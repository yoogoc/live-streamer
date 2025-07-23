use crate::event_bus::EventBus;
use crate::events::*;
use actix::prelude::*;
use log::info;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct DigitalHumanActor {
    pub id: Uuid,
    pub name: String,
    #[allow(unused)]
    pub personality: String,
    pub sessions: HashMap<Uuid, SessionData>,
    pub event_bus: Addr<EventBus>,
}

#[derive(Debug, Clone)]
pub struct SessionData {
    #[allow(unused)]
    pub session_id: Uuid,
    pub user_id: String,
    pub conversation_history: Vec<ConversationMessage>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ConversationMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DigitalHumanActor {
    pub fn new(name: String, personality: String, event_bus: Addr<EventBus>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            personality,
            sessions: HashMap::new(),
            event_bus,
        }
    }

    fn create_session(&mut self, session_id: Uuid, user_id: String) {
        let session_data = SessionData {
            session_id,
            user_id: user_id.clone(),
            conversation_history: Vec::new(),
            last_activity: chrono::Utc::now(),
        };

        self.sessions.insert(session_id, session_data);
        info!("Created new session {} for user {}", session_id, user_id);
    }

    fn remove_session(&mut self, session_id: &Uuid) {
        if let Some(session) = self.sessions.remove(session_id) {
            info!(
                "Removed session {} for user {}",
                session_id, session.user_id
            );
        }
    }

    fn add_message_to_history(&mut self, session_id: &Uuid, role: String, content: String) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            let message = ConversationMessage {
                role,
                content,
                timestamp: chrono::Utc::now(),
            };
            session.conversation_history.push(message);
            session.last_activity = chrono::Utc::now();
        }
    }

    fn process_text_input(&mut self, event: TextInputEvent) {
        let session_id = event.metadata.session_id.unwrap_or_default();

        // Add user message to history
        self.add_message_to_history(&session_id, "user".to_string(), event.text.clone());

        info!(
            "Processing text input for session {}: {}",
            session_id, event.text
        );

        // Create a simple response
        let response = format!(
            "Hello! I'm {}, and I received your message: '{}'",
            self.name, event.text
        );

        // Add AI response to history
        self.add_message_to_history(&session_id, "assistant".to_string(), response.clone());

        // Create LLM response event
        let llm_response = LLMResponseEvent {
            metadata: EventMetadata {
                session_id: Some(session_id),
                user_id: event.metadata.user_id.clone(),
                ..Default::default()
            },
            response: response.clone(),
            model: "digital_human".to_string(),
            tokens_used: None,
        };

        // Publish LLM response event through EventBus
        self.event_bus.do_send(llm_response);

        // Generate animation event based on response sentiment
        let animation_event = self.generate_animation_for_response(&response, &session_id, &event.metadata.user_id);
        self.event_bus.do_send(animation_event);

        // Generate emotion event (could be facial expression)
        let emotion_event = self.generate_emotion_for_response(&response, &session_id, &event.metadata.user_id);
        self.event_bus.do_send(emotion_event);
    }

    fn generate_animation_for_response(&self, response: &str, session_id: &Uuid, user_id: &Option<String>) -> AnimationEvent {
        // Simple animation selection based on content
        let animation_type = if response.contains("Hello") || response.contains("Hi") {
            "wave"
        } else if response.contains("?") {
            "thinking"
        } else {
            "talk"
        };

        AnimationEvent {
            metadata: EventMetadata {
                session_id: Some(*session_id),
                user_id: user_id.clone(),
                ..Default::default()
            },
            animation_type: animation_type.to_string(),
            duration: Some(2.0),
            parameters: serde_json::json!({
                "intensity": 0.8,
                "loop": false
            }),
        }
    }

    fn generate_emotion_for_response(&self, response: &str, session_id: &Uuid, user_id: &Option<String>) -> AnimationEvent {
        // Generate facial expression based on response
        let emotion = if response.contains("!") {
            "excited"
        } else if response.contains("?") {
            "curious"
        } else {
            "friendly"
        };

        AnimationEvent {
            metadata: EventMetadata {
                session_id: Some(*session_id),
                user_id: user_id.clone(),
                ..Default::default()
            },
            animation_type: format!("expression_{}", emotion),
            duration: Some(3.0),
            parameters: serde_json::json!({
                "emotion": emotion,
                "strength": 0.7
            }),
        }
    }
}

impl Actor for DigitalHumanActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!(
            "DigitalHumanActor '{}' started with ID: {}",
            self.name, self.id
        );
    }
}

impl Handler<UserConnectedEvent> for DigitalHumanActor {
    type Result = ();

    fn handle(&mut self, event: UserConnectedEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "User connected: {} in session {}",
            event.user_id, event.session_id
        );
        self.create_session(event.session_id, event.user_id);
    }
}

impl Handler<UserDisconnectedEvent> for DigitalHumanActor {
    type Result = ();

    fn handle(&mut self, event: UserDisconnectedEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "User disconnected: {} from session {}",
            event.user_id, event.session_id
        );
        self.remove_session(&event.session_id);
    }
}

impl Handler<TextInputEvent> for DigitalHumanActor {
    type Result = ();

    fn handle(&mut self, event: TextInputEvent, _ctx: &mut Context<Self>) -> Self::Result {
        self.process_text_input(event);
    }
}

impl Handler<AudioInputEvent> for DigitalHumanActor {
    type Result = ();

    fn handle(&mut self, event: AudioInputEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "Received audio input of {} bytes for session {:?}",
            event.audio_data.len(),
            event.metadata.session_id
        );

        // TODO: Implement speech-to-text processing
        // This would convert audio to text and then trigger a TextInputEvent
    }
}

// Message types for direct communication
#[derive(Message)]
#[rtype(result = "String")]
pub struct GetActorInfo;

impl Handler<GetActorInfo> for DigitalHumanActor {
    type Result = String;

    fn handle(&mut self, _msg: GetActorInfo, _ctx: &mut Context<Self>) -> Self::Result {
        format!(
            "DigitalHuman: {} (ID: {}), Active sessions: {}",
            self.name,
            self.id,
            self.sessions.len()
        )
    }
}
