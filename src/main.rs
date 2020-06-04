#![allow(dead_code)]

use std::path::PathBuf;
use structopt::StructOpt;

#[macro_use]
extern crate log;
extern crate lazy_static;

mod command;

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Subcommand,
}

#[derive(StructOpt, Debug)]
enum Subcommand {
    IntegrationTests(IntegrationTestsOpts),
}

#[derive(StructOpt, Debug)]
pub struct ParseWebserverLogsOpts {
    #[structopt(long, env = "MNGR_DATABASE_PATH")]
    db_path: String,
}

#[derive(StructOpt, Debug)]
pub struct IntegrationTestsOpts {
    #[structopt(long, default_value = "localhost")]
    url: String,
}

#[tokio::main]
async fn main() {
    match std::env::var("MNGR_HOME_DIR") {
        Ok(path) => {
            let mut home_dir = PathBuf::from(path);
            home_dir.push(".env");
            dotenv::from_path(home_dir).ok();
        }
        Err(_) => {
            // use current directory
            dotenv::dotenv().ok();
        }
    }

    pretty_env_logger::init();

    let opts = Opts::from_args();

    debug!("Execution with opts: {:#?}", opts);
    match opts.cmd {
        Subcommand::IntegrationTests(opts) => {
            let handler = command::IntegrationTests::new(opts);
            if let Err(message) = handler.run_tests().await {
                error!("failed with message: {}", message);
            } else {
                info!("tests succeeded");
            }
        }
    }
}
