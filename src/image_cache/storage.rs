use super::*;
use gpui::RenderImage;

type PreviewPlan = Vec<(String, CachedImage)>;

impl ImageCache {
    pub(super) fn prepare(
        &mut self,
        target: String,
        result: Result<CachedImage, String>,
    ) -> (Option<PreviewPlan>, Vec<Arc<RenderImage>>) {
        self.revision = self.revision.wrapping_add(1);
        let image = match result {
            Ok(image) => image,
            Err(error) => {
                self.fail(target, error);
                return (None, Vec::new());
            }
        };
        let incoming = image.bytes();
        let mut evicted = Vec::new();
        while self.bytes + incoming > MAX_CACHE_BYTES {
            let oldest = self
                .entries
                .iter()
                .filter(|(key, entry)| {
                    !self.demand.contains(*key) && matches!(entry.status, ImageStatus::Ready(_))
                })
                .min_by_key(|(_, entry)| entry.used)
                .map(|(key, _)| key.clone());
            let Some(oldest) = oldest else { break };
            if let Some(Entry {
                status: ImageStatus::Ready(image),
                ..
            }) = self.entries.remove(&oldest)
            {
                self.bytes -= image.bytes();
                evicted.push(image.image);
            }
        }
        if self.bytes + incoming <= MAX_CACHE_BYTES {
            self.bytes += incoming;
            self.clock = self.clock.wrapping_add(1);
            self.entries.insert(
                target,
                Entry {
                    status: ImageStatus::Ready(image),
                    used: self.clock,
                },
            );
            return (None, evicted);
        }
        let mut plan = Vec::new();
        for (key, entry) in &self.entries {
            if let ImageStatus::Ready(image) = &entry.status {
                plan.push((key.clone(), image.clone()));
            }
        }
        plan.push((target, image));
        (Some(plan), evicted)
    }

    pub(super) fn fail(&mut self, target: String, error: String) {
        self.revision = self.revision.wrapping_add(1);
        self.clock = self.clock.wrapping_add(1);
        self.entries.insert(
            target,
            Entry {
                status: ImageStatus::Failed(error),
                used: self.clock,
            },
        );
    }

    pub(super) fn apply(&mut self, images: PreviewPlan) -> Vec<Arc<RenderImage>> {
        self.revision = self.revision.wrapping_add(1);
        let mut evicted = Vec::new();
        for (target, image) in images {
            self.bytes += image.bytes();
            self.clock = self.clock.wrapping_add(1);
            if let Some(Entry {
                status: ImageStatus::Ready(old),
                ..
            }) = self.entries.insert(
                target,
                Entry {
                    status: ImageStatus::Ready(image),
                    used: self.clock,
                },
            ) {
                self.bytes -= old.bytes();
                evicted.push(old.image);
            }
        }
        evicted
    }
}
