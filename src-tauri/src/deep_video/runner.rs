use crate::deep_video::artifacts::{create_artifact_paths, write_json, write_text};
use crate::deep_video::candidate_mining::mine_candidate_segments;
use crate::deep_video::contact_sheet::generate_contact_sheet;
use crate::deep_video::evidence_timeline::build_evidence_timeline;
use crate::deep_video::frame_sampling::sample_interval_frames;
use crate::deep_video::report::build_markdown_report;
use crate::deep_video::types::{
    CandidateSegment, DeepVideoAnalysisRequest, DeepVideoAnalysisResult, DeepVideoArtifacts,
    DeepVideoSource, EvidenceFrame, EvidenceSheet,
};
use std::path::PathBuf;

pub async fn run_deep_video_analysis(
    request: DeepVideoAnalysisRequest,
) -> Result<DeepVideoAnalysisResult, String> {
    let task_id = request
        .task_id
        .clone()
        .unwrap_or_else(|| format!("deep-video-{}", chrono::Utc::now().timestamp_millis()));
    let paths = create_artifact_paths(&task_id)?;
    let candidates = mine_candidate_segments(
        request.transcript.as_ref(),
        &request.ocr_items,
        request.reference_text.as_deref(),
    );
    let (frames, sheet) = if request.use_frame_analysis {
        let video_path = source_video_path(&request.source)
            .ok_or_else(|| "Frame evidence requires a video source".to_string())?;
        if !video_path.exists() {
            return Err(format!(
                "Video source does not exist: {}",
                video_path.display()
            ));
        }

        let options = request.profile.normalized_options();
        let frames = sample_interval_frames(&video_path, &paths.frames_dir, &options).await?;
        let sheet = generate_contact_sheet(&frames, &paths.evidence_sheet_jpg)?;
        (frames, Some(sheet))
    } else {
        (Vec::new(), None)
    };

    let artifacts = DeepVideoArtifacts {
        request_json: paths.request_json.to_string_lossy().to_string(),
        frame_result_json: paths.frame_result_json.to_string_lossy().to_string(),
        evidence_sheet_jpg: request
            .use_frame_analysis
            .then(|| paths.evidence_sheet_jpg.to_string_lossy().to_string()),
        candidate_segments_json: paths.candidate_segments_json.to_string_lossy().to_string(),
        vision_result_json: None,
        evidence_timeline_json: paths.evidence_timeline_json.to_string_lossy().to_string(),
        result_json: paths.result_json.to_string_lossy().to_string(),
        report_md: paths.report_md.to_string_lossy().to_string(),
    };

    let result = assemble_analysis_result(&task_id, &request, frames, sheet, candidates, artifacts);

    write_json(&paths.request_json, &request)?;
    write_json(&paths.frame_result_json, &result.frames)?;
    write_json(&paths.candidate_segments_json, &result.candidates)?;
    write_json(&paths.evidence_timeline_json, &result.timeline)?;
    write_text(&paths.report_md, &result.report_markdown)?;
    write_json(&paths.result_json, &result)?;

    Ok(result)
}

pub fn assemble_analysis_result(
    task_id: &str,
    request: &DeepVideoAnalysisRequest,
    frames: Vec<EvidenceFrame>,
    evidence_sheet: Option<EvidenceSheet>,
    candidates: Vec<CandidateSegment>,
    artifacts: DeepVideoArtifacts,
) -> DeepVideoAnalysisResult {
    let timeline = build_evidence_timeline(
        request.transcript.as_ref(),
        &request.ocr_items,
        &frames,
        &candidates,
    );
    let report_markdown = build_markdown_report(
        &request.title,
        &timeline,
        &candidates,
        evidence_sheet.is_some(),
    );
    DeepVideoAnalysisResult {
        task_id: task_id.to_string(),
        title: request.title.clone(),
        source_video_path: source_video_path(&request.source)
            .map(|path| path.to_string_lossy().to_string()),
        profile: request.profile.clone(),
        frames,
        evidence_sheet,
        candidates,
        timeline,
        report_markdown,
        artifacts,
    }
}

fn source_video_path(source: &DeepVideoSource) -> Option<PathBuf> {
    match source {
        DeepVideoSource::LocalVideo { video_path } => Some(PathBuf::from(video_path)),
        DeepVideoSource::DownloadedDouyinVideo { video_path, .. } => {
            Some(PathBuf::from(video_path))
        }
        DeepVideoSource::TextOnly { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_video::types::{
        AnalysisProfile, CandidateCategory, CandidateSegment, CandidateSource,
        DeepVideoAnalysisRequest, DeepVideoArtifacts, DeepVideoSource, EvidenceFrame,
        EvidenceSheet, FrameSource,
    };
    use chrono::Utc;

    #[test]
    fn assembles_result_from_components() {
        let request = DeepVideoAnalysisRequest {
            source: DeepVideoSource::LocalVideo {
                video_path: "sample.mp4".to_string(),
            },
            task_id: Some("task-1".to_string()),
            title: "sample.mp4".to_string(),
            profile: AnalysisProfile::Economy,
            use_frame_analysis: true,
            transcript: None,
            ocr_items: Vec::new(),
            reference_text: None,
        };
        let frames = vec![EvidenceFrame::new(
            1,
            Some(0.0),
            "frame-001.jpg".to_string(),
            FrameSource::Opening,
        )];
        let sheet = EvidenceSheet {
            image_path: "evidence_sheet.jpg".to_string(),
            frames: frames.clone(),
            generated_at: Utc::now(),
        };
        let candidates = vec![CandidateSegment {
            segment_id: "candidate-001".to_string(),
            category: CandidateCategory::Hook,
            start_seconds: Some(0.0),
            end_seconds: Some(2.0),
            reason: "opening".to_string(),
            text_excerpt: "opening".to_string(),
            source: CandidateSource::Asr,
        }];

        let result = assemble_analysis_result(
            "task-1",
            &request,
            frames,
            Some(sheet),
            candidates,
            DeepVideoArtifacts {
                request_json: "request.json".to_string(),
                frame_result_json: "frame_result.json".to_string(),
                evidence_sheet_jpg: Some("evidence_sheet.jpg".to_string()),
                candidate_segments_json: "candidate_segments.json".to_string(),
                vision_result_json: None,
                evidence_timeline_json: "timeline.json".to_string(),
                result_json: "result.json".to_string(),
                report_md: "report.md".to_string(),
            },
        );

        assert_eq!(result.task_id, "task-1");
        assert!(result
            .report_markdown
            .contains("sample.mp4 Deep Video Analysis"));
        assert_eq!(result.timeline.len(), 2);
    }

    #[tokio::test]
    async fn text_only_analysis_does_not_require_existing_video_source() {
        let request = DeepVideoAnalysisRequest {
            source: DeepVideoSource::LocalVideo {
                video_path: "C:/tmp/does-not-exist/text-only.mp4".to_string(),
            },
            task_id: Some(format!(
                "text-only-{}",
                chrono::Utc::now().timestamp_millis()
            )),
            title: "text only".to_string(),
            profile: AnalysisProfile::Balanced,
            use_frame_analysis: false,
            transcript: Some(crate::deep_video::types::TranscriptInput {
                text: "Limited offer. Tap now to claim it.".to_string(),
                segments: vec![crate::deep_video::types::TranscriptSegment {
                    text: "Limited offer. Tap now to claim it.".to_string(),
                    start_seconds: Some(1.0),
                    end_seconds: Some(4.0),
                }],
            }),
            ocr_items: Vec::new(),
            reference_text: None,
        };

        let result = run_deep_video_analysis(request).await.unwrap();

        assert!(result.frames.is_empty());
        assert!(result.evidence_sheet.is_none());
        assert!(result.artifacts.evidence_sheet_jpg.is_none());
        assert!(result
            .timeline
            .iter()
            .any(|item| item.kind == crate::deep_video::types::EvidenceKind::Candidate));
        assert!(!result.report_markdown.contains("contact sheet"));
        assert!(!result.report_markdown.contains("frame IDs"));
    }
}
