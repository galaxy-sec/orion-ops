use derive_more::From;

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

impl From<usize> for UpdateOptions {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::new(UpdateScope::InElm),
            1 => Self::new(UpdateScope::InMod),
            2 => Self::new(UpdateScope::InProj),
            3 => Self::new(UpdateScope::InHost),
            _ => Self::new(UpdateScope::InHost),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateOptions {
    scope_level: UpdateScope,
}
impl Default for UpdateOptions {
    fn default() -> Self {
        Self {
            scope_level: UpdateScope::default(),
        }
    }
}
impl UpdateOptions {
    pub fn new(re_level: UpdateScope) -> Self {
        Self {
            scope_level: re_level,
        }
    }
    pub fn for_test() -> Self {
        Self {
            scope_level: UpdateScope::InProj,
        }
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
    pub(crate) fn clean_exist_ref_mod(&self) -> bool {
        !match &self.scope_level {
            UpdateScope::InElm => true,
            UpdateScope::InMod => false,
            UpdateScope::InProj => false,
            UpdateScope::InHost => false,
        }
    }
}
