pub mod cache;
pub mod converter;

use anyhow::Result;
use image::{DynamicImage, Rgba, RgbaImage};
use ratatui::style::Color;
use std::path::PathBuf;

const PIXELATION_BLOCK_SIZE: u32 = 8;

#[derive(Clone)]
pub struct ArtworkManager {
    cache: cache::ArtworkCache,
}

impl ArtworkManager {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache: cache::ArtworkCache::new(cache_dir, 100),
        }
    }

    pub async fn get_artwork_themed_v2(
        &self,
        url: &str,
        dark: Color,
        light: Color,
        theme_name: &str,
        mosaic: bool,
        is_retro: bool,
    ) -> Result<DynamicImage> {
        let themed_url = format!(
            "{}-{}-mosaic-{}-retro-{}",
            theme_name, url, mosaic, is_retro
        );

        if let Some(img) = self.cache.get(&themed_url) {
            return Ok(img);
        }

        let response = reqwest::get(url).await?.bytes().await?;
        let img = image::load_from_memory(&response)?;

        // Apply duotone theme only for retro themes
        let processed_img = if is_retro {
            apply_duotone_theme(img, dark, light)
        } else {
            img
        };

        // Optionally apply mosaic effect on top
        let themed_img = if mosaic {
            apply_pixelation(processed_img)
        } else {
            processed_img
        };

        self.cache.insert(themed_url, themed_img.clone());
        Ok(themed_img)
    }
}

fn apply_pixelation(img: DynamicImage) -> DynamicImage {
    let source = img.to_rgba8();
    let (width, height) = source.dimensions();
    let block_size = pixelation_block_size(width, height);
    let mut output = RgbaImage::new(width, height);

    for block_y in (0..height).step_by(block_size as usize) {
        for block_x in (0..width).step_by(block_size as usize) {
            let x_end = (block_x + block_size).min(width);
            let y_end = (block_y + block_size).min(height);
            let color = average_block_color(&source, block_x, block_y, x_end, y_end);

            for y in block_y..y_end {
                for x in block_x..x_end {
                    output.put_pixel(x, y, color);
                }
            }
        }
    }

    DynamicImage::ImageRgba8(output)
}

fn pixelation_block_size(_width: u32, _height: u32) -> u32 {
    PIXELATION_BLOCK_SIZE
}

fn average_block_color(
    img: &RgbaImage,
    x_start: u32,
    y_start: u32,
    x_end: u32,
    y_end: u32,
) -> Rgba<u8> {
    let mut sum_r = 0_u64;
    let mut sum_g = 0_u64;
    let mut sum_b = 0_u64;
    let mut sum_a = 0_u64;
    let mut count = 0_u64;

    for y in y_start..y_end {
        for x in x_start..x_end {
            let pixel = img.get_pixel(x, y);
            let alpha = pixel[3] as u64;

            sum_r += pixel[0] as u64 * alpha;
            sum_g += pixel[1] as u64 * alpha;
            sum_b += pixel[2] as u64 * alpha;
            sum_a += alpha;
            count += 1;
        }
    }

    if count == 0 || sum_a == 0 {
        return Rgba([0, 0, 0, 0]);
    }

    let avg_a = ((sum_a + count / 2) / count) as u8;
    let avg_r = ((sum_r + sum_a / 2) / sum_a) as u8;
    let avg_g = ((sum_g + sum_a / 2) / sum_a) as u8;
    let avg_b = ((sum_b + sum_a / 2) / sum_a) as u8;

    Rgba([avg_r, avg_g, avg_b, avg_a])
}

fn get_relative_luminance(r: f32, g: f32, b: f32) -> f32 {
    // Relative luminance formula (ITU-R BT.709)
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn apply_duotone_theme(img: DynamicImage, dark: Color, light: Color) -> DynamicImage {
    let (_d_r, _d_g, _d_b) = extract_rgb(dark);
    let (l_r, l_g, l_b) = extract_rgb(light);

    let mut grayscale = img.grayscale().to_rgba8();
    let (width, height) = grayscale.dimensions();

    // Calculate perceived brightness of the light theme color
    let luminance = get_relative_luminance(l_r, l_g, l_b);

    // Normalize brightness and gamma based on the theme color's luminance.
    // Darker colors (like Red) need higher brightness boost and lower gamma
    // to bring out mid-tone details that would otherwise be lost.
    //
    // Reference Luminance (Amber-like): ~150-180
    // Red Alert Luminance: ~54
    // Cyan VFD Luminance: ~255

    let base_brightness = 0.8;
    let base_gamma = 0.45;

    let (brightness_factor, gamma) = if luminance < 160.0 {
        // Boost factor: scale from 1.0 (at 160) up to 1.4 (at 50)
        let boost = (1.4 - (luminance - 50.0) * (0.4 / 110.0)).clamp(1.0, 1.4);
        (base_brightness * boost, base_gamma * (1.0 / boost.sqrt()))
    } else {
        (base_brightness, base_gamma)
    };

    // Define black point threshold - MORE AGGRESSIVE
    // Pixels below this threshold map to pure BLACK/GRAY (no color tint)
    // Pixels above this threshold map to the theme color
    let black_point = 0.35;

    for y in 0..height {
        for x in 0..width {
            let pixel = grayscale.get_pixel(x, y);
            let raw_intensity = pixel[0] as f32 / 255.0;

            // Apply linear scaling then gamma correction
            let intensity = (raw_intensity * brightness_factor).powf(gamma);

            let (r, g, b) = if intensity < black_point {
                // CRITICAL: Dark regions map to pure BLACK to DARK GRAY
                // NO THEME COLOR TINT in the shadows!
                // This creates the true "black point" with high contrast
                let shadow_value = (intensity / black_point * 10.0).clamp(0.0, 10.0) as u8;
                (shadow_value, shadow_value, shadow_value)
            } else {
                // Bright regions map to theme color
                // Remap intensity from [black_point, 1.0] to [0.0, 1.0]
                let remapped = (intensity - black_point) / (1.0 - black_point);

                let r = (l_r * remapped).clamp(0.0, 255.0) as u8;
                let g = (l_g * remapped).clamp(0.0, 255.0) as u8;
                let b = (l_b * remapped).clamp(0.0, 255.0) as u8;
                (r, g, b)
            };

            grayscale.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    DynamicImage::ImageRgba8(grayscale)
}

fn extract_rgb(color: Color) -> (f32, f32, f32) {
    match color {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        Color::Indexed(idx) => {
            // Approximate indexed colors based on standard terminal palette
            // Using darker values to reduce gamma/brightness
            match idx {
                0 => (0.0, 0.0, 0.0),        // Black
                1 => (160.0, 35.0, 35.0),    // Red (darker)
                2 => (35.0, 160.0, 35.0),    // Green
                3 => (160.0, 160.0, 0.0),    // Yellow
                4 => (20.0, 100.0, 200.0),   // Blue (darker dodger blue)
                5 => (160.0, 70.0, 160.0),   // Magenta
                6 => (40.0, 170.0, 160.0),   // Cyan (darker turquoise)
                7 => (180.0, 180.0, 180.0),  // White
                8 => (90.0, 90.0, 90.0),     // Bright Black (gray)
                9 => (200.0, 70.0, 50.0),    // Bright Red
                10 => (100.0, 190.0, 100.0), // Bright Green
                11 => (200.0, 200.0, 70.0),  // Bright Yellow
                12 => (80.0, 160.0, 210.0),  // Bright Blue
                13 => (200.0, 130.0, 200.0), // Bright Magenta
                14 => (120.0, 190.0, 190.0), // Bright Cyan
                15 => (220.0, 220.0, 220.0), // Bright White
                _ => (255.0, 176.0, 0.0),    // Default to amber
            }
        }
        Color::Reset => (180.0, 180.0, 180.0), // Default to light gray
        _ => (255.0, 176.0, 0.0),              // Default to amber for any other variant
    }
}

#[allow(dead_code)]
fn lerp(a: f32, b: f32, t: f32) -> u8 {
    (a + (b - a) * t).clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn album_sized_images_use_the_original_mosaic_block_size() {
        assert_eq!(pixelation_block_size(600, 600), 8);
    }

    #[test]
    fn pixelation_averages_block_colours_without_filters() {
        let mut img = RgbaImage::new(PIXELATION_BLOCK_SIZE, PIXELATION_BLOCK_SIZE);

        for y in 0..PIXELATION_BLOCK_SIZE {
            for x in 0..PIXELATION_BLOCK_SIZE {
                let color = if x < PIXELATION_BLOCK_SIZE / 2 {
                    Rgba([0, 0, 0, 255])
                } else {
                    Rgba([255, 255, 255, 255])
                };
                img.put_pixel(x, y, color);
            }
        }

        let pixelated = apply_pixelation(DynamicImage::ImageRgba8(img)).to_rgba8();

        for pixel in pixelated.pixels() {
            assert_eq!(*pixel, Rgba([128, 128, 128, 255]));
        }
    }

    #[test]
    fn pixelation_preserves_independent_partial_edge_blocks() {
        let mut img = RgbaImage::new(PIXELATION_BLOCK_SIZE + 1, 1);

        for x in 0..PIXELATION_BLOCK_SIZE {
            let color = if x < PIXELATION_BLOCK_SIZE / 2 {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            };
            img.put_pixel(x, 0, color);
        }
        img.put_pixel(PIXELATION_BLOCK_SIZE, 0, Rgba([255, 0, 0, 255]));

        let pixelated = apply_pixelation(DynamicImage::ImageRgba8(img)).to_rgba8();

        assert_eq!(*pixelated.get_pixel(0, 0), Rgba([128, 128, 128, 255]));
        assert_eq!(
            *pixelated.get_pixel(PIXELATION_BLOCK_SIZE, 0),
            Rgba([255, 0, 0, 255])
        );
    }

    #[test]
    fn pixelation_preserves_image_dimensions() {
        let img = RgbaImage::from_pixel(97, 53, Rgba([64, 128, 192, 255]));
        let pixelated = apply_pixelation(DynamicImage::ImageRgba8(img));

        assert_eq!(pixelated.width(), 97);
        assert_eq!(pixelated.height(), 53);
    }
}
