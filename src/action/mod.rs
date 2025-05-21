pub mod act;
pub mod bsh;
pub mod gxl;
pub mod prj;

const HOST_SETUP_GXL: &str = include_str!("init/host/actions/setup.gxl");
const HOST_UPDATE_GXL: &str = include_str!("init/host/actions/update.gxl");
const HOST_SETUP_SH: &str = include_str!("init/host/actions/setup.sh");
const HOST_PRJ: &str = include_str!("init/host/_gal/work.gxl");

const K8S_SETUP_GXL: &str = include_str!("init/k8s/actions/setup.gxl");
const K8S_UPDATE_GXL: &str = include_str!("init/k8s/actions/update.gxl");
const K8S_PRJ: &str = include_str!("init/k8s/_gal/work.gxl");
