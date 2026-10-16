use std::time::Instant;

#[derive(Clone, Copy)]
pub(crate) struct Timing {
    start: Instant,
    enabled: bool,
}

impl Timing {
    pub(crate) fn start() -> Self {
        Self {
            start: Instant::now(),
            enabled: std::env::var_os("MIRA_TIMINGS").is_some(),
        }
    }

    pub(crate) fn mark(self, event: &str) {
        if self.enabled {
            eprintln!(
                "MIRA {event} {:.3}ms",
                self.start.elapsed().as_secs_f64() * 1000.
            );
        }
    }
}
