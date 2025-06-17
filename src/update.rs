use derive_more::From;

//use super::predule::*;
#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum UpdateLevel {
    #[default]
    All,
    Mod,
    Elm,
}

#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum RedoLevel {
    #[default]
    ReLocal,
    ReAll,
    ReChange,
}

impl From<usize> for RedoLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => RedoLevel::ReChange,
            1 => RedoLevel::ReLocal,
            2 => RedoLevel::ReAll,
            _ => RedoLevel::ReAll,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateOptions {
    re_level: RedoLevel,
    up_level: UpdateLevel,
}
impl Default for UpdateOptions {
    fn default() -> Self {
        Self {
            re_level: RedoLevel::default(),
            up_level: UpdateLevel::All,
        }
    }
}
impl UpdateOptions {
    pub fn new(re_level: RedoLevel, up_level: UpdateLevel) -> Self {
        Self { re_level, up_level }
    }
    pub fn redo_level(&self) -> RedoLevel {
        self.re_level.clone()
    }
    pub fn level(&self) -> UpdateLevel {
        self.up_level.clone()
    }
    pub fn for_test() -> Self {
        Self {
            re_level: RedoLevel::default(),
            up_level: UpdateLevel::All,
        }
    }
    pub fn for_depend() -> Self {
        Self {
            re_level: RedoLevel::ReChange,
            up_level: UpdateLevel::All,
        }
    }
}
