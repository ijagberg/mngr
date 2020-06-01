use crate::{db, ParseWebserverLogsOpts};
use db::{Db, MetricRow};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    io::{self, BufRead},
};

lazy_static! {
    static ref METRIC_REGEX: Regex = Regex::new(r#".*metric:(?P<metric_csv>.+)"#).unwrap();
}

type Metrics = HashMap<String, Vec<(f64, i64)>>;

pub struct ParseWebserverLogs {
    opts: ParseWebserverLogsOpts,
}

impl ParseWebserverLogs {
    pub fn new(opts: ParseWebserverLogsOpts) -> Self {
        Self { opts }
    }

    pub fn parse(&self) -> Result<(), String> {
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
        for (name, value, timestamp) in metric_lines.iter().filter_map(|m| Self::parse_line(m).ok())
        {
            metrics
                .entry(name)
                .or_insert_with(|| Vec::new())
                .push((value, timestamp));
        }

        self.insert_in_db(metrics)?;

        Ok(())
    }

    fn insert_in_db(&self, metrics: Metrics) -> Result<(), String> {
        let timer = std::time::Instant::now();
        let db: Db<MetricRow> = Db::new(self.opts.db_path.clone());

        let mut successful_inserts = 0;
        for (name, values) in &metrics {
            for (value, timestamp) in values {
                if let Err(e) = db.insert_metric(MetricRow::new(name.clone(), *value, *timestamp)) {
                    error!(
                        "failed to insert metric '{}' with error message '{}'",
                        name, e
                    );
                } else {
                    successful_inserts += 1;
                }
            }
        }

        info!(
            "inserted {} metrics in {:?}",
            successful_inserts,
            timer.elapsed()
        );
        Ok(())
    }

    fn parse_line(line: &str) -> Result<(String, f64, i64), String> {
        let parts: Vec<_> = line.split(";").collect();

        if parts.len() != 3 {
            return Err("wrong number of values in metric csv".into());
        }

        let name: String = parts[0].to_owned();
        let value: f64 = parts[1].parse().map_err(|e| format!("{:?}", e))?;
        let timestamp: i64 = parts[2].parse().map_err(|e| format!("{:?}", e))?;

        Ok((name, value, timestamp))
    }
}
