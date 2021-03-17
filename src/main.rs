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
pub struct IntegrationTestsOpts {
    #[structopt(long, default_value = "localhost")]
    url: String,
    #[structopt(long, env = "WEBSERVER_KEY_NAME")]
    key_name: String,
    #[structopt(long, env = "WEBSERVER_KEY_VALUE")]
    key_value: String,
}

#[derive(StructOpt, Debug)]
pub struct CleanMqttStoreDbOpts {
    #[structopt(long)]
    db_path: String,
    #[structopt(long, default_value = "7")]
    days: u8,
    #[structopt(long)]
    vacuum: bool,
}

#[derive(StructOpt, Debug)]
pub struct CollectMetricsOpts {
    #[structopt(long, default_value = "5000")]
    sleep_ms: u64,
    #[structopt(long, env = "MNGR_INFLUX_URL")]
    influx_url: String,
    #[structopt(long, env = "MNGR_INFLUX_KEY")]
    influx_key: String,
    #[structopt(long, env = "MNGR_INFLUX_ORG")]
    influx_org: String,
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
