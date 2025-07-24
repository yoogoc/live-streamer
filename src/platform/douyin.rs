use crate::platform::LiveStreamConfig;
use crate::platform::PlatformListener;
use log::info;

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
