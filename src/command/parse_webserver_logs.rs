use std::io::{self, BufRead};

pub struct ParseWebserverLogs;

impl ParseWebserverLogs {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self) -> Result<(), ()> {
        let stdin = io::stdin();

        info!("parsing webserver logs...");

        for line in stdin.lock().lines().filter_map(|l| l.ok()) {
            info!("{}", line);
        }

        Ok(())
    }
}
