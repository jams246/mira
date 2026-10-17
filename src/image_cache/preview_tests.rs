use super::*;

fn cached(width: u32, height: u32) -> CachedImage {
    CachedImage {
        image: Arc::new(RenderImage::new([image::Frame::new(
            image::RgbaImage::new(width, height),
        )])),
        width,
        height,
    }
}

#[test]
fn three_32_mib_images_fit_64_mib_and_preserve_natural_dimensions() {
    let originals: Vec<_> = (0..3)
        .map(|index| (index.to_string(), cached(4096, 2048)))
        .collect();
    assert_eq!(
        originals
            .iter()
            .map(|(_, image)| image.image.as_bytes(0).unwrap().len())
            .sum::<usize>(),
        96 * 1024 * 1024
    );
    let fitted = fit_all(originals, 64 * 1024 * 1024).unwrap();
    assert!(
        fitted
            .iter()
            .map(|(_, image)| image.image.as_bytes(0).unwrap().len())
            .sum::<usize>()
            <= 64 * 1024 * 1024
    );
    assert!(
        fitted
            .iter()
            .all(|(_, image)| (image.width, image.height) == (4096, 2048))
    );
}

#[test]
fn decoded_rgba_becomes_native_bgra_before_preview_storage() {
    let image = from_decoded(
        mira::local_image::DecodedImage {
            width: 1,
            height: 1,
            rgba: vec![12, 34, 56, 78],
        },
        1,
    )
    .unwrap();
    assert_eq!(image.image.as_bytes(0), Some([56, 34, 12, 78].as_slice()));
}

#[test]
fn very_thin_images_respect_small_positive_pixel_budgets() {
    for (width, height) in [(8192, 1), (1, 8192), (8192, 2), (2, 8192), (8192, 8192)] {
        for budget in [1, 2, 3, 7, 64, 1024] {
            let (scaled_width, scaled_height) = dimensions(width, height, budget);
            assert!(scaled_width >= 1 && scaled_height >= 1);
            assert!(scaled_width <= width && scaled_height <= height);
            assert!(u64::from(scaled_width) * u64::from(scaled_height) <= budget);
        }
    }
}

#[test]
fn resizing_a_thin_frame_keeps_one_pixel_height_and_original_dimensions() {
    let fitted = fit(cached(8192, 1), 3);
    let size = fitted.image.size(0);
    assert_eq!((size.width.0, size.height.0, fitted.bytes()), (3, 1, 12));
    assert_eq!((fitted.width, fitted.height), (8192, 1));
}

#[test]
fn fitting_never_upscales_or_changes_original_metadata() {
    let mut image = cached(8, 4);
    image.width = 800;
    image.height = 400;
    let original = Arc::clone(&image.image);
    let fitted = fit(image, 128);
    assert!(Arc::ptr_eq(&fitted.image, &original));
    assert_eq!((fitted.width, fitted.height), (800, 400));
}

#[test]
fn proportional_fit_uses_one_pixel_minimum_without_budget_overrun() {
    let fitted = fit_all(
        vec![
            ("large".into(), cached(100, 100)),
            ("small".into(), cached(1, 1)),
        ],
        13,
    )
    .unwrap();
    assert_eq!(
        fitted
            .iter()
            .map(|(_, image)| image.image.as_bytes(0).unwrap().len())
            .sum::<usize>(),
        8
    );
}

#[test]
fn impossible_minimum_and_zero_decode_budgets_return_errors() {
    assert!(fit_all(vec![("image".into(), cached(1, 1))], 3).is_err());
    assert!(
        from_decoded(
            mira::local_image::DecodedImage {
                width: 1,
                height: 1,
                rgba: vec![0; 4]
            },
            0
        )
        .is_err()
    );
    assert!(fit_all(Vec::new(), 0).unwrap().is_empty());
}
