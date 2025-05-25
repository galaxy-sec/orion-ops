pub mod init;
pub mod metrc;
pub mod refs;
pub mod spec;
pub mod target;
pub mod work;
use std::{fmt::Display, str::FromStr};

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TargetNodeType {
    Host,
    K8s,
}
impl FromStr for TargetNodeType {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "host" => Ok(TargetNodeType::Host),
            "k8s" => Ok(TargetNodeType::K8s),
            _ => Err(()),
        }
    }
}
impl Display for TargetNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetNodeType::Host => write!(f, "host"),
            TargetNodeType::K8s => write!(f, "k8s"),
        }
    }
}
