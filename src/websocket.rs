use crate::event_bus::{EventBus, PublishEvent};
use crate::events::*;
use actix::prelude::*;
use log::info;
use std::collections::HashMap;
use uuid::Uuid;

pub struct WebSocketManager {
    connections: HashMap<Uuid, String>,
    event_bus: Addr<EventBus>,
}

impl WebSocketManager {
    pub fn new(event_bus: Addr<EventBus>) -> Self {
        Self {
            connections: HashMap::new(),
            event_bus,
        }
    }

    fn add_connection(&mut self, session_id: Uuid, user_id: String) {
        self.connections.insert(session_id, user_id.clone());
        info!(
            "Added WebSocket connection for session: {} user: {}",
            session_id, user_id
        );
    }

    fn remove_connection(&mut self, session_id: &Uuid) {
        if let Some(user_id) = self.connections.remove(session_id) {
            info!(
                "Removed WebSocket connection for session: {} user: {}",
                session_id, user_id
            );
        }
    }
}

impl Actor for WebSocketManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocketManager started");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterConnection {
    pub session_id: Uuid,
    pub user_id: String,
}

impl Handler<RegisterConnection> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: RegisterConnection, _ctx: &mut Context<Self>) -> Self::Result {
        self.add_connection(msg.session_id, msg.user_id);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterConnection {
    pub session_id: Uuid,
}

impl Handler<UnregisterConnection> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: UnregisterConnection, _ctx: &mut Context<Self>) -> Self::Result {
        self.remove_connection(&msg.session_id);
    }
}

impl Handler<LLMResponseEvent> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, event: LLMResponseEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let session_id = event.metadata.session_id.unwrap_or_default();

        if let Some(_user_id) = self.connections.get(&session_id) {
            let message = serde_json::json!({
                "type": "llm_response",
                "data": {
                    "response": event.response,
                    "model": event.model,
                    "timestamp": event.metadata.timestamp
                }
            });

            info!(
                "Broadcasting LLM response to session {}: {}",
                session_id, message
            );
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleTextMessage {
    pub session_id: Uuid,
    pub user_id: String,
    pub text: String,
}

impl Handler<HandleTextMessage> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: HandleTextMessage, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Received text message from {}: {}", msg.user_id, msg.text);

        // Try to parse as JSON for structured messages
        if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&msg.text) {
            if let Some(msg_type) = json_msg.get("type").and_then(|t| t.as_str()) {
                match msg_type {
                    "text_input" => {
                        if let Some(content) = json_msg.get("content").and_then(|c| c.as_str()) {
                            let event = TextInputEvent {
                                metadata: EventMetadata {
                                    session_id: Some(msg.session_id),
                                    user_id: Some(msg.user_id.clone()),
                                    ..Default::default()
                                },
                                text: content.to_string(),
                                language: json_msg
                                    .get("language")
                                    .and_then(|l| l.as_str())
                                    .map(|s| s.to_string()),
                            };
                            self.event_bus.do_send(PublishEvent(event));
                        }
                    }
                    _ => {
                        info!("Unknown message type: {}", msg_type);
                    }
                }
            }
        } else {
            // Treat as plain text input
            let event = TextInputEvent {
                metadata: EventMetadata {
                    session_id: Some(msg.session_id),
                    user_id: Some(msg.user_id.clone()),
                    ..Default::default()
                },
                text: msg.text.to_string(),
                language: None,
            };
            self.event_bus.do_send(PublishEvent(event));
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleUserConnect {
    pub session_id: Uuid,
    pub user_id: String,
}

impl Handler<HandleUserConnect> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: HandleUserConnect, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "WebSocket connection started for user: {} session: {}",
            msg.user_id, msg.session_id
        );

        // Register this connection
        self.add_connection(msg.session_id, msg.user_id.clone());

        // Publish user connected event
        let event = UserConnectedEvent {
            metadata: EventMetadata {
                session_id: Some(msg.session_id),
                user_id: Some(msg.user_id.clone()),
                ..Default::default()
            },
            session_id: msg.session_id,
            user_id: msg.user_id,
        };

        self.event_bus.do_send(PublishEvent(event));
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleUserDisconnect {
    pub session_id: Uuid,
    pub user_id: String,
}

impl Handler<HandleUserDisconnect> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: HandleUserDisconnect, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "WebSocket connection ended for user: {} session: {}",
            msg.user_id, msg.session_id
        );

        // Unregister this connection
        self.remove_connection(&msg.session_id);

        // Publish user disconnected event
        let event = UserDisconnectedEvent {
            metadata: EventMetadata {
                session_id: Some(msg.session_id),
                user_id: Some(msg.user_id.clone()),
                ..Default::default()
            },
            session_id: msg.session_id,
            user_id: msg.user_id,
        };

        self.event_bus.do_send(PublishEvent(event));
    }
}
