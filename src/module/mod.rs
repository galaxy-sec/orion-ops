pub mod metrc;
pub mod refs;
pub mod spec;
pub mod target;
pub mod work;
use std::{fmt::Display, str::FromStr};

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeType {
    Host,
    K8s,
}
impl FromStr for NodeType {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "host" => Ok(NodeType::Host),
            "k8s" => Ok(NodeType::K8s),
            _ => Err(()),
        }
    }
}
impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Host => write!(f, "host"),
            NodeType::K8s => write!(f, "k8s"),
        }
    }
}
