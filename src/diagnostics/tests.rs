use super::*;

struct HeldWriter {
    entered: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
    bytes: Vec<u8>,
}

impl Write for HeldWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.bytes.is_empty() {
            self.entered.send(()).unwrap();
            self.release.recv().unwrap();
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn stalled_output_does_not_block_producers_or_reorder_paint_and_frame_markers() {
    let (sender, receiver) = mpsc::channel();
    let (entered, waiting) = mpsc::channel();
    let (release, resume) = mpsc::channel();
    let writer = HeldWriter {
        entered,
        release: resume,
        bytes: Vec::new(),
    };
    let worker = std::thread::spawn(move || {
        let mut writer = writer;
        write_records(receiver, &mut writer);
        writer.bytes
    });
    sender
        .send(Record::Mira(
            "first_content_paint",
            Duration::from_millis(12),
        ))
        .unwrap();
    waiting.recv_timeout(Duration::from_secs(2)).unwrap();
    sender
        .send(Record::Native("frame duration", Duration::from_millis(3)))
        .unwrap();
    sender.send(Record::Scale(1.5)).unwrap();
    release.send(()).unwrap();
    drop(sender);
    assert_eq!(
        String::from_utf8(worker.join().unwrap()).unwrap(),
        "MIRA first_content_paint 12.000ms\nframe duration: 3ms\nMIRA window_scale_factor 1.5\n"
    );
}

#[test]
fn flush_acknowledges_all_preceding_records() {
    let (sender, receiver) = mpsc::channel();
    let (completed, acknowledged) = mpsc::sync_channel(0);
    sender.send(Record::Scale(2.)).unwrap();
    sender.send(Record::Flush(completed)).unwrap();
    drop(sender);
    let writer = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        write_records(receiver, &mut bytes);
        bytes
    });
    acknowledged.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(writer.join().unwrap(), b"MIRA window_scale_factor 2\n");
}
