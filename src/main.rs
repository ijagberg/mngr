use std::path::PathBuf;
use structopt::StructOpt;

#[macro_use]
extern crate log;
extern crate lazy_static;

mod command;
mod db;

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Subcommand,
}

#[derive(StructOpt, Debug)]
enum Subcommand {
    ParseWebserverLogs(ParseWebserverLogsOpts),
}

#[derive(StructOpt, Debug)]
pub struct ParseWebserverLogsOpts {
    #[structopt(long, env = "MNGR_DATABASE_PATH")]
    db_path: String,
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
        Subcommand::ParseWebserverLogs(opts) => {
            let handler = command::ParseWebserverLogs::new(opts);
            handler.parse().unwrap();
        }
    }
}
