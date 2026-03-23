pub mod cache;
pub mod converter;

use anyhow::Result;
use image::{DynamicImage, GenericImageView, Rgba};
use ratatui::style::Color;
use std::path::PathBuf;

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
    // Simple pixelation effect - reduce resolution then scale back up
    let (width, height) = img.dimensions();
    let scale_factor = 8; // Adjust for more/less pixelation

    let small_width = (width / scale_factor).max(1);
    let small_height = (height / scale_factor).max(1);

    let small = image::imageops::resize(
        &img,
        small_width,
        small_height,
        image::imageops::FilterType::Nearest,
    );

    image::imageops::resize(&small, width, height, image::imageops::FilterType::Nearest).into()
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
