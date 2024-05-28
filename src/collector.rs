// collector is a thread (entirely separate from axum) which gathers system data and saves it into redis

use byte_unit::{Byte, Unit};
use sqlx::PgPool;
use sysinfo::{Networks, System};
use tokio::{task, time};

use crate::config::CollectorConfig;

pub async fn start(config: CollectorConfig, db: PgPool) -> anyhow::Result<()> {
    // TODO(hito): create only with specific informations
    let mut sys = System::new_all();
    let mut networks = Networks::new_with_refreshed_list();
    time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

    let forever = task::spawn(async move {
        let mut interval = time::interval(config.interval);

        loop {
            interval.tick().await;

            // TODO(hito): refresh only specific things
            sys.refresh_all();
            networks.refresh();

            // system info
            let cpu = sys.global_cpu_info().cpu_usage();
            let memory = memory_percentage(&sys);

            // network info
            let (trans, recv) = network_usage(&networks);
            let trans_mbit = Byte::from_u64(trans).get_adjusted_unit(Unit::Mbit).get_value();
            let recv_mbit = Byte::from_u64(recv).get_adjusted_unit(Unit::Mbit).get_value();

            sqlx::query!("INSERT INTO datapoints (cpu, memory, transmitted, received) VALUES ($1, $2, $3, $4)", cpu, memory, trans_mbit, recv_mbit)
                .execute(&db)
                .await
                .unwrap();
        }
    });

    forever.await?;
    Ok(())
}

fn memory_percentage(sys: &System) -> f32 {
    let used = sys.used_memory();
    let total = sys.total_memory();

    let percentage = (used as f32 / total as f32) * 100.0;

    percentage
}

fn network_usage(net: &Networks) -> (u64, u64) {
    let mut trans = Vec::new();
    let mut recv = Vec::new();

    for (_, data) in net {
        trans.push(data.transmitted());
        recv.push(data.received());
    }

    let trans_total = trans.iter().sum();
    let recv_total = recv.iter().sum();
    (trans_total, recv_total)
}
