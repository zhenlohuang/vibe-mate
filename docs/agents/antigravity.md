# Antigravity (Google Cloud Code Assist) Agent 设计文档

## 概述

Antigravity Agent 是 Google Advanced Agentic Coding 的官方实现，由 Google Deepmind 团队开发。它使用 Google OAuth2 认证，通过 Cloud Code Assist API 访问，支持包括 Claude 和 Gemini 在内的多个模型。与 Gemini CLI 类似，但包含更高级的功能和项目管理能力。

## 1. 认证机制 (Authentication)

### 1.1 OAuth2 认证流程

#### 关键配置
```rust
const ANTIGRAVITY_CLIENT_ID: &str = "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
const ANTIGRAVITY_CLIENT_SECRET: &str = "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";
const ANTIGRAVITY_CALLBACK_PORT: u16 = 51121;
```

#### OAuth Scopes
```rust
const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    "https://www.googleapis.com/auth/cclog",                  // Cloud Code 日志
    "https://www.googleapis.com/auth/experimentsandconfigs"   // 实验性配置
];
```

#### 认证步骤

1. **启动本地回调服务器**
   - 监听端口: `51121`
   - 回调路径: `/oauth-callback`
   - 支持动态端口分配
   - 超时时间: 5 分钟

2. **构建授权 URL**
   ```
   https://accounts.google.com/o/oauth2/v2/auth?
     access_type=offline
     &client_id={CLIENT_ID}
     &prompt=consent
     &redirect_uri=http://localhost:51121/oauth-callback
     &response_type=code
     &scope={SCOPES}
     &state={STATE}
   ```

3. **接收回调并交换令牌**
   ```http
   POST https://oauth2.googleapis.com/token
   Content-Type: application/x-www-form-urlencoded

   code={AUTH_CODE}
   &client_id={CLIENT_ID}
   &client_secret={CLIENT_SECRET}
   &redirect_uri={REDIRECT_URI}
   &grant_type=authorization_code
   ```

4. **获取用户信息**
   ```http
   GET https://www.googleapis.com/oauth2/v1/userinfo?alt=json
   Authorization: Bearer {ACCESS_TOKEN}
   ```

5. **获取 GCP 项目 ID**（自动发现）

   调用 `loadCodeAssist` API:
   ```http
   POST https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist
   Authorization: Bearer {ACCESS_TOKEN}
   Content-Type: application/json

   {
     "metadata": {
       "ideType": "ANTIGRAVITY",
       "platform": "PLATFORM_UNSPECIFIED",
       "pluginType": "GEMINI"
     }
   }
   ```

   响应:
   ```json
   {
     "cloudaicompanionProject": "projects/1234567890",
     "allowedTiers": [
       {
         "id": "tier-1",
         "isDefault": true
       }
     ]
   }
   ```

6. **用户入驻（如果需要）**

   如果 `loadCodeAssist` 未返回项目，调用 `onboardUser`:
   ```http
   POST https://cloudcode-pa.googleapis.com/v1internal:onboardUser
   Authorization: Bearer {ACCESS_TOKEN}
   Content-Type: application/json

   {
     "tierId": "tier-1",
     "metadata": {
       "ideType": "ANTIGRAVITY",
       "platform": "PLATFORM_UNSPECIFIED",
       "pluginType": "GEMINI"
     }
   }
   ```

   轮询响应直到完成:
   ```json
   {
     "done": true,
     "response": {
       "cloudaicompanionProject": {
         "id": "projects/1234567890"
       }
     }
   }
   ```

### 1.2 认证状态维持

#### Token 存储结构
```rust
pub struct AntigravityAuth {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub timestamp: i64,        // 毫秒时间戳
    pub expired: String,       // RFC3339
    pub email: String,
    pub project_id: String,
}
```

#### 文件持久化
- **存储位置**: `{auth_dir}/antigravity-{email_sanitized}.json`
- **文件格式**: JSON
- **文件名规则**: `antigravity-` + 邮箱（`@` 和 `.` 替换为 `_`）

#### Token 刷新机制

1. **触发条件**
   - 距离过期时间 < 3000 秒（约 50 分钟）
   - 使用 `refreshSkew` 提前刷新

2. **刷新流程**
   ```rust
   fn ensure_access_token(
       &self,
       ctx: &Context,
       auth: &mut Auth
   ) -> Result<(String, Option<Auth>)> {
       let now = SystemTime::now();
       let timestamp = auth.metadata.get("timestamp").as_i64()?;
       let expires_in = auth.metadata.get("expires_in").as_i64()?;

       let expiry = timestamp + (expires_in * 1000);
       let refresh_skew = 3000 * 1000; // 3000 秒

       if now.duration_since(UNIX_EPOCH)?.as_millis() < (expiry - refresh_skew) as u128 {
           // Token 仍然有效
           let token = auth.metadata.get("access_token").as_str()?;
           return Ok((token.to_string(), None));
       }

       // 需要刷新
       let refresh_token = auth.metadata.get("refresh_token").as_str()?;
       let new_token = self.refresh_antigravity_token(ctx, refresh_token)?;

       auth.metadata["access_token"] = new_token.access_token;
       auth.metadata["refresh_token"] = new_token.refresh_token;
       auth.metadata["expires_in"] = new_token.expires_in;
       auth.metadata["timestamp"] = SystemTime::now().as_millis();
       auth.metadata["expired"] = calculate_expiry(new_token.expires_in);

       Ok((new_token.access_token, Some(auth.clone())))
   }
   ```

3. **刷新 API 调用**
   ```http
   POST https://oauth2.googleapis.com/token
   Content-Type: application/x-www-form-urlencoded

   client_id={CLIENT_ID}
   &client_secret={CLIENT_SECRET}
   &refresh_token={REFRESH_TOKEN}
   &grant_type=refresh_token
   ```

### 1.3 项目自动发现和入驻

#### loadCodeAssist API
用于检查用户是否已有关联项目：

```rust
pub async fn load_code_assist(
    ctx: &Context,
    access_token: &str
) -> Result<LoadCodeAssistResponse> {
    let endpoint = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";

    let request_body = json!({
        "metadata": {
            "ideType": "ANTIGRAVITY",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI"
        }
    });

    let response = http_client.post(endpoint)
        .bearer_auth(access_token)
        .json(&request_body)
        .send()
        .await?;

    let data: LoadCodeAssistResponse = response.json().await?;
    Ok(data)
}

pub struct LoadCodeAssistResponse {
    pub cloudaicompanion_project: Option<ProjectInfo>,
    pub allowed_tiers: Vec<TierInfo>,
}

pub struct ProjectInfo {
    pub id: String,
}

pub struct TierInfo {
    pub id: String,
    pub is_default: bool,
}
```

#### onboardUser API
如果用户没有项目，通过入驻流程创建：

```rust
pub async fn onboard_user(
    ctx: &Context,
    access_token: &str,
    tier_id: &str
) -> Result<String> {
    let endpoint = "https://cloudcode-pa.googleapis.com/v1internal:onboardUser";

    let request_body = json!({
        "tierId": tier_id,
        "metadata": {
            "ideType": "ANTIGRAVITY",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI"
        }
    });

    // 轮询直到完成（最多 5 次）
    for attempt in 1..=5 {
        let response = http_client.post(endpoint)
            .bearer_auth(access_token)
            .json(&request_body)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        let data: OnboardResponse = response.json().await?;

        if data.done {
            if let Some(project) = data.response.cloudaicompanion_project {
                return Ok(project.id);
            }
            return Err("No project ID in response");
        }

        // 等待 2 秒后重试
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Err("Onboarding timeout")
}

pub struct OnboardResponse {
    pub done: bool,
    pub response: OnboardResponseData,
}

pub struct OnboardResponseData {
    pub cloudaicompanion_project: Option<ProjectInfo>,
}
```

## 2. 额度查询 (Quota/Usage)

### 2.1 模型配额查询 API

#### 认证文件位置
```
~/.cli-proxy-api/antigravity-*.json
```

#### API 调用

**端点**:
```
POST https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels
```

**Headers**:
```http
Authorization: Bearer {access_token}
User-Agent: antigravity/1.11.3 Darwin/arm64
Content-Type: application/json
```

**请求示例**:
```json
{
  "project": "projects/1234567890"
}
```

**注意**: `project` 字段可选，提供后返回该项目的配额信息。

**响应示例**:
```json
{
  "models": {
    "gemini-3-pro-high": {
      "quotaInfo": {
        "remainingFraction": 0.85,
        "resetTime": "2025-01-18T12:00:00.000Z"
      }
    },
    "claude-sonnet-4-5": {
      "quotaInfo": {
        "remainingFraction": 0.75,
        "resetTime": "2025-01-19T00:00:00.000Z"
      }
    }
  }
}
```

#### 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `remainingFraction` | float | 剩余配额比例（0.0-1.0，需乘以100转为百分比） |
| `resetTime` | string | ISO8601 重置时间 |

#### 支持的模型

- **Gemini**: `gemini-3-pro`, `gemini-3-pro-high`, `gemini-3-flash`, `gemini-3-flash-high`
- **Claude**: `claude-sonnet-4-5`, `claude-opus-4`, `claude-opus-4-5`, `claude-4-sonnet`, `claude-4-opus`
- **Thinking variants**: `*-thinking` 变体

#### Token 刷新

当 access_token 过期时（401），使用 refresh_token 刷新：

```http
POST https://oauth2.googleapis.com/token
Content-Type: application/x-www-form-urlencoded

client_id=1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com
&client_secret=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
&grant_type=refresh_token
&refresh_token={REFRESH_TOKEN}
```

#### 错误处理

| Status | 处理方式 |
|--------|---------|
| 200 | 解析配额数据 |
| 401 | 使用 refresh_token 刷新 |
| 403 | 配额已耗尽 |
| 429 | 重试3次，间隔1秒 |
| 5xx | 使用缓存数据 |

#### 使用示例

```rust
pub async fn fetch_antigravity_quota(
    access_token: &str,
    project_id: Option<&str>
) -> Result<AntigravityQuotaInfo> {
    let mut body = json!({});
    if let Some(project) = project_id {
        body["project"] = json!(project);
    }

    let response = client
        .post("https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels")
        .bearer_auth(access_token)
        .header("User-Agent", "antigravity/1.11.3 Darwin/arm64")
        .json(&body)
        .send()
        .await?;

    let data: FetchModelsResponse = response.json().await?;

    // 转换 remainingFraction (0-1) 为百分比
    let quotas = data.models.iter().map(|(name, model)| {
        ModelQuota {
            name: name.clone(),
            remaining_percentage: model.quota_info.remaining_fraction * 100.0,
            reset_time: model.quota_info.reset_time,
        }
    }).collect();

    Ok(AntigravityQuotaInfo { quotas })
}
```

### 2.2 订阅信息查询 API

**端点**:
```
POST https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist
```

**请求**:
```json
{
  "metadata": {
    "ideType": "ANTIGRAVITY"
  }
}
```

**响应**:
```json
{
  "currentTier": {
    "id": "free_user_tier",
    "name": "Free"
  },
  "cloudaicompanionProject": "projects/1234567890"
}
```

### 2.3 Token 计数 API

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
        "parts": [{"text": "Hello"}]
      }
    ]
  }
}
```

**注意**: 与 Gemini CLI 类似，不包含 `project` 和 `model` 字段。

### 2.4 使用量统计

```rust
pub struct AntigravityUsage {
    pub prompt_token_count: i64,
    pub candidates_token_count: i64,
    pub total_token_count: i64,
}

fn parse_antigravity_usage(data: &[u8]) -> AntigravityUsage {
    AntigravityUsage {
        prompt_token_count: data.get("usageMetadata.promptTokenCount").unwrap_or(0),
        candidates_token_count: data.get("usageMetadata.candidatesTokenCount").unwrap_or(0),
        total_token_count: data.get("usageMetadata.totalTokenCount").unwrap_or(0),
    }
}
```

## 3. 模型使用和暴露

### 3.1 API 端点

#### 多环境支持
```rust
pub enum AntigravityEnvironment {
    Production,     // https://cloudcode-pa.googleapis.com
    Daily,          // https://daily-cloudcode-pa.googleapis.com
    DailySandbox,   // https://daily-cloudcode-pa.sandbox.googleapis.com
}

fn antigravity_base_url_fallback_order(auth: &Auth) -> Vec<String> {
    // 优先使用配置的环境
    // 失败时回退到其他环境
    vec![
        "https://cloudcode-pa.googleapis.com".to_string(),
        // 可选的 daily/sandbox 环境
    ]
}
```

#### 可用端点
```rust
pub enum AntigravityEndpoint {
    StreamGenerateContent,   // /v1internal:streamGenerateContent
    GenerateContent,         // /v1internal:generateContent
    CountTokens,            // /v1internal:countTokens
    FetchAvailableModels,   // /v1internal:fetchAvailableModels
}
```

### 3.2 请求格式

#### Headers
```rust
fn apply_antigravity_headers(req: &mut Request, access_token: &str) {
    req.header("Content-Type", "application/json");
    req.header("Authorization", format!("Bearer {}", access_token));
    req.header("User-Agent", "google-api-nodejs-client/9.15.1");
    req.header("X-Goog-Api-Client", "google-cloud-sdk vscode_cloudshelleditor/0.1");
    req.header("Client-Metadata", r#"{"ideType":"IDE_UNSPECIFIED","platform":"PLATFORM_UNSPECIFIED","pluginType":"GEMINI"}"#);
}
```

#### 请求体结构
```json
{
  "project": "projects/1234567890",
  "model": "gemini-2.5-pro",
  "request": {
    "contents": [
      {
        "role": "user",
        "parts": [{"text": "Hello"}]
      }
    ],
    "systemInstruction": {
      "parts": [
        {
          "text": "You are Antigravity, a powerful agentic AI coding assistant..."
        }
      ]
    },
    "generationConfig": {
      "temperature": 1.0,
      "topP": 0.95,
      "maxOutputTokens": 8192
    }
  }
}
```

### 3.3 系统指令

Antigravity 使用专门的系统指令：

```rust
const SYSTEM_INSTRUCTION: &str = r#"You are Antigravity, a powerful agentic AI coding assistant designed by the Google Deepmind team working on Advanced Agentic Coding.

You are pair programming with a USER to solve their coding task. The task may require creating a new codebase, modifying or debugging an existing codebase, or simply answering a question.

**Absolute paths only**
**Proactiveness**"#;

// 在请求中应用
fn apply_system_instruction(payload: &mut Vec<u8>) {
    payload.set("request.systemInstruction.parts.0.text", SYSTEM_INSTRUCTION);
}
```

### 3.4 Claude 模型支持

Antigravity 支持 Claude 模型，使用特殊处理：

#### 检测 Claude 模型
```rust
fn is_claude_model(model: &str) -> bool {
    model.to_lowercase().contains("claude") || model.contains("gemini-3-pro")
}
```

#### Claude 流式处理
对于 Claude 模型，使用流式翻译模式：

```rust
async fn execute_claude_stream(
    &self,
    ctx: &Context,
    auth: &mut Auth,
    req: Request
) -> Result<impl Stream<Item = StreamChunk>> {
    // 使用 streaming=true 进行请求翻译
    let translated = translate_request(from, to, model, payload, true);

    // 流式响应处理
    let stream = stream! {
        let mut scanner = BufReader::new(response.body);
        let mut buffer = Vec::new();
        let mut param = None;

        while let Some(line) = scanner.lines().next().await {
            let line = line?;

            if line.starts_with("data:") {
                let chunks = translate_stream(to, from, model, original, body, line, &mut param);
                for chunk in chunks {
                    yield Ok(StreamChunk { payload: chunk });
                }
            }
        }
    };

    Ok(stream)
}
```

### 3.5 环境回退机制

当请求失败时，自动尝试其他环境：

```rust
async fn execute_with_fallback(
    &self,
    ctx: &Context,
    auth: &mut Auth,
    req: Request
) -> Result<Response> {
    let base_urls = antigravity_base_url_fallback_order(auth);

    for (idx, base_url) in base_urls.iter().enumerate() {
        match self.execute_with_base_url(ctx, auth, &req, base_url).await {
            Ok(response) => return Ok(response),
            Err(e) if e.is_rate_limited() && idx + 1 < base_urls.len() => {
                log::debug!("Rate limited on {}, trying fallback", base_url);
                continue;
            }
            Err(e) if e.is_network_error() && idx + 1 < base_urls.len() => {
                log::debug!("Network error on {}, trying fallback", base_url);
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Err(Error::AllEndpointsFailed)
}
```

## 4. 错误处理

### 4.1 认证错误

```rust
pub enum AntigravityAuthError {
    OAuthTimeout,
    ProjectDiscoveryFailed,
    OnboardingFailed,
    TokenRefreshFailed,
}
```

### 4.2 API 错误

```rust
match status {
    200..=299 => Ok(response),
    400 => Err("Invalid request"),
    401 => Err("Unauthorized"),
    403 => Err("Permission denied"),
    404 => Err("Model or endpoint not found"),
    429 => {
        let retry_after = parse_retry_delay(body);
        Err(RateLimitError { retry_after })
    },
    500..=599 => Err("Server error"),
    _ => Err("Unknown error"),
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
oauth2 = "4.4"
async-stream = "0.3"
futures-core = "0.3"
regex = "1.10"
```

### 5.2 核心结构

```rust
use oauth2::{basic::BasicClient, AuthUrl, TokenUrl, ClientId, ClientSecret};

pub struct AntigravityClient {
    http_client: reqwest::Client,
    oauth_client: BasicClient,
    token_manager: Arc<RwLock<TokenManager>>,
    project_id: String,
    environment: AntigravityEnvironment,
}

pub struct TokenManager {
    access_token: String,
    refresh_token: String,
    expires_at: Instant,
    refresh_skew: Duration,  // 3000 秒
}

impl TokenManager {
    pub async fn get_token(&mut self) -> Result<String> {
        let now = Instant::now();
        if now + self.refresh_skew < self.expires_at {
            return Ok(self.access_token.clone());
        }

        // 需要刷新
        self.refresh().await?;
        Ok(self.access_token.clone())
    }

    async fn refresh(&mut self) -> Result<()> {
        let token_response = self.oauth_client
            .exchange_refresh_token(&RefreshToken::new(self.refresh_token.clone()))
            .request_async(async_http_client)
            .await?;

        self.access_token = token_response.access_token().secret().clone();
        if let Some(refresh) = token_response.refresh_token() {
            self.refresh_token = refresh.secret().clone();
        }
        self.expires_at = Instant::now() + Duration::from_secs(
            token_response.expires_in()
                .map(|d| d.as_secs())
                .unwrap_or(3600)
        );

        Ok(())
    }
}

impl AntigravityClient {
    pub async fn new(environment: AntigravityEnvironment) -> Result<Self>;
    pub async fn authenticate(&self) -> Result<Auth>;
    pub async fn discover_project(&self, token: &str) -> Result<String>;
    pub async fn onboard_user(&self, token: &str, tier: &str) -> Result<String>;
    pub async fn execute(&self, req: AntigravityRequest) -> Result<AntigravityResponse>;
    pub async fn execute_stream(&self, req: AntigravityRequest) -> Result<impl Stream<Item = StreamChunk>>;
    pub async fn count_tokens(&self, req: AntigravityRequest) -> Result<i64>;
}
```

### 5.3 项目发现实现

```rust
pub async fn discover_project(
    &self,
    access_token: &str
) -> Result<String> {
    // 1. 尝试 loadCodeAssist
    let load_response = self.load_code_assist(access_token).await?;

    if let Some(project) = load_response.cloudaicompanion_project {
        return Ok(project.id);
    }

    // 2. 如果没有项目，选择默认 tier 并入驻
    let tier_id = load_response.allowed_tiers
        .iter()
        .find(|t| t.is_default)
        .map(|t| t.id.clone())
        .unwrap_or_else(|| "legacy-tier".to_string());

    // 3. 入驻用户
    let project_id = self.onboard_user(access_token, &tier_id).await?;

    Ok(project_id)
}

async fn load_code_assist(
    &self,
    access_token: &str
) -> Result<LoadCodeAssistResponse> {
    let url = format!("{}/v1internal:loadCodeAssist", self.base_url());

    let response = self.http_client
        .post(&url)
        .bearer_auth(access_token)
        .json(&json!({
            "metadata": {
                "ideType": "ANTIGRAVITY",
                "platform": "PLATFORM_UNSPECIFIED",
                "pluginType": "GEMINI"
            }
        }))
        .send()
        .await?;

    Ok(response.json().await?)
}

async fn onboard_user(
    &self,
    access_token: &str,
    tier_id: &str
) -> Result<String> {
    let url = format!("{}/v1internal:onboardUser", self.base_url());

    for attempt in 1..=5 {
        let response = self.http_client
            .post(&url)
            .bearer_auth(access_token)
            .timeout(Duration::from_secs(30))
            .json(&json!({
                "tierId": tier_id,
                "metadata": {
                    "ideType": "ANTIGRAVITY",
                    "platform": "PLATFORM_UNSPECIFIED",
                    "pluginType": "GEMINI"
                }
            }))
            .send()
            .await?;

        let data: OnboardResponse = response.json().await?;

        if data.done {
            if let Some(project) = data.response.cloudaicompanion_project {
                return Ok(project.id);
            }
            return Err(Error::NoProjectInResponse);
        }

        // 等待 2 秒后重试
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Err(Error::OnboardingTimeout)
}
```

### 5.4 请求构建

```rust
pub struct AntigravityRequest {
    pub project: String,
    pub model: String,
    pub request: AntigravityRequestBody,
}

pub struct AntigravityRequestBody {
    pub contents: Vec<Content>,
    pub system_instruction: Option<SystemInstruction>,
    pub generation_config: Option<GenerationConfig>,
    pub safety_settings: Option<Vec<SafetySetting>>,
}

pub struct SystemInstruction {
    pub parts: Vec<Part>,
}

impl AntigravityRequest {
    pub fn new(project: String, model: String) -> Self {
        Self {
            project,
            model,
            request: AntigravityRequestBody {
                contents: vec![],
                system_instruction: Some(SystemInstruction {
                    parts: vec![Part::Text {
                        text: SYSTEM_INSTRUCTION.to_string()
                    }]
                }),
                generation_config: None,
                safety_settings: None,
            }
        }
    }
}
```

## 6. 安全注意事项

1. **项目管理**
   - 项目 ID 必须正确配置
   - 检查项目权限和配额

2. **Token 刷新**
   - 提前 3000 秒刷新（约 50 分钟）
   - 使用 `refresh_skew` 避免边界情况

3. **环境隔离**
   - 生产环境与测试环境分离
   - 正确配置环境 URL

4. **错误处理**
   - 实现完整的回退机制
   - 处理入驻流程的异步性质

## 7. 测试建议

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_project_discovery() {
        let client = AntigravityClient::new(AntigravityEnvironment::Production).await.unwrap();
        let token = "test_token";
        let project = client.discover_project(token).await.unwrap();
        assert!(!project.is_empty());
    }

    #[tokio::test]
    async fn test_onboard_user() {
        let client = AntigravityClient::new(AntigravityEnvironment::Production).await.unwrap();
        let token = "test_token";
        let project = client.onboard_user(token, "tier-1").await.unwrap();
        assert!(project.starts_with("projects/"));
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let mut manager = TokenManager::new(expired_token);
        let token = manager.get_token().await.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_claude_model_detection() {
        assert!(is_claude_model("claude-sonnet-4"));
        assert!(is_claude_model("gemini-3-pro"));
        assert!(!is_claude_model("gemini-2.5-pro"));
    }
}
```

## 8. 与 Gemini CLI 的区别

| 特性 | Antigravity | Gemini CLI |
|------|------------|-----------|
| Client ID | 1071006060591-... | 681255809395-... |
| Scopes | 包含 cclog 和 experimentsandconfigs | 标准 cloud-platform scopes |
| 项目发现 | 自动通过 loadCodeAssist/onboardUser | 需要手动配置 |
| 系统指令 | 专门的 Antigravity 指令 | 无特殊指令 |
| Claude 支持 | 内置支持 | 不支持 |
| 环境支持 | 多环境（prod/daily/sandbox） | 仅生产环境 |
| Token 刷新阈值 | 3000 秒 | 自动管理 |

## 9. 高级功能

### 9.1 Claude 模型集成

Antigravity 原生支持 Claude 模型，自动处理格式转换：

```rust
fn should_use_claude_mode(model: &str) -> bool {
    model.to_lowercase().contains("claude") || model.contains("gemini-3-pro")
}

// 自动选择处理模式
match should_use_claude_mode(&req.model) {
    true => self.execute_claude_stream(ctx, auth, req).await,
    false => self.execute_gemini_stream(ctx, auth, req).await,
}
```

### 9.2 多环境部署

支持在不同环境之间切换和回退：

```rust
pub enum DeploymentTier {
    Production,
    Daily,
    Sandbox,
}

impl AntigravityClient {
    pub fn with_tier(tier: DeploymentTier) -> Self {
        let base_url = match tier {
            DeploymentTier::Production => "https://cloudcode-pa.googleapis.com",
            DeploymentTier::Daily => "https://daily-cloudcode-pa.googleapis.com",
            DeploymentTier::Sandbox => "https://daily-cloudcode-pa.sandbox.googleapis.com",
        };
        // ...
    }
}
```
