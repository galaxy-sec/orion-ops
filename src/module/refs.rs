use crate::error::SpecError;
use crate::predule::*;

use async_trait::async_trait;
use contracts::requires;
use orion_error::UvsLogicFrom;

use super::ModelSTD;
use crate::types::LocalizeOptions;
use crate::{
    addr::AddrType,
    const_vars::MOD_DIR,
    error::SpecResult,
    module::model::ModModelSpec,
    tools::make_clean_path,
    types::{Localizable, Persistable, UnitUpdateable, ValuePath},
};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModuleSpecRef {
    name: String,
    addr: AddrType,
    #[serde(alias = "node")]
    model: ModelSTD,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    enable: Option<bool>,
    #[serde(skip)]
    local: Option<PathBuf>,
}

impl ModuleSpecRef {
    pub fn from<S: Into<String>, A: Into<AddrType>>(
        name: S,
        addr: A,
        node: ModelSTD,
    ) -> ModuleSpecRef {
        Self {
            name: name.into(),
            addr: addr.into(),
            model: node,
            enable: None,
            local: None,
        }
    }
    pub fn with_enable(mut self, effective: bool) -> Self {
        self.enable = Some(effective);
        self
    }

    pub fn is_enable(&self) -> bool {
        self.enable.unwrap_or(true)
    }
    pub fn spec_path(&self, root: &Path) -> PathBuf {
        root.join("mods").join(self.name.as_str())
    }
    pub fn set_local(&mut self, local: PathBuf) {
        self.local = Some(local);
    }
    pub fn get_target_spec(&self) -> SpecResult<Option<ModModelSpec>> {
        if self.is_enable() {
            if let Some(local) = &self.local {
                let target_root = local.join(self.name());
                let target_path = target_root.join(self.model().to_string());
                if target_path.exists() {
                    let spec = ModModelSpec::load_from(&target_path).with(&target_root)?;
                    return Ok(Some(spec));
                }
            }
        }
        Ok(None)
    }
}
impl ModuleSpecRef {
    #[requires(self.local.is_some())]
    pub async fn update(
        &self,
        _sys_root: &Path,
        options: &UpdateOptions,
    ) -> SpecResult<UnitUpdateValue> {
        //trace!(target: "spec/mod/",  "{:?}",self );
        if let Some(local) = &self.local {
            let mut flag = auto_exit_log!(
                info!(target: "/mod/ref",  "update mod ref {} success!", self.name ),
                error!(target: "/mod/ref", "update mod ref {} fail!", self.name )
            );
            std::fs::create_dir_all(local).owe_res().with(local)?;
            let target_root = local.join(self.name());
            let target_path = target_root.join(self.model().to_string());
            if !target_path.exists() || options.clean_exist_ref_mod() {
                let tmp_name = "__mod";
                let prj_path = self.addr.update_rename(local, tmp_name, options).await?;
                let mod_path = prj_path.position().join(MOD_DIR);
                let tmp_path = local.join(tmp_name);
                make_clean_path(&target_root)?;

                std::fs::rename(&mod_path, &target_root)
                    .owe_logic()
                    .with(("from", &mod_path))
                    .with(("to", &target_root))?;
                if tmp_path.exists() {
                    std::fs::remove_dir_all(tmp_path).owe_sys()?;
                }
            }

            debug!(target: "mod/ref",  "update target success!" );
            //let target_path = target_root.join(self.node().to_string());
            let spec = ModModelSpec::load_from(&target_path).with(&target_root)?;
            let _x = spec.update_local(&target_path, options).await?;
            ModModelSpec::clean_other(&target_root, self.model())?;
            flag.mark_suc();
            return Ok(_x);
        } else {
            Err(SpecError::from_logic(
                "no local value in ModuleSpecRef ".into(),
            ))
        }
    }

    pub fn spec_value_path(&self, parent: ValuePath) -> ValuePath {
        let value = PathBuf::from(self.name());
        parent.join(value)
    }
}

#[async_trait]
impl Localizable for ModuleSpecRef {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> SpecResult<()> {
        if self.enable.is_none_or(|x| x) {
            if let Some(local) = &self.local {
                let mut flag = auto_exit_log!(
                    info!(target: "spec/mod/", "localize mod {} success!", self.name ),
                    error!(target: "spec/mod/", "localize mod {} fail!", self.name )
                );
                let mod_path = local.join(self.name.as_str());
                let target_path = mod_path.join(self.model().to_string());
                let spec = ModModelSpec::load_from(&target_path)?;
                //if let Some(dst) = &dst_path {
                //    spec.save_main(dst.local(), None)?;
                //}
                let value = PathBuf::from(self.name());
                //let local = PathBuf::from(self.name()).join("local");
                let cur_dst_path = dst_path.map(|x| x.join(value));
                spec.localize(cur_dst_path, options).await?;
                flag.mark_suc();
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}
