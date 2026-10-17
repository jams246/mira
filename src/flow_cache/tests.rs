use super::*;
use crate::render_text::TextKind;

#[test]
fn zero_byte_images_estimate_height_by_occurrence_and_advance_by_piece() {
    let document = mira::markdown::parse("![](one.png)".repeat(10_000));
    let source = Source::whole(0, None);
    let index = Arc::new(FlowIndex::new(&document, source.runs(&document)));
    let mut entry = Entry {
        config: Config {
            width: 936.,
            font_size: 16.,
            line_height: 24.,
            format: TextKind::Body.into(),
            images: 0,
        },
        index: None,
        rows: Vec::new(),
        cursor: 0,
        height: 0.,
        len: 0,
        images: 10_000,
        complete: false,
        scheduled: false,
        pending: None,
    };
    assert!(entry.estimated_height() > 700.);
    entry.index = Some(index);
    entry.cursor = 100;
    entry.height = 300.;
    assert!(entry.estimated_height() >= 30_000.);
    entry.complete = true;
    assert_eq!(entry.estimated_height(), 300.);
}
