use crate::actor::*;
use crate::websocket::*;
use actix::Addr;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_ws;
use futures_util::StreamExt as _;
use log::{info, warn};
use uuid::Uuid;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .route("/ws/{user_id}", web::get().to(websocket_handler))
            .route("/digital-human/info", web::get().to(get_digital_human_info)),
    );
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "digital-human",
        "timestamp": chrono::Utc::now()
    })))
}

async fn websocket_handler(
    req: HttpRequest,
    path: web::Path<String>,
    stream: web::Payload,
    ws_manager: web::Data<Addr<WebSocketManager>>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    info!("WebSocket connection request from user: {}", user_id);

    let (response, session, stream) = actix_ws::handle(&req, stream)?;

    let session_id = Uuid::new_v4();
    let ws_manager_addr = ws_manager.get_ref().clone();

    actix_web::rt::spawn(handle_websocket_session(
        session,
        stream,
        session_id,
        user_id,
        ws_manager_addr,
    ));

    Ok(response)
}

async fn handle_websocket_session(
    mut session: actix_ws::Session,
    mut stream: actix_ws::MessageStream,
    session_id: Uuid,
    user_id: String,
    ws_manager: Addr<WebSocketManager>,
) {
    // Send connection event
    ws_manager.do_send(HandleUserConnect {
        session_id,
        user_id: user_id.clone(),
    });

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(actix_ws::Message::Text(text)) => {
                info!("Received text: {}", text);
                // Handle text message through WebSocketManager
                ws_manager.do_send(HandleTextMessage {
                    session_id,
                    user_id: user_id.clone(),
                    text: text.to_string(),
                });

                // Echo back for now
                if let Err(e) = session.text(format!("Echo: {}", text.to_string())).await {
                    warn!("Failed to send echo: {}", e);
                    break;
                }
            }
            Ok(actix_ws::Message::Binary(bin)) => {
                info!("Received binary data: {} bytes", bin.len());
                // Handle binary message (audio)
            }
            Ok(actix_ws::Message::Ping(bytes)) => {
                if let Err(e) = session.pong(&bytes).await {
                    warn!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(actix_ws::Message::Pong(_)) => {
                // Pong received
            }
            Ok(actix_ws::Message::Close(reason)) => {
                info!("WebSocket closed: {:?}", reason);
                break;
            }
            Ok(actix_ws::Message::Continuation(_)) => {
                // Handle continuation frames
            }
            Ok(actix_ws::Message::Nop) => {
                // Handle nop frames
            }
            Err(e) => {
                warn!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Send disconnect event
    ws_manager.do_send(HandleUserDisconnect {
        session_id,
        user_id,
    });
    info!("WebSocket session ended");
}

async fn get_digital_human_info() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "name": "Digital Human Assistant",
        "version": "1.0.0",
        "capabilities": [
            "text_conversation",
            "audio_input",
            "text_to_speech",
            "animation_control"
        ],
        "supported_languages": ["en", "zh-CN"],
        "websocket_endpoint": "/api/v1/ws/{user_id}"
    })))
}
