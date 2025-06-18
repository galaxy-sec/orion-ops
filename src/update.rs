use derive_more::From;

//use super::predule::*;
#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum KeepDuration {
    #[default]
    DurLong,
    DurProj,
    DurMod,
}

#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum KeepScope {
    #[default]
    InProj,
    InHost,
}

impl From<usize> for UpdateOptions {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::new(KeepScope::InHost, KeepDuration::DurLong),
            1 => Self::new(KeepScope::InProj, KeepDuration::DurLong),
            2 => Self::new(KeepScope::InProj, KeepDuration::DurProj),
            3 => Self::new(KeepScope::InProj, KeepDuration::DurMod),
            _ => Self::new(KeepScope::InProj, KeepDuration::DurMod),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateOptions {
    scope_level: KeepScope,
    durat_level: KeepDuration,
}
impl Default for UpdateOptions {
    fn default() -> Self {
        Self {
            scope_level: KeepScope::default(),
            durat_level: KeepDuration::DurLong,
        }
    }
}
impl UpdateOptions {
    pub fn new(re_level: KeepScope, up_level: KeepDuration) -> Self {
        Self {
            scope_level: re_level,
            durat_level: up_level,
        }
    }
    pub fn redo_level(&self) -> KeepScope {
        self.scope_level.clone()
    }
    pub fn level(&self) -> KeepDuration {
        self.durat_level.clone()
    }
    pub fn for_test() -> Self {
        Self {
            scope_level: KeepScope::InProj,
            durat_level: KeepDuration::DurProj,
        }
    }
}
impl UpdateOptions {
    pub fn clean_git_cache(&self) -> bool {
        !match (&self.scope_level, &self.durat_level) {
            (KeepScope::InProj, KeepDuration::DurLong) => false,
            (KeepScope::InProj, KeepDuration::DurProj) => false,
            (KeepScope::InProj, KeepDuration::DurMod) => false,
            (KeepScope::InHost, KeepDuration::DurLong) => true,
            (KeepScope::InHost, KeepDuration::DurProj) => false,
            (KeepScope::InHost, KeepDuration::DurMod) => false,
        }
    }
    pub fn clean_exists_depend(&self) -> bool {
        !match (&self.scope_level, &self.durat_level) {
            (KeepScope::InProj, KeepDuration::DurLong) => true,
            (KeepScope::InProj, KeepDuration::DurProj) => true,
            (KeepScope::InProj, KeepDuration::DurMod) => false,
            (KeepScope::InHost, KeepDuration::DurLong) => true,
            (KeepScope::InHost, KeepDuration::DurProj) => true,
            (KeepScope::InHost, KeepDuration::DurMod) => false,
        }
    }
    pub fn reuse_remote_file(&self) -> bool {
        match (&self.scope_level, &self.durat_level) {
            (KeepScope::InProj, KeepDuration::DurLong) => true,
            (KeepScope::InProj, KeepDuration::DurProj) => true,
            (KeepScope::InProj, KeepDuration::DurMod) => false,
            (KeepScope::InHost, KeepDuration::DurLong) => true,
            (KeepScope::InHost, KeepDuration::DurProj) => true,
            (KeepScope::InHost, KeepDuration::DurMod) => false,
        }
    }
    pub fn copy_to_exists_path(&self) -> bool {
        match (&self.scope_level, &self.durat_level) {
            (KeepScope::InProj, KeepDuration::DurLong) => true,
            (KeepScope::InProj, KeepDuration::DurProj) => true,
            (KeepScope::InProj, KeepDuration::DurMod) => false,
            (KeepScope::InHost, KeepDuration::DurLong) => true,
            (KeepScope::InHost, KeepDuration::DurProj) => true,
            (KeepScope::InHost, KeepDuration::DurMod) => false,
        }
    }
}
