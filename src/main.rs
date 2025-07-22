use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use env_logger::Env;
use eyre::Result;

mod actor;
mod event_bus;
mod events;
mod llm;
mod routes;
mod websocket;

use actor::DigitalHumanActor;
use event_bus::EventBus;
use websocket::WebSocketManager;

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    log::info!("Starting Digital Human Service...");

    // Create and start the event bus
    let event_bus = EventBus::new().start();
    log::info!("EventBus started");

    // Create and start the WebSocket manager
    let ws_manager = WebSocketManager::new(event_bus.clone()).start();
    log::info!("WebSocketManager started");

    // Create and start digital human actors
    let digital_human = DigitalHumanActor::new(
        "Maya".to_string(),
        "I am a helpful and friendly digital assistant with a warm personality. I enjoy helping users with their questions and providing engaging conversation.".to_string(),
        event_bus.clone()
    ).start();
    log::info!("DigitalHumanActor 'Maya' started");

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(ws_manager.clone()))
            .app_data(web::Data::new(event_bus.clone()))
            .app_data(web::Data::new(digital_human.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .configure(routes::configure_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(App::new().configure(routes::configure_routes)).await;

        let req = test::TestRequest::get().uri("/api/v1/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_digital_human_info() {
        let app = test::init_service(App::new().configure(routes::configure_routes)).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/digital-human/info")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
