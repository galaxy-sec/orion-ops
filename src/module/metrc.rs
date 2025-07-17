use derive_getters::Getters;
use indexmap::IndexMap;

#[derive(Getters, Clone, Debug)]
pub struct WorkLoad {
    metrics: IndexMap<String, bool>,
}
impl WorkLoad {
    pub fn tpl_init() -> Self {
        let mut metrics = IndexMap::new();
        metrics.insert("request".to_string(), true);
        Self { metrics }
    }
}
