use crate::event_bus::{EventBus, PublishEvent};
use crate::events::*;
use actix::prelude::*;
use log::{info, warn};
use std::collections::HashMap;
use uuid::Uuid;

pub struct WebSocketManager {
    connections: HashMap<Uuid, (String, Addr<WebSocketSessionActor>)>,
    event_bus: Addr<EventBus>,
}

impl WebSocketManager {
    pub fn new(event_bus: Addr<EventBus>) -> Self {
        Self {
            connections: HashMap::new(),
            event_bus,
        }
    }

    fn add_connection(
        &mut self,
        session_id: Uuid,
        user_id: String,
        session_actor: Addr<WebSocketSessionActor>,
    ) {
        self.connections
            .insert(session_id, (user_id.clone(), session_actor));
        info!(
            "Added WebSocket connection for session: {} user: {}",
            session_id, user_id
        );
    }

    fn remove_connection(&mut self, session_id: &Uuid) {
        if let Some((user_id, _)) = self.connections.remove(session_id) {
            info!(
                "Removed WebSocket connection for session: {} user: {}",
                session_id, user_id
            );
        }
    }
}

// New WebSocket Session Actor
pub struct WebSocketSessionActor {
    session: actix_ws::Session,
    session_id: Uuid,
    user_id: String,
}

impl WebSocketSessionActor {
    pub fn new(session: actix_ws::Session, session_id: Uuid, user_id: String) -> Self {
        Self {
            session,
            session_id,
            user_id,
        }
    }
}

impl Actor for WebSocketSessionActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!(
            "WebSocket session actor started for user: {} session: {}",
            self.user_id, self.session_id
        );
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub message: String,
}

impl Handler<SendMessage> for WebSocketSessionActor {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Context<Self>) -> Self::Result {
        let mut session = self.session.clone();
        let message = msg.message;
        let session_id = self.session_id;

        // Use spawn to handle the async operation
        let fut = async move {
            if let Err(e) = session.text(message).await {
                warn!("Failed to send message to session {}: {}", session_id, e);
            }
        };
        ctx.spawn(fut.into_actor(self));
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
    pub session_actor: Addr<WebSocketSessionActor>,
}

impl Handler<RegisterConnection> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: RegisterConnection, _ctx: &mut Context<Self>) -> Self::Result {
        self.add_connection(msg.session_id, msg.user_id, msg.session_actor);
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

        if let Some((user_id, session_actor)) = self.connections.get(&session_id) {
            let message = serde_json::json!({
                "type": "llm_response",
                "data": {
                    "response": event.response,
                    "model": event.model,
                    "timestamp": event.metadata.timestamp
                }
            });

            let message_str = message.to_string();
            info!(
                "Sending LLM response to session {} (user {}): {}",
                session_id, user_id, message_str
            );

            // Send the message through WebSocket session actor
            session_actor.do_send(SendMessage {
                message: message_str,
            });
        } else {
            warn!("No active connection found for session {}", session_id);
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
    pub session_actor: Addr<WebSocketSessionActor>,
}

impl Handler<HandleUserConnect> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: HandleUserConnect, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "WebSocket connection started for user: {} session: {}",
            msg.user_id, msg.session_id
        );

        // Register this connection
        self.add_connection(msg.session_id, msg.user_id.clone(), msg.session_actor);

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

impl Handler<TTSResponseEvent> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, event: TTSResponseEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let session_id = event.metadata.session_id.unwrap_or_default();

        if let Some((user_id, session_actor)) = self.connections.get(&session_id) {
            let message = serde_json::json!({
                "type": "tts_response",
                "data": {
                    "text": event.text,
                    "voice": event.voice,
                    "audio_data_length": event.audio_data.len(),
                    "timestamp": event.metadata.timestamp
                }
            });

            let message_str = message.to_string();
            info!(
                "Sending TTS response to session {} (user {}): {}",
                session_id, user_id, message_str
            );

            // Send the message through WebSocket session actor
            session_actor.do_send(SendMessage {
                message: message_str,
            });
        } else {
            warn!("No active connection found for session {}", session_id);
        }
    }
}

impl Handler<AnimationEvent> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, event: AnimationEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let session_id = event.metadata.session_id.unwrap_or_default();

        if let Some((user_id, session_actor)) = self.connections.get(&session_id) {
            let message = serde_json::json!({
                "type": "animation",
                "data": {
                    "animation_type": event.animation_type,
                    "duration": event.duration,
                    "parameters": event.parameters,
                    "timestamp": event.metadata.timestamp
                }
            });

            let message_str = message.to_string();
            info!(
                "Sending animation event to session {} (user {}): {}",
                session_id, user_id, message_str
            );

            // Send the message through WebSocket session actor
            session_actor.do_send(SendMessage {
                message: message_str,
            });
        } else {
            warn!("No active connection found for session {}", session_id);
        }
    }
}
