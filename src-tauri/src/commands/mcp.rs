// MCP 服务相关命令

use crate::core::asr_engine::{AsrConfig, AsrEngine};
use crate::core::mcp_client::{
    BatchProgress, DouyinVideoInfo, LinkParseResult, McpClient, McpConfig, SearchResult,
};
use crate::core::video_processor::VideoProcessor;
use crate::utils::paths::get_app_paths;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

/// 全局 MCP 客户端实例
static MCP_CLIENT: Lazy<RwLock<McpClient>> = Lazy::new(|| RwLock::new(McpClient::default()));

/// 批量解析进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseProgressEvent {
    pub current: usize,
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub current_link: String,
    pub current_result: Option<LinkParseResult>,
}

/// 批量解析统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchParseStats {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub results: Vec<LinkParseResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractDouyinContentResult {
    pub transcript: String,
    pub video_path: String,
}

/// 更新 MCP 配置
#[tauri::command]
pub fn update_mcp_config(
    dy_mcp_url: Option<String>,
    undoom_mcp_url: Option<String>,
    request_interval_ms: Option<u64>,
    max_retries: Option<u32>,
    timeout_secs: Option<u64>,
) -> Result<(), String> {
    let mut client = MCP_CLIENT.write();
    let mut config = client.config().clone();

    if let Some(url) = dy_mcp_url {
        config.dy_mcp_url = url;
    }
    if let Some(url) = undoom_mcp_url {
        config.undoom_mcp_url = url;
    }
    if let Some(interval) = request_interval_ms {
        config.request_interval_ms = interval;
    }
    if let Some(retries) = max_retries {
        config.max_retries = retries;
    }
    if let Some(timeout) = timeout_secs {
        config.timeout_secs = timeout;
    }

    client.update_config(config);
    Ok(())
}

/// 获取当前 MCP 配置
#[tauri::command]
pub fn get_mcp_config() -> McpConfig {
    MCP_CLIENT.read().config().clone()
}

/// 解析单个抖音链接
#[tauri::command]
pub async fn parse_douyin_link(link: String) -> Result<DouyinVideoInfo, String> {
    // Clone the client config to create a new client for this request
    // This avoids holding the lock across await points
    let config = MCP_CLIENT.read().config().clone();
    let client = McpClient::new(config);

    client
        .parse_douyin_link(&link)
        .await
        .map_err(|e| e.to_string())
}

/// 批量解析抖音链接（带进度事件）
#[tauri::command]
pub async fn parse_douyin_links_batch(
    app: AppHandle,
    links: Vec<String>,
) -> Result<BatchParseStats, String> {
    use crate::data::task_queue::TaskType;

    // 过滤空链接
    let links: Vec<String> = links
        .into_iter()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if links.is_empty() {
        return Err("没有有效的链接".to_string());
    }

    let total = links.len();

    // 获取全局任务队列并添加任务
    let task_queue = crate::commands::task_queue::get_task_queue();
    let task_id = task_queue
        .add_task(TaskType::LinkParsing {
            links: links.clone(),
        })
        .await;

    // 启动任务
    let _ = task_queue.start_next_task().await;

    // Clone the client config to create a new client for this request
    let config = MCP_CLIENT.read().config().clone();
    let client = McpClient::new(config);

    // 使用进度回调
    let app_clone = app.clone();
    let links_clone = links.clone();
    let task_id_clone = task_id.clone();

    let results = client
        .parse_links_batch(links, move |progress: BatchProgress| {
            let current_link = links_clone
                .get(progress.current.saturating_sub(1))
                .cloned()
                .unwrap_or_default();

            let _ = app_clone.emit(
                "mcp:parse-progress",
                ParseProgressEvent {
                    current: progress.current,
                    total: progress.total,
                    success: progress.success,
                    failed: progress.failed,
                    current_link,
                    current_result: None,
                },
            );

            // 更新任务队列进度
            let progress_pct = progress.current as f32 / progress.total as f32;
            crate::commands::task_queue::emit_task_progress(
                &app_clone,
                &task_id_clone,
                progress_pct,
                "running",
            );
        })
        .await;

    let success = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    // 更新任务队列为完成
    task_queue
        .complete_current_task(Some(format!("解析完成: {} 成功, {} 失败", success, failed)))
        .await;
    crate::commands::task_queue::emit_task_completed(
        &app,
        &task_id,
        Some(&format!("{}/{} 成功", success, total)),
    );

    // 发送完成事件
    let _ = app.emit(
        "mcp:parse-complete",
        BatchParseStats {
            total,
            success,
            failed,
            results: results.clone(),
        },
    );

    Ok(BatchParseStats {
        total,
        success,
        failed,
        results,
    })
}

/// 搜索抖音视频
#[tauri::command]
pub async fn search_douyin_videos(
    keyword: String,
    count: u32,
    scroll_times: u32,
) -> Result<SearchResult, String> {
    // Clone the client config to create a new client for this request
    let config = MCP_CLIENT.read().config().clone();
    let client = McpClient::new(config);

    client
        .search_videos(&keyword, count, scroll_times)
        .await
        .map_err(|e| e.to_string())
}

/// 获取用户视频列表
#[tauri::command]
pub async fn get_user_videos(user_id: String) -> Result<Vec<DouyinVideoInfo>, String> {
    // Clone the client config to create a new client for this request
    let config = MCP_CLIENT.read().config().clone();
    let client = McpClient::new(config);

    client
        .get_user_videos(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// 检查 dy-mcp 服务是否可用
#[tauri::command]
pub async fn check_dy_mcp_health() -> bool {
    // Clone the client config to create a new client for this request
    let config = MCP_CLIENT.read().config().clone();
    let client = McpClient::new(config);

    client.check_dy_mcp_health().await
}

/// 检查 Undoom MCP 服务是否可用
#[tauri::command]
pub async fn check_undoom_mcp_health() -> bool {
    // Clone the client config to create a new client for this request
    let config = MCP_CLIENT.read().config().clone();
    let client = McpClient::new(config);

    client.check_undoom_mcp_health().await
}

/// 验证链接格式
#[tauri::command]
pub fn validate_douyin_link(link: String) -> bool {
    McpClient::is_valid_douyin_link(&link)
}

/// 提取抖音视频文案（下载 -> 提取音频 -> 转写）
#[tauri::command]
pub async fn extract_douyin_content(
    app: AppHandle,
    url: String,
    filename: String,
) -> Result<ExtractDouyinContentResult, String> {
    eprintln!("[Extract] 开始处理: {}", filename);
    let processor = VideoProcessor::new().map_err(|e| e.to_string())?;

    // 1. 设置临时路径
    let temp_dir = processor.temp_dir();
    let video_path = temp_dir.join(format!("{}.mp4", filename));

    // 2. 下载视频 (使用 Python Sidecar)
    eprintln!("[Extract] 正在通过 Python 助手下载视频...");

    // 获取 MCP 客户端
    let config = MCP_CLIENT.read().config().clone();
    let client = McpClient::new(config);

    client
        .download_video_via_sidecar(&url, &video_path)
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    // 3. 提取音频
    eprintln!("[Extract] 正在提取音频...");
    let audio_result = processor
        .extract_audio(&video_path, None)
        .await
        .map_err(|e| format!("音频提取失败: {}", e))?;

    // 4. 初始化 ASR 引擎
    // 使用 AppPaths 统一管理路径
    let models_dir = if let Ok(paths) = get_app_paths() {
        paths.get_model_path("asr")
    } else {
        app.path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("models")
            .join("asr")
    };

    let app_config = crate::commands::settings::get_settings()
        .await
        .map_err(|e| e.to_string())?;

    let config = AsrConfig {
        models_dir,
        num_threads: app_config.gpu_threads as usize,
        use_gpu: app_config.gpu_enabled,
        gpu_device_id: app_config.gpu_device_id,
        ..Default::default()
    };

    let engine = AsrEngine::new(config).map_err(|e| e.to_string())?;

    // 5. 转写音频
    eprintln!("[Extract] 正在转写音频...");
    let result = engine
        .transcribe(&audio_result.output_path, None::<fn(f32)>)
        .await
        .map_err(|e| format!("转写失败: {}", e))?;

    // 6. 清理临时文件 (可选)
    // let _ = std::fs::remove_file(video_path);
    // let _ = std::fs::remove_file(audio_result.output_path);

    eprintln!("[Extract] 完成! 文案长度: {}", result.text.len());
    Ok(ExtractDouyinContentResult {
        transcript: result.text,
        video_path: video_path.to_string_lossy().to_string(),
    })
}
