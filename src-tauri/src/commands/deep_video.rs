use crate::commands::task_queue::{
    emit_task_completed, emit_task_failed, emit_task_progress, get_task_queue,
};
use crate::data::task_queue::TaskType;
use crate::deep_video::runner::run_deep_video_analysis;
use crate::deep_video::types::{
    DeepVideoAnalysisRequest, DeepVideoAnalysisResult, DeepVideoSource,
};
use tauri::AppHandle;

#[tauri::command]
pub async fn start_deep_video_analysis(
    app: AppHandle,
    request: DeepVideoAnalysisRequest,
) -> Result<String, String> {
    let (video_path, video_name) = match &request.source {
        DeepVideoSource::LocalVideo { video_path } => {
            let name = std::path::Path::new(video_path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("video")
                .to_string();
            (video_path.clone(), name)
        }
        DeepVideoSource::DownloadedDouyinVideo { video_path, .. } => {
            let name = std::path::Path::new(video_path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("video")
                .to_string();
            (video_path.clone(), name)
        }
    };

    let task_id = get_task_queue()
        .add_task(TaskType::DeepVideoAnalysis {
            video_path,
            video_name,
            profile: request.profile.clone(),
            transcript_task_id: request.task_id.clone(),
        })
        .await;

    let mut request = request;
    request.task_id = Some(task_id.clone());
    let app_for_task = app.clone();
    let task_id_for_task = task_id.clone();

    tauri::async_runtime::spawn(async move {
        emit_task_progress(&app_for_task, &task_id_for_task, 0.1, "running");
        get_task_queue()
            .start_task_by_id(&task_id_for_task)
            .await
            .ok();

        match run_deep_video_analysis(request).await {
            Ok(result) => {
                let result_path = result.artifacts.result_json.clone();
                get_task_queue()
                    .complete_task_by_id(&task_id_for_task, Some(result_path.clone()))
                    .await;
                emit_task_progress(&app_for_task, &task_id_for_task, 1.0, "completed");
                emit_task_completed(&app_for_task, &task_id_for_task, Some(&result_path));
            }
            Err(error) => {
                get_task_queue()
                    .fail_task_by_id(&task_id_for_task, error.clone())
                    .await;
                emit_task_failed(&app_for_task, &task_id_for_task, &error);
            }
        }
    });

    Ok(task_id)
}

#[tauri::command]
pub async fn get_deep_video_result(result_path: String) -> Result<DeepVideoAnalysisResult, String> {
    let content = std::fs::read_to_string(result_path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| error.to_string())
}
