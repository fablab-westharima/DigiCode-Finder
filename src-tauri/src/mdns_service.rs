use crate::types::DigiCodeDevice;
use chrono::Utc;
use log::{error, info};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

/// mDNS サービスタイプ
const SERVICE_TYPE: &str = "_digicode._tcp.local.";

/// デバイス管理用の共有状態
pub struct MdnsState {
    pub devices: Arc<RwLock<HashMap<String, DigiCodeDevice>>>,
    pub start_time: Instant,
    pub is_searching: Arc<RwLock<bool>>,
}

impl MdnsState {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
            is_searching: Arc::new(RwLock::new(false)),
        }
    }
}

impl Default for MdnsState {
    fn default() -> Self {
        Self::new()
    }
}

/// mDNS 検索を開始（ブロッキング）
pub async fn start_mdns_search(app_handle: AppHandle, state: Arc<MdnsState>) {
    let mdns = match ServiceDaemon::new() {
        Ok(daemon) => daemon,
        Err(e) => {
            error!("Failed to create mDNS daemon: {}", e);
            return;
        }
    };

    let receiver = match mdns.browse(SERVICE_TYPE) {
        Ok(recv) => recv,
        Err(e) => {
            error!("Failed to browse mDNS services: {}", e);
            return;
        }
    };

    *state.is_searching.write().await = true;
    info!("Started mDNS search for {}", SERVICE_TYPE);

    loop {
        match receiver.recv() {
            Ok(event) => {
                handle_mdns_event(event, &app_handle, &state).await;
            }
            Err(e) => {
                error!("mDNS receiver error: {}", e);
                break;
            }
        }
    }

    *state.is_searching.write().await = false;
}

/// mDNS イベントを処理
async fn handle_mdns_event(event: ServiceEvent, app_handle: &AppHandle, state: &Arc<MdnsState>) {
    match event {
        ServiceEvent::ServiceResolved(info) => {
            let name = info.get_fullname().to_string();

            // TXT レコードを変換
            let mut txt = HashMap::new();
            for prop in info.get_properties().iter() {
                let val = prop.val_str();
                txt.insert(prop.key().to_string(), val.to_string());
            }

            // IP アドレスを取得
            let addresses: Vec<String> = info.get_addresses().iter().map(|a| a.to_string()).collect();

            let device = DigiCodeDevice {
                name: name.clone(),
                host: info.get_hostname().to_string(),
                addresses,
                port: info.get_port(),
                txt,
                last_seen: Utc::now(),
            };

            info!("Device found: {} at {}", device.name, device.host);

            // デバイス一覧に追加
            state.devices.write().await.insert(name.clone(), device.clone());

            // フロントエンドに通知
            if let Err(e) = app_handle.emit("device-found", &device) {
                error!("Failed to emit device-found event: {}", e);
            }
        }
        ServiceEvent::ServiceRemoved(_, fullname) => {
            info!("Device removed: {}", fullname);

            // デバイス一覧から削除
            state.devices.write().await.remove(&fullname);

            // フロントエンドに通知
            if let Err(e) = app_handle.emit("device-removed", &fullname) {
                error!("Failed to emit device-removed event: {}", e);
            }
        }
        _ => {}
    }
}

/// 検索をリフレッシュ（デバイス一覧をクリアして再検索）
pub async fn refresh_search(state: &Arc<MdnsState>) {
    state.devices.write().await.clear();
    info!("Device list cleared for refresh");
}

/// 現在のデバイス一覧を取得
pub async fn get_devices(state: &Arc<MdnsState>) -> Vec<DigiCodeDevice> {
    state.devices.read().await.values().cloned().collect()
}

/// 検索中かどうかを取得
pub async fn is_searching(state: &Arc<MdnsState>) -> bool {
    *state.is_searching.read().await
}

/// 検索開始からの経過時間（ミリ秒）を取得
pub fn get_search_duration(state: &Arc<MdnsState>) -> u64 {
    state.start_time.elapsed().as_millis() as u64
}

/// 検索開始からの経過時間（秒）を取得
pub fn get_uptime(state: &Arc<MdnsState>) -> u64 {
    state.start_time.elapsed().as_secs()
}
