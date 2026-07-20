use crate::utils::paths::get_app_paths;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DeepVideoArtifactPaths {
    pub task_dir: PathBuf,
    pub frames_dir: PathBuf,
    pub request_json: PathBuf,
    pub frame_result_json: PathBuf,
    pub evidence_sheet_jpg: PathBuf,
    pub candidate_segments_json: PathBuf,
    pub vision_result_json: PathBuf,
    pub evidence_timeline_json: PathBuf,
    pub result_json: PathBuf,
    pub report_md: PathBuf,
}

impl DeepVideoArtifactPaths {
    pub fn from_root(root: &Path, task_id: &str) -> Self {
        let task_dir = root.join("deep-video").join(task_id);
        Self {
            frames_dir: task_dir.join("frames"),
            request_json: task_dir.join("deep_analysis_request.json"),
            frame_result_json: task_dir.join("frame_result.json"),
            evidence_sheet_jpg: task_dir.join("evidence_sheet.jpg"),
            candidate_segments_json: task_dir.join("candidate_segments.json"),
            vision_result_json: task_dir.join("vision_result.json"),
            evidence_timeline_json: task_dir.join("evidence_timeline.json"),
            result_json: task_dir.join("deep_analysis_result.json"),
            report_md: task_dir.join("report.md"),
            task_dir,
        }
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.frames_dir)
            .map_err(|error| format!("Failed to create deep video artifact dirs: {error}"))
    }
}

pub fn create_artifact_paths(task_id: &str) -> Result<DeepVideoArtifactPaths, String> {
    let paths = get_app_paths().map_err(|error| error.to_string())?;
    create_artifact_paths_from_root(&paths.data_dir, task_id)
}

pub fn create_artifact_paths_from_root(
    root: &Path,
    task_id: &str,
) -> Result<DeepVideoArtifactPaths, String> {
    let artifact_paths = DeepVideoArtifactPaths::from_root(root, task_id);
    artifact_paths.ensure_dirs()?;
    Ok(artifact_paths)
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    std::fs::write(path, content).map_err(|error| error.to_string())
}

pub fn write_text(path: &Path, value: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(path, value).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn creates_expected_deep_video_filenames() {
        let root = std::path::PathBuf::from("C:/tmp/test-app-data");
        let paths = DeepVideoArtifactPaths::from_root(&root, "task-123");

        assert!(paths.task_dir.ends_with("deep-video/task-123"));
        assert!(paths.request_json.ends_with("deep_analysis_request.json"));
        assert!(paths.evidence_sheet_jpg.ends_with("evidence_sheet.jpg"));
        assert!(paths.report_md.ends_with("report.md"));
    }

    #[test]
    fn writes_json_and_text_files() {
        let temp = tempfile::tempdir().unwrap();
        let json_path = temp.path().join("value.json");
        let text_path = temp.path().join("report.md");

        write_json(&json_path, &json!({"ok": true})).unwrap();
        write_text(&text_path, "# report").unwrap();

        assert!(std::fs::read_to_string(json_path)
            .unwrap()
            .contains("\"ok\": true"));
        assert_eq!(std::fs::read_to_string(text_path).unwrap(), "# report");
    }
}
