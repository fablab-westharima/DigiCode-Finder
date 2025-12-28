use crate::types::DigiCodeDevice;
use chrono::Utc;
use log::{error, info, warn};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;

/// mDNS サービスタイプ
const SERVICE_TYPE: &str = "_digicode._tcp";

/// 検索タイムアウト（秒）
/// ESP32のHTTPサーバーが3-20秒かかる可能性があるため、余裕を持たせる
const SEARCH_TIMEOUT_SECS: u64 = 15;

/// デバイスの到達性を確認（HTTP接続テスト）
async fn check_device_reachable(ip: &str, port: u16) -> bool {
    let url = format!("http://{}:{}/", ip, port);
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(&url).send().await {
        Ok(_) => {
            info!("Device {}:{} is reachable", ip, port);
            true
        }
        Err(e) => {
            info!("Device {}:{} is NOT reachable: {}", ip, port, e);
            false
        }
    }
}

/// デバイス管理用の共有状態
pub struct MdnsState {
    pub devices: Arc<RwLock<HashMap<String, DigiCodeDevice>>>,
    pub start_time: Instant,
    pub is_searching: Arc<RwLock<bool>>,
    pub browse_process: Arc<RwLock<Option<Child>>>,
}

impl MdnsState {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
            is_searching: Arc::new(RwLock::new(false)),
            browse_process: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for MdnsState {
    fn default() -> Self {
        Self::new()
    }
}

/// mDNS 検索を開始（タイムアウト付き）
pub async fn start_mdns_search(app_handle: AppHandle, state: Arc<MdnsState>) {
    // 既に検索中なら何もしない
    if *state.is_searching.read().await {
        info!("Search already in progress, skipping");
        return;
    }

    info!("Starting mDNS search (timeout: {}s)", SEARCH_TIMEOUT_SECS);

    // デバイスリストをクリア
    state.devices.write().await.clear();
    *state.is_searching.write().await = true;

    // Browse でサービスを発見
    run_dns_sd_browse(app_handle, state).await;
}

/// dns-sd -B でサービスをブラウズ（タイムアウト付き）
async fn run_dns_sd_browse(app_handle: AppHandle, state: Arc<MdnsState>) {
    #[cfg(target_os = "macos")]
    let cmd = "dns-sd";
    #[cfg(target_os = "windows")]
    let cmd = "dns-sd.exe";
    #[cfg(target_os = "linux")]
    let cmd = "avahi-browse";

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let args = vec!["-B", SERVICE_TYPE, "local."];
    #[cfg(target_os = "linux")]
    let args = vec!["-r", "-p", SERVICE_TYPE];

    info!("Running: {} {:?}", cmd, args);

    let mut child = match Command::new(cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to spawn dns-sd command: {}", e);
            *state.is_searching.write().await = false;
            return;
        }
    };

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let mut reader = BufReader::new(stdout).lines();

    // プロセスを保存
    *state.browse_process.write().await = Some(child);

    let state_clone = state.clone();
    let app_clone = app_handle.clone();

    // タイムアウト付きで読み取る
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(SEARCH_TIMEOUT_SECS),
        async {
            while let Ok(Some(line)) = reader.next_line().await {
                if line.contains("Add") && line.contains(SERVICE_TYPE) {
                    if let Some(instance_name) = parse_browse_line(&line) {
                        info!("Discovered service: {}", instance_name);

                        let detail_state = state_clone.clone();
                        let detail_app = app_clone.clone();
                        let name = instance_name.clone();

                        tokio::spawn(async move {
                            resolve_service(detail_app, detail_state, name).await;
                        });
                    }
                } else if line.contains("Rmv") && line.contains(SERVICE_TYPE) {
                    if let Some(instance_name) = parse_browse_line(&line) {
                        info!("Service removed: {}", instance_name);
                        let full_name = format!("{}._digicode._tcp.local.", instance_name);
                        state_clone.devices.write().await.remove(&full_name);

                        if let Err(e) = app_clone.emit("device-removed", &full_name) {
                            error!("Failed to emit device-removed: {}", e);
                        }
                    }
                }
            }
        },
    )
    .await;

    // タイムアウトまたは終了
    if result.is_err() {
        info!("Search timeout reached ({}s)", SEARCH_TIMEOUT_SECS);
    }

    // プロセスを終了
    stop_browse_process(&state).await;

    *state.is_searching.write().await = false;
    info!("Search completed");
}

/// ブラウズプロセスを停止
async fn stop_browse_process(state: &Arc<MdnsState>) {
    if let Some(mut child) = state.browse_process.write().await.take() {
        let _ = child.kill().await;
        info!("Browse process stopped");
    }
}

/// Browse 出力からインスタンス名を抽出
fn parse_browse_line(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 7 {
        Some(parts.last()?.to_string())
    } else {
        None
    }
}

/// dns-sd -L でサービスの詳細を取得
async fn resolve_service(app_handle: AppHandle, state: Arc<MdnsState>, instance_name: String) {
    #[cfg(target_os = "macos")]
    let cmd = "dns-sd";
    #[cfg(target_os = "windows")]
    let cmd = "dns-sd.exe";
    #[cfg(target_os = "linux")]
    let cmd = "avahi-resolve";

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let args = vec!["-L", &instance_name, SERVICE_TYPE, "local."];
    #[cfg(target_os = "linux")]
    let linux_arg = format!("{}.{}.local", instance_name, SERVICE_TYPE);
    #[cfg(target_os = "linux")]
    let args = vec!["-n", linux_arg.as_str()];

    info!("Resolving: {} {:?}", cmd, args);

    let mut child = match Command::new(cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to spawn resolve command for {}: {}", instance_name, e);
            return;
        }
    };

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let mut reader = BufReader::new(stdout).lines();
    let mut output_lines = Vec::new();
    let mut got_record = false;

    let timeout = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while let Ok(Some(line)) = reader.next_line().await {
            info!("Resolve line: {}", line);
            output_lines.push(line.clone());

            if line.contains("can be reached at") {
                got_record = true;
            } else if got_record && line.starts_with(' ') {
                break;
            }
        }
    });

    let _ = timeout.await;
    let _ = child.kill().await;

    let output = output_lines.join("\n");
    info!("Resolve output for {}: {}", instance_name, output);

    if let Some(device) = parse_resolve_output(&instance_name, &output) {
        let full_name = device.name.clone();

        // デバイス情報を一時保存（device-foundはIP解決とヘルスチェック後に送信）
        state.devices.write().await.insert(full_name.clone(), device.clone());
        info!("Device info stored: {} (pending verification)", full_name);
    }

    // IP解決とヘルスチェックを実行（ここでdevice-foundまたはdevice-removedを送信）
    resolve_ip(app_handle, state, instance_name).await;
}

#[cfg(target_os = "linux")]
async fn resolve_ip(_app_handle: AppHandle, _state: Arc<MdnsState>, _instance_name: String) {
    info!("IP resolution skipped on Linux");
}

#[cfg(not(target_os = "linux"))]
async fn resolve_ip(app_handle: AppHandle, state: Arc<MdnsState>, instance_name: String) {
    let hostname = format!("{}.local.", instance_name.replace("digicode-", "digicode-"));

    #[cfg(target_os = "macos")]
    let cmd = "dns-sd";
    #[cfg(target_os = "windows")]
    let cmd = "dns-sd.exe";

    let args = vec!["-t", "2", "-G", "v4", &hostname];

    let output = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        Command::new(cmd).args(&args).output()
    ).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            warn!("Failed to resolve IP for {}: {}", hostname, e);
            return;
        }
        Err(_) => {
            warn!("Timeout resolving IP for {}", hostname);
            return;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let full_name = format!("{}._digicode._tcp.local.", instance_name);

    if let Some(ip) = extract_ip_from_output(&stdout) {
        // デバイスの到達性をチェック
        let port = {
            let devices = state.devices.read().await;
            devices.get(&full_name).map(|d| d.port).unwrap_or(80)
        };

        let is_reachable = check_device_reachable(&ip, port).await;

        if is_reachable {
            // 到達可能: デバイスを更新
            let mut devices = state.devices.write().await;
            if let Some(device) = devices.get_mut(&full_name) {
                if !device.addresses.contains(&ip) {
                    device.addresses.push(ip.clone());
                }
                device.is_online = true;
                info!("Device {} is online at {}", instance_name, ip);

                if let Err(e) = app_handle.emit("device-found", &device.clone()) {
                    error!("Failed to emit device update: {}", e);
                }
            }
        } else {
            // 到達不能: デバイスを削除
            info!("Device {} is offline (cached mDNS entry), removing", instance_name);
            state.devices.write().await.remove(&full_name);

            if let Err(e) = app_handle.emit("device-removed", &full_name) {
                error!("Failed to emit device-removed: {}", e);
            }
        }
    } else {
        // IPが解決できない場合もデバイスを削除
        info!("Could not resolve IP for {}, removing", instance_name);
        state.devices.write().await.remove(&full_name);
    }
}

fn parse_resolve_output(instance_name: &str, output: &str) -> Option<DigiCodeDevice> {
    let mut host = String::new();
    let mut port: u16 = 80;
    let mut txt: HashMap<String, String> = HashMap::new();
    let addresses: Vec<String> = Vec::new();

    for line in output.lines() {
        if line.contains("can be reached at") {
            if let Some(at_pos) = line.find("can be reached at") {
                let after = &line[at_pos + 17..];
                let parts: Vec<&str> = after.trim().split(':').collect();
                if parts.len() >= 2 {
                    host = parts[0].trim().to_string();
                    port = parts[1].split_whitespace().next()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(80);
                }
            }
        }

        if line.starts_with(' ') && line.contains('=') {
            for pair in line.trim().split_whitespace() {
                if let Some((k, v)) = pair.split_once('=') {
                    txt.insert(k.to_string(), v.to_string());
                }
            }
        }
    }

    if host.is_empty() {
        return None;
    }

    Some(DigiCodeDevice {
        name: format!("{}._digicode._tcp.local.", instance_name),
        host,
        addresses,
        port,
        txt,
        last_seen: Utc::now(),
        is_online: true, // 発見直後はオンラインとみなす
    })
}

fn extract_ip_from_output(output: &str) -> Option<String> {
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            if part.split('.').count() == 4 {
                if part.split('.').all(|s| s.parse::<u8>().is_ok()) {
                    return Some(part.to_string());
                }
            }
        }
    }
    None
}

/// 現在のデバイス一覧を取得
pub async fn get_devices(state: &Arc<MdnsState>) -> Vec<DigiCodeDevice> {
    state.devices.read().await.values().cloned().collect()
}

/// 検索中かどうか
pub async fn is_searching(state: &Arc<MdnsState>) -> bool {
    *state.is_searching.read().await
}

/// 検索をリフレッシュ（HTTP API用 - デバイスリストをクリアするのみ）
pub async fn refresh_search(state: &Arc<MdnsState>) {
    state.devices.write().await.clear();
    info!("Device list cleared for refresh");
}

/// 検索開始からの経過時間（ミリ秒）
pub fn get_search_duration(state: &Arc<MdnsState>) -> u64 {
    state.start_time.elapsed().as_millis() as u64
}

/// 起動からの経過時間（秒）
pub fn get_uptime(state: &Arc<MdnsState>) -> u64 {
    state.start_time.elapsed().as_secs()
}
