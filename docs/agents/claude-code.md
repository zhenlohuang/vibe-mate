# Claude Code Agent 设计文档

## 概述

Claude Code Agent 是 Anthropic Claude API 的官方 CLI 实现，支持通过 OAuth2 认证或 API Key 两种方式访问 Claude API。主要用于代码辅助、对话和工具调用场景。

## 1. 认证机制 (Authentication)

### 1.1 认证方式

Claude Code 支持两种认证方式：

#### 方式 1: OAuth2 认证

##### 关键配置
```rust
const ANTHROPIC_AUTH_URL: &str = "https://claude.ai/oauth/authorize";
const ANTHROPIC_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const REDIRECT_URI: &str = "http://localhost:54545/callback";
const CALLBACK_PORT: u16 = 54545;  // 动态分配
```

##### OAuth Scopes
```rust
const SCOPES: &[&str] = &[
    "org:create_api_key",  // 创建 API Key
    "user:profile",        // 用户信息
    "user:inference"       // 推理权限
];
```

##### 认证步骤

1. **生成 PKCE 代码**
   - 使用与 Codex 相同的 PKCE 流程
   - `code_challenge_method`: S256

2. **启动本地回调服务器**
   - 支持动态端口分配
   - 默认端口: `54545`
   - 超时时间: 5 分钟

3. **构建授权 URL**
   ```
   https://claude.ai/oauth/authorize?
     code=true
     &client_id={CLIENT_ID}
     &response_type=code
     &redirect_uri=http://localhost:54545/callback
     &scope=org:create_api_key+user:profile+user:inference
     &code_challenge={CHALLENGE}
     &code_challenge_method=S256
     &state={STATE}
   ```

4. **接收回调**
   - 回调可能包含 code 分段（使用 `#` 分隔）
   - 需要解析: `code#state`

5. **交换令牌**
   ```http
   POST https://console.anthropic.com/v1/oauth/token
   Content-Type: application/json

   {
     "code": "{AUTH_CODE}",
     "state": "{STATE}",
     "grant_type": "authorization_code",
     "client_id": "{CLIENT_ID}",
     "redirect_uri": "{REDIRECT_URI}",
     "code_verifier": "{VERIFIER}"
   }
   ```

   响应:
   ```json
   {
     "access_token": "sk-ant-oat...",
     "refresh_token": "...",
     "token_type": "bearer",
     "expires_in": 3600,
     "organization": {
       "uuid": "...",
       "name": "..."
     },
     "account": {
       "uuid": "...",
       "email_address": "user@example.com"
     }
   }
   ```

#### 方式 2: API Key 认证

```rust
// 直接使用 Anthropic API Key
const API_KEY: &str = "sk-ant-api03-...";
```

- API Key 以 `sk-ant-api` 开头
- OAuth Token 以 `sk-ant-oat` 开头
- 通过前缀区分认证方式

### 1.2 认证状态维持

#### Token 存储结构
```rust
pub struct ClaudeTokenStorage {
    pub access_token: String,
    pub refresh_token: String,
    pub email: String,
    pub last_refresh: String,  // RFC3339
    pub expire: String,         // RFC3339
}
```

#### 文件持久化
- **存储位置**: `{auth_dir}/claude-{email}.json`
- **文件格式**: JSON
- **文件名规则**: `claude-` + 用户邮箱

#### Token 刷新机制

1. **触发条件**
   - 距离过期时间 < 5 分钟
   - 或 `access_token` 已失效

2. **刷新流程**
   ```http
   POST https://console.anthropic.com/v1/oauth/token
   Content-Type: application/json

   {
     "client_id": "{CLIENT_ID}",
     "grant_type": "refresh_token",
     "refresh_token": "{REFRESH_TOKEN}"
   }
   ```

3. **重试策略**
   - 最大重试次数: 3
   - 重试间隔: 指数退避 (1s, 2s, 3s)

4. **状态更新**
   ```rust
   auth.metadata["access_token"] = new_token.access_token;
   auth.metadata["refresh_token"] = new_token.refresh_token;
   auth.metadata["email"] = new_token.email;
   auth.metadata["expired"] = expiry_time;
   auth.metadata["last_refresh"] = now_rfc3339;
   auth.metadata["type"] = "claude";
   ```

## 2. 额度查询 (Quota/Usage)

### 2.1 OAuth Usage API（额度查询）

#### 认证文件位置
```
~/.cli-proxy-api/claude-*.json
```

#### API 调用

**端点**:
```
GET https://api.anthropic.com/api/oauth/usage
```

**Headers**:
```http
Authorization: Bearer {access_token}
Accept: application/json
anthropic-beta: oauth-2025-04-20
```

**响应示例**:
```json
{
  "five_hour": {
    "utilization": 75.5,
    "resets_at": "2025-01-18T12:00:00Z"
  },
  "seven_day": {
    "utilization": 45.2,
    "resets_at": "2025-01-25T00:00:00Z"
  },
  "seven_day_sonnet": {
    "utilization": 30.0,
    "resets_at": "2025-01-25T00:00:00Z"
  },
  "seven_day_opus": {
    "utilization": 60.0,
    "resets_at": "2025-01-25T00:00:00Z"
  },
  "extra_usage": {
    "is_enabled": true,
    "monthly_limit": 2000,
    "used_credits": 150,
    "utilization": 7.5
  }
}
```

#### 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `utilization` | float | 已使用百分比（0-100） |
| `resets_at` | string | ISO8601 重置时间 |

#### 配额类型

- **five_hour**: 5小时滚动窗口配额
- **seven_day**: 7天总配额
- **seven_day_sonnet**: Sonnet 模型专用配额
- **seven_day_opus**: Opus 模型专用配额
- **extra_usage**: 额外/付费配额

#### 错误处理

| Status | 处理方式 |
|--------|---------|
| 200 | 解析配额数据 |
| 401 | 需要重新认证（token 约1小时过期，无 refresh token） |
| 403 | 配额已耗尽 |
| 5xx | 使用缓存数据 |

#### 使用示例

```rust
pub async fn fetch_claude_quota(access_token: &str) -> Result<ClaudeQuotaInfo> {
    let response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await?;

    let data: ClaudeUsageResponse = response.json().await?;

    // 转换 utilization (已使用) 为 remaining (剩余)
    Ok(ClaudeQuotaInfo {
        five_hour_remaining: 100.0 - data.five_hour.utilization,
        seven_day_remaining: 100.0 - data.seven_day.utilization,
        // ...
    })
}
```

#### 缓存建议

- **TTL**: 5 分钟
- **失败时**: 使用缓存数据
- **不缓存**: 401 认证错误

### 2.2 Token 计数 API

Claude 提供专门的 token 计数端点：

#### 端点
```
POST https://api.anthropic.com/v1/messages/count_tokens?beta=true
```

#### 请求示例
```json
{
  "model": "claude-sonnet-4-5-20250929",
  "system": [
    {"type": "text", "text": "You are a helpful assistant."}
  ],
  "messages": [
    {
      "role": "user",
      "content": "Hello, how are you?"
    }
  ],
  "tools": []
}
```

#### 响应示例
```json
{
  "input_tokens": 25
}
```

### 2.3 使用量统计

#### 从响应中提取
```rust
pub struct ClaudeUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_input_tokens: i64,
    pub cache_read_input_tokens: i64,
}

fn parse_claude_usage(data: &[u8]) -> ClaudeUsage {
    ClaudeUsage {
        input_tokens: data.get("usage.input_tokens").unwrap_or(0),
        output_tokens: data.get("usage.output_tokens").unwrap_or(0),
        cache_creation_input_tokens: data.get("usage.cache_creation_input_tokens").unwrap_or(0),
        cache_read_input_tokens: data.get("usage.cache_read_input_tokens").unwrap_or(0),
    }
}
```

#### 流式响应中的使用量
```rust
fn parse_claude_stream_usage(line: &[u8]) -> Option<ClaudeUsage> {
    // 从 SSE 事件中提取
    // event: message_delta
    // data: {"type":"message_delta","delta":{},"usage":{"output_tokens":123}}
    if line.starts_with(b"data:") {
        let json = &line[5..].trim();
        if json.get("usage").exists() {
            return Some(parse_usage(json));
        }
    }
    None
}
```

## 3. 模型使用和暴露

### 3.1 API 端点

#### 基础 URL
```rust
const BASE_URL: &str = "https://api.anthropic.com";
```

#### 可用端点
```rust
pub enum ClaudeEndpoint {
    Messages,           // /v1/messages
    CountTokens,        // /v1/messages/count_tokens
    MessagesBeta,       // /v1/messages?beta=true
}
```

### 3.2 请求格式

#### Headers

```rust
fn apply_claude_headers(
    req: &mut Request,
    token: &str,
    is_api_key: bool,
    stream: bool,
    extra_betas: &[&str]
) {
    // 认证头
    if is_api_key {
        req.header("x-api-key", token);
    } else {
        req.header("Authorization", format!("Bearer {}", token));
    }

    req.header("Content-Type", "application/json");

    // Beta 功能
    let mut betas = vec![
        "claude-code-20250219",
        "oauth-2025-04-20",
        "interleaved-thinking-2025-05-14",
        "fine-grained-tool-streaming-2025-05-14"
    ];
    betas.extend(extra_betas);
    req.header("Anthropic-Beta", betas.join(","));

    // API 版本
    req.header("Anthropic-Version", "2023-06-01");
    req.header("Anthropic-Dangerous-Direct-Browser-Access", "true");

    // CLI 标识
    req.header("X-App", "cli");
    req.header("X-Stainless-Helper-Method", "stream");
    req.header("X-Stainless-Retry-Count", "0");
    req.header("X-Stainless-Runtime-Version", "v24.3.0");
    req.header("X-Stainless-Package-Version", "0.55.1");
    req.header("X-Stainless-Runtime", "node");
    req.header("X-Stainless-Lang", "js");
    req.header("X-Stainless-Arch", "arm64");
    req.header("X-Stainless-Os", "MacOS");
    req.header("X-Stainless-Timeout", "60");
    req.header("User-Agent", "claude-cli/1.0.83 (external, cli)");
    req.header("Connection", "keep-alive");
    req.header("Accept-Encoding", "gzip, deflate, br, zstd");

    if stream {
        req.header("Accept", "text/event-stream");
    } else {
        req.header("Accept", "application/json");
    }
}
```

#### 请求体结构
```json
{
  "model": "claude-sonnet-4-5-20250929",
  "max_tokens": 4096,
  "system": [
    {
      "type": "text",
      "text": "You are Claude Code, Anthropic's official CLI for Claude."
    }
  ],
  "messages": [
    {
      "role": "user",
      "content": "Hello"
    }
  ],
  "tools": [],
  "stream": true,
  "thinking": {
    "type": "enabled",
    "budget_tokens": 10000
  }
}
```

#### 关键字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `model` | string | 模型 ID |
| `max_tokens` | integer | 最大输出 token 数 |
| `system` | array | 系统指令（支持多段） |
| `messages` | array | 对话消息 |
| `tools` | array | 工具定义 |
| `stream` | boolean | 是否流式响应 |
| `thinking` | object | 扩展思考配置 |
| `temperature` | float | 温度参数 |
| `top_p` | float | 核采样参数 |
| `top_k` | integer | Top-K 采样 |

### 3.3 响应格式

#### 非流式响应
```json
{
  "id": "msg_01...",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "Hello! How can I assist you today?"
    }
  ],
  "model": "claude-sonnet-4-5-20250929",
  "stop_reason": "end_turn",
  "stop_sequence": null,
  "usage": {
    "input_tokens": 10,
    "output_tokens": 15
  }
}
```

#### 流式响应 (SSE)
```
event: message_start
data: {"type":"message_start","message":{"id":"msg_01...","model":"claude-sonnet-4-5-20250929","role":"assistant","content":[],"usage":{"input_tokens":10,"output_tokens":0}}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"!"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":15}}

event: message_stop
data: {"type":"message_stop"}
```

### 3.4 OAuth Token 工具前缀处理

OAuth Token (sk-ant-oat...) 需要在工具名称前添加 `proxy_` 前缀：

#### 请求处理
```rust
fn apply_claude_tool_prefix(body: &mut Vec<u8>, prefix: &str) {
    // 工具定义
    if let Some(tools) = body.get_mut("tools") {
        for tool in tools.iter_mut() {
            if let Some(name) = tool.get("name") {
                if !name.starts_with(prefix) {
                    tool["name"] = format!("{}{}", prefix, name);
                }
            }
        }
    }

    // tool_choice
    if let Some(tool_choice) = body.get_mut("tool_choice") {
        if tool_choice["type"] == "tool" {
            if let Some(name) = tool_choice.get("name") {
                if !name.starts_with(prefix) {
                    tool_choice["name"] = format!("{}{}", prefix, name);
                }
            }
        }
    }

    // 消息中的 tool_use
    if let Some(messages) = body.get_mut("messages") {
        for message in messages.iter_mut() {
            if let Some(content) = message.get_mut("content") {
                for part in content.iter_mut() {
                    if part["type"] == "tool_use" {
                        if let Some(name) = part.get("name") {
                            if !name.starts_with(prefix) {
                                part["name"] = format!("{}{}", prefix, name);
                            }
                        }
                    }
                }
            }
        }
    }
}
```

#### 响应处理
```rust
fn strip_claude_tool_prefix(body: &mut Vec<u8>, prefix: &str) {
    if let Some(content) = body.get_mut("content") {
        for part in content.iter_mut() {
            if part["type"] == "tool_use" {
                if let Some(name) = part.get("name") {
                    if name.starts_with(prefix) {
                        part["name"] = name.strip_prefix(prefix);
                    }
                }
            }
        }
    }
}
```

### 3.5 Thinking (扩展思考) 支持

#### 配置
```json
{
  "thinking": {
    "type": "enabled",
    "budget_tokens": 10000
  }
}
```

#### 约束条件

1. **tool_choice 限制**
   - 当 `tool_choice.type` 为 "any" 或 "tool" 时，必须禁用 thinking
   - "auto" 可以与 thinking 一起使用

2. **max_tokens 要求**
   - `max_tokens` 必须 > `thinking.budget_tokens`
   - 如果不满足，自动调整 `max_tokens` 到模型的 `MaxCompletionTokens`

```rust
fn ensure_max_tokens_for_thinking(body: &mut Vec<u8>, model: &str) {
    let thinking_type = body.get("thinking.type");
    if thinking_type != "enabled" {
        return;
    }

    let budget = body.get("thinking.budget_tokens").unwrap_or(0);
    let max_tokens = body.get("max_tokens").unwrap_or(0);

    if budget > 0 && max_tokens < budget {
        // 查询模型注册表
        let model_info = registry::lookup_model_info(model);
        let required_max = model_info.max_completion_tokens;
        body.set("max_tokens", required_max);
    }
}
```

### 3.6 响应压缩支持

Claude API 支持多种压缩格式：

```rust
pub enum CompressionFormat {
    Gzip,     // gzip
    Deflate,  // deflate
    Brotli,   // br
    Zstd,     // zstd
}

fn decode_response_body(
    body: impl Read,
    content_encoding: &str
) -> Result<Box<dyn Read>> {
    match content_encoding {
        "gzip" => Ok(Box::new(GzipDecoder::new(body))),
        "deflate" => Ok(Box::new(DeflateDecoder::new(body))),
        "br" => Ok(Box::new(BrotliDecoder::new(body))),
        "zstd" => Ok(Box::new(ZstdDecoder::new(body)?)),
        _ => Ok(Box::new(body)),
    }
}
```

## 4. 错误处理

### 4.1 认证错误

```rust
pub enum ClaudeAuthError {
    InvalidCode,          // 无效的授权码
    TokenExpired,         // Token 过期
    RefreshFailed,        // 刷新失败
    InvalidCredentials,   // 无效凭证
}
```

### 4.2 API 错误

标准 HTTP 状态码：
```rust
match status {
    200..=299 => Ok(response),
    400 => Err("Invalid request format"),
    401 => Err("Invalid API key or expired token"),
    403 => Err("Permission denied"),
    429 => Err("Rate limited - too many requests"),
    500 => Err("Server error"),
    529 => Err("Service overloaded"),
    _ => Err("Unknown error"),
}
```

## 5. Rust 实现建议

### 5.1 依赖项
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream", "gzip", "brotli", "deflate"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.21"
sha2 = "0.10"
uuid = { version = "1.0", features = ["v4"] }
flate2 = "1.0"        # gzip/deflate
brotli = "3.3"        # brotli
zstd = "0.13"         # zstd
async-stream = "0.3"  # 流式处理
```

### 5.2 核心结构

```rust
pub struct ClaudeClient {
    http_client: reqwest::Client,
    config: ClaudeConfig,
    auth: Arc<RwLock<ClaudeAuth>>,
}

pub struct ClaudeConfig {
    pub client_id: String,
    pub base_url: String,
    pub api_key: Option<String>,  // API Key 模式
}

pub enum ClaudeAuth {
    ApiKey(String),
    OAuth(ClaudeOAuthToken),
}

pub struct ClaudeOAuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
}

impl ClaudeClient {
    pub async fn new(config: ClaudeConfig) -> Result<Self>;
    pub async fn authenticate_oauth(&self) -> Result<()>;
    pub async fn refresh_token(&self) -> Result<()>;
    pub async fn execute(&self, req: ClaudeRequest) -> Result<ClaudeResponse>;
    pub async fn execute_stream(&self, req: ClaudeRequest) -> Result<ClaudeStreamResponse>;
    pub async fn count_tokens(&self, req: ClaudeRequest) -> Result<i64>;
}
```

### 5.3 请求构建

```rust
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: i32,
    pub system: Vec<SystemMessage>,
    pub messages: Vec<Message>,
    pub tools: Vec<Tool>,
    pub thinking: Option<ThinkingConfig>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
}

pub struct SystemMessage {
    pub r#type: String,  // "text"
    pub text: String,
}

pub struct Message {
    pub role: String,  // "user" | "assistant"
    pub content: MessageContent,
}

pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

pub struct ContentPart {
    pub r#type: String,  // "text" | "image" | "tool_use" | "tool_result"
    // ... 其他字段
}

pub struct ThinkingConfig {
    pub r#type: String,  // "enabled"
    pub budget_tokens: i64,
}
```

### 5.4 流式响应处理

```rust
use async_stream::stream;
use futures_core::stream::Stream;

pub async fn execute_stream(
    &self,
    req: ClaudeRequest
) -> Result<impl Stream<Item = Result<ClaudeStreamEvent>>> {
    let response = self.http_client
        .post(format!("{}/v1/messages", self.config.base_url))
        .json(&req)
        .send()
        .await?;

    let stream = stream! {
        let mut lines = response.bytes_stream();
        let mut buffer = Vec::new();

        while let Some(chunk) = lines.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);

            // 解析 SSE
            while let Some(idx) = buffer.iter().position(|&b| b == b'\n') {
                let line = buffer.drain(..=idx).collect::<Vec<_>>();

                if line.starts_with(b"event: ") {
                    let event_type = &line[7..line.len()-1];
                    // 读取下一行的 data
                    continue;
                }

                if line.starts_with(b"data: ") {
                    let data = &line[6..line.len()-1];
                    let event: ClaudeStreamEvent = serde_json::from_slice(data)?;
                    yield Ok(event);
                }
            }
        }
    };

    Ok(stream)
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClaudeStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: Message },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: usize, content_block: ContentBlock },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: Delta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDeltaData, usage: Usage },
    #[serde(rename = "message_stop")]
    MessageStop,
}
```

## 6. 安全注意事项

1. **Token 识别**
   - API Key: `sk-ant-api03-...`
   - OAuth Token: `sk-ant-oat-...`
   - 根据前缀选择认证方式

2. **工具前缀**
   - OAuth Token 必须添加 `proxy_` 前缀
   - 响应时需要移除前缀

3. **Beta 功能**
   - 某些功能需要特定 beta 头
   - `oauth-2025-04-20` 用于 OAuth 支持
   - `claude-code-20250219` 用于 CLI 特性

4. **压缩支持**
   - 请求头必须包含 `Accept-Encoding`
   - 正确解码响应的 `Content-Encoding`

## 7. 测试建议

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oauth_authentication() {
        let client = ClaudeClient::new(config).await.unwrap();
        let result = client.authenticate_oauth().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tool_prefix() {
        let mut body = json!({
            "tools": [{"name": "get_weather", "description": "..."}]
        });
        apply_claude_tool_prefix(&mut body, "proxy_");
        assert_eq!(body["tools"][0]["name"], "proxy_get_weather");
    }

    #[tokio::test]
    async fn test_stream_parsing() {
        let sse_data = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n";
        // 测试 SSE 解析
    }

    #[test]
    fn test_compression() {
        let data = b"Hello, world!";
        let compressed = gzip_compress(data);
        let decompressed = gzip_decompress(&compressed);
        assert_eq!(data, &decompressed[..]);
    }
}
```
