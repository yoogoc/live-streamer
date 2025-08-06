# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Live Streamer is a Rust-based digital human service built with the Actix framework. It provides real-time interactive capabilities through WebSocket connections and integrates with multiple live streaming platforms (Douyin, Bilibili, YouTube) to process danmaku (live chat) messages.

## Architecture

The system uses an event-driven architecture with the Actix actor model:

### Core Components

- **EventBus** (`src/event_bus.rs`) - Central message routing and event handling
- **DigitalHumanActor** (`src/actor.rs`) - Main AI logic processor, manages sessions and conversation history
- **WebSocketManager** (`src/websocket.rs`) - Manages WebSocket connections and session actors
- **LiveStreamManager** (`src/platform/manager.rs`) - Coordinates multiple platform listeners
- **TextValidator** (`src/validator.rs`) - Content validation and filtering system

### Event System

All events implement the `Event` trait from `src/events.rs` and include metadata:
- `UserConnectedEvent` / `UserDisconnectedEvent` - Connection lifecycle
- `TextInputEvent` / `AudioInputEvent` - User input events  
- `LLMResponseEvent` / `TTSResponseEvent` - AI responses
- `AnimationEvent` - Digital human animation triggers

## Development Commands

### Build and Run
```bash
# Build the project
cargo build

# Run the service (listens on 0.0.0.0:8080)
cargo run

# Run with logging
RUST_LOG=info cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_health_check
```

### Development Setup
```bash
# Check code formatting
cargo fmt --check

# Run clippy for lints
cargo clippy

# Check for unused dependencies
cargo machete  # if installed
```

## Key Directories

- `src/` - Main source code
- `src/platform/` - Live streaming platform integrations
- `src/llm/` - LLM service integrations (OpenAI module)
- `src/routes/` - HTTP route handlers

## API Endpoints

### REST API
- `GET /api/v1/health` - Health check
- `GET /api/v1/digital-human/info` - Digital human information
- `POST /api/v1/danmaku/{platform}` - Platform-specific danmaku callbacks

### WebSocket
- `WS /api/v1/ws/{user_id}` - Real-time user connection

## Platform Integration

The system supports multiple live streaming platforms through the `PlatformListener` trait:
- Each platform implements the trait with `start()`, `stop()`, and `is_running()` methods
- Danmaku messages are converted to unified `DanmakuMessage` format
- Platform listeners are managed by `LiveStreamManager`

## Actor Communication Flow

```
User Input → WebSocketSessionActor → EventBus → TextValidator → DigitalHumanActor
                                                                        ↓
WebSocketManager ← EventBus ← LLMResponseEvent/AnimationEvent ← Response Generation
```

## Configuration

Environment variables:
- `RUST_LOG` - Logging level (info, debug, warn, error)
- Service runs on port 8080 by default

## Dependencies

Key dependencies from Cargo.toml:
- `actix-web` 4.9 - Web framework
- `actix-ws` 0.3 - WebSocket support
- `serde` 1 - Serialization
- `uuid` 1.6 - Session/user identification
- `chrono` 0.4.30 - Timestamp handling
- `tokio` 1.24.2 - Async runtime

## Adding New Features

### New Event Types
1. Define event struct in `src/events.rs`
2. Implement `Event` trait
3. Add handlers in `EventBus` and relevant actors

### New Platform Integration
1. Implement `PlatformListener` trait in `src/platform/`
2. Add platform to `Platform` enum
3. Register with `LiveStreamManager`

### New Validation Rules
1. Extend `RuleType` enum in validator
2. Implement validation logic in `TextValidator::validate()`