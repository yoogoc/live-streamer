use crate::platform::*;
use crate::websocket::*;
use actix::prelude::*;
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
            .route("/digital-human/info", web::get().to(get_digital_human_info))
            .route("/danmaku/douyin", web::post().to(handle_douyin_danmaku))
            .route("/danmaku/bilibili", web::post().to(handle_bilibili_danmaku))
            .route("/platform/config", web::post().to(add_platform_config)),
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
    // Create WebSocket session actor
    let session_actor =
        WebSocketSessionActor::new(session.clone(), session_id, user_id.clone()).start();

    // Send connection event
    ws_manager.do_send(HandleUserConnect {
        session_id,
        user_id: user_id.clone(),
        session_actor: session_actor.clone(),
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

// 处理抖音弹幕的HTTP回调
async fn handle_douyin_danmaku(
    json: web::Json<serde_json::Value>,
    live_manager: web::Data<Addr<LiveStreamManager>>,
) -> Result<HttpResponse> {
    info!("Received Douyin danmaku: {:?}", json);

    // 解析抖音弹幕数据
    if let Ok(danmaku) = parse_douyin_danmaku(&json) {
        live_manager.do_send(ProcessDanmaku { danmaku });
        Ok(HttpResponse::Ok().json(serde_json::json!({"status": "success"})))
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid danmaku format"})))
    }
}

// 处理B站弹幕的HTTP回调
async fn handle_bilibili_danmaku(
    json: web::Json<serde_json::Value>,
    live_manager: web::Data<Addr<LiveStreamManager>>,
) -> Result<HttpResponse> {
    info!("Received Bilibili danmaku: {:?}", json);

    // 解析B站弹幕数据
    if let Ok(danmaku) = parse_bilibili_danmaku(&json) {
        live_manager.do_send(ProcessDanmaku { danmaku });
        Ok(HttpResponse::Ok().json(serde_json::json!({"status": "success"})))
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid danmaku format"})))
    }
}

// 添加平台配置
async fn add_platform_config(
    json: web::Json<LiveStreamConfig>,
    live_manager: web::Data<Addr<LiveStreamManager>>,
) -> Result<HttpResponse> {
    info!("Adding platform config: {:?}", json);

    live_manager.do_send(AddPlatformConfig {
        config: json.into_inner(),
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "success"})))
}

fn parse_douyin_danmaku(data: &serde_json::Value) -> Result<DanmakuMessage, String> {
    let message = data
        .get("message")
        .and_then(|m| m.as_str())
        .ok_or("Missing message field")?;

    let user_id = data
        .get("user_id")
        .and_then(|u| u.as_str())
        .unwrap_or("anonymous");

    let username = data
        .get("username")
        .and_then(|u| u.as_str())
        .unwrap_or("用户");

    let room_id = data
        .get("room_id")
        .and_then(|r| r.as_str())
        .unwrap_or("unknown");

    Ok(DanmakuMessage {
        platform: Platform::Douyin,
        room_id: room_id.to_string(),
        user_id: user_id.to_string(),
        username: username.to_string(),
        message: message.to_string(),
        timestamp: chrono::Utc::now(),
        user_level: data
            .get("user_level")
            .and_then(|l| l.as_u64())
            .map(|l| l as u32),
        is_vip: data
            .get("is_vip")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    })
}

fn parse_bilibili_danmaku(data: &serde_json::Value) -> Result<DanmakuMessage, String> {
    let info = data
        .get("info")
        .and_then(|i| i.as_array())
        .ok_or("Missing info array")?;

    let message = info
        .get(1)
        .and_then(|m| m.as_str())
        .ok_or("Missing message")?;

    let user_info = info
        .get(2)
        .and_then(|u| u.as_array())
        .ok_or("Missing user info")?;

    let user_id = user_info
        .get(0)
        .and_then(|u| u.as_u64())
        .map(|u| u.to_string())
        .unwrap_or("anonymous".to_string());

    let username = user_info.get(1).and_then(|u| u.as_str()).unwrap_or("用户");

    let room_id = data
        .get("roomid")
        .and_then(|r| r.as_u64())
        .map(|r| r.to_string())
        .unwrap_or("unknown".to_string());

    Ok(DanmakuMessage {
        platform: Platform::Bilibili,
        room_id,
        user_id,
        username: username.to_string(),
        message: message.to_string(),
        timestamp: chrono::Utc::now(),
        user_level: None,
        is_vip: false,
    })
}
