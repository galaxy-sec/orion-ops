mod args;
mod spec;
//mod vault;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use args::GInsCmd;
use clap::Parser;
use orion_error::ErrorOwe;
use orion_ops::error::{SpecResult, report_error};
use orion_variate::vars::setup_start_env_vars;
use spec::do_ins_cmd;

#[tokio::main]
async fn main() {
    use std::process;
    match GxOps::run().await {
        Err(e) => report_error(e),
        Ok(_) => {
            return;
        }
    }
    process::exit(-1);
}

pub struct GxOps {}
impl GxOps {
    pub async fn run() -> SpecResult<()> {
        setup_start_env_vars().owe_res()?;
        let cmd = GInsCmd::parse();
        debug!("galaxy flow running .....");
        do_ins_cmd(cmd).await?;
        Ok(())
    }
}
