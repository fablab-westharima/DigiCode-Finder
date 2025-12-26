use crate::types::DigiCodeDevice;
use chrono::Utc;
use log::{error, info, warn};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
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

/// mDNS 検索を開始
/// macOS: dns-sd コマンドを使用（OS の mDNSResponder を利用、entitlement 不要）
/// Windows: dns-sd.exe（Bonjour Print Services インストール必要）
/// Linux: avahi-browse
///
/// 重要: この関数は block_on から直接 await される必要がある
/// tokio::spawn を使うとランタイムが即座に終了してデバイスが検出されない
pub async fn start_mdns_search(app_handle: AppHandle, state: Arc<MdnsState>) {
    info!("Starting mDNS search using system command (dns-sd)");

    *state.is_searching.write().await = true;

    // Browse でサービスを発見（直接 await - spawn しない）
    run_dns_sd_browse(app_handle, state).await;
}

/// dns-sd -B でサービスをブラウズ
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

    while let Ok(Some(line)) = reader.next_line().await {
        // dns-sd -B の出力形式:
        // Timestamp     A/R    Flags  if Domain               Service Type         Instance Name
        // 17:02:42.508  Add        2   6 local.               _digicode._tcp.      digicode-UNKO-009

        if line.contains("Add") && line.contains(SERVICE_TYPE) {
            if let Some(instance_name) = parse_browse_line(&line) {
                info!("Discovered service: {}", instance_name);

                // サービス詳細を取得
                let detail_state = state.clone();
                let detail_app = app_handle.clone();
                let name = instance_name.clone();

                tokio::spawn(async move {
                    resolve_service(detail_app, detail_state, name).await;
                });
            }
        } else if line.contains("Rmv") && line.contains(SERVICE_TYPE) {
            if let Some(instance_name) = parse_browse_line(&line) {
                info!("Service removed: {}", instance_name);
                let full_name = format!("{}._digicode._tcp.local.", instance_name);
                state.devices.write().await.remove(&full_name);

                if let Err(e) = app_handle.emit("device-removed", &full_name) {
                    error!("Failed to emit device-removed: {}", e);
                }
            }
        }
    }

    warn!("dns-sd browse process ended");
    *state.is_searching.write().await = false;
}

/// Browse 出力からインスタンス名を抽出
fn parse_browse_line(line: &str) -> Option<String> {
    // 行末のインスタンス名を取得
    // 例: "17:02:42.508  Add        2   6 local.               _digicode._tcp.      digicode-UNKO-009"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 7 {
        // 最後の部分がインスタンス名
        Some(parts.last()?.to_string())
    } else {
        None
    }
}

/// dns-sd -L でサービスの詳細を取得
/// dns-sd -L は終了しないコマンドなので、必要な情報を取得したらプロセスを終了させる
async fn resolve_service(app_handle: AppHandle, state: Arc<MdnsState>, instance_name: String) {
    #[cfg(target_os = "macos")]
    let cmd = "dns-sd";
    #[cfg(target_os = "windows")]
    let cmd = "dns-sd.exe";
    #[cfg(target_os = "linux")]
    let cmd = "avahi-resolve";

    // dns-sd -L <instance> <type> <domain>
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let args = vec!["-L", &instance_name, SERVICE_TYPE, "local."];
    #[cfg(target_os = "linux")]
    let args = vec!["-n", &format!("{}.{}.local", instance_name, SERVICE_TYPE)];

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

    // タイムアウト付きで読み取る（最大5秒）
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while let Ok(Some(line)) = reader.next_line().await {
            info!("Resolve line: {}", line);
            output_lines.push(line.clone());

            // "can be reached at" を含む行が来たら、次の行（TXTレコード）も取得
            if line.contains("can be reached at") {
                got_record = true;
            } else if got_record && line.starts_with(' ') {
                // TXTレコードを取得したら終了
                break;
            }
        }
    });

    let _ = timeout.await; // タイムアウトしても続行

    // プロセスを終了
    let _ = child.kill().await;

    let output = output_lines.join("\n");
    info!("Resolve output for {}: {}", instance_name, output);

    // dns-sd -L の出力をパース
    // 例:
    // Lookup digicode-UNKO-009._digicode._tcp.local.
    // DATE: ---Fri 26 Dec 2025---
    // 17:30:00.000  digicode-UNKO-009._digicode._tcp.local. can be reached at digicode-UNKO-009.local.:80 (interface 6)
    //  version=1.8.0 name=UNKO-009 uuid=mnAP9bwi

    if let Some(device) = parse_resolve_output(&instance_name, &output) {
        let full_name = device.name.clone();

        state.devices.write().await.insert(full_name.clone(), device.clone());

        if let Err(e) = app_handle.emit("device-found", &device) {
            error!("Failed to emit device-found: {}", e);
        }

        info!("Device resolved: {} at {}:{}", device.name, device.addresses.first().unwrap_or(&"unknown".to_string()), device.port);
    }

    // IP アドレスも解決
    resolve_ip(app_handle, state, instance_name).await;
}

/// dns-sd -G でIPアドレスを取得
async fn resolve_ip(app_handle: AppHandle, state: Arc<MdnsState>, instance_name: String) {
    let hostname = format!("{}.local.", instance_name.replace("digicode-", "digicode-"));

    #[cfg(target_os = "macos")]
    let cmd = "dns-sd";
    #[cfg(target_os = "windows")]
    let cmd = "dns-sd.exe";

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        // dns-sd -G v4 <hostname> でIPv4アドレスを取得
        // タイムアウト付きで実行
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

        // IPアドレスを抽出して更新
        if let Some(ip) = extract_ip_from_output(&stdout) {
            let full_name = format!("{}._digicode._tcp.local.", instance_name);

            if let Some(device) = state.devices.write().await.get_mut(&full_name) {
                if !device.addresses.contains(&ip) {
                    device.addresses.push(ip.clone());
                    info!("Updated IP for {}: {}", instance_name, ip);

                    if let Err(e) = app_handle.emit("device-found", &device.clone()) {
                        error!("Failed to emit device update: {}", e);
                    }
                }
            }
        }
    }
}

/// dns-sd -L 出力をパース
fn parse_resolve_output(instance_name: &str, output: &str) -> Option<DigiCodeDevice> {
    let mut host = String::new();
    let mut port: u16 = 80;
    let mut txt: HashMap<String, String> = HashMap::new();
    let addresses: Vec<String> = Vec::new();

    for line in output.lines() {
        // "can be reached at" の行からホストとポートを取得
        if line.contains("can be reached at") {
            // digicode-UNKO-009.local.:80
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

        // TXT レコードの行（スペースで始まる）
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
    })
}

/// IPアドレスを出力から抽出
fn extract_ip_from_output(output: &str) -> Option<String> {
    // dns-sd -G v4 の出力から IPv4 アドレスを抽出
    for line in output.lines() {
        // 例: "digicode-UNKO-009.local. 192.168.50.100"
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            // IPv4 形式をチェック
            if part.split('.').count() == 4 {
                if part.split('.').all(|s| s.parse::<u8>().is_ok()) {
                    return Some(part.to_string());
                }
            }
        }
    }
    None
}

/// 検索をリフレッシュ
pub async fn refresh_search(state: &Arc<MdnsState>) {
    state.devices.write().await.clear();
    info!("Device list cleared for refresh");
}

/// 現在のデバイス一覧を取得
pub async fn get_devices(state: &Arc<MdnsState>) -> Vec<DigiCodeDevice> {
    state.devices.read().await.values().cloned().collect()
}

/// 検索中かどうか
pub async fn is_searching(state: &Arc<MdnsState>) -> bool {
    *state.is_searching.read().await
}

/// 検索開始からの経過時間（ミリ秒）
pub fn get_search_duration(state: &Arc<MdnsState>) -> u64 {
    state.start_time.elapsed().as_millis() as u64
}

/// 起動からの経過時間（秒）
pub fn get_uptime(state: &Arc<MdnsState>) -> u64 {
    state.start_time.elapsed().as_secs()
}
