use crate::deep_video::types::{
    CandidateCategory, CandidateSegment, CandidateSource, OcrInputItem, TranscriptInput,
};

pub fn mine_candidate_segments(
    transcript: Option<&TranscriptInput>,
    ocr_items: &[OcrInputItem],
    reference_text: Option<&str>,
) -> Vec<CandidateSegment> {
    let mut candidates = Vec::new();

    if let Some(transcript) = transcript {
        for segment in &transcript.segments {
            if let Some(category) = classify_text(&segment.text, candidates.is_empty()) {
                push_candidate(
                    &mut candidates,
                    category,
                    segment.start_seconds,
                    segment.end_seconds,
                    "ASR segment matched short-video structure keywords",
                    &segment.text,
                    CandidateSource::Asr,
                );
            }
        }

        if candidates.is_empty() && !transcript.text.trim().is_empty() {
            push_candidate(
                &mut candidates,
                CandidateCategory::Hook,
                None,
                None,
                "Transcript has no segments; full transcript kept as opening candidate",
                &transcript.text,
                CandidateSource::Asr,
            );
        }
    }

    for item in ocr_items {
        if let Some(category) = classify_text(&item.text, false) {
            push_candidate(
                &mut candidates,
                category,
                item.timestamp_seconds,
                item.timestamp_seconds,
                "OCR text matched visual evidence keywords",
                &item.text,
                CandidateSource::Ocr,
            );
        }
    }

    if let Some(reference_text) = reference_text {
        if let Some(category) = classify_text(reference_text, false) {
            push_candidate(
                &mut candidates,
                category,
                None,
                None,
                "Reference text matched analysis keywords",
                reference_text,
                CandidateSource::ReferenceText,
            );
        }
    }

    candidates
}

fn classify_text(text: &str, is_opening: bool) -> Option<CandidateCategory> {
    let normalized = text.trim().to_lowercase();
    if normalized.is_empty() {
        return None;
    }

    if is_opening || contains_any(&normalized, &["你是不是", "有没有", "别再", "为什么", "如果你"]) {
        return Some(CandidateCategory::Hook);
    }
    if contains_any(&normalized, &["痛点", "问题", "困扰", "踩坑", "失败"]) {
        return Some(CandidateCategory::PainPoint);
    }
    if contains_any(&normalized, &["好处", "收益", "提升", "解决", "省钱", "省时"]) {
        return Some(CandidateCategory::Benefit);
    }
    if contains_any(&normalized, &["但是", "其实", "反而", "关键是", "重点来了"]) {
        return Some(CandidateCategory::TurningPoint);
    }
    if contains_any(&normalized, &["案例", "证明", "实测", "数据", "对比"]) {
        return Some(CandidateCategory::Proof);
    }
    if contains_any(&normalized, &["福利", "优惠", "立减", "限时", "价格", "元"]) {
        return Some(CandidateCategory::Offer);
    }
    if contains_any(&normalized, &["注意", "风险", "不要", "警告", "避开"]) {
        return Some(CandidateCategory::Warning);
    }
    if contains_any(&normalized, &["关注", "评论", "私信", "下单", "领取"]) {
        return Some(CandidateCategory::Cta);
    }

    None
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn push_candidate(
    candidates: &mut Vec<CandidateSegment>,
    category: CandidateCategory,
    start_seconds: Option<f32>,
    end_seconds: Option<f32>,
    reason: &str,
    text_excerpt: &str,
    source: CandidateSource,
) {
    let segment_id = format!("candidate-{:03}", candidates.len() + 1);
    candidates.push(CandidateSegment {
        segment_id,
        category,
        start_seconds,
        end_seconds,
        reason: reason.to_string(),
        text_excerpt: text_excerpt.chars().take(180).collect(),
        source,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_video::types::{
        CandidateCategory, CandidateSource, OcrInputItem, TranscriptInput, TranscriptSegment,
    };

    #[test]
    fn mines_hook_from_opening_segment() {
        let transcript = TranscriptInput {
            text: "你是不是也遇到这个问题？后面我给你一个方法".to_string(),
            segments: vec![
                TranscriptSegment {
                    text: "你是不是也遇到这个问题？".to_string(),
                    start_seconds: Some(0.0),
                    end_seconds: Some(3.0),
                },
                TranscriptSegment {
                    text: "后面我给你一个方法".to_string(),
                    start_seconds: Some(3.0),
                    end_seconds: Some(8.0),
                },
            ],
        };

        let segments = mine_candidate_segments(Some(&transcript), &[], None);

        assert!(segments.iter().any(|item| {
            item.category == CandidateCategory::Hook
                && item.source == CandidateSource::Asr
                && item.start_seconds == Some(0.0)
        }));
    }

    #[test]
    fn mines_offer_from_ocr_text() {
        let ocr = vec![OcrInputItem {
            frame_index: Some(4),
            timestamp_seconds: Some(12.0),
            image_path: Some("frame-004.jpg".to_string()),
            text: "限时福利 立减 99 元".to_string(),
        }];

        let segments = mine_candidate_segments(None, &ocr, None);

        assert_eq!(segments[0].category, CandidateCategory::Offer);
        assert_eq!(segments[0].source, CandidateSource::Ocr);
        assert_eq!(segments[0].start_seconds, Some(12.0));
    }
}
