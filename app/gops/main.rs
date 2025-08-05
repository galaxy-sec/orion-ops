mod args;
mod spec;
//mod vault;

extern crate clap;
extern crate log;

use args::GInsCmd;
use clap::Parser;
use galaxy_ops::error::{MainResult, report_error};
use orion_error::ErrorOwe;
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
    pub async fn run() -> MainResult<()> {
        setup_start_env_vars().owe_res()?;
        let cmd = GInsCmd::parse();
        println!("gops: {}", env!("CARGO_PKG_VERSION"));
        do_ins_cmd(cmd).await?;
        Ok(())
    }
}
