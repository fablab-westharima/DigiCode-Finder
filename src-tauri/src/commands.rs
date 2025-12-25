use crate::mdns_service::{self, MdnsState};
use crate::types::DigiCodeDevice;
use std::sync::Arc;
use tauri::State;

/// デバイス一覧を取得
#[tauri::command]
pub async fn get_devices(state: State<'_, Arc<MdnsState>>) -> Result<Vec<DigiCodeDevice>, String> {
    Ok(mdns_service::get_devices(&state).await)
}

/// 検索を開始
#[tauri::command]
pub async fn start_search(state: State<'_, Arc<MdnsState>>) -> Result<bool, String> {
    mdns_service::refresh_search(&state).await;
    Ok(true)
}

/// ステータスを取得
#[tauri::command]
pub async fn get_status(
    state: State<'_, Arc<MdnsState>>,
) -> Result<(bool, usize), String> {
    let is_searching = mdns_service::is_searching(&state).await;
    let devices = mdns_service::get_devices(&state).await;
    Ok((is_searching, devices.len()))
}
