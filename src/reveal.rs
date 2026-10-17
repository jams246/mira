mod coarse;
mod horizontal;
#[cfg(test)]
mod tests;

pub(crate) use coarse::{coarse, horizontal_coarse};
pub(crate) use horizontal::horizontal;

use std::{cell::RefCell, ops::Range, rc::Rc};

use gpui::{
    App, Bounds, EntityId, ListOffset, ListState, Pixels, Point, ShapedLine, TextLayout, point, px,
};
use mira::reading_state::Location;

use crate::{
    render_context::RenderContext, selection::Position, text_map::TextMap, timing::Timing,
};

pub(crate) type RevealHandle = Rc<RefCell<Reveal>>;

pub(crate) struct Reveal {
    pub target: Option<Location>,
    pub reading: Location,
    pub viewport: Bounds<Pixels>,
    pub list: ListState,
    pub owner: EntityId,
    coarse_request: Option<(Location, Pixels)>,
    coarse_scheduled: bool,
    sample: Option<(Pixels, Pixels)>,
    resolved: bool,
    precise: bool,
    timing: Timing,
}

impl Reveal {
    pub fn new(list: ListState, owner: EntityId) -> RevealHandle {
        Rc::new(RefCell::new(Self {
            target: None,
            reading: Location::default(),
            viewport: Bounds::default(),
            list,
            owner,
            coarse_request: None,
            coarse_scheduled: false,
            sample: None,
            resolved: false,
            precise: false,
            timing: Timing::start(),
        }))
    }

    pub fn seek(&mut self, target: Location, cx: &mut App) {
        self.timing = Timing::start();
        self.target = Some(target);
        self.reading = target;
        self.list.scroll_to(ListOffset {
            item_ix: target.block,
            offset_in_item: px(0.),
        });
        cx.notify(self.owner);
    }

    pub fn begin_frame(&mut self, bounds: Bounds<Pixels>) {
        self.viewport = bounds;
        self.sample = None;
        self.resolved = false;
        self.precise = false;
    }

    fn finish(&mut self, target: Location, position: Point<Pixels>, cx: &mut App) {
        let Some(block) = self.list.bounds_for_item(target.block) else {
            return;
        };
        let offset = f32::from(position.y - block.top()) - target.offset;
        self.list.scroll_to(ListOffset {
            item_ix: target.block,
            offset_in_item: px(offset.max(0.)),
        });
        self.target = None;
        self.reading = target;
        self.resolved = true;
        self.timing.mark("semantic_reveal_ready");
        cx.notify(self.owner);
    }

    fn samples(&self, bounds: Bounds<Pixels>) -> bool {
        self.target.is_none()
            && !self.resolved
            && bounds.bottom() > self.viewport.top()
            && bounds.top() < self.viewport.bottom()
            && bounds.right() >= self.viewport.left()
            && bounds.left() < self.viewport.right()
    }

    fn record(&mut self, location: Location, point: Point<Pixels>) {
        let score = (
            point.y.max(self.viewport.top()),
            point.x.max(self.viewport.left()),
        );
        if self.sample.is_none_or(|old| score < old) {
            self.sample = Some(score);
            self.reading = location;
        }
    }
}

fn inside(target: Location, position: Position, cell: Option<usize>, len: usize) -> bool {
    target.block == position.block
        && target.cell == cell
        && target.byte >= position.byte
        && target.byte <= position.byte.saturating_add(len)
}

pub(crate) fn observe(
    resources: &RenderContext,
    position: Position,
    len: usize,
    mapping: &TextMap,
    layout: &TextLayout,
    text: &str,
    cx: &mut App,
) {
    let cell = resources.cell;
    let mut state = resources.reveal.borrow_mut();
    let bounds = layout.bounds();
    if bounds.size.height <= px(0.) {
        return;
    }
    if let Some(target) = state.target
        && inside(target, position, cell, len)
        && let Some(point) =
            layout.position_for_index(mapping.to_display(target.byte - position.byte))
    {
        state.precise = true;
        if horizontal(resources, position.block, point.x, cx) {
            return;
        }
        state.finish(target, point, cx);
        return;
    }
    if state.samples(bounds) {
        let sample = point(bounds.left(), state.viewport.top().max(bounds.top()));
        let byte = layout
            .index_for_position(sample)
            .unwrap_or_else(|nearest| nearest)
            .min(text.len());
        if let Some(point) = layout.position_for_index(byte) {
            let location = Location {
                block: position.block,
                cell,
                byte: position.byte + mapping.to_source(byte),
                offset: f32::from(point.y - state.viewport.top()),
            };
            state.record(location, point);
        }
    }
}

pub(crate) fn observe_code(
    resources: &RenderContext,
    position: Position,
    core: Range<usize>,
    mapping: &TextMap,
    line: &ShapedLine,
    geometry: (Point<Pixels>, Pixels),
    cx: &mut App,
) {
    let (origin, line_height) = geometry;
    let mut state = resources.reveal.borrow_mut();
    if let Some(target) = state.target
        && code_inside(
            target,
            position,
            &core,
            position.byte + mapping.to_source(line.text.len()),
        )
    {
        let point = point(
            origin.x + line.x_for_index(mapping.to_display(target.byte - position.byte)),
            origin.y,
        );
        state.precise = true;
        if horizontal(resources, position.block, point.x, cx) {
            return;
        }
        state.finish(target, point, cx);
        return;
    }
    let bounds = Bounds::new(origin, gpui::size(line.width, line_height));
    if state.samples(bounds) {
        let byte = line.closest_index_for_x((state.viewport.left() - origin.x).max(px(0.)));
        let location = Location {
            block: position.block,
            cell: None,
            byte: position.byte + mapping.to_source(byte),
            offset: f32::from(origin.y - state.viewport.top()),
        };
        let sample = point(origin.x + line.x_for_index(byte), origin.y);
        state.record(location, sample);
    }
}

fn code_inside(
    target: Location,
    position: Position,
    core: &Range<usize>,
    window_end: usize,
) -> bool {
    target.block == position.block
        && target.cell.is_none()
        && (core.contains(&target.byte) || target.byte == core.end && core.end == window_end)
}
