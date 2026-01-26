# Codex (OpenAI Codex) Agent 设计文档

## 概述

Codex Agent 是 OpenAI ChatGPT 的编程接口实现，使用 OpenAI 的 OAuth2 认证流程，通过 ChatGPT Backend API 与 Codex 模型进行交互。

## 1. 认证机制 (Authentication)

### 1.1 认证流程

Codex 使用 **OAuth2 + PKCE** (Proof Key for Code Exchange) 认证流程：

#### 关键配置
```rust
const OPENAI_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const OPENAI_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const OPENAI_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const CALLBACK_PORT: u16 = 1455;
const ORIGINATOR: &str = "codex_cli_rs";
```

#### OAuth Scopes
```rust
const SCOPES: &[&str] = &[
    "openid",
    "email",
    "profile",
    "offline_access"
];
```

#### 认证步骤

1. **生成 PKCE 代码**
   - 生成 `code_verifier` (随机字符串)
   - 计算 `code_challenge` = BASE64URL(SHA256(code_verifier))
   - 使用 S256 方法

2. **启动本地回调服务器**
   - 监听端口: `1455`
   - 回调路径: `/auth/callback`
   - 超时时间: 5 分钟

3. **构建授权 URL**
   ```
   https://auth.openai.com/oauth/authorize?
     client_id={CLIENT_ID}
     &response_type=code
     &redirect_uri=http://localhost:1455/auth/callback
     &scope=openid+email+profile+offline_access
     &state={RANDOM_STATE}
     &code_challenge={CHALLENGE}
     &code_challenge_method=S256
     &id_token_add_organizations=true
     &codex_cli_simplified_flow=true
     &originator=codex_cli_rs
   ```
   - `originator=codex_cli_rs` 必须包含，否则可能出现 token 交换 403

4. **打开浏览器**
   - 自动打开系统默认浏览器
   - 如果无浏览器，提供 SSH 隧道指导

5. **接收回调**
   - 等待用户在浏览器中完成认证
   - 提供手动输入回调 URL 的选项（15秒后）
   - 验证 `state` 参数防止 CSRF

6. **交换令牌**
   ```http
   POST https://auth.openai.com/oauth/token
   Content-Type: application/x-www-form-urlencoded

   grant_type=authorization_code
   &client_id={CLIENT_ID}
   &code={AUTH_CODE}
   &redirect_uri={REDIRECT_URI}
   &code_verifier={VERIFIER}
   ```

   响应:
   ```json
   {
     "access_token": "...",
     "refresh_token": "...",
     "id_token": "...",
     "token_type": "Bearer",
     "expires_in": 3600
   }
   ```

7. **解析 ID Token (JWT)**
   - 从 `id_token` 提取 `account_id` 和 `email`
   - 无需验证签名（信任来源）

### 1.2 认证状态维持

#### Token 存储结构
```rust
pub struct CodexTokenStorage {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub account_id: String,
    pub email: String,
    pub last_refresh: String,     // RFC3339 timestamp
    pub expire: String,            // RFC3339 timestamp
}
```

#### 文件持久化
- **存储位置**: `{auth_dir}/codex-{email}.json`
- **文件格式**: JSON
- **文件名规则**: `codex-` + 用户邮箱

#### Token 刷新机制

1. **触发条件**
   - 距离过期时间 < 5 天
   - 或 `access_token` 已过期

2. **刷新流程**
   ```http
   POST https://auth.openai.com/oauth/token
   Content-Type: application/x-www-form-urlencoded

   client_id={CLIENT_ID}
   &grant_type=refresh_token
   &refresh_token={REFRESH_TOKEN}
   &scope=openid+profile+email
   ```

3. **重试策略**
   - 最大重试次数: 3
   - 重试间隔: 指数退避 (1s, 2s, 3s)
   - 失败后返回错误

4. **状态更新**
   ```rust
   auth.metadata["access_token"] = new_token.access_token;
   auth.metadata["refresh_token"] = new_token.refresh_token; // 如果返回
   auth.metadata["id_token"] = new_token.id_token;
   auth.metadata["expired"] = expiry_time;
   auth.metadata["last_refresh"] = now_rfc3339;
   ```

## 2. 额度查询 (Quota/Usage)

### 2.1 Usage API（额度查询）

#### 认证文件位置
```
~/.codex/auth.json
```

#### API 调用

**端点**:
```
GET https://chatgpt.com/backend-api/wham/usage
```

**Headers**:
```http
Authorization: Bearer {access_token}
ChatGPT-Account-Id: {account_id}
Accept: application/json
```

**响应示例**:
```json
{
  "plan_type": "plus",
  "rate_limit": {
    "limit_reached": false,
    "primary_window": {
      "used_percent": 45,
      "reset_at": 1737201600
    },
    "secondary_window": {
      "used_percent": 20,
      "reset_at": 1737806400
    }
  }
}
```

#### 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `plan_type` | string | 订阅计划（free, plus, pro, team, enterprise） |
| `limit_reached` | boolean | 是否达到限制 |
| `primary_window.used_percent` | int | 主窗口已使用百分比（0-100） |
| `primary_window.reset_at` | int | Unix 时间戳（秒） |
| `secondary_window.used_percent` | int | 次窗口已使用百分比（0-100） |
| `secondary_window.reset_at` | int | Unix 时间戳（秒） |

#### 配额窗口说明

- **Primary Window**: 3小时滚动会话窗口
- **Secondary Window**: 每周配额

#### Token 刷新

当 access_token 过期时（401），使用 refresh_token 刷新：

```http
POST https://auth.openai.com/oauth/token
Content-Type: application/x-www-form-urlencoded

client_id=app_EMoamEEZ73f0CkXaXp7hrann
&grant_type=refresh_token
&refresh_token={REFRESH_TOKEN}
```

#### 错误处理

| Status | 处理方式 |
|--------|---------|
| 200 | 解析配额数据 |
| 401 | 使用 refresh_token 刷新 |
| 403 | 配额已耗尽 |
| 429 | 达到速率限制 |
| 5xx | 使用缓存数据 |

#### 使用示例

```rust
pub async fn fetch_codex_quota(
    access_token: &str,
    account_id: &str
) -> Result<CodexQuotaInfo> {
    let response = client
        .get("https://chatgpt.com/backend-api/wham/usage")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("ChatGPT-Account-Id", account_id)
        .send()
        .await?;

    let data: CodexUsageResponse = response.json().await?;

    // 转换 used_percent (已使用) 为 remaining (剩余)
    Ok(CodexQuotaInfo {
        plan_type: data.plan_type,
        session_remaining: 100 - data.rate_limit.primary_window.used_percent,
        weekly_remaining: 100 - data.rate_limit.secondary_window.used_percent,
        // ...
    })
}
```

### 2.2 Token 计数

Codex 提供本地 token 计数功能（不依赖 API）：

#### 实现方式
```rust
// 使用 tiktoken-rs 进行本地计数
fn count_tokens(model: &str, payload: &[u8]) -> Result<i64> {
    let encoder = get_tokenizer_for_model(model)?;
    let count = count_codex_input_tokens(&encoder, payload)?;
    Ok(count)
}
```

#### 支持的模型
```rust
fn tokenizer_for_codex_model(model: &str) -> Tokenizer {
    match model.to_lowercase() {
        m if m.starts_with("gpt-5") => Tokenizer::GPT5,
        m if m.starts_with("gpt-4.1") => Tokenizer::GPT41,
        m if m.starts_with("gpt-4o") => Tokenizer::GPT4o,
        m if m.starts_with("gpt-4") => Tokenizer::GPT4,
        m if m.starts_with("gpt-3.5") => Tokenizer::GPT35Turbo,
        _ => Tokenizer::Cl100kBase, // 默认
    }
}
```

#### 计数字段
- `instructions`: 系统指令
- `input[].content[].text`: 消息文本
- `input[].function_call`: 函数调用
- `input[].function_call_output`: 函数输出
- `tools[].name/description/parameters`: 工具定义
- `text.format.schema`: 结构化输出 schema

### 2.2 使用量统计

响应中包含使用量信息：

```json
{
  "type": "response.completed",
  "response": {
    "usage": {
      "input_tokens": 1234,
      "output_tokens": 567,
      "total_tokens": 1801
    }
  }
}
```

**提取逻辑**:
```rust
fn parse_codex_usage(data: &[u8]) -> UsageDetail {
    UsageDetail {
        input_tokens: data.get("response.usage.input_tokens"),
        output_tokens: data.get("response.usage.output_tokens"),
        total_tokens: data.get("response.usage.total_tokens"),
    }
}
```

## 3. 模型使用和暴露

### 3.1 API 端点

#### 基础 URL
```rust
const BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
```

#### 可用端点
```rust
pub enum CodexEndpoint {
    Responses,  // /responses - 生成响应
}
```

**完整 URL**: `https://chatgpt.com/backend-api/codex/responses`

### 3.2 请求格式

#### Headers
```rust
fn apply_codex_headers(req: &mut Request, token: &str, auth: &Auth) {
    req.header("Content-Type", "application/json");
    req.header("Authorization", format!("Bearer {}", token));
    req.header("Version", "0.21.0");
    req.header("Openai-Beta", "responses=experimental");
    req.header("Session_id", uuid::new_v4());
    req.header("User-Agent", "codex_cli_rs/0.50.0 (Mac OS 26.0.1; arm64) Apple_Terminal/464");
    req.header("Accept", "text/event-stream");
    req.header("Connection", "Keep-Alive");

    // 仅在使用 OAuth token 时添加
    if !is_api_key {
        req.header("Originator", "codex_cli_rs");
        req.header("Chatgpt-Account-Id", account_id);
    }

    // 缓存头
    req.header("Conversation_id", cache_id);
    req.header("Session_id", cache_id);
}
```

#### 请求体结构
```json
{
  "model": "gpt-4o",
  "stream": true,
  "instructions": "",
  "input": [
    {
      "type": "message",
      "role": "user",
      "content": [
        {"type": "text", "text": "Hello"}
      ]
    }
  ],
  "tools": [],
  "prompt_cache_key": "{uuid}"
}
```

#### 关键字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `model` | string | 模型 ID |
| `stream` | boolean | 是否流式响应 |
| `instructions` | string | 系统指令 |
| `input` | array | 输入消息列表 |
| `tools` | array | 可用工具定义 |
| `prompt_cache_key` | string | 提示缓存键（用于多轮对话） |

### 3.3 响应格式

#### 流式响应 (SSE)
```
data: {"type":"response.started","response_id":"..."}

data: {"type":"response.content_part.delta","delta":{"text":"Hello"}}

data: {"type":"response.content_part.delta","delta":{"text":" world"}}

data: {"type":"response.completed","response":{"usage":{"input_tokens":10,"output_tokens":5}}}
```

#### 事件类型
```rust
pub enum CodexEventType {
    ResponseStarted,           // 响应开始
    ContentPartDelta,          // 内容增量
    FunctionCallDelta,         // 函数调用增量
    FunctionCallOutput,        // 函数调用输出
    ResponseCompleted,         // 响应完成（包含 usage）
}
```

### 3.4 会话缓存机制

#### Claude 格式缓存
```rust
// 从 Claude metadata.user_id 生成缓存键
let cache_key = format!("{}-{}", model, user_id);
let cache = get_or_create_cache(cache_key);

// 缓存结构
struct CodexCache {
    id: String,          // UUID
    expire: Instant,     // 1 小时后过期
}
```

#### OpenAI 格式缓存
```rust
// 直接使用 prompt_cache_key
let cache_id = payload.get("prompt_cache_key");
```

#### 缓存头部设置
```rust
req.header("Conversation_id", cache_id);
req.header("Session_id", cache_id);
req.header("prompt_cache_key", cache_id);
```

### 3.5 Thinking (思考) 模式支持

Codex 支持扩展思考预算：

```rust
fn apply_thinking(payload: &mut Vec<u8>, model: &str) -> Result<()> {
    if model.ends_with("-thinking") || model.ends_with("-extended-thinking") {
        // 应用思考预算配置
        thinking::apply_thinking_provider_codex(payload, model)?;
    }
    Ok(())
}
```

## 4. 错误处理

### 4.1 认证错误

```rust
pub enum CodexAuthError {
    PortInUse,              // 端口被占用
    ServerStartFailed,      // 服务器启动失败
    CallbackTimeout,        // 回调超时
    InvalidState,           // State 不匹配
    CodeExchangeFailed,     // 代码交换失败
    BrowserOpenFailed,      // 浏览器打开失败
}
```

### 4.2 API 错误

状态码处理：
```rust
match status {
    200..=299 => Ok(response),
    400 => Err("Bad Request"),
    401 => Err("Unauthorized - token expired"),
    429 => Err("Rate Limited"),
    500..=599 => Err("Server Error"),
    _ => Err("Unknown Error"),
}
```

## 5. Rust 实现建议

### 5.1 依赖项
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.21"
sha2 = "0.10"
uuid = { version = "1.0", features = ["v4"] }
rand = "0.8"
jsonwebtoken = "9"  # 用于解析 JWT
tiktoken-rs = "0.5"  # Token 计数
```

### 5.2 核心结构

```rust
pub struct CodexClient {
    http_client: reqwest::Client,
    config: CodexConfig,
    auth_storage: Arc<RwLock<CodexTokenStorage>>,
}

pub struct CodexConfig {
    pub client_id: String,
    pub redirect_uri: String,
    pub callback_port: u16,
    pub base_url: String,
}

impl CodexClient {
    pub async fn authenticate(&self) -> Result<CodexAuth>;
    pub async fn refresh_token(&self) -> Result<()>;
    pub async fn execute(&self, req: CodexRequest) -> Result<CodexResponse>;
    pub async fn execute_stream(&self, req: CodexRequest) -> Result<StreamResponse>;
    pub async fn count_tokens(&self, req: CodexRequest) -> Result<i64>;
}
```

### 5.3 PKCE 实现

```rust
use rand::Rng;
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub struct PkceCodes {
    pub code_verifier: String,
    pub code_challenge: String,
}

pub fn generate_pkce_codes() -> Result<PkceCodes> {
    // 生成 code_verifier (43-128 字符)
    let verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    // 计算 code_challenge = BASE64URL(SHA256(verifier))
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let challenge = URL_SAFE_NO_PAD.encode(&hash);

    Ok(PkceCodes {
        code_verifier: verifier,
        code_challenge: challenge,
    })
}
```

### 5.4 JWT 解析

```rust
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    #[serde(rename = "https://api.openai.com/auth")]
    openai_auth: OpenAIAuth,
    email: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIAuth {
    organizations: Vec<Organization>,
}

pub fn parse_jwt_token(id_token: &str) -> Result<(String, String)> {
    // 仅解码，不验证签名
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format");
    }

    // 解码 payload
    let payload = base64::decode(parts[1])?;
    let claims: IdTokenClaims = serde_json::from_slice(&payload)?;

    let account_id = claims.openai_auth.organizations
        .first()
        .map(|org| org.id.clone())
        .unwrap_or_default();

    Ok((account_id, claims.email))
}
```

## 6. 安全注意事项

1. **Token 存储安全**
   - 使用系统密钥链 (macOS Keychain, Windows Credential Manager)
   - 或加密存储敏感字段

2. **PKCE 实现**
   - `code_verifier` 必须随机生成
   - 使用 SHA-256 计算 challenge
   - 使用 Base64URL 编码（无填充）

3. **State 验证**
   - 必须验证回调中的 state 参数
   - 防止 CSRF 攻击

4. **Token 刷新**
   - 提前 5 天刷新 token
   - 刷新失败后应提示用户重新登录

5. **网络安全**
   - 支持代理配置
   - 使用 HTTPS
   - 验证 SSL 证书

## 7. 测试建议

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pkce_generation() {
        let codes = generate_pkce_codes().unwrap();
        assert!(codes.code_verifier.len() >= 43);
        assert!(codes.code_challenge.len() > 0);
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let client = CodexClient::new(config);
        // Mock refresh token response
        let result = client.refresh_token().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_jwt_parsing() {
        let id_token = "eyJ...";  // Sample JWT
        let (account_id, email) = parse_jwt_token(id_token).unwrap();
        assert!(!account_id.is_empty());
        assert!(email.contains('@'));
    }
}
```
