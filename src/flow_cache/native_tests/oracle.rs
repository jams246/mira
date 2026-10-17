use gpui::{App, AvailableSpace, Bounds, Pixels, Window, px, size};

use super::Case;
use crate::{flow_cache::measure::MeasuredIds, flow_view::row};

fn geometry(
    case: &Case,
    pieces: std::ops::Range<usize>,
    measuring: bool,
    window: &mut Window,
    cx: &mut App,
) -> (Vec<(usize, Bounds<Pixels>)>, f32) {
    let ids = MeasuredIds::default();
    let mut native = row::build(
        &case.index,
        pieces,
        &case.input(),
        measuring,
        Some(&ids),
        window,
        cx,
    );
    let root = native.request_layout(window, cx);
    window.compute_layout(
        root,
        size(
            AvailableSpace::Definite(px(case.resources.width)),
            AvailableSpace::MinContent,
        ),
        cx,
    );
    let root_bounds = window.layout_bounds(root);
    let pieces = ids
        .borrow()
        .iter()
        .map(|(piece, id)| {
            let mut bounds = window.layout_bounds(*id);
            bounds.origin -= root_bounds.origin;
            (*piece, bounds)
        })
        .collect();
    (pieces, root_bounds.size.height.into())
}

pub(super) fn verify(case: &Case, window: &mut Window, cx: &mut App) {
    let count = case.index.pieces().len();
    let (expected, full_height) = geometry(case, 0..count, false, window, cx);
    assert_eq!(expected.len(), count);
    let (rows, height) = {
        let state = case.entry.borrow();
        assert!(state.pending.is_none());
        (state.rows.clone(), state.height)
    };
    assert!(
        (height - full_height).abs() < 0.05,
        "full native flow height"
    );
    let mut cursor = 0;
    for row in rows {
        assert_eq!(
            row.pieces.start, cursor,
            "contiguous committed flow ordinals"
        );
        for measuring in [false, true] {
            let (actual, _) = geometry(case, row.pieces.clone(), measuring, window, cx);
            for (piece, mut bounds) in actual {
                bounds.origin.y += px(row.top);
                assert_eq!(expected[piece].0, piece);
                let reference = expected[piece].1;
                for (actual, expected) in [
                    (bounds.origin.x, reference.origin.x),
                    (bounds.origin.y, reference.origin.y),
                    (bounds.size.width, reference.size.width),
                    (bounds.size.height, reference.size.height),
                ] {
                    assert!(
                        (f32::from(actual) - f32::from(expected)).abs() < 0.05,
                        "piece {piece}: geometry {bounds:?} differs from full native {reference:?}"
                    );
                }
            }
        }
        cursor = row.pieces.end;
    }
    assert_eq!(
        cursor, count,
        "every native flow piece is committed exactly once"
    );
}
