use crate::CleanMqttStoreDbOpts;
use chrono::Utc;
use rusqlite::params;

pub struct CleanMqttStoreDb {
    opts: CleanMqttStoreDbOpts,
    connection: rusqlite::Connection,
}

impl CleanMqttStoreDb {
    pub fn new(opts: CleanMqttStoreDbOpts) -> Self {
        let connection = rusqlite::Connection::open(&opts.db_path)
            .map_err(|e| format!("{:?}", e))
            .unwrap();
        Self { opts, connection }
    }

    pub fn clean_mqtt_store_db(&self) -> Result<(), String> {
        let current_timestamp = Utc::now().timestamp() as u32;
        let days_ago = current_timestamp - (24 * 60 * 60 * self.opts.days as u32);

        info!("deleting rows from before: {}", days_ago);

        let deleted_rows = self.delete_older_than(days_ago)?;

        info!("deleted {} rows from before: {}", deleted_rows, days_ago);

        if self.opts.vacuum {
            info!("vacuuming database...");
            let vacuumed_rows = self.vacuum_database()?;
            info!("vacuumed {} rows from the database", vacuumed_rows);
        }

        Ok(())
    }

    fn delete_older_than(&self, bucket: u32) -> Result<usize, String> {
        self.connection
            .execute("DELETE FROM message WHERE bucket < ?1", params![bucket])
            .map_err(|e| format!("error when deleting from database: {}", e))
    }

    fn vacuum_database(&self) -> Result<usize, String> {
        self.connection
            .execute("VACUUM", params![])
            .map_err(|e| format!("error when vacuuming database: {}", e))
    }
}
