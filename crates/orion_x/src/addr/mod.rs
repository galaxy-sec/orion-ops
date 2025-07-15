pub mod git;
pub mod http;
pub mod local;
pub mod types;

pub use git::GitAddr;
pub use http::HttpAddr;
pub use local::path_file_name;
pub use local::rename_path;
pub use local::LocalAddr;
pub use types::AddrType;
pub mod error;

pub use error::{AddrError, AddrReason, AddrResult};
