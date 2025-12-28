use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DigiCode デバイス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigiCodeDevice {
    /// mDNS サービス名
    pub name: String,
    /// ホスト名 (xxx.local)
    pub host: String,
    /// IP アドレス配列
    pub addresses: Vec<String>,
    /// サービスポート
    pub port: u16,
    /// TXT レコード
    pub txt: HashMap<String, String>,
    /// 最終検出時刻
    #[serde(rename = "lastSeen")]
    pub last_seen: DateTime<Utc>,
    /// オンライン状態（到達性確認済み）
    #[serde(rename = "isOnline", default = "default_online")]
    pub is_online: bool,
}

fn default_online() -> bool {
    true // デフォルトはオンライン（発見直後）
}

/// API レスポンス: デバイス一覧
#[derive(Debug, Serialize)]
pub struct DevicesResponse {
    pub success: bool,
    pub devices: Vec<DigiCodeDevice>,
    #[serde(rename = "searchDuration")]
    pub search_duration: u64,
    pub timestamp: DateTime<Utc>,
}

/// API レスポンス: 検索開始
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub success: bool,
    pub message: String,
    pub timeout: u64,
}

/// API レスポンス: ステータス
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub success: bool,
    pub version: String,
    #[serde(rename = "isSearching")]
    pub is_searching: bool,
    #[serde(rename = "deviceCount")]
    pub device_count: usize,
    pub uptime: u64,
}

/// API レスポンス: ヘルスチェック
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// API リクエスト: 検索開始
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub timeout: Option<u64>,
}
