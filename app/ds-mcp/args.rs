use orion_syspec::infra::DfxArgsGetter;

#[derive(Default)]
pub struct TempArgs {}
impl DfxArgsGetter for TempArgs {
    fn debug_level(&self) -> usize {
        5
    }

    fn log_setting(&self) -> Option<String> {
        None
    }
}
