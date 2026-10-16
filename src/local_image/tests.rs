use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use image::{ImageFormat, Rgba, RgbaImage};

use super::decode;

pub(super) struct Fixture {
    pub root: PathBuf,
    pub document: PathBuf,
}

impl Fixture {
    pub(super) fn new(name: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".verification/mira-plan/local-image-tests")
            .join(format!(
                "{}-{}-{name}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(root.join("book")).expect("Create synthetic fixture directory");
        let document = root.join("book/document.md");
        fs::write(&document, "![synthetic](image.png)").expect("Write synthetic Markdown");
        Self {
            root,
            document: fs::canonicalize(document).expect("Canonical fixture document"),
        }
    }

    pub(super) fn image(&self, name: &str, width: u32, height: u32) -> PathBuf {
        let path = self.document.parent().expect("Fixture parent").join(name);
        RgbaImage::from_pixel(width, height, Rgba([12, 34, 56, 78]))
            .save_with_format(&path, ImageFormat::Png)
            .expect("Write synthetic PNG");
        path
    }
}

#[test]
fn decode_preserves_aspect_rgba_and_source_bytes() {
    let fixture = Fixture::new("aspect");
    let path = fixture.image("image.png", 32, 18);
    let before = fs::read(&path).unwrap();
    let decoded = decode(&fixture.document, "image.png").unwrap();
    assert_eq!(
        (
            decoded.width,
            decoded.height,
            decoded.rgba.len(),
            &decoded.rgba[..4],
            fs::read(path).unwrap()
        ),
        (32, 18, 32 * 18 * 4, &[12, 34, 56, 78][..], before)
    );
}

#[test]
fn decode_accepts_percent_encoded_unicode_and_spaces() {
    let fixture = Fixture::new("unicode");
    fixture.image("画像 image.png", 3, 7);
    let decoded = decode(&fixture.document, "%E7%94%BB%E5%83%8F%20image.png").unwrap();
    assert_eq!((decoded.width, decoded.height), (3, 7));
}

#[test]
fn decode_reports_missing_and_corrupt_files() {
    let fixture = Fixture::new("errors");
    let missing = decode(&fixture.document, "missing.png").unwrap_err();
    fs::write(
        fixture.document.parent().unwrap().join("corrupt.png"),
        b"not an image",
    )
    .unwrap();
    let corrupt = decode(&fixture.document, "corrupt.png").unwrap_err();
    assert!(missing.contains("Cannot find image") && corrupt.contains("Unsupported image format"));
}

#[test]
fn decode_supports_all_five_declared_formats() {
    let fixture = Fixture::new("formats");
    for (name, format) in [
        ("png.png", ImageFormat::Png),
        ("jpeg.jpg", ImageFormat::Jpeg),
        ("gif.gif", ImageFormat::Gif),
        ("webp.webp", ImageFormat::WebP),
        ("bmp.bmp", ImageFormat::Bmp),
    ] {
        let path = fixture.document.parent().unwrap().join(name);
        image::RgbImage::from_pixel(2, 3, image::Rgb([12, 34, 56]))
            .save_with_format(path, format)
            .unwrap();
        let decoded = decode(&fixture.document, name).unwrap();
        assert_eq!(
            (decoded.width, decoded.height, decoded.rgba.len()),
            (2, 3, 24),
            "{name}"
        );
    }
}

#[test]
fn decode_reads_only_first_gif_frame() {
    let fixture = Fixture::new("first-frame");
    let path = fixture.document.parent().unwrap().join("animated.gif");
    let mut encoder = image::codecs::gif::GifEncoder::new(fs::File::create(path).unwrap());
    encoder
        .encode_frames([
            image::Frame::new(RgbaImage::from_pixel(2, 1, Rgba([255, 0, 0, 255]))),
            image::Frame::new(RgbaImage::from_pixel(2, 1, Rgba([0, 0, 255, 255]))),
        ])
        .unwrap();
    drop(encoder);
    assert_eq!(
        decode(&fixture.document, "animated.gif").unwrap().rgba,
        [255, 0, 0, 255, 255, 0, 0, 255]
    );
}

#[test]
fn decode_rejects_remote_absolute_unc_and_control_targets() {
    let fixture = Fixture::new("untrusted");
    for target in [
        "https://example.com/image.png",
        "file:///C:/image.png",
        "C:\\image.png",
        "\\\\server\\share\\image.png",
        "/image.png",
        "%2Fimage.png",
        "image.png:stream",
        "image%00.png",
        "bad\n.png",
        "%FF.png",
        "",
    ] {
        assert!(
            decode(&fixture.document, target).is_err(),
            "Unexpectedly accepted {target:?}"
        );
    }
}

#[test]
fn decode_rejects_parent_escape_before_reading() {
    let fixture = Fixture::new("parent-escape");
    RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255]))
        .save(fixture.root.join("outside.png"))
        .unwrap();
    assert!(
        decode(&fixture.document, "../outside.png")
            .unwrap_err()
            .contains("escapes")
    );
}

#[cfg(windows)]
#[test]
fn decode_rejects_symlink_escape_when_symlinks_are_available() {
    let fixture = Fixture::new("symlink");
    let outside = fixture.root.join("outside.png");
    RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255]))
        .save(&outside)
        .unwrap();
    let link = fixture.document.parent().unwrap().join("escape.png");
    if let Err(error) = std::os::windows::fs::symlink_file(outside, link) {
        eprintln!("Symlink escape test unavailable: {error}");
        return;
    }
    assert!(
        decode(&fixture.document, "escape.png")
            .unwrap_err()
            .contains("escapes")
    );
}
