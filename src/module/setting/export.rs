use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

use super::LocalizeConf;

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct Setting {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    localize: Option<LocalizeConf>,
}

impl Setting {
    pub fn example() -> Self {
        Self {
            localize: Some(LocalizeConf::example()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use crate::types::Configable;

    use super::*;
    use orion_error::TestAssert;

    #[test]
    fn test_setting_serialization() {
        let temp_dir = temp_dir();
        let save_path = temp_dir.join("setting.yml");
        let setting = Setting::example();
        setting.save_conf(&save_path).assert();
        println!("{}", std::fs::read_to_string(&save_path).unwrap());
        Setting::from_conf(&save_path).assert();
    }
}
