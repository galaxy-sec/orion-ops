use std::path::PathBuf;

use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use orion_exchange::vars::{ValueDict, ValueType};

use crate::{
    addr::{LocalAddr, path_file_name},
    const_vars::SYS_MODEL_SPC_ROOT,
    error::{SpecReason, SpecResult, ToErr},
    tpl::{TPlEngineType, TplRender},
    types::{AsyncUpdateable, JsonAble, Persistable, TomlAble},
};

use super::refs::SysModelSpecRef;

#[derive(Getters, Clone, Debug)]
pub struct RunningSystem {
    name: String,
    spec: SysModelSpecRef,
    value: ValueDict,
    local: Option<PathBuf>,
}
impl Persistable<RunningSystem> for RunningSystem {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        let root = path.join(self.name());
        std::fs::create_dir_all(&root).owe_conf()?;
        let spec_path = root.join("spec.toml");
        self.spec.save_toml(&spec_path)?;
        let json_path = root.join("value.json");
        self.value.save_json(&json_path)?;
        Ok(())
    }

    fn load_from(path: &PathBuf) -> SpecResult<Self> {
        let name = path_file_name(path)?;
        let spec_path = path.join("spec.toml");
        let spec = SysModelSpecRef::from_toml(&spec_path)?;
        let json_path = path.join("value.json");
        let value = ValueDict::from_json(&json_path)?;
        Ok(Self {
            name,
            spec,
            value,
            local: Some(path.clone()),
        })
    }
}
impl RunningSystem {
    pub fn new(spec: SysModelSpecRef, value: ValueDict) -> Self {
        let name = spec.name().clone();
        Self {
            name,
            spec,
            value,
            local: None,
        }
    }
    pub async fn update(&self) -> SpecResult<()> {
        let mut ctx = WithContext::want("sys spec update");
        let local = self
            .local
            .clone()
            .ok_or(SpecReason::Miss("local-path".into()).to_err().with(&ctx))?;
        let spec = local.join("spec");
        if spec.exists() {
            ctx.with_path("spec", &spec);
            std::fs::remove_dir_all(&spec).owe_res().with(&ctx)?;
        }
        self.spec.update_rename(&local, "spec").await?;
        Ok(())
    }

    pub async fn localize(&self) -> SpecResult<()> {
        let mut ctx = WithContext::want("modul localize");
        let local = self
            .local
            .clone()
            .ok_or(SpecReason::Miss("local-path".into()).to_err().with(&ctx))?;
        let tpl = local.join("spec");
        let dst = local.join("local");
        let data = local.join("value.json");

        ctx.with_path("local", &dst);
        std::fs::create_dir_all(&dst).owe_res().with(&ctx)?;
        TplRender::render_path(TPlEngineType::Handlebars, &tpl, &dst, &data)?;
        Ok(())
    }
}
pub fn make_runsystem_example() -> RunningSystem {
    let spec = SysModelSpecRef::from(
        "example-sys",
        LocalAddr::from(format!("{}/example-sys", SYS_MODEL_SPC_ROOT)),
    );
    let mut dict = ValueDict::new();
    dict.insert("SYS_KEY", ValueType::from("example-sys"));
    dict.insert("INS_MAX_MEM", ValueType::from(100));
    dict.insert("CACHE_SIZE", ValueType::from(4));
    let sys = RunningSystem::new(spec, dict);
    sys
}
#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use crate::{
        const_vars::SYS_MODEL_INS_ROOT, error::SpecResult, system::work::make_runsystem_example,
        types::Persistable,
    };

    use super::RunningSystem;

    #[tokio::test]
    async fn test_sys_running() -> SpecResult<()> {
        let path = PathBuf::from(SYS_MODEL_INS_ROOT);
        let sys = make_runsystem_example();
        sys.save_to(&path)?;
        let sys = RunningSystem::load_from(&path.join(sys.name()))?;
        sys.update().await?;
        sys.localize().await?;
        Ok(())
    }
}
