use douyin_creator_tools_lib::deep_video::runner::run_deep_video_analysis_with_artifact_root;
use douyin_creator_tools_lib::deep_video::types::{
    AnalysisProfile, AnalysisProfileOptions, DeepVideoAnalysisRequest, DeepVideoSource,
    EvidenceKind, TranscriptInput, TranscriptSegment,
};
use std::path::{Path, PathBuf};
use std::process::Command;

fn transcript_fixture() -> TranscriptInput {
    TranscriptInput {
        text: "Limited offer. Tap now to claim the creator toolkit bonus.".to_string(),
        segments: vec![TranscriptSegment {
            text: "Limited offer. Tap now to claim the creator toolkit bonus.".to_string(),
            start_seconds: Some(1.0),
            end_seconds: Some(4.0),
        }],
    }
}

async fn run_smoke_request(
    request: DeepVideoAnalysisRequest,
    artifact_root: &Path,
) -> Result<douyin_creator_tools_lib::deep_video::types::DeepVideoAnalysisResult, String> {
    run_deep_video_analysis_with_artifact_root(request, artifact_root).await
}

fn assert_path_stays_under(path: &str, root: &Path) {
    let path = Path::new(path);
    assert!(
        path.starts_with(root),
        "artifact path {} should stay under {}",
        path.display(),
        root.display()
    );
}

#[tokio::test]
async fn text_only_deep_video_smoke_writes_report_without_frame_artifacts() {
    let artifact_root = tempfile::tempdir().unwrap();
    let task_id = format!("smoke-text-only-{}", chrono::Utc::now().timestamp_millis());
    let request = DeepVideoAnalysisRequest {
        source: DeepVideoSource::TextOnly {
            source_url: Some("https://v.douyin.com/smoke/".to_string()),
        },
        task_id: Some(task_id),
        title: "smoke text only".to_string(),
        profile: AnalysisProfile::Balanced,
        use_frame_analysis: false,
        transcript: Some(transcript_fixture()),
        ocr_items: Vec::new(),
        reference_text: Some("Author: Smoke\nLikes: 42\nComments: 7\nShares: 3".to_string()),
    };

    let result = run_smoke_request(request, artifact_root.path())
        .await
        .unwrap();

    assert!(result.source_video_path.is_none());
    assert!(result.frames.is_empty());
    assert!(result.evidence_sheet.is_none());
    assert!(result.artifacts.evidence_sheet_jpg.is_none());
    assert!(result
        .timeline
        .iter()
        .any(|item| item.kind == EvidenceKind::Candidate));
    assert!(std::path::Path::new(&result.artifacts.request_json).exists());
    assert!(std::path::Path::new(&result.artifacts.result_json).exists());
    assert!(std::path::Path::new(&result.artifacts.report_md).exists());
    assert_path_stays_under(&result.artifacts.request_json, artifact_root.path());
    assert_path_stays_under(&result.artifacts.result_json, artifact_root.path());
    assert_path_stays_under(&result.artifacts.report_md, artifact_root.path());
    assert!(!result.report_markdown.contains("contact sheet"));
}

#[tokio::test]
async fn frame_evidence_smoke_requires_existing_video_source() {
    let artifact_root = tempfile::tempdir().unwrap();
    let request = DeepVideoAnalysisRequest {
        source: DeepVideoSource::LocalVideo {
            video_path: "C:/tmp/douyin-creator-toolkit-smoke/missing.mp4".to_string(),
        },
        task_id: Some(format!(
            "smoke-missing-video-{}",
            chrono::Utc::now().timestamp_millis()
        )),
        title: "missing frame source".to_string(),
        profile: AnalysisProfile::Economy,
        use_frame_analysis: true,
        transcript: Some(transcript_fixture()),
        ocr_items: Vec::new(),
        reference_text: None,
    };

    let error = run_smoke_request(request, artifact_root.path())
        .await
        .unwrap_err();

    assert!(error.contains("Video source does not exist"));
}

#[tokio::test]
#[ignore = "requires a local FFmpeg/FFprobe pair under src-tauri/resources/ffmpeg"]
async fn local_video_frame_evidence_smoke_writes_visual_artifacts() {
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();

    let artifact_root = local_smoke_root();
    std::fs::create_dir_all(&artifact_root).unwrap();
    let video_path = artifact_root.join("sample-local-video.mp4");
    generate_sample_video(&video_path);

    let task_id = format!(
        "smoke-local-video-{}",
        chrono::Utc::now().timestamp_millis()
    );
    let request = DeepVideoAnalysisRequest {
        source: DeepVideoSource::LocalVideo {
            video_path: video_path.to_string_lossy().to_string(),
        },
        task_id: Some(task_id),
        title: "sample local video".to_string(),
        profile: AnalysisProfile::Custom(AnalysisProfileOptions {
            max_frames: 4,
            interval_seconds: 1.0,
            candidate_window_seconds: 4.0,
            frames_per_candidate: 1,
            use_scene_boundaries: false,
            vision_passes: 0,
        }),
        use_frame_analysis: true,
        transcript: Some(transcript_fixture()),
        ocr_items: Vec::new(),
        reference_text: Some("Author: Local Smoke\nLikes: 12\nComments: 3\nShares: 1".to_string()),
    };

    let result = run_smoke_request(request, &artifact_root).await.unwrap();

    assert_eq!(
        result.source_video_path.as_deref(),
        Some(video_path.to_string_lossy().as_ref())
    );
    assert!(result.frames.len() >= 3);
    assert!(result
        .timeline
        .iter()
        .any(|item| item.kind == EvidenceKind::Frame));
    assert!(result.evidence_sheet.is_some());
    assert!(result.artifacts.evidence_sheet_jpg.is_some());
    assert!(result.report_markdown.contains("contact sheet"));
    assert!(result.report_markdown.contains("frame IDs"));

    for artifact in [
        &result.artifacts.request_json,
        &result.artifacts.frame_result_json,
        &result.artifacts.candidate_segments_json,
        &result.artifacts.evidence_timeline_json,
        &result.artifacts.result_json,
        &result.artifacts.report_md,
    ] {
        assert!(
            Path::new(artifact).exists(),
            "artifact should exist: {artifact}"
        );
        assert_path_stays_under(artifact, &artifact_root);
    }

    let sheet_path = result.artifacts.evidence_sheet_jpg.as_ref().unwrap();
    assert!(Path::new(sheet_path).exists());
    assert_path_stays_under(sheet_path, &artifact_root);
}

fn local_smoke_root() -> PathBuf {
    std::env::var_os("DEEP_VIDEO_LOCAL_SMOKE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("douyin-creator-toolkit-local-video-smoke"))
}

fn generate_sample_video(video_path: &Path) {
    let ffmpeg = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("ffmpeg")
        .join(if cfg!(windows) {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        });
    assert!(ffmpeg.exists(), "expected FFmpeg at {}", ffmpeg.display());

    let status = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=320x180:rate=10:duration=3.2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=880:duration=3.2",
            "-shortest",
            "-pix_fmt",
            "yuv420p",
            "-y",
            video_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();

    assert!(status.success(), "sample video generation should succeed");
    assert!(video_path.exists());
}
