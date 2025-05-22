use std::collections::HashMap;

use derive_getters::Getters;

#[derive(Getters, Clone, Debug)]
pub struct WorkLoad {
    metrics: HashMap<String, bool>,
}
impl WorkLoad {
    pub fn tpl_init() -> Self {
        let mut metrics = HashMap::new();
        metrics.insert("request".to_string(), true);
        Self { metrics }
    }
}
