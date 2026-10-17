use gpui::{Bounds, ShapedLine, point, px, size};

use super::actions::mouse_index;

#[test]
fn clicking_placeholder_cannot_select_bytes_outside_empty_query() {
    let placeholder = ShapedLine::default().with_len("Find in document".len());
    let bounds = Bounds::new(point(px(10.), px(10.)), size(px(200.), px(24.)));
    let click = point(px(80.), px(20.));
    assert!(placeholder.closest_index_for_x(click.x - bounds.left()) > 0);
    assert_eq!(
        mouse_index(0, Some(&bounds), Some(&placeholder), px(0.), click),
        0
    );
}

#[test]
fn query_hit_testing_preserves_native_unicode_end_and_empty_layout_fallback() {
    let text = "é日";
    let line = ShapedLine::default().with_len(text.len());
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(200.), px(24.)));
    let click = point(px(150.), px(12.));
    assert_eq!(
        mouse_index(text.len(), Some(&bounds), Some(&line), px(0.), click),
        text.len()
    );
    assert_eq!(
        mouse_index(text.len(), Some(&bounds), None, px(0.), click),
        0
    );
}
