use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub(crate) struct Timing {
    start: Instant,
    enabled: bool,
}

impl Timing {
    pub(crate) fn start() -> Self {
        let start = Instant::now();
        crate::diagnostics::initialize();
        Self {
            start,
            enabled: std::env::var_os("MIRA_TIMINGS").is_some(),
        }
    }

    pub(crate) fn mark(self, event: &'static str) {
        self.duration(event, self.start.elapsed());
    }

    pub(crate) fn scale(self, scale: f32) {
        if self.enabled {
            crate::diagnostics::scale(scale);
        }
    }

    pub(crate) fn duration(self, event: &'static str, duration: Duration) {
        if self.enabled {
            crate::diagnostics::duration(event, duration);
        }
    }
}
