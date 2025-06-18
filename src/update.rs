use derive_more::From;

//use super::predule::*;
#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum DurationLevel {
    #[default]
    DurLong,
    DurProj,
    DurMod,
}

#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum ScopeLevel {
    #[default]
    InProj,
    InHost,
}

impl From<usize> for UpdateOptions {
    fn from(value: usize) -> Self {
        match value {
            1 => Self::new(ScopeLevel::InProj, DurationLevel::DurProj),
            2 => Self::new(ScopeLevel::InHost, DurationLevel::DurLong),
            _ => Self::new(ScopeLevel::InHost, DurationLevel::DurLong),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateOptions {
    re_level: ScopeLevel,
    up_level: DurationLevel,
}
impl Default for UpdateOptions {
    fn default() -> Self {
        Self {
            re_level: ScopeLevel::default(),
            up_level: DurationLevel::DurLong,
        }
    }
}
impl UpdateOptions {
    pub fn new(re_level: ScopeLevel, up_level: DurationLevel) -> Self {
        Self { re_level, up_level }
    }
    pub fn redo_level(&self) -> ScopeLevel {
        self.re_level.clone()
    }
    pub fn level(&self) -> DurationLevel {
        self.up_level.clone()
    }
    pub fn for_test() -> Self {
        Self {
            re_level: ScopeLevel::default(),
            up_level: DurationLevel::DurLong,
        }
    }
    pub fn for_depend() -> Self {
        Self {
            re_level: ScopeLevel::ReChange,
            up_level: DurationLevel::DurLong,
        }
    }
}
impl UpdateOptions {
    pub fn clean_git_cache(&self) -> bool {
        todo!()
    }
    pub fn clean_exists_depend(&self) -> bool {
        todo!()
    }
    pub(crate) fn use_remote_file(&self) -> bool {
        todo!()
    }
}
