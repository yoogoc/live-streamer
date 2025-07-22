use crate::actor::DigitalHumanActor;
use crate::events::*;
use crate::websocket::WebSocketManager;
use actix::prelude::*;
use log::info;
use std::any::Any;
// use std::collections::HashMap;

#[derive(Debug)]
pub struct EventBus {
    // subscribers: HashMap<String, Vec<String>>,
    digital_human_actor: Option<Addr<DigitalHumanActor>>,
    websocket_manager: Option<Addr<WebSocketManager>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            // subscribers: HashMap::new(),
            digital_human_actor: None,
            websocket_manager: None,
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct PublishEvent<T: Event>(pub T);

impl<T: Event> Handler<PublishEvent<T>> for EventBus {
    type Result = ();

    fn handle(&mut self, msg: PublishEvent<T>, ctx: &mut Context<Self>) -> Self::Result {
        let event = msg.0;
        let event_type = event.event_type();

        info!("Handling publish request for event: {}", event_type);
        match event_type {
            "user_connected" => {
                if let Some(user_event) = (&event as &dyn Any).downcast_ref::<UserConnectedEvent>()
                {
                    ctx.address().do_send(user_event.clone());
                }
            }
            "user_disconnected" => {
                if let Some(user_event) =
                    (&event as &dyn Any).downcast_ref::<UserDisconnectedEvent>()
                {
                    ctx.address().do_send(user_event.clone());
                }
            }
            "text_input" => {
                if let Some(text_event) = (&event as &dyn Any).downcast_ref::<TextInputEvent>() {
                    ctx.address().do_send(text_event.clone());
                }
            }
            "audio_input" => {
                if let Some(audio_event) = (&event as &dyn Any).downcast_ref::<AudioInputEvent>() {
                    // Forward to DigitalHumanActor if registered
                    if let Some(ref digital_human) = self.digital_human_actor {
                        digital_human.do_send(audio_event.clone());
                    }
                }
            }
            "llm_response" => {
                if let Some(llm_event) = (&event as &dyn Any).downcast_ref::<LLMResponseEvent>() {
                    ctx.address().do_send(llm_event.clone());
                }
            }
            "tts_response" => {
                if let Some(tts_event) = (&event as &dyn Any).downcast_ref::<TTSResponseEvent>() {
                    // Forward to WebSocketManager if registered
                    if let Some(ref websocket_manager) = self.websocket_manager {
                        websocket_manager.do_send(tts_event.clone());
                    }
                }
            }
            "animation" => {
                if let Some(anim_event) = (&event as &dyn Any).downcast_ref::<AnimationEvent>() {
                    // Forward to WebSocketManager if registered
                    if let Some(ref websocket_manager) = self.websocket_manager {
                        websocket_manager.do_send(anim_event.clone());
                    }
                }
            }
            _ => {
                info!("Unknown event type: {}", event_type);
            }
        }
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

        // Forward to DigitalHumanActor
        if let Some(ref digital_human) = self.digital_human_actor {
            digital_human.do_send(event);
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
