#[macro_export]
macro_rules! auto_exit_log {
    ($suc_log:expr, $fail_log:expr) => {
        scopeguard::guard(orion_infra::logging::BoolFlag::default(), |flag| {
            if flag.is_suc() {
                $suc_log;
            } else {
                $fail_log;
            }
        })
    };
}

#[derive(Default)]
pub struct BoolFlag {
    is_suc: bool,
}
impl BoolFlag {
    pub fn mark_suc(&mut self) {
        self.is_suc = true;
    }
    pub fn is_suc(&self) -> bool {
        self.is_suc
    }
}
