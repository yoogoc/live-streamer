use crate::event_bus::EventBus;
use crate::events::*;
use crate::platform::bilibili::BilibiliListener;
use crate::platform::douyin::DouyinListener;
use crate::platform::websocket::WebSocketListener;
use crate::platform::youtube::YouTubeListener;
use crate::platform::{
    DanmakuMessage, LiveStreamConfig, Platform, PlatformListener, ProcessDanmaku,
};
use actix::prelude::*;
use log::info;
use std::collections::HashMap;
use uuid::Uuid;

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

// Message types for LiveStreamManager
#[derive(Message)]
#[rtype(result = "()")]
pub struct AddPlatformConfig {
    pub config: LiveStreamConfig,
}

impl Handler<AddPlatformConfig> for LiveStreamManager {
    type Result = ();

    fn handle(&mut self, msg: AddPlatformConfig, _ctx: &mut Context<Self>) -> Self::Result {
        self.add_platform_config(msg.config);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RemovePlatformConfig {
    pub config_id: String,
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
