mod api_server;
mod commands;
mod mdns_service;
mod types;

use log::info;
use mdns_service::MdnsState;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
use tokio::sync::RwLock;

/// アプリケーション全体の状態
pub struct AppState {
    pub mdns: Arc<MdnsState>,
    pub app_handle: Arc<RwLock<Option<AppHandle>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mdns: Arc::new(MdnsState::new()),
            app_handle: Arc::new(RwLock::new(None)),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState::new());
    let state_for_api = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_devices,
            commands::start_search,
            commands::get_status,
        ])
        .setup(move |app| {
            // カスタムメニューを作成（不要な項目を削除）
            let handle = app.handle();

            // アプリメニュー (DigiCode Finder)
            let app_menu = Submenu::with_items(
                handle,
                "DigiCode Finder",
                true,
                &[
                    &PredefinedMenuItem::about(handle, Some("About DigiCode Finder"), None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::hide(handle, Some("Hide DigiCode Finder"))?,
                    &PredefinedMenuItem::hide_others(handle, Some("Hide Others"))?,
                    &PredefinedMenuItem::show_all(handle, Some("Show All"))?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::quit(handle, Some("Quit DigiCode Finder"))?,
                ],
            )?;

            // ウィンドウメニュー
            let window_menu = Submenu::with_items(
                handle,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(handle, Some("Minimize"))?,
                    &PredefinedMenuItem::maximize(handle, Some("Zoom"))?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::fullscreen(handle, Some("Toggle Full Screen"))?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::close_window(handle, Some("Close Window"))?,
                ],
            )?;

            // メニュー全体を構築
            let menu = Menu::with_items(handle, &[&app_menu, &window_menu])?;
            app.set_menu(menu)?;
            let app_handle = app.handle().clone();

            // AppHandle を保存
            let state_clone = state_for_api.clone();
            tauri::async_runtime::spawn(async move {
                *state_clone.app_handle.write().await = Some(app_handle.clone());
            });

            // HTTP API サーバーを起動（別スレッド）
            let state_api = state_for_api.mdns.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(api_server::start_api_server(state_api));
            });

            // 起動時の自動検索は行わない（ユーザーが更新ボタンを押した時のみ検索）
            info!("DigiCode Finder started (press refresh to search for devices)");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
