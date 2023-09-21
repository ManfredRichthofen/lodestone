use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, CpuRefreshKind, DiskExt, SystemExt};

use tokio::time::sleep;

use crate::AppState;

// Since MemInfo is not serializable, we need to create a new struct that is serializable.
#[derive(Serialize, Deserialize)]
pub struct MemInfo {
    total: u64,
    free: u64,
}

pub async fn get_ram(axum::extract::State(state): axum::extract::State<AppState>) -> Json<MemInfo> {
    let mut sys = state.system.lock().await;
    sys.refresh_memory();
    Json(MemInfo {
        total: sys.total_memory(),
        free: sys.available_memory(),
    })
}

// Since DiskInfo is not serializable, we need to create a new struct that is serializable.
#[derive(Serialize, Deserialize)]
pub struct DiskInfo {
    total: u64,
    free: u64,
}

pub async fn get_disk(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<DiskInfo> {
    let mut sys = state.system.lock().await;
    sys.refresh_disks_list();
    let disks = sys.disks();
    Json(DiskInfo {
        total: disks.iter().fold(0, |acc, v| acc + v.total_space()),
        free: disks.iter().fold(0, |acc, v| acc + v.available_space()),
    })
}

#[derive(Serialize, Deserialize)]
pub struct CPUInfo {
    pub cpu_speed: u64,
    pub cpu_load: f32,
}

pub async fn get_cpu_info(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<CPUInfo> {
    let mut sys = state.system.lock().await;
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    sleep(tokio::time::Duration::from_millis(100)).await;
    sys.refresh_cpu();
    Json(CPUInfo {
        cpu_speed: {
            sys.cpus().iter().fold(0, |acc, v| acc + v.frequency()) / sys.cpus().len() as u64
        },
        cpu_load: sys.cpus().iter().fold(0.0, |acc, v| acc + v.cpu_usage())
            / sys.cpus().len() as f32,
    })
}

pub fn get_system_routes(state: AppState) -> Router {
    Router::new()
        .route("/system/ram", get(get_ram))
        .route("/system/disk", get(get_disk))
        .route("/system/cpu", get(get_cpu_info))
        .with_state(state)
}
