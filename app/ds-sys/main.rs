mod args;
mod spec;
//mod vault;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use args::GSysCmd;
use clap::Parser;
use orion_syspec::{
    error::{SpecResult, report_error},
    vars::setup_start_env_vars,
};
use spec::do_sys_cmd;

#[tokio::main]
async fn main() {
    use std::process;
    match GxSys::run().await {
        Err(e) => report_error(e),
        Ok(_) => {
            return;
        }
    }
    process::exit(-1);
}

pub struct GxSys {}
impl GxSys {
    pub async fn run() -> SpecResult<()> {
        setup_start_env_vars()?;
        let cmd = GSysCmd::parse();
        debug!("galaxy flow running .....");
        do_sys_cmd(cmd).await?;
        Ok(())
    }
}
