use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    io::{self, BufRead},
};

lazy_static! {
    static ref METRIC_REGEX: Regex = Regex::new(r#".*metric:(?P<metric_csv>.+)"#).unwrap();
}

type Metrics = HashMap<String, Vec<f64>>;

pub struct ParseWebserverLogs;

impl ParseWebserverLogs {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self) -> Result<Metrics, ()> {
        let stdin = io::stdin();

        info!("parsing webserver logs...");

        let metric_lines: Vec<String> = stdin
            .lock()
            .lines()
            .filter_map(|l| l.ok())
            .filter_map(|l| {
                METRIC_REGEX
                    .captures(&l)
                    .and_then(|c| c.name("metric_csv"))
                    .and_then(|m| Some(m.as_str().to_owned()))
            })
            .collect();

        info!("parsed {} metric lines...", metric_lines.len());

        let mut metrics = Metrics::new();
        for (name, value) in metric_lines.iter().filter_map(|m| Self::parse_line(m).ok()) {
            metrics
                .entry(name)
                .or_insert_with(|| Vec::new())
                .push(value);
        }

        Ok(metrics)
    }

    fn parse_line(line: &str) -> Result<(String, f64), ()> {
        let parts: Vec<_> = line.split(";").collect();

        if parts.len() != 2 {
            return Err(());
        }

        let name: String = parts[0].to_owned();
        let value: f64 = parts[1].parse().map_err(|_| ())?;

        Ok((name, value))
    }
}
