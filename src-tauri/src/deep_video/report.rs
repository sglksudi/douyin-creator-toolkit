use crate::deep_video::time_format::format_time_label;
use crate::deep_video::types::{CandidateSegment, EvidenceItem};

pub fn build_markdown_report(
    title: &str,
    timeline: &[EvidenceItem],
    candidates: &[CandidateSegment],
    has_frame_evidence: bool,
) -> String {
    let mut lines = vec![
        format!("# {title} Deep Video Analysis"),
        String::new(),
        "## Executive Summary".to_string(),
        format!(
            "- Built a no-vision evidence timeline with {} evidence items and {} text-guided candidates.",
            timeline.len(),
            candidates.len()
        ),
        if has_frame_evidence {
            "- Vision confirmation is disabled in Phase 1; claims are limited to transcript, OCR, and frame evidence.".to_string()
        } else {
            "- Frame evidence is disabled; claims are limited to transcript, OCR, and reference text.".to_string()
        },
        String::new(),
        "## Candidate Structure".to_string(),
    ];

    if candidates.is_empty() {
        lines.push("- No text-guided candidates were detected.".to_string());
    } else {
        for candidate in candidates {
            let range = crate::deep_video::types::EvidenceTimeRange {
                start_seconds: candidate.start_seconds,
                end_seconds: candidate.end_seconds,
            };
            lines.push(format!(
                "- {:?}: {} {}",
                candidate.category,
                candidate.text_excerpt,
                format_time_label(&range)
            ));
        }
    }

    lines.extend([String::new(), "## Evidence Timeline".to_string()]);

    for item in timeline {
        lines.push(format!(
            "- {:?}: {} {}",
            item.kind,
            item.excerpt,
            format_time_label(&item.time_range)
        ));
    }

    lines.extend([String::new(), "## Recommendations".to_string()]);
    if has_frame_evidence {
        lines.push(
            "- Add vision confirmation in Phase 2 for frame-level visual claims.".to_string(),
        );
        lines.push(
            "- Keep frame citations stable by referencing frame IDs from the contact sheet."
                .to_string(),
        );
    } else {
        lines.push(
            "- Enable frame evidence when visual proof is needed for the selected text candidates."
                .to_string(),
        );
        lines
            .push("- Keep text-only evidence separate from later visual confirmation.".to_string());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_video::types::{
        CandidateCategory, CandidateSegment, CandidateSource, EvidenceItem, EvidenceKind,
        EvidenceTimeRange,
    };

    #[test]
    fn report_contains_title_and_timestamp_citation() {
        let timeline = vec![EvidenceItem {
            kind: EvidenceKind::Candidate,
            title: "Hook candidate".to_string(),
            time_range: EvidenceTimeRange {
                start_seconds: Some(0.0),
                end_seconds: Some(3.0),
            },
            reference: "candidate-001".to_string(),
            excerpt: "你是不是也这样".to_string(),
            frame_ids: Vec::new(),
            thumbnail_path: None,
        }];
        let candidates = vec![CandidateSegment {
            segment_id: "candidate-001".to_string(),
            category: CandidateCategory::Hook,
            start_seconds: Some(0.0),
            end_seconds: Some(3.0),
            reason: "opening".to_string(),
            text_excerpt: "你是不是也这样".to_string(),
            source: CandidateSource::Asr,
        }];

        let report = build_markdown_report("sample.mp4", &timeline, &candidates, false);

        assert!(report.contains("# sample.mp4 Deep Video Analysis"));
        assert!(report.contains("[00:00.0-00:03.0]"));
    }

    #[test]
    fn text_only_report_omits_contact_sheet_guidance() {
        let report = build_markdown_report("sample.mp4", &[], &[], false);

        assert!(report.contains("Frame evidence is disabled"));
        assert!(!report.contains("contact sheet"));
        assert!(!report.contains("frame IDs"));
    }
}
