use std::path::PathBuf;

use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use orion_exchange::vars::{ValueDict, ValueType};

use crate::{
    addr::{GitAddr, path_file_name},
    error::ToErr,
    error::{SpecReason, SpecResult},
    modul::{ModuleSpecRef, NodeType},
    tpl::{TPlEngineType, TplRender},
    types::{AsyncUpdateable, JsonAble, Localizable, Persistable, TomlAble},
};

#[derive(Getters, Clone, Debug)]
pub struct RunningModule {
    name: String,
    spec: ModuleSpecRef,
    value: ValueDict,
    local: Option<PathBuf>,
}
impl Persistable<RunningModule> for RunningModule {
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
        let spec = ModuleSpecRef::from_toml(&spec_path)?;
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
impl RunningModule {
    pub fn new(spec: ModuleSpecRef, value: ValueDict) -> Self {
        let name = spec.name().clone();
        Self {
            name,
            spec,
            value,
            local: None,
        }
    }
    pub async fn update(&self) -> SpecResult<()> {
        let mut ctx = WithContext::want("modul update");
        let local = self
            .local
            .clone()
            .ok_or(SpecReason::Miss("local-path".into()).to_err().with(&ctx))?;
        let spec = local.join("spec");
        if spec.exists() {
            ctx.with_path("spec", &spec);
            std::fs::remove_dir_all(&spec).owe_res().with(&ctx)?;
        }
        ctx.with_path("local", &local);
        self.spec.update_rename(&local, "spec").await.with(&ctx)?;
        Ok(())
    }
}

#[async_trait]
impl Localizable for RunningModule {
    async fn localize(&self) -> SpecResult<()> {
        let mut ctx = WithContext::want("modul localize");
        let local = self
            .local
            .clone()
            .ok_or(SpecReason::Miss("local-path".into()).to_err().with(&ctx))?;
        let tpl = local.join("spec");
        let dst = local.join("local");
        let data = local.join("value.json");
        ctx.with_path("dst", &dst);
        TplRender::render_path(TPlEngineType::Handlebars, &tpl, &dst, &data).with(&ctx)?;
        Ok(())
    }
}
pub fn make_modins_example() -> SpecResult<RunningModule> {
    let spec = ModuleSpecRef::from(
        "mysql",
        //LocalAddr::from(format!("{}/mysql", MODULES_SPC_ROOT)),
        GitAddr::from("https://github/galaxy-sec/demo-mod").branch("master"),
        NodeType::Host,
    );
    let mut dict = ValueDict::new();
    dict.insert("key", ValueType::from("abc"));
    let sys = RunningModule::new(spec, dict);
    Ok(sys)
}
#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_exchange::vars::{ValueDict, ValueType};

    use crate::{
        addr::LocalAddr,
        const_vars::{MODULES_INS_ROOT, MODULES_SPC_ROOT},
        error::SpecResult,
        modul::{ModuleSpecRef, NodeType},
        types::{Localizable, Persistable},
    };

    use super::RunningModule;

    #[tokio::test]
    async fn test_mod_running() -> SpecResult<()> {
        let spec = ModuleSpecRef::from(
            "mysql",
            LocalAddr::from(format!("{}/mysql", MODULES_SPC_ROOT)),
            NodeType::Host,
        );
        let mut dict = ValueDict::new();
        dict.insert("key", ValueType::from("abc"));
        let sys = RunningModule::new(spec, dict);
        let path = PathBuf::from(MODULES_INS_ROOT);
        sys.save_to(&path)?;
        let sys = RunningModule::load_from(&path.join("mysql"))?;
        sys.update().await?;
        sys.localize().await?;
        Ok(())
    }
}
