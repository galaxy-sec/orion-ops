use std::path::PathBuf;

use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use orion_exchange::vars::ValueDict;

use crate::{
    addr::path_file_name,
    error::RunResult,
    modul::ModuleSpecRef,
    tpl::{TPlEngineType, TplRender},
    types::{AsyncUpdateable, JsonAble, Localizeable, SaveAble, TomlAble},
};

#[derive(Getters, Clone, Debug)]
pub struct ModRunning {
    name: String,
    spec: ModuleSpecRef,
    value: ValueDict,
}
impl SaveAble<ModRunning> for ModRunning {
    fn save_to(&self, path: &PathBuf) -> RunResult<()> {
        let root = path.join(self.name());
        std::fs::create_dir_all(&root).owe_conf()?;
        let spec_path = root.join("spec.toml");
        self.spec.save_toml(&spec_path)?;
        let json_path = root.join("value.json");
        self.value.save_json(&json_path)?;
        Ok(())
    }

    fn load_from(path: &PathBuf) -> RunResult<Self> {
        let name = path_file_name(path)?;
        let spec_path = path.join("spec.toml");
        let spec = ModuleSpecRef::from_toml(&spec_path)?;
        let json_path = path.join("value.json");
        let value = ValueDict::from_json(&json_path)?;
        Ok(Self { name, spec, value })
    }
}
impl ModRunning {
    pub fn new(spec: ModuleSpecRef, value: ValueDict) -> Self {
        let name = spec.name().clone();
        Self { name, spec, value }
    }
}

#[async_trait]
impl Localizeable for ModRunning {
    async fn localize(&self, path: &PathBuf) -> RunResult<()> {
        let mut ctx = WithContext::want("modul localize");

        let root = path.join(self.name());
        let tpl = root.join("spec");
        let dst = root.join("local");
        let data = root.join("value.json");
        ctx.with_path("dst", &dst);
        std::fs::create_dir_all(&dst).owe_res().with(&ctx)?;
        if tpl.exists() {
            ctx.with_path("tpl", &tpl);
            std::fs::remove_dir_all(&tpl).owe_res().with(&ctx)?;
        }
        ctx.with_path("root", &root);
        self.spec.update_rename(&root, "spec").await.with(&ctx)?;
        TplRender::render_path(TPlEngineType::Handlebars, &tpl, &dst, &data).with(&ctx)?;
        Ok(())
    }
}
#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_exchange::vars::{ValueDict, ValueType};

    use crate::{
        addr::LocalAddr,
        const_vars::{MODULES_INS_ROOT, MODULES_SPC_ROOT},
        error::RunResult,
        modul::{ModuleSpecRef, NodeType},
        types::{Localizeable, SaveAble},
    };

    use super::ModRunning;

    #[tokio::test]
    async fn test_mod_running() -> RunResult<()> {
        let spec = ModuleSpecRef::from(
            "mysql",
            LocalAddr::from(format!("{}/mysql", MODULES_SPC_ROOT)),
            NodeType::Host,
        );
        let mut dict = ValueDict::new();
        dict.insert("key", ValueType::from("abc"));
        let sys = ModRunning::new(spec, dict);
        let path = PathBuf::from(MODULES_INS_ROOT);
        sys.save_to(&path)?;
        sys.localize(&path).await?;
        Ok(())
    }
}
