use crate::commands::task_queue::{
    emit_task_completed, emit_task_failed, emit_task_progress, get_task_queue,
};
use crate::data::task_queue::TaskType;
use crate::deep_video::runner::run_deep_video_analysis;
use crate::deep_video::types::{
    DeepVideoAnalysisRequest, DeepVideoAnalysisResult, DeepVideoSource,
};
use futures_util::future::{AbortHandle, Abortable};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use tauri::AppHandle;

static DEEP_VIDEO_ABORT_HANDLES: Lazy<Mutex<HashMap<String, AbortHandle>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn abort_deep_video_task(task_id: &str) -> bool {
    if let Some(handle) = DEEP_VIDEO_ABORT_HANDLES.lock().remove(task_id) {
        handle.abort();
        return true;
    }

    false
}

#[tauri::command]
pub async fn start_deep_video_analysis(
    app: AppHandle,
    request: DeepVideoAnalysisRequest,
) -> Result<String, String> {
    let (source_path, source_name) = match &request.source {
        DeepVideoSource::LocalVideo { video_path } => {
            let name = std::path::Path::new(video_path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("video")
                .to_string();
            (Some(video_path.clone()), name)
        }
        DeepVideoSource::DownloadedDouyinVideo { video_path, .. } => {
            let name = std::path::Path::new(video_path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("video")
                .to_string();
            (Some(video_path.clone()), name)
        }
        DeepVideoSource::TextOnly { source_url } => {
            let name = source_url.clone().unwrap_or_else(|| request.title.clone());
            (None, name)
        }
    };

    let task_id = get_task_queue()
        .add_task(TaskType::DeepVideoAnalysis {
            source_path,
            source_name,
            profile: request.profile.clone(),
            transcript_task_id: request.task_id.clone(),
            use_frame_analysis: request.use_frame_analysis,
        })
        .await;

    let mut request = request;
    request.task_id = Some(task_id.clone());
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    DEEP_VIDEO_ABORT_HANDLES
        .lock()
        .insert(task_id.clone(), abort_handle);
    let app_for_task = app.clone();
    let task_id_for_task = task_id.clone();

    tauri::async_runtime::spawn(async move {
        if get_task_queue()
            .start_task_by_id(&task_id_for_task)
            .await
            .is_err()
        {
            DEEP_VIDEO_ABORT_HANDLES.lock().remove(&task_id_for_task);
            return;
        }

        if !matches!(
            get_task_queue().get_task_status(&task_id_for_task).await,
            Some(crate::data::task_queue::TaskStatus::Running)
        ) {
            DEEP_VIDEO_ABORT_HANDLES.lock().remove(&task_id_for_task);
            return;
        }

        get_task_queue()
            .update_task_progress_by_id(&task_id_for_task, 0.1)
            .await;
        emit_task_progress(&app_for_task, &task_id_for_task, 0.1, "running");

        match Abortable::new(run_deep_video_analysis(request), abort_registration).await {
            Ok(Ok(result)) => {
                let result_path = result.artifacts.result_json.clone();
                if get_task_queue()
                    .complete_task_by_id(&task_id_for_task, Some(result_path.clone()))
                    .await
                    .is_some()
                {
                    emit_task_progress(&app_for_task, &task_id_for_task, 1.0, "completed");
                    emit_task_completed(&app_for_task, &task_id_for_task, Some(&result_path));
                }
            }
            Ok(Err(error)) => {
                if get_task_queue()
                    .fail_task_by_id(&task_id_for_task, error.clone())
                    .await
                    .is_some()
                {
                    emit_task_failed(&app_for_task, &task_id_for_task, &error);
                }
            }
            Err(_) => {}
        }

        DEEP_VIDEO_ABORT_HANDLES.lock().remove(&task_id_for_task);
    });

    Ok(task_id)
}

#[tauri::command]
pub async fn get_deep_video_result(result_path: String) -> Result<DeepVideoAnalysisResult, String> {
    let content = std::fs::read_to_string(result_path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| error.to_string())
}
