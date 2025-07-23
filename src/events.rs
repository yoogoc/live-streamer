use std::any::Any;

use actix::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<Uuid>,
    pub user_id: Option<String>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: None,
            user_id: None,
        }
    }
}

pub trait Event: Message<Result = ()> + Clone + Send + Any + 'static {
    fn event_type(&self) -> &'static str;
    #[allow(unused)]
    fn metadata(&self) -> &EventMetadata;
    #[allow(unused)]
    fn set_metadata(&mut self, metadata: EventMetadata);
}

#[derive(Debug, Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct UserConnectedEvent {
    pub metadata: EventMetadata,
    pub session_id: Uuid,
    pub user_id: String,
}

impl Event for UserConnectedEvent {
    fn event_type(&self) -> &'static str {
        "user_connected"
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn set_metadata(&mut self, metadata: EventMetadata) {
        self.metadata = metadata;
    }
}

#[derive(Debug, Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct UserDisconnectedEvent {
    pub metadata: EventMetadata,
    pub session_id: Uuid,
    pub user_id: String,
}

impl Event for UserDisconnectedEvent {
    fn event_type(&self) -> &'static str {
        "user_disconnected"
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn set_metadata(&mut self, metadata: EventMetadata) {
        self.metadata = metadata;
    }
}

#[derive(Debug, Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct TextInputEvent {
    pub metadata: EventMetadata,
    pub text: String,
    pub language: Option<String>,
}

impl Event for TextInputEvent {
    fn event_type(&self) -> &'static str {
        "text_input"
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn set_metadata(&mut self, metadata: EventMetadata) {
        self.metadata = metadata;
    }
}

#[derive(Debug, Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct AudioInputEvent {
    pub metadata: EventMetadata,
    pub audio_data: Vec<u8>,
    pub format: String,
    pub sample_rate: u32,
}

impl Event for AudioInputEvent {
    fn event_type(&self) -> &'static str {
        "audio_input"
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn set_metadata(&mut self, metadata: EventMetadata) {
        self.metadata = metadata;
    }
}

#[derive(Debug, Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct LLMResponseEvent {
    pub metadata: EventMetadata,
    pub response: String,
    pub model: String,
    pub tokens_used: Option<u32>,
}

impl Event for LLMResponseEvent {
    fn event_type(&self) -> &'static str {
        "llm_response"
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn set_metadata(&mut self, metadata: EventMetadata) {
        self.metadata = metadata;
    }
}

#[derive(Debug, Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct TTSResponseEvent {
    pub metadata: EventMetadata,
    pub audio_data: Vec<u8>,
    pub text: String,
    pub voice: String,
}

impl Event for TTSResponseEvent {
    fn event_type(&self) -> &'static str {
        "tts_response"
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn set_metadata(&mut self, metadata: EventMetadata) {
        self.metadata = metadata;
    }
}

#[derive(Debug, Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct AnimationEvent {
    pub metadata: EventMetadata,
    pub animation_type: String,
    pub duration: Option<f32>,
    pub parameters: serde_json::Value,
}

impl Event for AnimationEvent {
    fn event_type(&self) -> &'static str {
        "animation"
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn set_metadata(&mut self, metadata: EventMetadata) {
        self.metadata = metadata;
    }
}
