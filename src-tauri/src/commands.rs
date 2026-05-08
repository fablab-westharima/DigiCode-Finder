/*
 * DigiCode Finder - mDNS Device Detector
 * Copyright (C) 2024-2026 DigiCo LLC
 *
 * Licensed under the GNU Affero General Public License version 3 or later.
 * See LICENSE file in the repository root for full terms.
 */

use crate::mdns_service;
use crate::types::DigiCodeDevice;
use crate::AppState;
use log::info;
use std::sync::Arc;
use std::time::Duration;
use tauri::State;

/// デバイス一覧を取得
#[tauri::command]
pub async fn get_devices(state: State<'_, Arc<AppState>>) -> Result<Vec<DigiCodeDevice>, String> {
    Ok(mdns_service::get_devices(&state.mdns).await)
}

/// 検索を開始
#[tauri::command]
pub async fn start_search(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let app_handle = state.app_handle.read().await.clone();

    if let Some(handle) = app_handle {
        let mdns_state = state.mdns.clone();

        // 検索をバックグラウンドで実行
        tauri::async_runtime::spawn(async move {
            mdns_service::start_mdns_search(handle, mdns_state).await;
        });

        Ok(true)
    } else {
        Err("App not initialized".to_string())
    }
}

/// ステータスを取得
#[tauri::command]
pub async fn get_status(
    state: State<'_, Arc<AppState>>,
) -> Result<(bool, usize), String> {
    let is_searching = mdns_service::is_searching(&state.mdns).await;
    let devices = mdns_service::get_devices(&state.mdns).await;
    Ok((is_searching, devices.len()))
}

/// デバイスの到達性を確認（HTTP接続テスト）
async fn check_device_reachable(ip: &str, port: u16) -> bool {
    let url = format!("http://{}:{}/", ip, port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    match client.get(&url).send().await {
        Ok(response) => {
            info!("Device {} responded with status: {}", ip, response.status());
            true
        }
        Err(e) => {
            info!("Device {} unreachable: {}", ip, e);
            false
        }
    }
}

/// 全デバイスのオンライン状態を検証し、オフラインのデバイスを削除
#[tauri::command]
pub async fn verify_devices(state: State<'_, Arc<AppState>>) -> Result<usize, String> {
    let mut devices = state.mdns.devices.write().await;
    let mut offline_devices: Vec<String> = Vec::new();

    // 各デバイスの到達性を確認
    for (name, device) in devices.iter_mut() {
        if let Some(ip) = device.addresses.first() {
            let is_online = check_device_reachable(ip, device.port).await;
            device.is_online = is_online;

            if !is_online {
                offline_devices.push(name.clone());
                info!("Device {} marked as offline", name);
            }
        } else {
            // IPアドレスがない場合はオフラインとみなす
            device.is_online = false;
            offline_devices.push(name.clone());
        }
    }

    // オフラインデバイスを削除
    let removed_count = offline_devices.len();
    for name in offline_devices {
        devices.remove(&name);
    }

    info!("Verification complete: {} devices removed", removed_count);
    Ok(removed_count)
}

/// 単一デバイスの到達性を確認
#[tauri::command]
pub async fn check_device_online(ip: String, port: u16) -> Result<bool, String> {
    Ok(check_device_reachable(&ip, port).await)
}
