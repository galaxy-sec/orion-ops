use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OsType {
    MacOs,
    Ubuntu,
}
