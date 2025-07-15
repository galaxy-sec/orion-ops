use derive_more::From;

use crate::vars::ValueDict;

//use super::predule::*;
#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum KeepDuration {
    #[default]
    DurProj,
}

#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum UpdateScope {
    InElm,
    InMod,
    #[default]
    InProj,
    InHost,
}

impl From<(usize, ValueDict)> for UpdateOptions {
    fn from(value: (usize, ValueDict)) -> Self {
        match value.0 {
            0 => Self::new(UpdateScope::InElm, value.1),
            1 => Self::new(UpdateScope::InMod, value.1),
            2 => Self::new(UpdateScope::InProj, value.1),
            3 => Self::new(UpdateScope::InHost, value.1),
            _ => Self::new(UpdateScope::InHost, value.1),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct UpdateOptions {
    scope_level: UpdateScope,
    values: ValueDict,
}
impl UpdateOptions {
    pub fn new(re_level: UpdateScope, values: ValueDict) -> Self {
        Self {
            scope_level: re_level,
            values,
        }
    }
    pub fn for_test() -> Self {
        Self {
            scope_level: UpdateScope::InProj,
            values: ValueDict::default(),
        }
    }
    pub fn values(&self) -> &ValueDict {
        &self.values
    }
}
impl UpdateOptions {
    pub fn clean_git_cache(&self) -> bool {
        !match &self.scope_level {
            UpdateScope::InElm => true,
            UpdateScope::InMod => true,
            UpdateScope::InProj => true,
            UpdateScope::InHost => false,
        }
    }
    pub fn clean_exists_depend(&self) -> bool {
        !match &self.scope_level {
            UpdateScope::InElm => true,
            UpdateScope::InMod => true,
            UpdateScope::InProj => false,
            UpdateScope::InHost => false,
        }
    }
    pub fn reuse_remote_file(&self) -> bool {
        match &self.scope_level {
            UpdateScope::InElm => true,
            UpdateScope::InMod => true,
            UpdateScope::InProj => true,
            UpdateScope::InHost => false,
        }
    }
    pub fn copy_to_exists_path(&self) -> bool {
        !match &self.scope_level {
            UpdateScope::InElm => true,
            UpdateScope::InMod => true,
            UpdateScope::InProj => true,
            UpdateScope::InHost => false,
        }
    }
    pub fn clean_exist_ref_mod(&self) -> bool {
        !match &self.scope_level {
            UpdateScope::InElm => true,
            UpdateScope::InMod => false,
            UpdateScope::InProj => false,
            UpdateScope::InHost => false,
        }
    }
}
