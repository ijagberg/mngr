use rusqlite::params;
use std::marker::PhantomData;

pub struct Db<T> {
    path: String,
    _phantom: PhantomData<T>,
}

impl Db<MetricRow> {
    pub fn new(path: String) -> Self {
        Self {
            path,
            _phantom: PhantomData,
        }
    }

    fn get_connection(&self) -> Result<rusqlite::Connection, String> {
        rusqlite::Connection::open(&self.path).map_err(|e| format!("{:?}", e))
    }

    pub fn insert_metric(&self, metric: MetricRow) -> Result<(), String> {
        let db = self.get_connection()?;

        db.execute(
            "
            INSERT INTO metric (name, value, timestamp) 
            VALUES (?1, ?2, ?3)",
            params![metric.name, metric.value, metric.timestamp],
        )
        .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }
}

pub struct MetricRow {
    name: String,
    value: f64,
    timestamp: i64,
}

impl MetricRow {
    pub fn new(name: String, value: f64, timestamp: i64) -> Self {
        Self {
            name,
            value,
            timestamp,
        }
    }
}
