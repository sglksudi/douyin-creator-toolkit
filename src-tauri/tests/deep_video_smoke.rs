use douyin_creator_tools_lib::deep_video::runner::run_deep_video_analysis;
use douyin_creator_tools_lib::deep_video::types::{
    AnalysisProfile, DeepVideoAnalysisRequest, DeepVideoSource, EvidenceKind, TranscriptInput,
    TranscriptSegment,
};

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

#[tokio::test]
async fn text_only_deep_video_smoke_writes_report_without_frame_artifacts() {
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

    let result = run_deep_video_analysis(request).await.unwrap();

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
    assert!(!result.report_markdown.contains("contact sheet"));
}

#[tokio::test]
async fn frame_evidence_smoke_requires_existing_video_source() {
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

    let error = run_deep_video_analysis(request).await.unwrap_err();

    assert!(error.contains("Video source does not exist"));
}
