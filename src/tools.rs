use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct GitRepo {}
#[allow(dead_code)]
impl GitRepo {
    pub(crate) fn pull(_repo: &str) -> PathBuf {
        PathBuf::from("./os.sh")
    }
}
#[derive(Default, Clone, Debug)]
pub struct GxShell {}
#[allow(dead_code)]
impl GxShell {
    pub(crate) fn exec(_file: &str, _fun: &str) -> String {
        r#"{"result": true}"#.to_string()
    }
}
#[derive(Default, Clone, Debug)]
pub struct Http {}
impl Http {}
