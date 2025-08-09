use std::path::{Path, PathBuf};

use derive_getters::Getters;

use glob::Pattern;
#[derive(Default, Clone, Debug, Getters)]
pub struct TemplatePath {
    includes: Vec<PathBuf>,
    excludes: Vec<PathBuf>,
}

impl TemplatePath {
    pub fn new(includes: Vec<PathBuf>, excludes: Vec<PathBuf>) -> Self {
        Self { includes, excludes }
    }
    pub fn exclude_mut(&mut self) -> &mut Vec<PathBuf> {
        &mut self.excludes
    }

    pub fn is_exclude(&self, dst: &Path) -> bool {
        for exclude in &self.excludes {
            if dst.starts_with(exclude) {
                return true;
            }
            if let Ok(pattern) = Pattern::new(exclude.to_str().unwrap())
                && pattern.matches(dst.to_str().unwrap())
            {
                return true;
            }
        }
        false
    }
    pub fn is_include(&self, dst: &Path) -> bool {
        if self.includes().is_empty() {
            return true;
        }
        for include in &self.includes {
            if dst.starts_with(include) {
                return true;
            }
            if let Ok(pattern) = Pattern::new(include.to_str().unwrap())
                && pattern.matches(dst.to_str().unwrap())
            {
                return true;
            }
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let includes = vec![PathBuf::from("include/path")];
        let excludes = vec![PathBuf::from("exclude/path")];
        let path = TemplatePath::new(includes.clone(), excludes.clone());

        assert_eq!(path.includes(), &includes);
        assert_eq!(path.excludes(), &excludes);
    }

    #[test]
    fn test_exclude_mut() {
        let mut path = TemplatePath::default();
        path.exclude_mut().push(PathBuf::from("test/path"));

        assert_eq!(path.excludes(), &vec![PathBuf::from("test/path")]);
    }

    #[test]
    fn test_is_exclude() {
        let path = TemplatePath::new(vec![], vec![PathBuf::from("exclude/path")]);

        assert!(path.is_exclude(&PathBuf::from("exclude/path/sub")));
        assert!(!path.is_exclude(&PathBuf::from("other/path")));
    }

    #[test]
    fn test_is_include() {
        let path = TemplatePath::new(vec![PathBuf::from("include/path")], vec![]);

        assert!(path.is_include(&PathBuf::from("include/path/sub")));
        assert!(!path.is_include(&PathBuf::from("other/path")));
    }

    #[test]
    fn test_wildcard_exclude() {
        let path = TemplatePath::new(vec![], vec![PathBuf::from("exclude/*.txt")]);

        assert!(path.is_exclude(&PathBuf::from("exclude/test.txt")));
        assert!(!path.is_exclude(&PathBuf::from("exclude/test.log")));
    }

    #[test]
    fn test_wildcard_include() {
        let path = TemplatePath::new(vec![PathBuf::from("include/*.txt")], vec![]);

        assert!(path.is_include(&PathBuf::from("include/test.txt")));
        assert!(!path.is_include(&PathBuf::from("include/test.log")));
    }
}
