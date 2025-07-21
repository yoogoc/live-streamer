use crate::events::*;
use actix::prelude::*;
use log::info;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct EventBus {
    subscribers: HashMap<String, Vec<String>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
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

    fn handle(&mut self, msg: PublishEvent<T>, _ctx: &mut Context<Self>) -> Self::Result {
        let event = msg.0;
        let event_type = event.event_type();

        info!("Handling publish request for event: {}", event_type);
    }
}

impl Handler<UserConnectedEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: UserConnectedEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received UserConnectedEvent: {} for session {}",
            event.user_id, event.session_id
        );
    }
}

impl Handler<UserDisconnectedEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: UserDisconnectedEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received UserDisconnectedEvent: {} for session {}",
            event.user_id, event.session_id
        );
    }
}

impl Handler<TextInputEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, event: TextInputEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "EventBus received TextInputEvent: {} for session {:?}",
            event.text, event.metadata.session_id
        );
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterActor {
    pub actor_type: String,
    pub addr: Recipient<PublishEvent<UserConnectedEvent>>,
}

impl Handler<RegisterActor> for EventBus {
    type Result = ();

    fn handle(&mut self, msg: RegisterActor, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Registered actor type: {}", msg.actor_type);
        // In a full implementation, you would store the addr for routing events
    }
}
