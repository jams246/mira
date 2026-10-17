mod budget;
mod preview;
mod storage;

#[cfg(test)]
mod tests;

use crate::{file::LoadedDocument, timing::Timing};
use gpui::{App, EntityId, Window};
use preview::CachedImage;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

pub(crate) type CacheHandle = Rc<RefCell<ImageCache>>;
pub(crate) const MAX_CACHE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone)]
pub(crate) enum ImageStatus {
    Loading,
    Ready(CachedImage),
    Failed(String),
}

struct Entry {
    status: ImageStatus,
    used: u64,
}

pub(crate) struct ImageCache {
    loaded: Arc<LoadedDocument>,
    owner: EntityId,
    entries: HashMap<String, Entry>,
    demand: HashSet<String>,
    bytes: usize,
    clock: u64,
    closed: bool,
}

impl ImageCache {
    pub(crate) fn new(loaded: Arc<LoadedDocument>, owner: EntityId) -> CacheHandle {
        Rc::new(RefCell::new(Self {
            loaded,
            owner,
            entries: HashMap::new(),
            demand: HashSet::new(),
            bytes: 0,
            clock: 0,
            closed: false,
        }))
    }

    pub(crate) fn begin_frame(&mut self) {
        self.demand.clear();
    }

    pub(crate) fn request(
        handle: &CacheHandle,
        target: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> ImageStatus {
        let mut cache = handle.borrow_mut();
        let Some(raw_target) = cache.loaded.document.targets.get(target).cloned() else {
            return ImageStatus::Failed("Invalid local image destination.".into());
        };
        let destination = match mira::links::decode(&raw_target) {
            Ok(destination) => destination,
            Err(error) => return ImageStatus::Failed(error),
        };
        cache.clock = cache.clock.wrapping_add(1);
        let used = cache.clock;
        cache.demand.insert(destination.clone());
        if let Some(entry) = cache.entries.get_mut(&destination) {
            entry.used = used;
            return entry.status.clone();
        }
        if cache.closed {
            return ImageStatus::Loading;
        }
        let Some(permit) = budget::Permit::acquire() else {
            return ImageStatus::Loading;
        };
        let origin = cache.loaded.path.clone();
        let owner = cache.owner;
        cache.entries.insert(
            destination.clone(),
            Entry {
                status: ImageStatus::Loading,
                used,
            },
        );
        drop(cache);
        let weak = Rc::downgrade(handle);
        let physical = window
            .viewport_size()
            .map(|value| f32::from(value) * window.scale_factor());
        let pixels = (physical.width * physical.height).max(1.) as u64;
        let executor = cx.background_executor().clone();
        let decode = executor.spawn(async move {
            let timing = Timing::start();
            let result = preview::decode(&origin, &raw_target, pixels);
            timing.mark("image_prepare");
            result
        });
        cx.spawn(async move |cx| {
            let result = decode.await;
            let plan = cx.update(|cx| {
                let handle = weak.upgrade()?;
                let mut cache = handle.borrow_mut();
                if cache.closed {
                    return None;
                }
                let (plan, evicted) = cache.prepare(destination.clone(), result);
                for image in evicted {
                    cx.drop_image(image, None);
                }
                plan
            });
            if let Ok(Some(images)) = plan {
                let resized = executor
                    .spawn(async move {
                        let timing = Timing::start();
                        let result = preview::fit_all(images, MAX_CACHE_BYTES);
                        timing.mark("image_rebalance");
                        result
                    })
                    .await;
                let _ = cx.update(|cx| {
                    if let Some(handle) = weak.upgrade() {
                        let mut cache = handle.borrow_mut();
                        if !cache.closed {
                            match resized {
                                Ok(images) => {
                                    for image in cache.apply(images) {
                                        cx.drop_image(image, None);
                                    }
                                }
                                Err(error) => cache.fail(destination, error),
                            }
                        }
                    }
                });
            }
            let _ = cx.update(|cx| {
                // A stale document may release the slot needed by the new viewport.
                drop(permit);
                cx.notify(owner);
            });
        })
        .detach();
        ImageStatus::Loading
    }

    pub(crate) fn clear(&mut self, window: &mut Window, cx: &mut App) {
        self.closed = true;
        for (_, entry) in self.entries.drain() {
            if let ImageStatus::Ready(image) = entry.status {
                cx.drop_image(image.image, Some(window));
            }
        }
        self.demand.clear();
        self.bytes = 0;
    }
}
