use crate::mdns_service::{self, MdnsState};
use crate::types::{DevicesResponse, HealthResponse, SearchRequest, SearchResponse, StatusResponse};
use axum::{
    extract::State,
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use log::{error, info};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;

/// アプリケーションバージョン
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// API サーバーのポート
const API_PORT: u16 = 31415;

/// 許可されたオリジン
const ALLOWED_ORIGINS: &[&str] = &[
    "http://localhost:5173",
    "http://localhost:5174",
    "https://app.digital-fab.jp",
    "https://digicode.pages.dev",
    "https://code.fablab-westharima.jp",
    "https://digicode-frontend.pages.dev",
];

/// HTTP API サーバーを起動
pub async fn start_api_server(state: Arc<MdnsState>) {
    // CORS 設定
    let origins: Vec<HeaderValue> = ALLOWED_ORIGINS
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    // Private Network Access 対応 (Chrome 138+)
    // プリフライトリクエストで Access-Control-Allow-Private-Network: true を返す
    let private_network_header = SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("access-control-allow-private-network"),
        HeaderValue::from_static("true"),
    );

    // ルーター設定
    let app = Router::new()
        .route("/api/devices", get(get_devices))
        .route("/api/search", post(start_search))
        .route("/api/status", get(get_status))
        .route("/health", get(health_check))
        .layer(ServiceBuilder::new().layer(private_network_header).layer(cors))
        .with_state(state);

    // サーバー起動
    let addr = format!("0.0.0.0:{}", API_PORT);
    info!("Starting HTTP API server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("HTTP server error: {}", e);
    }
}

/// GET /api/devices - デバイス一覧取得
async fn get_devices(State(state): State<Arc<MdnsState>>) -> Json<DevicesResponse> {
    let devices = mdns_service::get_devices(&state).await;
    let search_duration = mdns_service::get_search_duration(&state);

    Json(DevicesResponse {
        success: true,
        devices,
        search_duration,
        timestamp: Utc::now(),
    })
}

/// POST /api/search - 検索開始
async fn start_search(
    State(state): State<Arc<MdnsState>>,
    Json(payload): Json<Option<SearchRequest>>,
) -> Json<SearchResponse> {
    let timeout = payload.and_then(|p| p.timeout).unwrap_or(5000);

    mdns_service::refresh_search(&state).await;

    Json(SearchResponse {
        success: true,
        message: "Search started".to_string(),
        timeout,
    })
}

/// GET /api/status - ステータス確認
async fn get_status(State(state): State<Arc<MdnsState>>) -> Json<StatusResponse> {
    let is_searching = mdns_service::is_searching(&state).await;
    let devices = mdns_service::get_devices(&state).await;
    let uptime = mdns_service::get_uptime(&state);

    Json(StatusResponse {
        success: true,
        version: VERSION.to_string(),
        is_searching,
        device_count: devices.len(),
        uptime,
    })
}

/// GET /health - ヘルスチェック
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: VERSION.to_string(),
    })
}
