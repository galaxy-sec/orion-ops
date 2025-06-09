mod args;
mod spec;
//mod vault;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use args::GCustCmd;
use clap::Parser;
use orion_syspec::error::{SpecResult, report_error};
use spec::do_cust_cmd;

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
        let cmd = GCustCmd::parse();
        debug!("galaxy flow running .....");
        do_cust_cmd(cmd).await?;
        Ok(())
    }
}
