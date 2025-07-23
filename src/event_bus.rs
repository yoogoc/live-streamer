use crate::actor::DigitalHumanActor;
use crate::events::*;
use crate::validator::{TextValidator, ValidationResult};
use crate::websocket::WebSocketManager;
use actix::prelude::*;
use log::info;
// use std::collections::HashMap;

#[derive(Debug)]
pub struct EventBus {
    // subscribers: HashMap<String, Vec<String>>,
    digital_human_actor: Option<Addr<DigitalHumanActor>>,
    websocket_manager: Option<Addr<WebSocketManager>>,
    text_validator: TextValidator,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            // subscribers: HashMap::new(),
            digital_human_actor: None,
            websocket_manager: None,
            text_validator: TextValidator::new(),
        }
    }

    pub fn register_digital_human(&mut self, addr: Addr<DigitalHumanActor>) {
        self.digital_human_actor = Some(addr);
        info!("Registered DigitalHumanActor with EventBus");
    }

    pub fn register_websocket_manager(&mut self, addr: Addr<WebSocketManager>) {
        self.websocket_manager = Some(addr);
        info!("Registered WebSocketManager with EventBus");
    }
}

impl Actor for EventBus {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("EventBus started");
    }
}

impl Handler<UserConnectedEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: UserConnectedEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received UserConnectedEvent: {} for session {}",
            event.user_id, event.session_id
        );

        // Forward to DigitalHumanActor
        if let Some(ref digital_human) = self.digital_human_actor {
            digital_human.do_send(event);
        }
    }
}

impl Handler<UserDisconnectedEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: UserDisconnectedEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received UserDisconnectedEvent: {} for session {}",
            event.user_id, event.session_id
        );

        // Forward to DigitalHumanActor
        if let Some(ref digital_human) = self.digital_human_actor {
            digital_human.do_send(event);
        }
    }
}

impl Handler<TextInputEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: TextInputEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received TextInputEvent: {} for session {:?}",
            event.text, event.metadata.session_id
        );

        // 校验弹幕内容
        match self.text_validator.validate(&event) {
            ValidationResult::Allow => {
                // 允许：转发给DigitalHumanActor
                if let Some(ref digital_human) = self.digital_human_actor {
                    digital_human.do_send(event);
                }
            }
            ValidationResult::Ignore => {
                // 忽略：什么都不做
                info!("TextInputEvent ignored due to validation rules");
            }
            ValidationResult::Warn(warning_msg) => {
                // 警告：使用LLM生成警告文本
                let warning_response = LLMResponseEvent {
                    metadata: EventMetadata {
                        session_id: event.metadata.session_id,
                        user_id: event.metadata.user_id,
                        ..Default::default()
                    },
                    response: format!("⚠️ {}", warning_msg),
                    model: "validation_system".to_string(),
                    tokens_used: None,
                };

                // 发送警告消息
                if let Some(ref websocket_manager) = self.websocket_manager {
                    websocket_manager.do_send(warning_response);
                }
            }
        }
    }
}

impl Handler<AudioInputEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: AudioInputEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received AudioInputEvent: {} for session {:?}",
            event.format, event.metadata.session_id
        );

        // Forward to DigitalHumanActor
        if let Some(ref digital_human) = self.digital_human_actor {
            digital_human.do_send(event);
        }
    }
}

impl Handler<TTSResponseEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: TTSResponseEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received TTSResponseEvent: {} for session {:?}",
            event.text, event.metadata.session_id
        );

        // Forward to WebSocketManager to send back to client
        if let Some(ref websocket_manager) = self.websocket_manager {
            websocket_manager.do_send(event);
        }
    }
}

impl Handler<AnimationEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: AnimationEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received AnimationEvent: {} for session {:?}",
            event.animation_type, event.metadata.session_id
        );

        // Forward to WebSocketManager to send back to client
        if let Some(ref websocket_manager) = self.websocket_manager {
            websocket_manager.do_send(event);
        }
    }
}

impl Handler<LLMResponseEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: LLMResponseEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received LLMResponseEvent: {} for session {:?}",
            event.response, event.metadata.session_id
        );

        // Forward to WebSocketManager to send back to client
        if let Some(ref websocket_manager) = self.websocket_manager {
            websocket_manager.do_send(event);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterDigitalHuman {
    pub addr: Addr<DigitalHumanActor>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterWebSocketManager {
    pub addr: Addr<WebSocketManager>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterActor {
    pub actor_type: String,
    // pub addr: Recipient<PublishEvent<UserConnectedEvent>>,
}

impl Handler<RegisterDigitalHuman> for EventBus {
    type Result = ();

    fn handle(&mut self, msg: RegisterDigitalHuman, _ctx: &mut Context<Self>) -> Self::Result {
        self.register_digital_human(msg.addr);
    }
}

impl Handler<RegisterWebSocketManager> for EventBus {
    type Result = ();

    fn handle(&mut self, msg: RegisterWebSocketManager, _ctx: &mut Context<Self>) -> Self::Result {
        self.register_websocket_manager(msg.addr);
    }
}

impl Handler<RegisterActor> for EventBus {
    type Result = ();

    fn handle(&mut self, msg: RegisterActor, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Registered actor type: {}", msg.actor_type);
        // In a full implementation, you would store the addr for routing events
    }
}
