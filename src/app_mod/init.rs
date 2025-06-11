use std::path::Path;

use orion_error::{ErrorOwe, ErrorWith};

use crate::error::SpecResult;

pub const MOD_APP_GAL_WORK: &str = include_str!("init_tpl/_gal/work.gxl");
const MOD_APP_GITIGNORE: &str = include_str!("init_tpl/.gitignore");

pub fn mod_app_gitignore(path: &Path) -> SpecResult<()> {
    let ignore_path = path.join(".gitignore");
    if !ignore_path.exists() {
        std::fs::write(&ignore_path, MOD_APP_GITIGNORE)
            .owe_res()
            .with(&ignore_path)?;
    }
    Ok(())
}
