use crate::platform::LiveStreamConfig;
use crate::platform::PlatformListener;
use log::info;

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
