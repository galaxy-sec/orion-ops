#[macro_export]
macro_rules! log_flag {
    ($suc_log:expr, $fail_log:expr) => {
        scopeguard::guard(crate::tools::BoolFlag::default(), |flag| {
            if flag.is_suc() {
                $suc_log;
            } else {
                $fail_log;
            }
        })
    };
}
