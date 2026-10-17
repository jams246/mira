use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

static SINK: OnceLock<fn(&'static str, Duration)> = OnceLock::new();

/// Routes optional frame diagnostics to the application's nonblocking writer.
pub fn set_measurement_sink(sink: fn(&'static str, Duration)) {
    let _ = SINK.set(sink);
}

pub(crate) fn measure<R>(label: &'static str, work: impl FnOnce() -> R) -> R {
    if let Some(sink) = SINK.get() {
        let start = Instant::now();
        let result = work();
        sink(label, start.elapsed());
        result
    } else {
        ::util::measure(label, work)
    }
}
