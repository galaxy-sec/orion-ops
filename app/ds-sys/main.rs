mod args;
mod spec;
//mod vault;

extern crate log;
#[macro_use]
extern crate clap;

use args::GSysCmd;
use clap::Parser;
use orion_error::ErrorOwe;
use orion_ops::error::{MainResult, report_error};
use orion_variate::vars::setup_start_env_vars;
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
    pub async fn run() -> MainResult<()> {
        setup_start_env_vars().owe_res()?;
        let cmd = GSysCmd::parse();
        println!("ds-sys: {}", env!("CARGO_PKG_VERSION"));
        do_sys_cmd(cmd).await?;
        Ok(())
    }
}
