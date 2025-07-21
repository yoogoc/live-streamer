# Digital Human Service (数字人服务)

一个基于 Rust 和 Actix 框架的事件驱动数字人服务，提供实时的人机交互能力。

## 项目架构

### 核心组件

#### 1. 事件系统 (`src/events.rs`)
- 定义了完整的事件类型系统
- 包含用户连接/断开、文本输入、音频输入、LLM响应等事件
- 所有事件都包含元数据（时间戳、会话ID、用户ID等）

**主要事件类型：**
- `UserConnectedEvent` - 用户连接事件
- `UserDisconnectedEvent` - 用户断开连接事件
- `TextInputEvent` - 文本输入事件
- `AudioInputEvent` - 音频输入事件
- `LLMResponseEvent` - LLM响应事件
- `TTSResponseEvent` - 语音合成响应事件
- `AnimationEvent` - 动画事件

#### 2. 事件总线 (`src/event_bus.rs`)
- 中央事件分发系统
- 处理各种事件的路由和处理
- 支持事件发布和订阅机制
- 负责在不同组件间传递消息

#### 3. 数字人核心 (`src/actor.rs`)
- `DigitalHumanActor` - 主要的数字人逻辑处理器
- 管理用户会话和对话历史
- 处理文本输入并生成响应
- 支持个性化配置（名称、性格特征）
- 维护会话状态和对话上下文

**核心功能：**
- 会话创建和管理
- 对话历史记录
- 文本输入处理
- 智能响应生成

#### 4. WebSocket管理 (`src/websocket.rs`)
- `WebSocketManager` - 管理所有WebSocket连接
- 处理用户连接的生命周期
- 支持实时消息传递
- 处理连接注册和注销

**消息处理：**
- 文本消息处理
- 二进制消息处理（音频数据）
- 连接状态管理
- 事件广播

#### 5. HTTP路由 (`src/routes/mod.rs`)
- REST API端点（健康检查、数字人信息）
- WebSocket升级处理
- 支持跨域访问
- 实时会话管理

### 技术栈

- **Rust** - 系统级编程语言，保证内存安全和高性能
- **Actix** - 高性能的Actor模型框架
- **Actix-Web** - 快速的异步Web框架
- **Actix-WS** - WebSocket支持
- **Serde** - 序列化/反序列化框架
- **UUID** - 唯一标识符生成
- **Chrono** - 时间处理
- **Log/Env_logger** - 日志系统

## 主要功能特性

✅ **事件驱动架构** - 使用Actix actor模型实现完全异步的事件处理  
✅ **实时通信** - WebSocket支持实现与用户的实时交互  
✅ **会话管理** - 自动管理用户会话和对话历史  
✅ **可扩展设计** - 模块化架构便于扩展新功能  
✅ **类型安全** - 完整的Rust类型系统保证代码安全性  
✅ **高性能** - 基于Actix的异步处理，支持高并发  
✅ **内存安全** - Rust的所有权系统防止内存泄漏和竞态条件

## API 接口

### REST API

#### 健康检查
```
GET /api/v1/health
```
返回服务健康状态

#### 数字人信息
```
GET /api/v1/digital-human/info
```
返回数字人的基本信息和能力

### WebSocket API

#### 连接端点
```
WS /api/v1/ws/{user_id}
```

#### 消息格式

**文本输入消息：**
```json
{
  "type": "text_input",
  "content": "Hello, how are you?",
  "language": "en"
}
```

**响应消息：**
```json
{
  "type": "llm_response",
  "data": {
    "response": "Hello! I'm doing well, thank you for asking.",
    "model": "digital_human",
    "timestamp": "2023-12-07T10:30:00Z"
  }
}
```

## 快速开始

### 环境要求

- Rust 1.70+ 
- Cargo

### 安装依赖

```bash
cargo build
```

### 运行服务

```bash
cargo run
```

服务将在 `http://127.0.0.1:8080` 启动

### 测试连接

1. **健康检查：**
```bash
curl http://127.0.0.1:8080/api/v1/health
```

2. **获取数字人信息：**
```bash
curl http://127.0.0.1:8080/api/v1/digital-human/info
```

3. **WebSocket连接测试：**
```javascript
const ws = new WebSocket('ws://127.0.0.1:8080/api/v1/ws/test_user');

ws.onopen = function() {
    console.log('Connected to digital human');
    
    // 发送文本消息
    ws.send(JSON.stringify({
        type: 'text_input',
        content: 'Hello, digital human!',
        language: 'en'
    }));
};

ws.onmessage = function(event) {
    const response = JSON.parse(event.data);
    console.log('Received:', response);
};
```

## 开发和扩展

### 添加新的事件类型

1. 在 `src/events.rs` 中定义新的事件结构
2. 实现 `Event` trait
3. 在相关的Actor中添加事件处理器

### 集成外部服务

项目设计支持轻松集成：

- **LLM服务** - OpenAI GPT、本地模型等
- **语音服务** - 语音识别和语音合成
- **数据库** - 会话持久化和用户数据存储
- **缓存系统** - Redis等缓存解决方案

### 扩展功能示例

```rust
// 添加新的事件类型
#[derive(Debug, Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct CustomEvent {
    pub metadata: EventMetadata,
    pub custom_data: String,
}

impl Event for CustomEvent {
    fn event_type(&self) -> &'static str { "custom_event" }
    fn metadata(&self) -> &EventMetadata { &self.metadata }
    fn set_metadata(&mut self, metadata: EventMetadata) { self.metadata = metadata; }
}
```

## 项目结构

```
src/
├── main.rs              # 应用入口点
├── events.rs            # 事件定义和类型
├── event_bus.rs         # 事件总线实现
├── actor.rs             # 数字人核心逻辑
├── websocket.rs         # WebSocket连接管理
├── llm/                 # LLM集成模块
│   ├── mod.rs
│   └── openai.rs
└── routes/              # HTTP路由处理
    └── mod.rs
```

## 配置

服务配置可以通过环境变量进行调整：

```bash
# 设置日志级别
RUST_LOG=info

# 设置服务端口
PORT=8080

# 设置绑定地址
BIND_ADDRESS=127.0.0.1
```

## 测试

运行测试套件：

```bash
cargo test
```

## 性能特性

- **异步处理** - 基于Tokio的异步运行时
- **零拷贝** - 高效的消息传递机制
- **内存管理** - Rust的所有权系统确保内存安全
- **并发安全** - Actor模型保证线程安全

## 许可证

本项目基于 MIT 许可证开源。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这个项目。

---

**注意：** 这是一个基础版本的数字人服务，提供了完整的架构基础。您可以在此基础上集成具体的LLM、TTS、STT等服务来构建完整的数字人解决方案。