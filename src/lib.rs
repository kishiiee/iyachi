#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::*;
use image::{DynamicImage, GenericImageView, RgbaImage, ImageOutputFormat};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use reqwest;
use tokio;

#[napi(object)]
pub struct User {
    pub name: String,
    pub avatar_url: String,
}

#[napi(object)]
pub struct ShipImageInput {
    pub user1: User,
    pub user2: User,
    pub percentage: u8,
    pub background_image: String,
}

#[napi]
pub async fn create_ship_image(input: ShipImageInput) -> Result<Buffer> {
    let bg_image = fetch_image(&input.background_image).await?;
    let user1_image = fetch_image(&input.user1.avatar_url).await?;
    let user2_image = fetch_image(&input.user2.avatar_url).await?;

    let composed = compose_image(
        bg_image,
        user1_image,
        user2_image,
        &input.user1.name,
        &input.user2.name,
        input.percentage,
    )?;

    let mut buffer = Vec::new();
    composed.write_to(&mut buffer, ImageOutputFormat::Png)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(Buffer::from(buffer))
}

async fn fetch_image(url: &str) -> Result<DynamicImage> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| Error::from_reason(format!("Failed to fetch {}: {}", url, e)))?;

    let bytes = resp.bytes()
        .await
        .map_err(|e| Error::from_reason(format!("Failed to read bytes: {}", e)))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| Error::from_reason(format!("Failed to decode image: {}", e)))?;

    Ok(img)
}

fn compose_image(
    bg: DynamicImage,
    user1: DynamicImage,
    user2: DynamicImage,
    name1: &str,
    name2: &str,
    percentage: u8,
) -> Result<RgbaImage> {
    let mut base = bg.resize_exact(600, 400, image::imageops::FilterType::Lanczos3).to_rgba8();

    let avatar_size = 128;
    let u1 = user1.resize_exact(avatar_size, avatar_size, image::imageops::FilterType::Lanczos3);
    let u2 = user2.resize_exact(avatar_size, avatar_size, image::imageops::FilterType::Lanczos3);

    image::imageops::overlay(&mut base, &u1, 80, 100);
    image::imageops::overlay(&mut base, &u2, 600 - avatar_size - 80, 100);

    // Draw names
    let font_data = include_bytes!("../assets/Roboto-Regular.ttf"); // You must include this font file
    let font = Font::try_from_bytes(font_data as &[u8])
        .ok_or_else(|| Error::from_reason("Failed to load font"))?;
    let scale = Scale::uniform(32.0);

    draw_text_mut(&mut base, image::Rgba([255, 255, 255, 255]), 80, 240, scale, &font, name1);
    draw_text_mut(&mut base, image::Rgba([255, 255, 255, 255]), (600 - avatar_size - 80) as i32, 240, scale, &font, name2);

    // Draw percentage text in the middle
    let percentage_text = format!("❤️ {}%", percentage);
    draw_text_mut(&mut base, image::Rgba([255, 0, 100, 255]), 250, 180, Scale::uniform(40.0), &font, &percentage_text);

    Ok(base)
}
