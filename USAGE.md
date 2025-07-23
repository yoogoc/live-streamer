# 数字人直播服务 API 使用示例

这个项目实现了一个数字人直播服务，支持多平台弹幕集成和实时互动。

## 主要功能

1. **弹幕校验系统** - 实时过滤和验证弹幕内容
2. **多平台集成** - 支持抖音、B站、YouTube等直播平台
3. **数字人交互** - LLM驱动的对话和动画生成
4. **WebSocket连接** - 实时双向通信

## API接口

### 1. WebSocket连接
```javascript
// 连接到数字人服务
const ws = new WebSocket('ws://localhost:8080/api/v1/ws/user123');

// 发送文本消息
ws.send(JSON.stringify({
    type: "text_input",
    content: "你好，数字人！",
    language: "zh-CN"
}));

// 接收响应
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('收到回复:', data);
};
```

### 2. 抖音弹幕集成
```bash
# 抖音弹幕回调接口
POST /api/v1/danmaku/douyin
Content-Type: application/json

{
    "message": "主播好棒！",
    "user_id": "douyin_user_123",
    "username": "小明",
    "room_id": "room_456",
    "user_level": 5,
    "is_vip": false
}
```

### 3. B站弹幕集成
```bash
# B站弹幕回调接口
POST /api/v1/danmaku/bilibili
Content-Type: application/json

{
    "info": [
        [], 
        "弹幕内容", 
        [123456, "用户名", 0, 0, 0, 10000, 1, ""]
    ],
    "roomid": 789
}
```

### 4. 平台配置管理
```bash
# 添加直播平台配置
POST /api/v1/platform/config
Content-Type: application/json

{
    "platform": "Douyin",
    "room_id": "room_123",
    "api_key": "your_api_key",
    "webhook_url": "http://your-server.com/webhook",
    "enabled": true
}
```

## 运行服务

```bash
# 启动服务
RUST_LOG=info cargo run

# 服务将在 http://0.0.0.0:8080 启动
```

## 弹幕校验规则

系统内置以下校验规则：

1. **敏感词过滤** - 自动检测和处理敏感内容
2. **频率限制** - 防止刷屏，每用户最多10条/分钟
3. **长度限制** - 弹幕长度1-200字符

### 校验结果处理：
- **Allow**: 正常处理，发送给数字人AI
- **Ignore**: 静默忽略
- **Warn**: 返回警告消息给用户

## 数字人响应流程

```
弹幕输入 → 校验规则 → EventBus → DigitalHumanActor
                                        ↓
生成动画 ← 生成表情 ← 生成回复 ← LLM处理
    ↓         ↓         ↓
WebSocket ← WebSocket ← WebSocket
```

## WebSocket消息格式

### 客户端发送：
```json
{
    "type": "text_input",
    "content": "消息内容",
    "language": "zh-CN"
}
```

### 服务端响应：
```json
{
    "type": "llm_response",
    "data": {
        "response": "你好！我是数字人Maya",
        "model": "digital_human",
        "timestamp": "2024-01-01T12:00:00Z"
    }
}
```

### 动画事件：
```json
{
    "type": "animation",
    "data": {
        "animation_type": "wave",
        "duration": 2.0,
        "parameters": {
            "intensity": 0.8,
            "loop": false
        }
    }
}
```

## 健康检查

```bash
GET /api/v1/health

Response:
{
    "status": "healthy",
    "service": "digital-human",
    "timestamp": "2024-01-01T12:00:00Z"
}
```

## 系统架构

- **EventBus**: 事件分发中心，处理所有事件路由
- **TextValidator**: 弹幕内容校验器
- **DigitalHumanActor**: 数字人AI处理器
- **LiveStreamManager**: 直播平台集成管理器
- **WebSocketManager**: WebSocket连接管理器

所有组件采用Actor模型，支持高并发和异步处理。