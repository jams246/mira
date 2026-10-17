use super::*;
use std::{path::PathBuf, time::Duration};

fn cache() -> CacheHandle {
    ImageCache::new(
        Arc::new(LoadedDocument {
            path: PathBuf::new(),
            document: mira::markdown::parse(String::new()),
            read_time: Duration::ZERO,
            parse_time: Duration::ZERO,
            code: HashMap::new(),
            code_index_time: Duration::ZERO,
        }),
        EntityId::from(1),
    )
}

fn image(width: u32, height: u32) -> CachedImage {
    CachedImage {
        image: Arc::new(gpui::RenderImage::new([image::Frame::new(
            image::RgbaImage::new(width, height),
        )])),
        width,
        height,
    }
}

#[test]
fn decoded_cache_evicts_offscreen_least_recently_used_image_at_64_mib() {
    let handle = cache();
    let mut cache = handle.borrow_mut();
    for target in 0..64 {
        assert!(
            cache
                .prepare(target.to_string(), Ok(image(512, 512)))
                .0
                .is_none()
        );
    }
    cache.entries.get_mut("0").unwrap().used = 100;
    cache.demand.insert("0".into());
    let (plan, evicted) = cache.prepare("64".into(), Ok(image(512, 512)));
    assert!(plan.is_none());
    assert_eq!(evicted.len(), 1);
    assert!(!cache.entries.contains_key("1"));
    assert!(cache.entries.contains_key("0"));
    assert_eq!(cache.bytes, MAX_CACHE_BYTES);
}

#[test]
fn visible_working_set_over_budget_settles_without_eviction_or_redecode() {
    let handle = cache();
    let mut cache = handle.borrow_mut();
    for target in ["a", "b", "c"] {
        cache.demand.insert(target.into());
    }
    cache.prepare("a".into(), Ok(image(4096, 2048)));
    cache.prepare("b".into(), Ok(image(4096, 2048)));
    let (plan, evicted) = cache.prepare("c".into(), Ok(image(4096, 2048)));
    assert!(evicted.is_empty());
    let previews = preview::fit_all(plan.unwrap(), MAX_CACHE_BYTES).unwrap();
    cache.apply(previews);
    for _ in 0..12 {
        cache.begin_frame();
        for target in ["a", "b", "c"] {
            cache.demand.insert(target.into());
            let ImageStatus::Ready(image) = &cache.entries[target].status else {
                panic!("ready preview");
            };
            assert_eq!((image.width, image.height), (4096, 2048));
        }
        assert!(cache.bytes <= MAX_CACHE_BYTES);
    }
}

#[test]
fn repeated_destination_failures_reuse_one_entry() {
    let handle = cache();
    let mut cache = handle.borrow_mut();
    for _ in 0..1000 {
        cache.prepare("same.png".into(), Err("Cannot decode image.".into()));
    }
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.bytes, 0);
}
