use std::path::PathBuf;
use structopt::StructOpt;

#[macro_use]
extern crate log;

mod command;

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Subcommand,
}

#[derive(StructOpt, Debug)]
enum Subcommand {
    ParseWebserverLogs,
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
        Subcommand::ParseWebserverLogs => {
            let handler = command::ParseWebserverLogs::new();
            handler.parse().unwrap();
        }
    }
}
