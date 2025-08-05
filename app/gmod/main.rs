mod args;
mod spec;
//mod vault;

extern crate log;
#[macro_use]
extern crate clap;

use crate::args::GxModCmd;
use clap::Parser;
use galaxy_ops::error::{MainResult, report_error};
use orion_error::ErrorOwe;
use orion_variate::vars::setup_start_env_vars;
use spec::do_mod_cmd;

#[tokio::main]
async fn main() {
    use std::process;
    match GxMod::run().await {
        Err(e) => report_error(e),
        Ok(_) => {
            return;
        }
    }
    process::exit(-1);
}

pub struct GxMod {}
impl GxMod {
    pub async fn run() -> MainResult<()> {
        setup_start_env_vars().owe_res()?;
        println!("gmod: {}", env!("CARGO_PKG_VERSION"));
        let cmd = GxModCmd::parse();
        do_mod_cmd(cmd).await?;
        Ok(())
    }
}
