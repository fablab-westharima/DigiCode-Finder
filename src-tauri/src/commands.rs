use crate::mdns_service;
use crate::types::DigiCodeDevice;
use crate::AppState;
use std::sync::Arc;
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
