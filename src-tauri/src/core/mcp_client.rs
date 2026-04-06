// MCP 客户端
// 用于调用抖音数据分析服务（dy-mcp 和 Undoom MCP）

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// 本地 Sidecar API 端口
pub const SIDECAR_PORT: u16 = 38080;
/// 本地 Sidecar API 基础 URL
pub const SIDECAR_BASE_URL: &str = "http://127.0.0.1:38080";

#[derive(Error, Debug, Clone)]
pub enum McpError {
    #[error("网络请求失败: {0}")]
    NetworkError(String),
    #[error("链接解析失败: {0}")]
    ParseError(String),
    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),
    #[error("频率限制")]
    RateLimited,
    #[error("链接无效: {0}")]
    InvalidLink(String),
    #[error("请求超时")]
    Timeout,
    #[error("响应解析失败: {0}")]
    ResponseParseError(String),
}

/// MCP 客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub dy_mcp_url: String,
    pub undoom_mcp_url: String,
    pub request_interval_ms: u64,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            dy_mcp_url: "http://localhost:3000".to_string(),
            undoom_mcp_url: "http://localhost:3001".to_string(),
            request_interval_ms: 1000,
            max_retries: 3,
            timeout_secs: 30,
        }
    }
}

/// 抖音视频信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DouyinVideoInfo {
    pub video_url: String,
    pub title: String,
    pub author: String,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    #[serde(default)]
    pub cover_url: Option<String>,
    #[serde(default)]
    pub duration: Option<u64>,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub videos: Vec<DouyinVideoInfo>,
    pub total: u64,
}

/// 批量处理进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub current: usize,
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

/// 单个链接解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkParseResult {
    pub link: String,
    pub success: bool,
    pub video_info: Option<DouyinVideoInfo>,
    pub error: Option<String>,
    pub retry_count: u32,
}

/// dy-mcp 服务响应格式
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct DyMcpResponse {
    status: String,
    data: Option<DyMcpVideoData>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
struct DyMcpVideoData {
    #[serde(default)]
    video_url: Option<String>,
    #[serde(default, alias = "no_watermark_url")]
    no_watermark_video_url: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default, alias = "nickname")]
    author: Option<String>,
    #[serde(default, alias = "digg_count")]
    likes: Option<u64>,
    #[serde(default, alias = "comment_count")]
    comments: Option<u64>,
    #[serde(default, alias = "share_count")]
    shares: Option<u64>,
    #[serde(default)]
    cover_url: Option<String>,
    #[serde(default)]
    duration: Option<u64>,
}

/// Undoom MCP 服务响应格式
#[derive(Debug, Deserialize)]
struct UndoomMcpResponse {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    videos: Option<Vec<UndoomVideoData>>,
    #[serde(default)]
    total: Option<u64>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UndoomVideoData {
    #[serde(default)]
    video_url: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    likes: Option<u64>,
    #[serde(default)]
    comments: Option<u64>,
    #[serde(default)]
    shares: Option<u64>,
    #[serde(default)]
    cover_url: Option<String>,
    #[serde(default)]
    duration: Option<u64>,
}

/// MCP 客户端
pub struct McpClient {
    config: McpConfig,
    http_client: Client,
}

impl McpClient {
    /// 创建新的 MCP 客户端实例
    pub fn new(config: McpConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config,
            http_client,
        }
    }

    /// 获取配置
    pub fn config(&self) -> &McpConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: McpConfig) {
        self.config = config;
    }

    /// 解析抖音链接（使用内置 Rust 解析器，无需外部服务）
    pub async fn parse_douyin_link(&self, link: &str) -> Result<DouyinVideoInfo, McpError> {
        // 验证链接格式
        if !Self::is_valid_douyin_link(link) {
            return Err(McpError::InvalidLink(link.to_string()));
        }

        // 使用内置解析器
        let parser = super::douyin_parser::DouyinParser::new();

        match parser.parse_link(link).await {
            Ok(data) => Ok(DouyinVideoInfo {
                video_url: data.no_watermark_url.clone(),
                title: data.title,
                author: data.author,
                likes: data.likes,
                comments: data.comments,
                shares: data.shares,
                cover_url: Some(data.cover_url),
                duration: Some(data.duration),
            }),
            Err(e) => match e {
                super::douyin_parser::DouyinError::Timeout => Err(McpError::Timeout),
                super::douyin_parser::DouyinError::InvalidLink(l) => Err(McpError::InvalidLink(l)),
                super::douyin_parser::DouyinError::VideoNotFound => {
                    Err(McpError::ParseError("视频不存在或已删除".to_string()))
                }
                super::douyin_parser::DouyinError::AuthRequired => Err(
                    McpError::ServiceUnavailable("需要登录验证，请稍后重试".to_string()),
                ),
                _ => Err(McpError::ParseError(e.to_string())),
            },
        }
    }

    /// 带重试的解析
    pub async fn parse_with_retry(&self, link: &str) -> LinkParseResult {
        let mut last_error = String::from("未知错误");
        let mut retry_count = 0;

        for attempt in 0..self.config.max_retries {
            retry_count = attempt;

            match self.parse_douyin_link(link).await {
                Ok(info) => {
                    return LinkParseResult {
                        link: link.to_string(),
                        success: true,
                        video_info: Some(info),
                        error: None,
                        retry_count,
                    };
                }
                Err(e) => {
                    last_error = e.to_string();

                    // 对于某些错误不需要重试
                    match &e {
                        McpError::InvalidLink(_) => break,
                        McpError::RateLimited => {
                            // 频率限制时等待更长时间
                            tokio::time::sleep(Duration::from_millis(
                                self.config.request_interval_ms * 2,
                            ))
                            .await;
                        }
                        _ => {
                            // 其他错误等待标准间隔后重试
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            }
        }

        LinkParseResult {
            link: link.to_string(),
            success: false,
            video_info: None,
            error: Some(last_error),
            retry_count,
        }
    }

    /// 批量解析链接（带重试和进度回调）
    pub async fn parse_links_batch<F>(
        &self,
        links: Vec<String>,
        progress_callback: F,
    ) -> Vec<LinkParseResult>
    where
        F: Fn(BatchProgress),
    {
        let total = links.len();
        let mut results = Vec::with_capacity(total);
        let mut success = 0;
        let mut failed = 0;

        for (i, link) in links.into_iter().enumerate() {
            let result = self.parse_with_retry(&link).await;

            if result.success {
                success += 1;
            } else {
                failed += 1;
            }

            results.push(result);

            progress_callback(BatchProgress {
                current: i + 1,
                total,
                success,
                failed,
            });

            // 请求间隔（最后一个不需要等待）
            if i < total - 1 {
                tokio::time::sleep(Duration::from_millis(self.config.request_interval_ms)).await;
            }
        }

        results
    }

    /// 搜索关键词视频（调用 Undoom MCP 服务）
    pub async fn search_videos(
        &self,
        keyword: &str,
        count: u32,
        scroll_times: u32,
    ) -> Result<SearchResult, McpError> {
        let url = format!("{}/search", self.config.undoom_mcp_url);

        let response = self
            .http_client
            .post(&url)
            .json(&serde_json::json!({
                "keyword": keyword,
                "count": count,
                "scroll_times": scroll_times
            }))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    McpError::Timeout
                } else if e.is_connect() {
                    McpError::ServiceUnavailable(format!("无法连接到 Undoom MCP 服务: {}", e))
                } else {
                    McpError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status();
        if status.as_u16() == 429 {
            return Err(McpError::RateLimited);
        }
        if !status.is_success() {
            return Err(McpError::NetworkError(format!("HTTP 错误: {}", status)));
        }

        let mcp_response: UndoomMcpResponse = response
            .json()
            .await
            .map_err(|e| McpError::ResponseParseError(e.to_string()))?;

        if !mcp_response.success {
            return Err(McpError::ParseError(
                mcp_response.error.unwrap_or_else(|| "搜索失败".to_string()),
            ));
        }

        let videos = mcp_response
            .videos
            .unwrap_or_default()
            .into_iter()
            .map(|v| DouyinVideoInfo {
                video_url: v.video_url.unwrap_or_default(),
                title: v.title.unwrap_or_else(|| "未知标题".to_string()),
                author: v.author.unwrap_or_else(|| "未知作者".to_string()),
                likes: v.likes.unwrap_or(0),
                comments: v.comments.unwrap_or(0),
                shares: v.shares.unwrap_or(0),
                cover_url: v.cover_url,
                duration: v.duration,
            })
            .collect();

        Ok(SearchResult {
            videos,
            total: mcp_response.total.unwrap_or(0),
        })
    }

    /// 获取用户视频列表（调用 Undoom MCP 服务）
    pub async fn get_user_videos(&self, user_id: &str) -> Result<Vec<DouyinVideoInfo>, McpError> {
        let url = format!("{}/user/videos", self.config.undoom_mcp_url);

        let response = self
            .http_client
            .post(&url)
            .json(&serde_json::json!({ "user_id": user_id }))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    McpError::Timeout
                } else if e.is_connect() {
                    McpError::ServiceUnavailable(format!("无法连接到 Undoom MCP 服务: {}", e))
                } else {
                    McpError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status();
        if status.as_u16() == 429 {
            return Err(McpError::RateLimited);
        }
        if !status.is_success() {
            return Err(McpError::NetworkError(format!("HTTP 错误: {}", status)));
        }

        let mcp_response: UndoomMcpResponse = response
            .json()
            .await
            .map_err(|e| McpError::ResponseParseError(e.to_string()))?;

        if !mcp_response.success {
            return Err(McpError::ParseError(
                mcp_response
                    .error
                    .unwrap_or_else(|| "获取用户视频失败".to_string()),
            ));
        }

        let videos = mcp_response
            .videos
            .unwrap_or_default()
            .into_iter()
            .map(|v| DouyinVideoInfo {
                video_url: v.video_url.unwrap_or_default(),
                title: v.title.unwrap_or_else(|| "未知标题".to_string()),
                author: v.author.unwrap_or_else(|| "未知作者".to_string()),
                likes: v.likes.unwrap_or(0),
                comments: v.comments.unwrap_or(0),
                shares: v.shares.unwrap_or(0),
                cover_url: v.cover_url,
                duration: v.duration,
            })
            .collect();

        Ok(videos)
    }

    /// 验证抖音链接格式
    pub fn is_valid_douyin_link(link: &str) -> bool {
        let link = link.trim().to_lowercase();
        link.contains("douyin.com")
            || link.contains("v.douyin.com")
            || link.contains("iesdouyin.com")
    }

    /// 检查 dy-mcp 服务是否可用（内置解析器始终可用）
    /// 检查 dy-mcp 服务是否可用（内置解析器始终可用）
    pub async fn check_dy_mcp_health(&self) -> bool {
        // 检查本地 Sidecar API
        let health_url = format!("{}/health", SIDECAR_BASE_URL);
        match self
            .http_client
            .get(&health_url)
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// 通过 Python Sidecar 下载视频 (避免 403 问题)
    pub async fn download_video_via_sidecar(
        &self,
        url: &str,
        save_path: &std::path::Path,
    ) -> Result<(), McpError> {
        let api_url = format!("{}/download", SIDECAR_BASE_URL);
        let sidecar_token = crate::core::sidecar_manager::sidecar_token()
            .ok_or_else(|| McpError::ServiceUnavailable("Python Sidecar 尚未初始化".to_string()))?;

        let path_str = save_path
            .to_str()
            .ok_or_else(|| McpError::NetworkError("无效的文件路径".to_string()))?;

        let response = self
            .http_client
            .post(api_url)
            .header("X-Sidecar-Token", sidecar_token)
            .json(&serde_json::json!({
                "url": url,
                "path": path_str
            }))
            .timeout(Duration::from_secs(600)) // 下载可能需要较长时间
            .send()
            .await
            .map_err(|e| McpError::NetworkError(format!("连接 Python 下载服务失败: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(McpError::NetworkError(format!(
                "Python 下载服务返回错误 ({}): {}",
                status, error_text
            )));
        }

        // 检查返回的 JSON 状态
        let json_resp: serde_json::Value = response
            .json()
            .await
            .map_err(|e| McpError::ResponseParseError(e.to_string()))?;

        if let Some(status) = json_resp.get("status") {
            if status.as_str() == Some("success") {
                Ok(())
            } else {
                let error = json_resp
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("未知错误");
                Err(McpError::NetworkError(format!(
                    "Python 下载服务执行失败: {}",
                    error
                )))
            }
        } else {
            Err(McpError::ResponseParseError("响应格式无效".to_string()))
        }
    }

    /// 检查 Undoom MCP 服务是否可用
    pub async fn check_undoom_mcp_health(&self) -> bool {
        let url = format!("{}/health", self.config.undoom_mcp_url);
        self.http_client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new(McpConfig::default())
    }
}
