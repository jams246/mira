use std::fs;
use std::io::{self, Read};

use super::tests::Fixture;
use super::{
    MAX_COMPRESSED_BYTES, MAX_DIMENSION, MAX_PIXELS, MAX_RGBA_BYTES, decode, read_bounded,
    validate_dimensions,
};

fn bmp_header(width: u32, height: u32) -> Vec<u8> {
    let mut header = vec![0; 54];
    header[..2].copy_from_slice(b"BM");
    header[2..6].copy_from_slice(&54u32.to_le_bytes());
    header[10..14].copy_from_slice(&54u32.to_le_bytes());
    header[14..18].copy_from_slice(&40u32.to_le_bytes());
    header[18..22].copy_from_slice(&width.to_le_bytes());
    header[22..26].copy_from_slice(&height.to_le_bytes());
    header[26..28].copy_from_slice(&1u16.to_le_bytes());
    header[28..30].copy_from_slice(&24u16.to_le_bytes());
    header
}

#[test]
fn decode_rejects_oversized_dimensions_from_header() {
    let fixture = Fixture::new("dimension-limit");
    fs::write(
        fixture.document.parent().unwrap().join("large.bmp"),
        bmp_header(MAX_DIMENSION + 1, 1),
    )
    .unwrap();
    let error = decode(&fixture.document, "large.bmp").unwrap_err();
    assert!(error.to_lowercase().contains("limit"), "{error}");
}

#[test]
fn decode_rejects_excess_pixel_count_before_reading_pixels() {
    let fixture = Fixture::new("pixel-limit");
    fs::write(
        fixture.document.parent().unwrap().join("large.bmp"),
        bmp_header(4096, 2049),
    )
    .unwrap();
    assert!(
        decode(&fixture.document, "large.bmp")
            .unwrap_err()
            .contains("8,388,608-pixel limit")
    );
}

#[test]
fn decode_rejects_oversized_compressed_file() {
    let fixture = Fixture::new("compressed-limit");
    fs::File::create(fixture.document.parent().unwrap().join("large.png"))
        .unwrap()
        .set_len(MAX_COMPRESSED_BYTES as u64 + 1)
        .unwrap();
    assert!(
        decode(&fixture.document, "large.png")
            .unwrap_err()
            .contains("16 MiB limit")
    );
}

#[test]
fn bounded_read_stops_after_limit_and_one_detection_byte() {
    struct Endless {
        read: usize,
    }
    impl Read for Endless {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            buffer.fill(0);
            self.read += buffer.len();
            Ok(buffer.len())
        }
    }
    let mut reader = Endless { read: 0 };
    let error = read_bounded(&mut reader).unwrap_err();
    assert_eq!(
        (reader.read, error.as_str()),
        (
            MAX_COMPRESSED_BYTES + 1,
            "Image file exceeds the 16 MiB limit."
        )
    );
}

#[test]
fn maximum_pixel_boundary_matches_rgba_bound_without_overflow() {
    assert_eq!(
        (
            validate_dimensions(MAX_DIMENSION, 1024).unwrap(),
            MAX_PIXELS * 4
        ),
        (MAX_RGBA_BYTES, MAX_RGBA_BYTES as u64)
    );
}

#[test]
fn dimension_checks_reject_zero_and_integer_extremes() {
    assert!(
        [(0, 1), (1, 0), (u32::MAX, u32::MAX)]
            .into_iter()
            .all(|(width, height)| validate_dimensions(width, height).is_err())
    );
}
