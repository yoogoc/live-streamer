use crate::platform::LiveStreamConfig;
use crate::platform::PlatformListener;
use log::info;
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
