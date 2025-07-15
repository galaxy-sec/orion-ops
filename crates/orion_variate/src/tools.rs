use url::Url;

#[derive(Default, Clone, Debug)]
pub struct Http {}
impl Http {}
pub fn get_repo_name(url_str: &str) -> Option<String> {
    // 先尝试处理SSH格式的Git地址
    if url_str.starts_with("git@") {
        if let Some(repo_part) = url_str.split(':').next_back() {
            return repo_part.split('/').next_back().map(String::from);
        }
    }

    // 原有HTTP/HTTPS URL处理逻辑
    let url = Url::parse(url_str).ok()?;
    let last = url.path_segments()?.rev().find(|s| !s.is_empty());
    last.map(String::from)
}

pub fn test_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
