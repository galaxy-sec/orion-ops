use std::path::PathBuf;

use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith};
use orion_exchange::vars::ValueDict;

use crate::{
    addr::path_file_name,
    error::SpecResult,
    system::SysModelSpecRef,
    tpl::{TPlEngineType, TplRender},
    types::{AsyncUpdateable, JsonAble, SaveAble, TomlAble},
};

#[derive(Getters, Clone, Debug)]
pub struct SysRunning {
    name: String,
    spec: SysModelSpecRef,
    value: ValueDict,
}
impl SaveAble<SysRunning> for SysRunning {
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
        Ok(Self { name, spec, value })
    }
}
impl SysRunning {
    pub fn new(spec: SysModelSpecRef, value: ValueDict) -> Self {
        let name = spec.name().clone();
        Self { name, spec, value }
    }
    pub async fn update(&self, path: &PathBuf) -> SpecResult<()> {
        let root = path.join(self.name());
        let tpl = root.join("spec");
        if tpl.exists() {
            std::fs::remove_dir_all(&tpl).owe_res().with(&tpl)?;
        }
        self.spec.update_rename(&root, "spec").await?;
        Ok(())
    }

    pub async fn localize(&self, path: &PathBuf) -> SpecResult<()> {
        let root = path.join(self.name());
        let tpl = root.join("spec");
        let dst = root.join("local");
        let data = root.join("value.json");
        std::fs::create_dir_all(&dst).owe_res()?;
        TplRender::render_path(TPlEngineType::Handlebars, &tpl, &dst, &data)?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_exchange::vars::{ValueDict, ValueType};

    use crate::{
        addr::LocalAddr,
        const_vars::{SYS_MODEL_INS_ROOT, SYS_MODEL_SPC_ROOT},
        error::SpecResult,
        system::SysModelSpecRef,
        types::SaveAble,
    };

    use super::SysRunning;

    #[tokio::test]
    async fn test_sys_running() -> SpecResult<()> {
        let spec = SysModelSpecRef::from(
            "x-gateway",
            LocalAddr::from(format!("{}/x-gateway", SYS_MODEL_SPC_ROOT)),
        );
        let mut dict = ValueDict::new();
        dict.insert("key", ValueType::from("abc"));
        let sys = SysRunning::new(spec, dict);
        let path = PathBuf::from(SYS_MODEL_INS_ROOT);
        sys.save_to(&path)?;
        sys.update(&path).await?;
        sys.localize(&path).await?;
        Ok(())
    }
}
