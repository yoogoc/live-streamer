use crate::event_bus::EventBus;
use crate::events::*;
use actix::prelude::*;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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

pub struct LiveStreamManager {
    configs: HashMap<String, LiveStreamConfig>,
    event_bus: Addr<EventBus>,
    active_listeners: HashMap<String, Box<dyn PlatformListener>>,
}

impl LiveStreamManager {
    pub fn new(event_bus: Addr<EventBus>) -> Self {
        Self {
            configs: HashMap::new(),
            event_bus,
            active_listeners: HashMap::new(),
        }
    }

    pub fn add_platform_config(&mut self, config: LiveStreamConfig) {
        let config_id = format!("{:?}_{}", config.platform, config.room_id);
        info!("Adding platform config: {}", config_id);

        if config.enabled {
            self.start_listener(&config_id, &config);
        }

        self.configs.insert(config_id, config);
    }

    pub fn remove_platform_config(&mut self, config_id: &str) {
        if let Some(_config) = self.configs.remove(config_id) {
            self.stop_listener(config_id);
            info!("Removed platform config: {}", config_id);
        }
    }

    fn start_listener(&mut self, config_id: &str, config: &LiveStreamConfig) {
        info!("Starting listener for: {}", config_id);

        let listener: Box<dyn PlatformListener> = match config.platform {
            Platform::Douyin => Box::new(DouyinListener::new(config.clone())),
            Platform::Bilibili => Box::new(BilibiliListener::new(config.clone())),
            Platform::YouTube => Box::new(YouTubeListener::new(config.clone())),
            Platform::WebSocket => Box::new(WebSocketListener::new(config.clone())),
        };

        self.active_listeners
            .insert(config_id.to_string(), listener);
    }

    fn stop_listener(&mut self, config_id: &str) {
        if let Some(_listener) = self.active_listeners.remove(config_id) {
            info!("Stopped listener for: {}", config_id);
        }
    }

    pub fn process_danmaku(&mut self, danmaku: DanmakuMessage) {
        info!(
            "Processing danmaku from {:?}: {}",
            danmaku.platform, danmaku.message
        );

        let text_event = TextInputEvent {
            metadata: EventMetadata {
                session_id: Some(Uuid::new_v4()),
                user_id: Some(format!(
                    "{}_{}",
                    danmaku.platform.to_string(),
                    danmaku.user_id
                )),
                ..Default::default()
            },
            text: danmaku.message,
            language: Some("zh-CN".to_string()),
        };

        self.event_bus.do_send(text_event);
    }
}

impl Actor for LiveStreamManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("LiveStreamManager started");
    }
}

#[allow(unused)]
pub trait PlatformListener: Send {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn stop(&mut self);
    fn is_running(&self) -> bool;
}

pub struct DouyinListener {
    config: LiveStreamConfig,
    running: bool,
}

impl DouyinListener {
    pub fn new(config: LiveStreamConfig) -> Self {
        Self {
            config,
            running: false,
        }
    }
}

impl PlatformListener for DouyinListener {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Douyin listener for room: {}", self.config.room_id);
        self.running = true;

        // TODO: 实现抖音弹幕监听
        // 这里需要连接到抖音的弹幕API或使用第三方服务

        Ok(())
    }

    fn stop(&mut self) {
        info!("Stopping Douyin listener");
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

pub struct BilibiliListener {
    config: LiveStreamConfig,
    running: bool,
}

impl BilibiliListener {
    pub fn new(config: LiveStreamConfig) -> Self {
        Self {
            config,
            running: false,
        }
    }
}

impl PlatformListener for BilibiliListener {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Starting Bilibili listener for room: {}",
            self.config.room_id
        );
        self.running = true;

        // TODO: 实现B站弹幕监听
        // 可以使用bilibili-live-danmaku crate或WebSocket连接

        Ok(())
    }

    fn stop(&mut self) {
        info!("Stopping Bilibili listener");
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

pub struct YouTubeListener {
    config: LiveStreamConfig,
    running: bool,
}

impl YouTubeListener {
    pub fn new(config: LiveStreamConfig) -> Self {
        Self {
            config,
            running: false,
        }
    }
}

impl PlatformListener for YouTubeListener {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Starting YouTube listener for stream: {}",
            self.config.room_id
        );
        self.running = true;

        // TODO: 实现YouTube直播聊天监听
        // 需要使用YouTube Live Streaming API

        Ok(())
    }

    fn stop(&mut self) {
        info!("Stopping YouTube listener");
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

pub struct WebSocketListener {
    config: LiveStreamConfig,
    running: bool,
}

impl WebSocketListener {
    pub fn new(config: LiveStreamConfig) -> Self {
        Self {
            config,
            running: false,
        }
    }
}

impl PlatformListener for WebSocketListener {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Starting WebSocket listener on: {}",
            self.config
                .webhook_url
                .as_ref()
                .unwrap_or(&"N/A".to_string())
        );
        self.running = true;

        // TODO: 实现WebSocket弹幕监听
        // 这已经通过现有的WebSocketManager实现了

        Ok(())
    }

    fn stop(&mut self) {
        info!("Stopping WebSocket listener");
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

// Message types for LiveStreamManager
#[derive(Message)]
#[rtype(result = "()")]
pub struct AddPlatformConfig {
    pub config: LiveStreamConfig,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RemovePlatformConfig {
    pub config_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessDanmaku {
    pub danmaku: DanmakuMessage,
}

impl Handler<AddPlatformConfig> for LiveStreamManager {
    type Result = ();

    fn handle(&mut self, msg: AddPlatformConfig, _ctx: &mut Context<Self>) -> Self::Result {
        self.add_platform_config(msg.config);
    }
}

impl Handler<RemovePlatformConfig> for LiveStreamManager {
    type Result = ();

    fn handle(&mut self, msg: RemovePlatformConfig, _ctx: &mut Context<Self>) -> Self::Result {
        self.remove_platform_config(&msg.config_id);
    }
}

impl Handler<ProcessDanmaku> for LiveStreamManager {
    type Result = ();

    fn handle(&mut self, msg: ProcessDanmaku, _ctx: &mut Context<Self>) -> Self::Result {
        self.process_danmaku(msg.danmaku);
    }
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

// 示例：抖音弹幕服务集成
#[allow(unused)]
pub struct DouyinDanmakuService {
    live_stream_manager: Addr<LiveStreamManager>,
}

#[allow(unused)]
impl DouyinDanmakuService {
    pub fn new(live_stream_manager: Addr<LiveStreamManager>) -> Self {
        Self {
            live_stream_manager,
        }
    }

    // 模拟接收抖音弹幕的HTTP回调
    pub async fn handle_danmaku_webhook(
        &self,
        danmaku_data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(danmaku) = self.parse_douyin_danmaku(danmaku_data) {
            self.live_stream_manager.do_send(ProcessDanmaku { danmaku });
        }
        Ok(())
    }

    fn parse_douyin_danmaku(
        &self,
        data: serde_json::Value,
    ) -> Result<DanmakuMessage, Box<dyn std::error::Error>> {
        let message = data
            .get("message")
            .and_then(|m| m.as_str())
            .ok_or("Missing message field")?;

        let user_id = data
            .get("user_id")
            .and_then(|u| u.as_str())
            .unwrap_or("anonymous");

        let username = data
            .get("username")
            .and_then(|u| u.as_str())
            .unwrap_or("用户");

        let room_id = data
            .get("room_id")
            .and_then(|r| r.as_str())
            .unwrap_or("unknown");

        Ok(DanmakuMessage {
            platform: Platform::Douyin,
            room_id: room_id.to_string(),
            user_id: user_id.to_string(),
            username: username.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            user_level: data
                .get("user_level")
                .and_then(|l| l.as_u64())
                .map(|l| l as u32),
            is_vip: data
                .get("is_vip")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }
}
