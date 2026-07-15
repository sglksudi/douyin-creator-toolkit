use crate::deep_video::types::{
    CandidateSegment, EvidenceFrame, EvidenceItem, EvidenceKind, EvidenceTimeRange, OcrInputItem,
    TranscriptInput,
};

pub fn build_evidence_timeline(
    transcript: Option<&TranscriptInput>,
    ocr_items: &[OcrInputItem],
    frames: &[EvidenceFrame],
    candidates: &[CandidateSegment],
) -> Vec<EvidenceItem> {
    let mut items = Vec::new();

    if let Some(transcript) = transcript {
        for (index, segment) in transcript.segments.iter().enumerate() {
            items.push(EvidenceItem {
                kind: EvidenceKind::Asr,
                title: format!("ASR segment {}", index + 1),
                time_range: EvidenceTimeRange {
                    start_seconds: segment.start_seconds,
                    end_seconds: segment.end_seconds,
                },
                reference: format!("asr-{:03}", index + 1),
                excerpt: segment.text.clone(),
                frame_ids: Vec::new(),
                thumbnail_path: None,
            });
        }
    }

    for (index, item) in ocr_items.iter().enumerate() {
        items.push(EvidenceItem {
            kind: EvidenceKind::Ocr,
            title: format!("OCR text {}", item.frame_index.unwrap_or(index + 1)),
            time_range: EvidenceTimeRange {
                start_seconds: item.timestamp_seconds,
                end_seconds: item.timestamp_seconds,
            },
            reference: item
                .image_path
                .clone()
                .unwrap_or_else(|| format!("ocr-{:03}", index + 1)),
            excerpt: item.text.clone(),
            frame_ids: item
                .frame_index
                .map(|value| vec![format!("#{value:03}")])
                .unwrap_or_default(),
            thumbnail_path: item.image_path.clone(),
        });
    }

    for candidate in candidates {
        items.push(EvidenceItem {
            kind: EvidenceKind::Candidate,
            title: format!("{:?} candidate", candidate.category),
            time_range: EvidenceTimeRange {
                start_seconds: candidate.start_seconds,
                end_seconds: candidate.end_seconds,
            },
            reference: candidate.segment_id.clone(),
            excerpt: candidate.text_excerpt.clone(),
            frame_ids: Vec::new(),
            thumbnail_path: None,
        });
    }

    for frame in frames {
        items.push(EvidenceItem {
            kind: EvidenceKind::Frame,
            title: format!("Key frame {}", frame.frame_id),
            time_range: EvidenceTimeRange {
                start_seconds: frame.timestamp_seconds,
                end_seconds: frame.timestamp_seconds,
            },
            reference: frame.image_path.clone(),
            excerpt: format!("{:?} frame", frame.source),
            frame_ids: vec![frame.frame_id.clone()],
            thumbnail_path: Some(frame.image_path.clone()),
        });
    }

    items.sort_by(compare_evidence_items);
    items
}

fn compare_evidence_items(left: &EvidenceItem, right: &EvidenceItem) -> std::cmp::Ordering {
    let left_known = left.time_range.start_seconds.is_some();
    let right_known = right.time_range.start_seconds.is_some();
    match (left_known, right_known) {
        (true, false) => return std::cmp::Ordering::Less,
        (false, true) => return std::cmp::Ordering::Greater,
        _ => {}
    }

    let left_time = left.time_range.start_seconds.unwrap_or(f32::MAX);
    let right_time = right.time_range.start_seconds.unwrap_or(f32::MAX);
    left_time
        .partial_cmp(&right_time)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then(kind_priority(&left.kind).cmp(&kind_priority(&right.kind)))
}

fn kind_priority(kind: &EvidenceKind) -> usize {
    match kind {
        EvidenceKind::Asr => 0,
        EvidenceKind::Ocr => 1,
        EvidenceKind::Candidate => 2,
        EvidenceKind::Frame => 3,
        EvidenceKind::Scene => 4,
        EvidenceKind::Screenshot => 5,
        EvidenceKind::Vision => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_video::types::{
        CandidateCategory, CandidateSegment, CandidateSource, EvidenceFrame, EvidenceKind,
        FrameSource,
    };

    #[test]
    fn timeline_sorts_known_times_before_unknown_times() {
        let frames = vec![
            EvidenceFrame::new(1, None, "frame-001.jpg".to_string(), FrameSource::Scene),
            EvidenceFrame::new(
                2,
                Some(3.0),
                "frame-002.jpg".to_string(),
                FrameSource::Interval,
            ),
        ];
        let candidates = vec![CandidateSegment {
            segment_id: "candidate-001".to_string(),
            category: CandidateCategory::Hook,
            start_seconds: Some(1.0),
            end_seconds: Some(2.0),
            reason: "opening".to_string(),
            text_excerpt: "你是不是也这样".to_string(),
            source: CandidateSource::Asr,
        }];

        let timeline = build_evidence_timeline(None, &[], &frames, &candidates);

        assert_eq!(timeline[0].kind, EvidenceKind::Candidate);
        assert_eq!(timeline[1].kind, EvidenceKind::Frame);
        assert_eq!(timeline[2].time_range.start_seconds, None);
    }

    #[test]
    fn timeline_includes_frame_ids_and_thumbnails() {
        let frames = vec![EvidenceFrame::new(
            7,
            Some(12.0),
            "frame-007.jpg".to_string(),
            FrameSource::Interval,
        )];

        let timeline = build_evidence_timeline(None, &[], &frames, &[]);

        assert_eq!(timeline[0].frame_ids, vec!["#007"]);
        assert_eq!(
            timeline[0].thumbnail_path,
            Some("frame-007.jpg".to_string())
        );
    }
}
