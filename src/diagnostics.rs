use std::{
    io::Write,
    sync::{OnceLock, mpsc},
    time::Duration,
};

enum Record {
    Mira(&'static str, Duration),
    Native(&'static str, Duration),
    Scale(f32),
    Flush(mpsc::SyncSender<()>),
}

static WRITER: OnceLock<Option<mpsc::Sender<Record>>> = OnceLock::new();

pub(crate) fn initialize() {
    WRITER.get_or_init(|| {
        let native =
            std::env::var("ZED_MEASUREMENTS").is_ok_and(|value| value == "1" || value == "true");
        if !native && std::env::var_os("MIRA_TIMINGS").is_none() {
            return None;
        }
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("mira-diagnostics".into())
            .spawn(move || write_records(receiver, std::io::stderr()))
            .ok()?;
        if native {
            gpui::set_measurement_sink(|event, duration| send(Record::Native(event, duration)));
        }
        Some(sender)
    });
}

pub(crate) fn duration(event: &'static str, duration: Duration) {
    send(Record::Mira(event, duration));
}

pub(crate) fn scale(scale: f32) {
    send(Record::Scale(scale));
}

pub(crate) fn flush() {
    if let Some(Some(writer)) = WRITER.get() {
        let (sent, received) = mpsc::sync_channel(0);
        if writer.send(Record::Flush(sent)).is_ok() {
            let _ = received.recv_timeout(Duration::from_secs(2));
        }
    }
}

fn send(record: Record) {
    if let Some(Some(writer)) = WRITER.get() {
        let _ = writer.send(record);
    }
}

fn write_records(records: mpsc::Receiver<Record>, mut output: impl Write) {
    for record in records {
        match record {
            Record::Mira(event, duration) => {
                let _ = writeln!(
                    output,
                    "MIRA {event} {:.3}ms",
                    duration.as_secs_f64() * 1000.
                );
            }
            Record::Native(event, duration) => {
                let _ = writeln!(output, "{event}: {duration:?}");
            }
            Record::Scale(scale) => {
                let _ = writeln!(output, "MIRA window_scale_factor {scale}");
            }
            Record::Flush(completed) => {
                let _ = output.flush();
                let _ = completed.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests;
