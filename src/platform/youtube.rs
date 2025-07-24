use crate::platform::LiveStreamConfig;
use crate::platform::PlatformListener;
use log::info;

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
