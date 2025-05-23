pub mod git;
pub mod http;
pub mod local;
pub mod types;

pub use git::GitAddr;
pub use http::HttpAddr;
pub use local::LocalAddr;
pub use local::path_file_name;
pub use local::rename_path;
pub use types::AddrType;
