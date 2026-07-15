use crate::deep_video::types::{EvidenceFrame, EvidenceTimeRange};

pub fn format_time_label(range: &EvidenceTimeRange) -> String {
    match (range.start_seconds, range.end_seconds) {
        (Some(start), Some(end)) if (start - end).abs() >= 0.05 => {
            format!("[{}-{}]", format_seconds(start), format_seconds(end))
        }
        (Some(start), _) => format!("[{}]", format_seconds(start)),
        _ => "[time unknown]".to_string(),
    }
}

pub fn format_frame_citation(frame: &EvidenceFrame) -> String {
    let range = EvidenceTimeRange {
        start_seconds: frame.timestamp_seconds,
        end_seconds: frame.timestamp_seconds,
    };

    format!("{} {}", frame.frame_id, format_time_label(&range))
}

pub fn format_seconds(seconds: f32) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "time unknown".to_string();
    }

    let tenths = (seconds * 10.0).round() as u64;
    let total_seconds = tenths / 10;
    let tenth = tenths % 10;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    format!("{minutes:02}:{seconds:02}.{tenth}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_video::types::{EvidenceFrame, EvidenceTimeRange, FrameSource};

    #[test]
    fn formats_single_time_with_rollover() {
        let range = EvidenceTimeRange {
            start_seconds: Some(59.95),
            end_seconds: Some(59.95),
        };

        assert_eq!(format_time_label(&range), "[01:00.0]");
    }

    #[test]
    fn formats_time_range_with_rollover() {
        let range = EvidenceTimeRange {
            start_seconds: Some(119.95),
            end_seconds: Some(125.24),
        };

        assert_eq!(format_time_label(&range), "[02:00.0-02:05.2]");
    }

    #[test]
    fn formats_unknown_time() {
        let range = EvidenceTimeRange {
            start_seconds: None,
            end_seconds: None,
        };

        assert_eq!(format_time_label(&range), "[time unknown]");
    }

    #[test]
    fn formats_frame_citation() {
        let frame = EvidenceFrame::new(
            2,
            Some(5.4),
            "deep-video/task/frames/frame-002.jpg".to_string(),
            FrameSource::Interval,
        );

        assert_eq!(format_frame_citation(&frame), "#002 [00:05.4]");
    }
}
