use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

use crate::addr::AddrType;
#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    af_cep: String,
    af_img: Option<String>,
    af_bin: Option<String>,
}
impl Artifact {
    pub fn new<S: Into<String>>(cep: S) -> Self {
        Self {
            af_cep: cep.into(),
            af_img: None,
            af_bin: None,
        }
    }
}

impl From<(&str, &str, &str)> for Artifact {
    fn from(value: (&str, &str, &str)) -> Self {
        Self {
            af_cep: value.0.into(),
            af_img: Some(value.1.into()),
            af_bin: Some(value.2.into()),
        }
    }
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct DockImage {
    cep: String,
    addr: AddrType,
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct BinPackage {
    cep: String,
    addr: AddrType,
}
