use crate::types::DigiCodeDevice;
use astro_dnssd::{ServiceBrowserBuilder, ServiceEventType};
use chrono::Utc;
use dns_lookup::lookup_host;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

/// mDNS サービスタイプ
const SERVICE_TYPE: &str = "_digicode._tcp";

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
/// astro-dnssd はネイティブAPIを使用するため、entitlement不要
pub async fn start_mdns_search(app_handle: AppHandle, state: Arc<MdnsState>) {
    info!("Starting mDNS search for {} using native API (astro-dnssd)", SERVICE_TYPE);

    *state.is_searching.write().await = true;

    // ブラウザを作成
    let browser = match ServiceBrowserBuilder::new(SERVICE_TYPE)
        .with_domain("local")
        .browse()
    {
        Ok(browser) => browser,
        Err(e) => {
            error!("Failed to create mDNS browser: {:?}", e);
            *state.is_searching.write().await = false;
            return;
        }
    };

    info!("mDNS browser started successfully");

    // ポーリングでサービスを受信
    loop {
        match browser.recv_timeout(Duration::from_millis(500)) {
            Ok(service) => {
                // Added イベントのみ処理
                if !matches!(service.event_type, ServiceEventType::Added) {
                    // Removed の場合はスキップまたは削除処理
                    if matches!(service.event_type, ServiceEventType::Removed) {
                        let name = format!("{}._digicode._tcp.local.", service.name);
                        info!("Device removed: {}", name);
                        state.devices.write().await.remove(&name);
                        if let Err(e) = app_handle.emit("device-removed", &name) {
                            error!("Failed to emit device-removed event: {}", e);
                        }
                    }
                    continue;
                }

                // サービス発見時の処理
                let name = format!("{}._digicode._tcp.local.", service.name);

                // TXT レコードを取得（astro-dnssd は HashMap<String, String> で提供）
                let txt = service.txt_record.clone().unwrap_or_default();

                // ホスト名からIPアドレスを解決する
                let hostname = &service.hostname;
                let addresses: Vec<String> = match lookup_host(hostname) {
                    Ok(ips) => ips
                        .into_iter()
                        .filter(|ip| ip.is_ipv4()) // IPv4のみ使用
                        .map(|ip| ip.to_string())
                        .collect(),
                    Err(e) => {
                        warn!("Failed to resolve hostname {}: {:?}", hostname, e);
                        vec![]
                    }
                };

                let device = DigiCodeDevice {
                    name: name.clone(),
                    host: service.hostname.clone(),
                    addresses,
                    port: service.port,
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
            Err(e) => {
                // タイムアウトは正常（サービスが見つからなかっただけ）
                // エラーログは出さない
                let err_str = format!("{:?}", e);
                if !err_str.contains("Timeout") && !err_str.contains("timeout") {
                    warn!("mDNS browse error: {:?}", e);
                }
            }
        }

        // 短いスリープを入れてCPU負荷を下げる
        tokio::time::sleep(Duration::from_millis(100)).await;
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
