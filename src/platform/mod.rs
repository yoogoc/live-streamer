mod bilibili;
mod douyin;
mod manager;
mod websocket;
mod youtube;

use actix::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub use {
    bilibili::BilibiliListener, douyin::DouyinListener, manager::AddPlatformConfig,
    manager::LiveStreamManager, manager::RemovePlatformConfig, websocket::WebSocketListener,
    youtube::YouTubeListener,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessDanmaku {
    pub danmaku: DanmakuMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanmakuMessage {
    pub platform: Platform,
    pub room_id: String,
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub user_level: Option<u32>,
    pub is_vip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Douyin,
    Bilibili,
    YouTube,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStreamConfig {
    pub platform: Platform,
    pub room_id: String,
    pub api_key: Option<String>,
    pub webhook_url: Option<String>,
    pub enabled: bool,
}

#[allow(unused)]
pub trait PlatformListener: Send {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn stop(&mut self);
    fn is_running(&self) -> bool;
}

impl Platform {
    pub fn to_string(&self) -> String {
        match self {
            Platform::Douyin => "douyin".to_string(),
            Platform::Bilibili => "bilibili".to_string(),
            Platform::YouTube => "youtube".to_string(),
            Platform::WebSocket => "websocket".to_string(),
        }
    }
}
