use std::{path::Path, sync::Arc};

use gpui::RenderImage;
use image::{ImageBuffer, Rgba, imageops};

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;

#[derive(Clone)]
pub(crate) struct CachedImage {
    pub image: Arc<RenderImage>,
    pub width: u32,
    pub height: u32,
}

impl CachedImage {
    pub(crate) fn bytes(&self) -> usize {
        self.image.as_bytes(0).map_or(0, <[u8]>::len)
    }
}

pub(crate) fn decode(origin: &Path, target: &str, max_pixels: u64) -> Result<CachedImage, String> {
    from_decoded(mira::local_image::decode(origin, target)?, max_pixels)
}

fn from_decoded(
    mut decoded: mira::local_image::DecodedImage,
    max_pixels: u64,
) -> Result<CachedImage, String> {
    if max_pixels == 0 {
        return Err("Image preview budget must allow at least one pixel.".into());
    }
    for pixel in decoded.rgba.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let buffer = image::RgbaImage::from_raw(decoded.width, decoded.height, decoded.rgba)
        .ok_or("Invalid decoded image buffer.")?;
    let (width, height) = dimensions(buffer.width(), buffer.height(), max_pixels);
    let buffer = if buffer.dimensions() == (width, height) {
        buffer
    } else {
        imageops::resize(&buffer, width, height, imageops::FilterType::Triangle)
    };
    Ok(CachedImage {
        image: Arc::new(RenderImage::new([image::Frame::new(buffer)])),
        width: decoded.width,
        height: decoded.height,
    })
}

/// Shrinks a valid cached first frame to a positive pixel budget without upscaling.
pub(crate) fn fit(mut image: CachedImage, max_pixels: u64) -> CachedImage {
    let Some(bytes) = image.image.as_bytes(0) else {
        return image;
    };
    let size = image.image.size(0);
    let (old_width, old_height) = (size.width.0 as u32, size.height.0 as u32);
    let (width, height) = dimensions(old_width, old_height, max_pixels.max(1));
    if (width, height) == (old_width, old_height) {
        return image;
    }
    let Some(buffer) = ImageBuffer::<Rgba<u8>, &[u8]>::from_raw(old_width, old_height, bytes)
    else {
        return image;
    };
    let resized = imageops::resize(&buffer, width, height, imageops::FilterType::Triangle);
    image.image = Arc::new(RenderImage::new([image::Frame::new(resized)]));
    image
}

pub(crate) fn fit_all(
    images: Vec<(String, CachedImage)>,
    max_bytes: usize,
) -> Result<Vec<(String, CachedImage)>, String> {
    let budget = max_bytes / 4;
    if images.len() > budget {
        return Err("Image cache budget cannot hold one pixel per visible image.".into());
    }
    let pixels =
        |image: &CachedImage| image.image.as_bytes(0).map_or(0, |bytes| bytes.len() / 4) as u128;
    let total: u128 = images.iter().map(|(_, image)| pixels(image)).sum();
    if total <= budget as u128 {
        return Ok(images);
    }
    let remaining = (budget - images.len()) as u128;
    Ok(images
        .into_iter()
        .map(|(target, image)| {
            let allowance = 1 + remaining * pixels(&image) / total;
            (target, fit(image, allowance as u64))
        })
        .collect())
}

fn dimensions(width: u32, height: u32, budget: u64) -> (u32, u32) {
    let pixels = u64::from(width) * u64::from(height);
    if pixels <= budget {
        return (width, height);
    }
    let scale = (budget as f64 / pixels as f64).sqrt();
    let width = ((f64::from(width) * scale).floor() as u32)
        .max(1)
        .min(width)
        .min(budget.min(u64::from(width)) as u32);
    let height = ((f64::from(height) * scale).floor() as u32)
        .max(1)
        .min(height);
    (
        width,
        u64::from(height).min(budget / u64::from(width)) as u32,
    )
}
