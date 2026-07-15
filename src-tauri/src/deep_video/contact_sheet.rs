use crate::deep_video::time_format::format_seconds;
use crate::deep_video::types::{EvidenceFrame, EvidenceSheet};
use chrono::Utc;
use font8x8::UnicodeFonts;
use image::{imageops, DynamicImage, GenericImage, ImageBuffer, Rgb, RgbImage};
use std::path::Path;

const CELL_WIDTH: u32 = 240;
const CELL_HEIGHT: u32 = 170;
const LABEL_HEIGHT: u32 = 26;
const COLUMNS: u32 = 4;

pub fn generate_contact_sheet(
    frames: &[EvidenceFrame],
    output_path: &Path,
) -> Result<EvidenceSheet, String> {
    if frames.is_empty() {
        return Err("No frames available for contact sheet".to_string());
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let rows = ((frames.len() as u32) + COLUMNS - 1) / COLUMNS;
    let mut sheet: RgbImage =
        ImageBuffer::from_pixel(COLUMNS * CELL_WIDTH, rows * CELL_HEIGHT, Rgb([18, 18, 20]));

    for (position, frame) in frames.iter().enumerate() {
        let col = (position as u32) % COLUMNS;
        let row = (position as u32) / COLUMNS;
        let x = col * CELL_WIDTH;
        let y = row * CELL_HEIGHT;

        let source = image::open(&frame.image_path)
            .map_err(|error| format!("Failed to read frame {}: {error}", frame.image_path))?;
        paste_frame(&mut sheet, source, x, y)?;
        draw_label_bar(&mut sheet, frame, x, y + CELL_HEIGHT - LABEL_HEIGHT);
    }

    DynamicImage::ImageRgb8(sheet)
        .save(output_path)
        .map_err(|error| error.to_string())?;

    Ok(EvidenceSheet {
        image_path: output_path.to_string_lossy().to_string(),
        frames: frames.to_vec(),
        generated_at: Utc::now(),
    })
}

fn paste_frame(sheet: &mut RgbImage, source: DynamicImage, x: u32, y: u32) -> Result<(), String> {
    let available_height = CELL_HEIGHT - LABEL_HEIGHT;
    let resized = source.resize(CELL_WIDTH, available_height, imageops::FilterType::Triangle);
    let rgb = resized.to_rgb8();
    sheet
        .copy_from(&rgb, x, y)
        .map_err(|error| error.to_string())
}

fn draw_label_bar(sheet: &mut RgbImage, frame: &EvidenceFrame, x: u32, y: u32) {
    for yy in y..(y + LABEL_HEIGHT).min(sheet.height()) {
        for xx in x..(x + CELL_WIDTH).min(sheet.width()) {
            sheet.put_pixel(xx, yy, Rgb([0, 0, 0]));
        }
    }

    let timestamp = frame
        .timestamp_seconds
        .map(format_seconds)
        .unwrap_or_else(|| "time unknown".to_string());
    let label = format!("{} {} {:?}", frame.frame_id, timestamp, frame.source);
    draw_text(sheet, x + 6, y + 7, &label, Rgb([255, 255, 255]));
}

fn draw_text(image: &mut RgbImage, x: u32, y: u32, text: &str, color: Rgb<u8>) {
    let mut cursor_x = x;
    for character in text.chars() {
        draw_char(image, cursor_x, y, character, color);
        cursor_x += 8;
        if cursor_x + 8 >= image.width() {
            break;
        }
    }
}

fn draw_char(image: &mut RgbImage, x: u32, y: u32, character: char, color: Rgb<u8>) {
    if let Some(glyph) = font8x8::BASIC_FONTS.get(character) {
        for (row, byte) in glyph.iter().enumerate() {
            for col in 0..8u32 {
                if byte & (1u8 << col) != 0 {
                    let px = x + col;
                    let py = y + row as u32;
                    if px < image.width() && py < image.height() {
                        image.put_pixel(px, py, color);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_video::types::{EvidenceFrame, FrameSource};
    use image::{ImageBuffer, Rgb};

    #[test]
    fn creates_contact_sheet_for_numbered_frames() {
        let temp = tempfile::tempdir().unwrap();
        let frame_path = temp.path().join("frame-001.jpg");
        let output_path = temp.path().join("evidence_sheet.jpg");

        let image = ImageBuffer::from_pixel(80, 60, Rgb([200u8, 210, 220]));
        image.save(&frame_path).unwrap();

        let frames = vec![EvidenceFrame::new(
            1,
            Some(5.0),
            frame_path.to_string_lossy().to_string(),
            FrameSource::Interval,
        )];

        let sheet = generate_contact_sheet(&frames, &output_path).unwrap();

        assert!(output_path.exists());
        assert_eq!(sheet.frames[0].frame_id, "#001");
        assert_eq!(sheet.image_path, output_path.to_string_lossy());
    }
}
