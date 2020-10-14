use std::time::Instant;

use influx::{InfluxClient, Measurement};
use sysinfo::SystemExt;

pub struct CollectMetrics {
    opts: crate::CollectMetricsOpts,
    influx_client: InfluxClient,
}

impl CollectMetrics {
    pub fn new(opts: crate::CollectMetricsOpts) -> Self {
        let influx_client = InfluxClient::builder(
            opts.influx_url.clone(),
            opts.influx_key.clone(),
            opts.influx_org.clone(),
        )
        .build()
        .unwrap();
        Self {
            opts,
            influx_client,
        }
    }

    pub async fn collect_metrics(self) {
        let mut sys = sysinfo::System::new_all();
        loop {
            let timer = Instant::now();
            info!("publishing server metrics to influx...");
            let used_mem = UsedMemory::new(sys.get_used_memory());
            let response = self
                .influx_client
                .send_batch("mngr", &[Measurement::from(used_mem)])
                .await;

            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap();
                error!(
                    "error response from influx: '{}' with body '{}'",
                    status, body
                );
            } else {
                info!(
                    "success response from influx: '{}' after {:?}",
                    status,
                    timer.elapsed()
                );
            }
            tokio::time::delay_for(std::time::Duration::from_millis(self.opts.sleep_ms)).await;
            sys.refresh_all();
        }
    }
}

struct UsedMemory {
    kb: u64,
}

impl UsedMemory {
    fn new(kb: u64) -> Self {
        Self { kb }
    }
}

impl From<UsedMemory> for Measurement {
    fn from(v: UsedMemory) -> Self {
        Measurement::builder("used_mem".into())
            .with_field_u128("kb".into(), v.kb as u128)
            .build()
            .unwrap()
    }
}