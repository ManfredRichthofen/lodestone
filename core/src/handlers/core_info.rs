use std::env;

use crate::{prelude::VERSION, AppState};
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

#[derive(Serialize, Deserialize)]
pub struct CoreInfo {
    version: semver::Version,
    is_setup: bool,
    os: String,
    arch: String,
    cpu: String,
    cpu_count: u32,
    total_ram: u64,
    total_disk: u64,
    host_name: String,
    uuid: String,
    core_name: String,
    up_since: i64,
}

pub async fn get_core_info(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<CoreInfo> {
    let sys = System::new_all();
    Json(CoreInfo {
        version: VERSION.with(|v| v.clone()),
        is_setup: state.first_time_setup_key.lock().await.is_none(),
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        cpu: {
            let cpu_str = sys
                .cpus()
                .first()
                .map_or_else(|| "Unknown CPU", |v| v.brand());
            if cpu_str.is_empty() {
                "Unknown CPU".to_string()
            } else {
                cpu_str.to_string()
            }
        },
        cpu_count: sys.cpus().len() as u32,
        host_name: sys
            .host_name()
            .unwrap_or_else(|| "Unknown Hostname".to_string()),
        total_ram: sys.total_memory(),
        total_disk: sys.disks().iter().fold(0, |acc, v| acc + v.total_space()),
        core_name: state.global_settings.lock().await.core_name(),
        uuid: state.uuid.clone(),
        up_since: state.up_since,
    })
}

pub fn get_core_info_routes(state: AppState) -> Router {
    Router::new()
        .route("/info", get(get_core_info))
        .with_state(state)
}
