# Gemini CLI (Google Cloud Code Assist) Agent 设计文档

## 概述

Gemini CLI Agent 是 Google Cloud Code Assist 的命令行接口实现，使用 Google OAuth2 认证，通过 Cloud Code Assist API 访问 Gemini 模型。该实现与 Google Cloud Platform 深度集成，需要 GCP 项目支持。

## 1. 认证机制 (Authentication)

### 1.1 OAuth2 认证流程

Gemini CLI 使用标准 Google OAuth2 认证：

#### 关键配置
```rust
const GEMINI_OAUTH_CLIENT_ID: &str = "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com";
const GEMINI_OAUTH_CLIENT_SECRET: &str = "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";
const GEMINI_CALLBACK_PORT: u16 = 8085;  // 默认回调端口
const GEMINI_CALLBACK_PATH: &str = "/oauth2callback";
```

#### OAuth Scopes
```rust
const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile"
];
```

#### 认证步骤

1. **启动本地回调服务器**
   - 监听端口: `8085`（可配置）
   - 回调路径: `/oauth2callback`
   - 超时时间: 5 分钟

2. **构建授权 URL**
   ```
   https://accounts.google.com/o/oauth2/v2/auth?
     client_id={CLIENT_ID}
     &redirect_uri=http://localhost:8085/oauth2callback
     &scope=https://www.googleapis.com/auth/cloud-platform+userinfo.email+userinfo.profile
     &response_type=code
     &state=state-token
     &access_type=offline
     &prompt=consent
   ```

   参数说明：
   - `access_type=offline`: 获取 refresh_token
   - `prompt=consent`: 强制显示同意页面

3. **接收回调**
   - 等待用户在浏览器中完成认证
   - 提供手动输入回调 URL 的选项（15秒后）

4. **交换令牌**
   ```http
   POST https://oauth2.googleapis.com/token
   Content-Type: application/x-www-form-urlencoded

   code={AUTH_CODE}
   &client_id={CLIENT_ID}
   &client_secret={CLIENT_SECRET}
   &redirect_uri={REDIRECT_URI}
   &grant_type=authorization_code
   ```

   响应:
   ```json
   {
     "access_token": "ya29.a0...",
     "refresh_token": "1//0g...",
     "expires_in": 3600,
     "token_type": "Bearer",
     "scope": "..."
   }
   ```

5. **获取用户信息**
   ```http
   GET https://www.googleapis.com/oauth2/v1/userinfo?alt=json
   Authorization: Bearer {ACCESS_TOKEN}
   ```

   响应:
   ```json
   {
     "id": "...",
     "email": "user@gmail.com",
     "verified_email": true,
     "name": "User Name",
     "picture": "https://..."
   }
   ```

### 1.2 认证状态维持

#### Token 存储结构
```rust
pub struct GeminiTokenStorage {
    pub token: HashMap<String, Value>,  // 完整的 OAuth2 token
    pub project_id: String,              // GCP 项目 ID
    pub email: String,                   // 用户邮箱
}

// token 字段包含：
// {
//   "access_token": "...",
//   "refresh_token": "...",
//   "expiry": "2025-01-19T...",
//   "token_type": "Bearer",
//   "token_uri": "https://oauth2.googleapis.com/token",
//   "client_id": "...",
//   "client_secret": "...",
//   "scopes": [...],
//   "universe_domain": "googleapis.com"
// }
```

#### 文件持久化
- **存储位置**: `{auth_dir}/gemini-cli-{email}.json`
- **文件格式**: JSON
- **文件名规则**: `gemini-cli-` + 用户邮箱

#### Token 刷新机制

1. **自动刷新**
   - 使用 `golang.org/x/oauth2` 的 `TokenSource` 自动管理
   - 在 token 过期前自动刷新

2. **刷新流程**
   ```rust
   pub struct OAuth2Config {
       client_id: String,
       client_secret: String,
       scopes: Vec<String>,
       endpoint: GoogleEndpoint,
   }

   impl OAuth2Config {
       pub fn token_source(&self, ctx: Context, token: &Token) -> TokenSource {
           // 创建自动刷新的 TokenSource
           conf.token_source(ctx, token)
       }
   }

   // TokenSource 会自动：
   // 1. 检查 token 是否过期
   // 2. 如果过期，使用 refresh_token 刷新
   // 3. 返回有效的 access_token
   ```

3. **状态更新**
   ```rust
   fn update_gemini_cli_token_metadata(
       auth: &mut Auth,
       base: &HashMap<String, Value>,
       token: &Token
   ) {
       let merged = merge_token_map(base, token);
       auth.metadata["access_token"] = token.access_token;
       auth.metadata["token_type"] = token.token_type;
       auth.metadata["refresh_token"] = token.refresh_token;
       auth.metadata["expiry"] = token.expiry.to_rfc3339();
       auth.metadata["token"] = merged;
   }
   ```

### 1.3 GCP 项目集成

Gemini CLI 需要关联 GCP 项目：

#### 存储项目 ID
```rust
// 项目 ID 存储在 metadata 中
auth.metadata["project_id"] = project_id;
```

#### 在请求中使用
```rust
fn resolve_gemini_project_id(auth: &Auth) -> String {
    // 从 metadata 中获取
    auth.metadata.get("project_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
}

// 请求体中包含
{
  "project": "{project_id}",
  "model": "gemini-2.5-pro",
  "request": { ... }
}
```

## 2. 额度查询 (Quota/Usage)

### 2.1 配额查询 API

**重要提示**: Gemini CLI 目前**没有公开的配额查询 API**。

#### 认证文件位置
```
~/.gemini/oauth_creds.json
~/.gemini/google_accounts.json
```

#### Token 结构
```json
{
  "id_token": "jwt_token",
  "access_token": "string",
  "refresh_token": "string",
  "expiry_date": 1737201600000
}
```

#### 替代方案

由于没有公开的配额 API，可以通过以下方式获取账户信息：

1. **JWT 解析获取邮箱**

   从 `id_token` 中解码用户信息：

   ```rust
   pub fn parse_gemini_account_info(id_token: &str) -> Result<GeminiAccountInfo> {
       let segments: Vec<&str> = id_token.split('.').collect();
       let payload = base64::decode(segments[1])?;
       let claims: Value = serde_json::from_slice(&payload)?;

       Ok(GeminiAccountInfo {
           email: claims["email"].as_str()?.to_string(),
           name: claims.get("name").and_then(|v| v.as_str()).map(String::from),
       })
   }
   ```

2. **CLI 状态检查**

   检查 Gemini CLI 是否已认证：

   ```rust
   pub async fn check_gemini_cli_status() -> Result<GeminiStatus> {
       let auth_path = home_dir()?.join(".gemini/oauth_creds.json");

       if !auth_path.exists() {
           return Ok(GeminiStatus::NotAuthenticated);
       }

       let auth: GeminiAuthFile = serde_json::from_str(&fs::read_to_string(&auth_path)?)?;

       if Utc::now() > DateTime::from_timestamp_millis(auth.expiry_date)? {
           return Ok(GeminiStatus::TokenExpired);
       }

       Ok(GeminiStatus::Authenticated)
   }
   ```

3. **占位符数据**

   返回占位符配额数据（`-1` 表示未知）：

   ```rust
   pub struct GeminiQuotaPlaceholder {
       pub email: String,
       pub quota_available: i32,  // -1 表示未知
       pub plan_type: String,     // "Google Account"
   }
   ```

### 2.2 Token 计数 API

虽然没有配额查询 API，但提供了 token 计数功能：

#### 端点
```
POST https://cloudcode-pa.googleapis.com/v1internal:countTokens
```

#### 请求示例
```json
{
  "request": {
    "contents": [
      {
        "role": "user",
        "parts": [
          {"text": "Hello, how are you?"}
        ]
      }
    ]
  }
}
```

**注意**: countTokens 请求不包含 `project` 和 `model` 字段。

#### 响应示例
```json
{
  "totalTokens": 25
}
```

### 2.2 使用量统计

#### 从响应中提取
```rust
pub struct GeminiUsage {
    pub total_tokens: i64,
    pub prompt_tokens: i64,
    pub candidates_tokens: i64,
}

fn parse_gemini_cli_usage(data: &[u8]) -> GeminiUsage {
    GeminiUsage {
        total_tokens: data.get("usageMetadata.totalTokenCount").unwrap_or(0),
        prompt_tokens: data.get("usageMetadata.promptTokenCount").unwrap_or(0),
        candidates_tokens: data.get("usageMetadata.candidatesTokenCount").unwrap_or(0),
    }
}
```

#### 流式响应中的使用量
```rust
fn parse_gemini_cli_stream_usage(line: &[u8]) -> Option<GeminiUsage> {
    // SSE 格式: data: {...}
    if line.starts_with(b"data:") {
        let json = &line[5..].trim();
        if json.get("usageMetadata").exists() {
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
const CODE_ASSIST_ENDPOINT: &str = "https://cloudcode-pa.googleapis.com";
const CODE_ASSIST_VERSION: &str = "v1internal";
```

#### 可用端点
```rust
pub enum GeminiCLIEndpoint {
    StreamGenerateContent,  // :streamGenerateContent
    GenerateContent,        // :generateContent
    CountTokens,           // :countTokens
    FetchAvailableModels,  // :fetchAvailableModels
}

// 完整 URL 格式
// https://cloudcode-pa.googleapis.com/v1internal:{action}
```

### 3.2 请求格式

#### Headers
```rust
fn apply_gemini_cli_headers(req: &mut Request) {
    req.header("Content-Type", "application/json");
    req.header("Authorization", format!("Bearer {}", access_token));
    req.header("User-Agent", "google-api-nodejs-client/9.15.1");
    req.header("X-Goog-Api-Client", "gl-node/22.17.0");
    req.header("Client-Metadata", "ideType=IDE_UNSPECIFIED,platform=PLATFORM_UNSPECIFIED,pluginType=GEMINI");

    // 流式请求
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
  "project": "{project_id}",
  "model": "gemini-2.5-pro",
  "request": {
    "contents": [
      {
        "role": "user",
        "parts": [
          {"text": "Hello"}
        ]
      }
    ],
    "generationConfig": {
      "temperature": 1.0,
      "topP": 0.95,
      "topK": 40,
      "maxOutputTokens": 8192
    },
    "safetySettings": [
      {
        "category": "HARM_CATEGORY_HARASSMENT",
        "threshold": "BLOCK_NONE"
      }
    ]
  }
}
```

#### 关键字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `project` | string | GCP 项目 ID |
| `model` | string | 模型 ID |
| `request` | object | Gemini 请求体 |
| `request.contents` | array | 对话内容 |
| `request.generationConfig` | object | 生成配置 |
| `request.safetySettings` | array | 安全设置 |

### 3.3 响应格式

#### 非流式响应
```json
{
  "candidates": [
    {
      "content": {
        "role": "model",
        "parts": [
          {"text": "Hello! How can I assist you today?"}
        ]
      },
      "finishReason": "STOP",
      "safetyRatings": [...]
    }
  ],
  "usageMetadata": {
    "promptTokenCount": 10,
    "candidatesTokenCount": 15,
    "totalTokenCount": 25
  }
}
```

#### 流式响应 (SSE)
```
data: {"candidates":[{"content":{"role":"model","parts":[{"text":"Hello"}]}}]}

data: {"candidates":[{"content":{"role":"model","parts":[{"text":"!"}]}}]}

data: {"candidates":[{"content":{"role":"model","parts":[{"text":" How"}]}}],"usageMetadata":{"promptTokenCount":10,"candidatesTokenCount":3,"totalTokenCount":13}}
```

### 3.4 模型回退机制

#### 预览模型回退顺序
```rust
fn cli_preview_fallback_order(model: &str) -> Vec<String> {
    match model {
        "gemini-2.5-pro" => vec![
            // 可以添加预览版本
            // "gemini-2.5-pro-preview-05-06".to_string(),
        ],
        "gemini-2.5-flash" => vec![
            // "gemini-2.5-flash-preview-04-17".to_string(),
        ],
        "gemini-2.5-flash-lite" => vec![
            // "gemini-2.5-flash-lite-preview-06-17".to_string(),
        ],
        _ => vec![],
    }
}
```

#### 回退逻辑
```rust
async fn execute_with_fallback(
    &self,
    base_model: &str,
    payload: &[u8]
) -> Result<Response> {
    let models = cli_preview_fallback_order(base_model);
    let mut models_to_try = vec![base_model.to_string()];
    models_to_try.extend(models);

    for (idx, model) in models_to_try.iter().enumerate() {
        let result = self.execute_with_model(model, payload).await;

        match result {
            Ok(response) => return Ok(response),
            Err(e) if e.status_code() == 429 => {
                // 速率限制，尝试下一个模型
                if idx + 1 < models_to_try.len() {
                    log::debug!("Rate limited, retrying with {}", models_to_try[idx + 1]);
                    continue;
                }
                return Err(e);
            }
            Err(e) => return Err(e),
        }
    }

    Err(Error::NoModelAvailable)
}
```

### 3.5 图像生成特殊处理

对于 `gemini-2.5-flash-image-preview` 模型：

```rust
fn fix_gemini_cli_image_aspect_ratio(
    model: &str,
    payload: &mut Vec<u8>
) -> Vec<u8> {
    if model != "gemini-2.5-flash-image-preview" {
        return payload.clone();
    }

    let aspect_ratio = payload.get("request.generationConfig.imageConfig.aspectRatio");
    if !aspect_ratio.exists() {
        return payload.clone();
    }

    // 检查是否已有图像输入
    let has_inline_data = check_has_inline_data(payload);

    if !has_inline_data {
        // 创建空白图像作为基础
        let white_image = create_white_image_base64(aspect_ratio.as_str());

        // 插入提示和空白图像
        let instruction = r#"Based on the following requirements, create an image within the uploaded picture. The new content *MUST* completely cover the entire area of the original picture, maintaining its exact proportions, and *NO* blank areas should appear."#;

        // 修改 request.contents[0].parts
        payload.set("request.contents.0.parts.0", json!({"text": instruction}));
        payload.set("request.contents.0.parts.1", json!({
            "inlineData": {
                "mimeType": "image/png",
                "data": white_image
            }
        }));

        // 设置响应模式
        payload.set("request.generationConfig.responseModalities", json!(["IMAGE", "TEXT"]));
    }

    // 移除 imageConfig
    payload.delete("request.generationConfig.imageConfig");

    payload.clone()
}
```

### 3.6 速率限制处理

#### 提取重试延迟
```rust
fn parse_retry_delay(error_body: &[u8]) -> Option<Duration> {
    // 方法 1: 从 RetryInfo 提取
    let details = error_body.get("error.details");
    if let Some(details_array) = details.as_array() {
        for detail in details_array {
            if detail.get("@type") == "type.googleapis.com/google.rpc.RetryInfo" {
                if let Some(retry_delay) = detail.get("retryDelay").as_str() {
                    // 格式: "0.847655010s"
                    return parse_duration(retry_delay);
                }
            }
        }

        // 方法 2: 从 ErrorInfo.metadata.quotaResetDelay 提取
        for detail in details_array {
            if detail.get("@type") == "type.googleapis.com/google.rpc.ErrorInfo" {
                if let Some(quota_reset) = detail.get("metadata.quotaResetDelay").as_str() {
                    return parse_duration(quota_reset);
                }
            }
        }
    }

    // 方法 3: 从错误消息中提取 "Your quota will reset after Xs."
    let message = error_body.get("error.message").as_str()?;
    let re = Regex::new(r"after\s+(\d+)s\.?").ok()?;
    if let Some(captures) = re.captures(message) {
        if let Some(seconds) = captures.get(1).and_then(|m| m.as_str().parse::<u64>().ok()) {
            return Some(Duration::from_secs(seconds));
        }
    }

    None
}
```

## 4. 错误处理

### 4.1 认证错误

```rust
pub enum GeminiAuthError {
    OAuthFlowTimeout,     // OAuth 流程超时
    CallbackFailed,       // 回调失败
    TokenExchangeFailed,  // Token 交换失败
    InvalidProject,       // 无效的 GCP 项目
}
```

### 4.2 API 错误

```rust
match status {
    200..=299 => Ok(response),
    400 => Err("Invalid request"),
    401 => Err("Unauthorized - invalid credentials"),
    403 => Err("Permission denied - check GCP project"),
    404 => Err("Model not found"),
    429 => {
        let retry_after = parse_retry_delay(body);
        Err(RateLimitError { retry_after })
    },
    500..=599 => Err("Server error"),
    _ => Err("Unknown error"),
}
```

### 4.3 速率限制错误

```rust
pub struct RateLimitError {
    pub retry_after: Option<Duration>,
    pub message: String,
}

impl RateLimitError {
    pub fn should_retry(&self) -> bool {
        self.retry_after.is_some()
    }

    pub fn wait_duration(&self) -> Duration {
        self.retry_after.unwrap_or(Duration::from_secs(60))
    }
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
oauth2 = "4.4"           # OAuth2 客户端
base64 = "0.21"
regex = "1.10"
chrono = "0.4"
async-stream = "0.3"
```

### 5.2 核心结构

```rust
use oauth2::{
    AuthUrl, TokenUrl, ClientId, ClientSecret, RedirectUrl,
    AuthorizationCode, TokenResponse, RefreshToken,
    basic::BasicClient,
};
use oauth2::reqwest::async_http_client;

pub struct GeminiCLIClient {
    http_client: reqwest::Client,
    oauth_client: BasicClient,
    token_source: Arc<RwLock<TokenSource>>,
    project_id: String,
}

pub struct TokenSource {
    token: Token,
    oauth_config: BasicClient,
}

impl TokenSource {
    pub async fn token(&mut self) -> Result<String> {
        // 检查是否过期
        if self.token.is_expired() {
            self.refresh().await?;
        }
        Ok(self.token.access_token.clone())
    }

    async fn refresh(&mut self) -> Result<()> {
        let refresh_token = RefreshToken::new(self.token.refresh_token.clone());
        let token_response = self.oauth_config
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await?;

        self.token = Token {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response.refresh_token()
                .map(|t| t.secret().clone())
                .unwrap_or(self.token.refresh_token.clone()),
            expiry: Utc::now() + Duration::seconds(token_response.expires_in()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(3600)),
            token_type: "Bearer".to_string(),
        };

        Ok(())
    }
}

impl GeminiCLIClient {
    pub async fn new(project_id: String) -> Result<Self>;
    pub async fn authenticate(&self) -> Result<Token>;
    pub async fn execute(&self, req: GeminiRequest) -> Result<GeminiResponse>;
    pub async fn execute_stream(&self, req: GeminiRequest) -> Result<GeminiStreamResponse>;
    pub async fn count_tokens(&self, req: GeminiRequest) -> Result<i64>;
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>>;
}
```

### 5.3 OAuth2 认证实现

```rust
pub async fn authenticate_oauth() -> Result<Token> {
    let oauth_client = BasicClient::new(
        ClientId::new(GEMINI_OAUTH_CLIENT_ID.to_string()),
        Some(ClientSecret::new(GEMINI_OAUTH_CLIENT_SECRET.to_string())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(
        format!("http://localhost:{}/oauth2callback", GEMINI_CALLBACK_PORT)
    )?);

    // 生成授权 URL
    let (authorize_url, csrf_state) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/cloud-platform".to_string()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string()))
        .set_pkce_challenge(PkceCodeChallenge::new_random_sha256())
        .url();

    // 启动回调服务器
    let (tx, rx) = tokio::sync::oneshot::channel();
    let server = start_callback_server(GEMINI_CALLBACK_PORT, tx).await?;

    // 打开浏览器
    open::that(authorize_url.as_str())?;

    // 等待回调
    let auth_code = rx.await?;

    // 关闭服务器
    server.shutdown().await?;

    // 交换 token
    let token_response = oauth_client
        .exchange_code(AuthorizationCode::new(auth_code))
        .request_async(async_http_client)
        .await?;

    Ok(Token::from_token_response(token_response))
}
```

### 5.4 请求构建

```rust
pub struct GeminiRequest {
    pub project: String,
    pub model: String,
    pub request: GeminiRequestBody,
}

pub struct GeminiRequestBody {
    pub contents: Vec<Content>,
    pub generation_config: Option<GenerationConfig>,
    pub safety_settings: Option<Vec<SafetySetting>>,
}

pub struct Content {
    pub role: String,  // "user" | "model"
    pub parts: Vec<Part>,
}

pub enum Part {
    Text { text: String },
    InlineData { mime_type: String, data: String },
    FunctionCall { name: String, args: HashMap<String, Value> },
    FunctionResponse { name: String, response: HashMap<String, Value> },
}

pub struct GenerationConfig {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub max_output_tokens: Option<i32>,
    pub stop_sequences: Option<Vec<String>>,
}
```

## 6. 安全注意事项

1. **GCP 项目管理**
   - 确保项目已启用 Cloud Code Assist API
   - 检查项目权限和配额

2. **Token 安全**
   - 使用系统密钥链存储敏感 token
   - 定期刷新 access_token

3. **速率限制**
   - 实现指数退避重试
   - 遵守 API 配额限制

4. **错误处理**
   - 正确解析速率限制错误
   - 提供清晰的用户提示

## 7. 测试建议

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oauth_flow() {
        let client = GeminiCLIClient::new("test-project".to_string()).await.unwrap();
        // Mock OAuth 流程
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let mut token_source = TokenSource::new(expired_token, oauth_config);
        let token = token_source.token().await.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_retry_delay_parsing() {
        let error = json!({
            "error": {
                "details": [{
                    "@type": "type.googleapis.com/google.rpc.RetryInfo",
                    "retryDelay": "5.5s"
                }]
            }
        });
        let delay = parse_retry_delay(&error).unwrap();
        assert_eq!(delay, Duration::from_millis(5500));
    }

    #[test]
    fn test_image_aspect_ratio_fix() {
        let mut payload = json!({
            "model": "gemini-2.5-flash-image-preview",
            "request": {
                "generationConfig": {
                    "imageConfig": {
                        "aspectRatio": "16:9"
                    }
                }
            }
        });
        fix_gemini_cli_image_aspect_ratio("gemini-2.5-flash-image-preview", &mut payload);
        // 验证修改
    }
}
```
