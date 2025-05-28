use std::path::{Path, PathBuf};

use crate::vars::{ValueDict, ValueType};
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, WithContext};

use crate::{
    addr::{GitAddr, path_file_name},
    error::{SpecReason, SpecResult, ToErr},
    tpl::{TPlEngineType, TplRender},
    types::{AsyncUpdateable, Configable, JsonAble, Persistable},
};

use super::{refs::SysModelSpecRef, spec::SysModelSpec};

#[derive(Getters, Clone, Debug)]
pub struct RunningSystem {
    name: String,
    spec_ref: SysModelSpecRef,
    value: ValueDict,
    local: Option<PathBuf>,
}
impl Persistable<RunningSystem> for RunningSystem {
    fn save_to(&self, path: &Path) -> SpecResult<()> {
        let root = path.join(self.name());
log::debug!("Creating system directory at: {}", root.display());
std::fs::create_dir_all(&root).owe_conf()
    .map_err(|e| {
        log::error!("Failed to create system directory at {}: {}", root.display(), e);
        e
    })?;

let spec_path = root.join("spec.yml");
log::debug!("Saving system spec to: {}", spec_path.display());
self.spec_ref.save_conf(&spec_path)
    .map_err(|e| {
        log::error!("Failed to save system spec to {}: {}", spec_path.display(), e);
        e
    })?;

let json_path = root.join("value.json");
log::debug!("Saving system values to: {}", json_path.display());
self.value.save_json(&json_path)
    .map_err(|e| {
        log::error!("Failed to save system values to {}: {}", json_path.display(), e);
        e
    })?;
        Ok(())
    }

    fn load_from(path: &Path) -> SpecResult<Self> {
        log::debug!("Loading system from path: {}", path.display());
let name = path_file_name(path)
    .map_err(|e| {
        log::error!("Failed to get path name from {}: {}", path.display(), e);
        e
    })?;

let spec_path = path.join("spec.yml");
log::debug!("Loading system spec from: {}", spec_path.display());
let spec = SysModelSpecRef::from_conf(&spec_path)
    .map_err(|e| {
        log::error!("Failed to load system spec from {}: {}", spec_path.display(), e);
        e
    })?;

let json_path = path.join("value.json");
log::debug!("Loading system values from: {}", json_path.display());
let value = ValueDict::from_json(&json_path)
    .map_err(|e| {
        log::error!("Failed to load system values from {}: {}", json_path.display(), e);
        e
    })?;
        Ok(Self {
            name,
            spec_ref: spec,
            value,
            local: Some(path.to_path_buf()),
        })
    }
}
impl RunningSystem {
    pub fn new(spec: SysModelSpecRef, value: ValueDict) -> Self {
        let name = spec.name().clone();
        Self {
            name,
            spec_ref: spec,
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
log::debug!("Preparing to update system spec at: {}", spec.display());
if spec.exists() {
    ctx.with_path("spec", &spec);
    log::debug!("Removing existing spec directory at: {}", spec.display());
    std::fs::remove_dir_all(&spec).owe_res()
        .map_err(|e| {
            log::error!("Failed to remove spec directory at {}: {}", spec.display(), e);
            e
        })
        .with(&ctx)?;
}

log::info!("Updating system spec from remote repository");
self.spec_ref
    .update_rename(&local, "spec")
    .await
    .with(&ctx)?;

ctx.with("action", "sys-spec load");
log::debug!("Loading updated system spec from: {}", spec.display());
let spec = SysModelSpec::load_from(&spec)
    .map_err(|e| {
        log::error!("Failed to load updated system spec from {}: {}", spec.display(), e);
        e
    })
    .with(&ctx)?;

ctx.with("action", "spec update");
log::info!("Applying system spec updates");
spec.update_local().await
    .map_err(|e| {
        log::error!("Failed to update system spec: {}", e);
        e
    })
    .with(&ctx)?;
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

log::debug!("Localizing system resources from {} to {}", tpl.display(), dst.display());

ctx.with_path("local", &dst);
log::debug!("Creating localization directory at: {}", dst.display());
std::fs::create_dir_all(&dst).owe_res()
    .map_err(|e| {
        log::error!("Failed to create localization directory at {}: {}", dst.display(), e);
        e
    })
    .with(&ctx)?;

log::info!("Rendering system templates");
TPlEngineType::Handlebars
    .render_path(&tpl, &dst, &data)
    .map_err(|e| {
        log::error!(
            "Failed to render templates from {} to {}: {}", 
            tpl.display(), 
            dst.display(), 
            e
        );
        e
    })?;
        Ok(())
    }
}
pub fn make_runsystem_example() -> RunningSystem {
    let target = "example-sys-x1";
    let spec = SysModelSpecRef::from(
        target,
        GitAddr::from("https://e.coding.net/dy-sec/galaxy-open/spec_example_sys.git").path(target),
    );
    let mut dict = ValueDict::new();
    dict.insert("SYS_KEY", ValueType::from("example-sys"));
    dict.insert("INS_MAX_MEM", ValueType::from(100));
    dict.insert("CACHE_SIZE", ValueType::from(4));
    RunningSystem::new(spec, dict)
}

pub fn make_runsystem_new(repo: &str, path: &str) -> RunningSystem {
    //let name = get_last_segment(repo).unwrap_or("unknow".into());
    let spec = SysModelSpecRef::from(path, GitAddr::from(repo).path(path));
    let mut dict = ValueDict::new();
    dict.insert("SYS_KEY", ValueType::from("example-sys"));
    dict.insert("INS_MAX_MEM", ValueType::from(100));
    dict.insert("CACHE_SIZE", ValueType::from(4));
    RunningSystem::new(spec, dict)
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
