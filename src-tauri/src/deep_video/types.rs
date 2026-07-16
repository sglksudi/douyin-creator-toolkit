use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeepVideoSource {
    LocalVideo {
        video_path: String,
    },
    DownloadedDouyinVideo {
        video_path: String,
        source_url: String,
    },
    TextOnly {
        source_url: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeepVideoAnalysisRequest {
    pub source: DeepVideoSource,
    pub task_id: Option<String>,
    pub title: String,
    pub profile: AnalysisProfile,
    #[serde(default)]
    pub use_frame_analysis: bool,
    pub transcript: Option<TranscriptInput>,
    #[serde(default)]
    pub ocr_items: Vec<OcrInputItem>,
    pub reference_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptInput {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub segments: Vec<TranscriptSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptSegment {
    pub text: String,
    pub start_seconds: Option<f32>,
    pub end_seconds: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OcrInputItem {
    pub frame_index: Option<usize>,
    pub timestamp_seconds: Option<f32>,
    pub image_path: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisProfile {
    Economy,
    Balanced,
    Precise,
    Custom(AnalysisProfileOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisProfileOptions {
    pub max_frames: usize,
    pub interval_seconds: f32,
    pub candidate_window_seconds: f32,
    pub frames_per_candidate: usize,
    pub use_scene_boundaries: bool,
    pub vision_passes: usize,
}

impl AnalysisProfile {
    pub fn normalized_options(&self) -> AnalysisProfileOptions {
        let options = match self {
            AnalysisProfile::Economy => AnalysisProfileOptions {
                max_frames: 12,
                interval_seconds: 8.0,
                candidate_window_seconds: 6.0,
                frames_per_candidate: 1,
                use_scene_boundaries: false,
                vision_passes: 1,
            },
            AnalysisProfile::Balanced => AnalysisProfileOptions {
                max_frames: 24,
                interval_seconds: 5.0,
                candidate_window_seconds: 8.0,
                frames_per_candidate: 2,
                use_scene_boundaries: true,
                vision_passes: 1,
            },
            AnalysisProfile::Precise => AnalysisProfileOptions {
                max_frames: 48,
                interval_seconds: 3.0,
                candidate_window_seconds: 10.0,
                frames_per_candidate: 3,
                use_scene_boundaries: true,
                vision_passes: 2,
            },
            AnalysisProfile::Custom(options) => options.clone(),
        };

        options.clamped()
    }
}

impl AnalysisProfileOptions {
    pub fn clamped(self) -> Self {
        Self {
            max_frames: self.max_frames.clamp(1, 96),
            interval_seconds: self.interval_seconds.clamp(1.0, 30.0),
            candidate_window_seconds: self.candidate_window_seconds.clamp(2.0, 30.0),
            frames_per_candidate: self.frames_per_candidate.clamp(1, 5),
            use_scene_boundaries: self.use_scene_boundaries,
            vision_passes: self.vision_passes.clamp(0, 3),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrameSource {
    Interval,
    Scene,
    TextCandidate,
    Opening,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceFrame {
    pub frame_id: String,
    pub index: usize,
    pub timestamp_seconds: Option<f32>,
    pub image_path: String,
    pub source: FrameSource,
}

impl EvidenceFrame {
    pub fn new(
        index: usize,
        timestamp_seconds: Option<f32>,
        image_path: String,
        source: FrameSource,
    ) -> Self {
        Self {
            frame_id: format!("#{index:03}"),
            index,
            timestamp_seconds,
            image_path,
            source,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceSheet {
    pub image_path: String,
    pub frames: Vec<EvidenceFrame>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateCategory {
    Hook,
    PainPoint,
    Benefit,
    TurningPoint,
    Proof,
    Offer,
    Warning,
    Cta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateSource {
    Asr,
    Ocr,
    ReferenceText,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateSegment {
    pub segment_id: String,
    pub category: CandidateCategory,
    pub start_seconds: Option<f32>,
    pub end_seconds: Option<f32>,
    pub reason: String,
    pub text_excerpt: String,
    pub source: CandidateSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Asr,
    Ocr,
    Candidate,
    Frame,
    Scene,
    Screenshot,
    Vision,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceTimeRange {
    pub start_seconds: Option<f32>,
    pub end_seconds: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceItem {
    pub kind: EvidenceKind,
    pub title: String,
    pub time_range: EvidenceTimeRange,
    pub reference: String,
    pub excerpt: String,
    #[serde(default)]
    pub frame_ids: Vec<String>,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeepVideoArtifacts {
    pub request_json: String,
    pub frame_result_json: String,
    pub evidence_sheet_jpg: Option<String>,
    pub candidate_segments_json: String,
    pub vision_result_json: Option<String>,
    pub evidence_timeline_json: String,
    pub result_json: String,
    pub report_md: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeepVideoAnalysisResult {
    pub task_id: String,
    pub title: String,
    pub source_video_path: Option<String>,
    pub profile: AnalysisProfile,
    pub frames: Vec<EvidenceFrame>,
    pub evidence_sheet: Option<EvidenceSheet>,
    pub candidates: Vec<CandidateSegment>,
    pub timeline: Vec<EvidenceItem>,
    pub report_markdown: String,
    pub artifacts: DeepVideoArtifacts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_defaults_match_product_tiers() {
        assert_eq!(
            AnalysisProfile::Economy.normalized_options(),
            AnalysisProfileOptions {
                max_frames: 12,
                interval_seconds: 8.0,
                candidate_window_seconds: 6.0,
                frames_per_candidate: 1,
                use_scene_boundaries: false,
                vision_passes: 1,
            }
        );
        assert_eq!(
            AnalysisProfile::Balanced.normalized_options().max_frames,
            24
        );
        assert_eq!(AnalysisProfile::Precise.normalized_options().max_frames, 48);
    }

    #[test]
    fn custom_profile_values_are_clamped() {
        let profile = AnalysisProfile::Custom(AnalysisProfileOptions {
            max_frames: 500,
            interval_seconds: 0.2,
            candidate_window_seconds: 99.0,
            frames_per_candidate: 9,
            use_scene_boundaries: true,
            vision_passes: 7,
        });

        assert_eq!(
            profile.normalized_options(),
            AnalysisProfileOptions {
                max_frames: 96,
                interval_seconds: 1.0,
                candidate_window_seconds: 30.0,
                frames_per_candidate: 5,
                use_scene_boundaries: true,
                vision_passes: 3,
            }
        );
    }

    #[test]
    fn evidence_frame_assigns_stable_frame_id() {
        let frame = EvidenceFrame::new(
            3,
            Some(10.25),
            "deep-video/task/frames/frame-003.jpg".to_string(),
            FrameSource::Interval,
        );

        assert_eq!(frame.frame_id, "#003");
        assert_eq!(frame.index, 3);
        assert_eq!(frame.timestamp_seconds, Some(10.25));
    }

    #[test]
    fn request_defaults_frame_analysis_to_disabled() {
        let request: DeepVideoAnalysisRequest = serde_json::from_value(serde_json::json!({
            "source": { "local_video": { "video_path": "sample.mp4" } },
            "task_id": null,
            "title": "sample.mp4",
            "profile": "balanced",
            "transcript": null,
            "ocr_items": [],
            "reference_text": null
        }))
        .unwrap();

        assert!(!request.use_frame_analysis);
    }
}
