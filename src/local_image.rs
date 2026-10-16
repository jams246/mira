//! Bounded, read-only decoding of images inside the Markdown directory.

mod path;

#[cfg(test)]
mod resource_tests;
#[cfg(test)]
mod tests;

use std::io::{Cursor, Read};
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};

/// Maximum compressed image file size: 16 MiB.
pub const MAX_COMPRESSED_BYTES: usize = 16 * 1024 * 1024;
/// Maximum width or height of an image, in pixels.
pub const MAX_DIMENSION: u32 = 8192;
/// Maximum decoded pixel count: 8,388,608 pixels.
pub const MAX_PIXELS: u64 = 8 * 1024 * 1024;
/// Maximum final RGBA buffer size: 32 MiB.
pub const MAX_RGBA_BYTES: usize = 32 * 1024 * 1024;

/// One image frame in straight-alpha RGBA byte order.
#[derive(Debug)]
pub struct DecodedImage {
    /// Original image width, in pixels.
    pub width: u32,
    /// Original image height, in pixels.
    pub height: u32,
    /// Four bytes per pixel in red, green, blue, alpha order.
    pub rgba: Vec<u8>,
}

/// Decodes the first frame of a relative local PNG, JPEG, GIF, WebP, or BMP.
///
/// Call off the UI thread. The canonical image must stay inside the canonical
/// Markdown file's parent directory. Remote paths, escapes, oversized inputs,
/// and invalid images return a displayable error without modifying any file.
/// Decoder auxiliary allocations also receive image's best-effort 32 MiB limit;
/// source and final-buffer bounds are checked explicitly.
pub fn decode(document_path: &Path, target: &str) -> Result<DecodedImage, String> {
    let file = path::open(document_path, target)?;
    if file
        .metadata()
        .map_err(|error| format!("Cannot inspect image: {error}"))?
        .len()
        > MAX_COMPRESSED_BYTES as u64
    {
        return Err("Image file exceeds the 16 MiB limit.".into());
    }
    let encoded = read_bounded(file)?;
    let mut reader = ImageReader::new(Cursor::new(&encoded))
        .with_guessed_format()
        .map_err(|error| format!("Cannot identify image: {error}"))?;
    if !matches!(
        reader.format(),
        Some(
            ImageFormat::Png
                | ImageFormat::Jpeg
                | ImageFormat::Gif
                | ImageFormat::WebP
                | ImageFormat::Bmp
        )
    ) {
        return Err("Unsupported image format. Use PNG, JPEG, GIF, WebP, or BMP.".into());
    }
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_RGBA_BYTES as u64);
    reader.limits(limits);
    let decoder = reader
        .into_decoder()
        .map_err(|error| format!("Cannot decode image: {error}"))?;
    let (width, height) = decoder.dimensions();
    let rgba_bytes = validate_dimensions(width, height)?;
    if decoder.total_bytes() > MAX_RGBA_BYTES as u64 {
        return Err("Decoded image exceeds the 32 MiB limit.".into());
    }
    let rgba = DynamicImage::from_decoder(decoder)
        .map_err(|error| format!("Cannot decode image: {error}"))?
        .into_rgba8()
        .into_raw();
    if rgba.len() != rgba_bytes {
        return Err("Decoded image has an invalid pixel buffer.".into());
    }
    Ok(DecodedImage {
        width,
        height,
        rgba,
    })
}

fn validate_dimensions(width: u32, height: u32) -> Result<usize, String> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err("Image dimensions exceed the 8192-pixel limit or are empty.".into());
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_PIXELS {
        return Err("Image exceeds the 8,388,608-pixel limit.".into());
    }
    Ok((pixels * 4) as usize)
}

fn read_bounded(mut reader: impl Read) -> Result<Vec<u8>, String> {
    let mut encoded = Vec::new();
    reader
        .by_ref()
        .take(MAX_COMPRESSED_BYTES as u64)
        .read_to_end(&mut encoded)
        .map_err(|error| format!("Cannot read image: {error}"))?;
    if encoded.len() == MAX_COMPRESSED_BYTES
        && reader
            .read(&mut [0])
            .map_err(|error| format!("Cannot read image: {error}"))?
            != 0
    {
        return Err("Image file exceeds the 16 MiB limit.".into());
    }
    Ok(encoded)
}
