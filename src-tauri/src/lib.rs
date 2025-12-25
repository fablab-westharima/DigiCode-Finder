mod api_server;
mod commands;
mod mdns_service;
mod types;

use log::info;
use mdns_service::MdnsState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(MdnsState::new());
    let state_for_api = state.clone();
    let state_for_mdns = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_devices,
            commands::start_search,
            commands::get_status,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // HTTP API サーバーを起動（別スレッド）
            let state_api = state_for_api.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(api_server::start_api_server(state_api));
            });

            // mDNS 検索を開始（別スレッド）
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    mdns_service::start_mdns_search(app_handle, state_for_mdns).await;
                });
            });

            info!("DigiCode Helper started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
