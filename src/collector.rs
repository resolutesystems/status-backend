// collector is a thread (entirely separate from axum) which gathers system data and saves it into redis
// (code needs to be cleaned A LOT)

use byte_unit::{Byte, Unit};
use chrono::Utc;
use redis::AsyncCommands;
use sysinfo::{Networks, System};
use tokio::{task, time};

use crate::{config::CollectorConfig, models::{AllDataPoints, DataSet}};

pub async fn start(config: CollectorConfig, redis: redis::Client) -> anyhow::Result<()> {
    // TODO(hito): create only with specific informations
    let mut sys = System::new_all();
    let mut networks = Networks::new_with_refreshed_list();
    time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

    let forever = task::spawn(async move {
        let mut con = redis.get_multiplexed_async_connection().await.unwrap();
        let mut interval = time::interval(config.interval);

        loop {
            interval.tick().await;

            // TODO(hito): refresh only specific things
            sys.refresh_all();
            networks.refresh();

            // fetch current data from redis
            let all_dps: Option<String> = con.get(&config.redis_key).await.unwrap();
            let mut all_dps = if let Some(all_dps) = all_dps {
                let all_dps: AllDataPoints = serde_json::from_str(&all_dps).unwrap();
                all_dps
            } else {
                AllDataPoints::default()
            };

            // cpu
            all_dps.cpu.timestamps.push(Utc::now());
            let mut cpu_datas = match all_dps.cpu.datasets.first() {
                Some(datas) => datas.clone(),
                None => {
                    let datas = DataSet { label: String::from("Usage (%)"), data: vec![] };
                    datas
                }
            };
            cpu_datas.data.push(sys.global_cpu_info().cpu_usage());
            all_dps.cpu.datasets = vec![cpu_datas];

            // memory
            all_dps.memory.timestamps.push(Utc::now());
            let mut mem_datas = match all_dps.memory.datasets.first() {
                Some(datas) => datas.clone(),
                None => {
                    let datas = DataSet { label: String::from("Usage (%)"), data: vec![] };
                    datas
                }
            };
            mem_datas.data.push(memory_percentage(&sys));
            all_dps.memory.datasets = vec![mem_datas];

            // network info
            let (trans, recv) = network_usage(&networks);
            all_dps.network.timestamps.push(Utc::now());

            let mut nin_datas = match all_dps.network.datasets.get(0) {
                Some(datas) => datas.clone(),
                None => {
                    let datas = DataSet { label: String::from("Received (mb/s)"), data: vec![] };
                    datas
                }
            };
            let recv_mb = Byte::from_u64(recv);
            nin_datas.data.push(recv_mb.get_adjusted_unit(Unit::Mbit).get_value() as f32);

            let mut nout_datas = match all_dps.network.datasets.get(1) {
                Some(datas) => datas.clone(),
                None => {
                    let datas = DataSet { label: String::from("Transmitted (mb/s)"), data: vec![] };
                    datas
                }
            };
            let trans_mb = Byte::from_u64(trans);
            nout_datas.data.push(trans_mb.get_adjusted_unit(Unit::Mbit).get_value() as f32);

            all_dps.network.datasets = vec![nin_datas, nout_datas];


            // saving data into redis (ultra efficient performance x3000 code, written at 4am)
            if all_dps.cpu.timestamps.len() > config.records {
                all_dps.cpu.timestamps = all_dps.cpu.timestamps.split_off(all_dps.cpu.timestamps.len() - config.records);
            }
            for datas in all_dps.cpu.datasets.iter_mut() {
                if datas.data.len() > config.records {
                    datas.data = datas.data.split_off(datas.data.len() - config.records);
                }
            }

            if all_dps.memory.timestamps.len() > config.records {
                all_dps.memory.timestamps = all_dps.memory.timestamps.split_off(all_dps.memory.timestamps.len() - config.records);
            }
            for datas in all_dps.memory.datasets.iter_mut() {
                if datas.data.len() > config.records {
                    datas.data = datas.data.split_off(datas.data.len() - config.records);
                }
            }

            if all_dps.network.timestamps.len() > config.records {
                all_dps.network.timestamps = all_dps.network.timestamps.split_off(all_dps.network.timestamps.len() - config.records);
            }
            for datas in all_dps.network.datasets.iter_mut() {
                if datas.data.len() > config.records {
                    datas.data = datas.data.split_off(datas.data.len() - config.records);
                }
            }

            let _: () = con.set(&config.redis_key, serde_json::to_string(&all_dps).unwrap()).await.unwrap();
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
