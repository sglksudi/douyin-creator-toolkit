use crate::core::video_processor::VideoProcessor;
use crate::deep_video::types::{AnalysisProfileOptions, EvidenceFrame, FrameSource};
use std::path::Path;

pub async fn sample_interval_frames(
    video_path: &Path,
    frames_dir: &Path,
    options: &AnalysisProfileOptions,
) -> Result<Vec<EvidenceFrame>, String> {
    std::fs::create_dir_all(frames_dir).map_err(|error| error.to_string())?;

    let processor = VideoProcessor::new().map_err(|error| error.to_string())?;
    let info = processor
        .get_video_info(video_path)
        .await
        .map_err(|error| error.to_string())?;

    let timestamps = build_interval_timestamps(info.duration_ms, options);
    let mut frames = Vec::new();

    for (position, timestamp) in timestamps.into_iter().enumerate() {
        let index = position + 1;
        let image_path = frames_dir.join(format!("frame-{index:03}.jpg"));
        processor
            .generate_thumbnail(video_path, &image_path, timestamp, Some(480))
            .await
            .map_err(|error| error.to_string())?;
        frames.push(EvidenceFrame::new(
            index,
            Some(timestamp),
            image_path.to_string_lossy().to_string(),
            if index == 1 {
                FrameSource::Opening
            } else {
                FrameSource::Interval
            },
        ));
    }

    Ok(frames)
}

pub fn build_interval_timestamps(duration_ms: u64, options: &AnalysisProfileOptions) -> Vec<f32> {
    let duration_seconds = duration_ms as f32 / 1000.0;
    if duration_seconds <= 0.0 {
        return vec![0.0];
    }

    let mut timestamps = Vec::new();
    let mut current = 0.0;
    while current < duration_seconds && timestamps.len() < options.max_frames {
        timestamps.push(round_to_tenth(current));
        current += options.interval_seconds;
    }

    if timestamps.is_empty() {
        timestamps.push(0.0);
    }

    timestamps
}

pub fn parse_showinfo_timestamps(stderr: &str) -> Vec<f32> {
    stderr
        .lines()
        .filter_map(|line| {
            let marker = "pts_time:";
            let start = line.find(marker)? + marker.len();
            let rest = &line[start..];
            let value = rest.split_whitespace().next()?;
            value.parse::<f32>().ok()
        })
        .collect()
}

fn round_to_tenth(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_video::types::AnalysisProfileOptions;

    #[test]
    fn builds_interval_timestamps_from_duration_and_options() {
        let options = AnalysisProfileOptions {
            max_frames: 4,
            interval_seconds: 5.0,
            candidate_window_seconds: 8.0,
            frames_per_candidate: 2,
            use_scene_boundaries: true,
            vision_passes: 1,
        };

        assert_eq!(
            build_interval_timestamps(18_000, &options),
            vec![0.0, 5.0, 10.0, 15.0]
        );
    }

    #[test]
    fn parses_showinfo_pts_time_values() {
        let stderr = "[Parsed_showinfo_1] n: 0 pts: 1024 pts_time:2.133 pos:0\n[Parsed_showinfo_1] n: 1 pts: 2048 pts_time:4.266 pos:0";

        assert_eq!(parse_showinfo_timestamps(stderr), vec![2.133, 4.266]);
    }
}
