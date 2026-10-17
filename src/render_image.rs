use std::{rc::Rc, sync::Arc};

use gpui::{AnyElement, App, IntoElement, MouseButton, Window, div, img, prelude::*, px};
use mira::document::{Document, Inline};
use mira::inline_index::InlineIndex;

use crate::{
    flow_cache::span::MIN_SPAN_BYTES,
    image_cache::{ImageCache, ImageStatus},
    render_context::RenderContext,
    render_text::{self, TextKind},
    selection::Position,
    wrapped_text::WrappedText,
};

pub(crate) struct ImageContent<'a> {
    pub target: usize,
    pub alt: &'a [Inline],
    pub link: Option<usize>,
    pub occurrence: usize,
    pub span_source: Option<(Position, Arc<InlineIndex>)>,
}

pub(crate) fn image(
    document: &Document,
    position: Position,
    content: ImageContent<'_>,
    resources: &RenderContext,
    window: &mut Window,
    cx: &mut App,
    request: bool,
) -> AnyElement {
    let id = gpui::SharedString::from(format!(
        "image-{}-{}-{}",
        position.block, position.byte, content.occurrence
    ));
    let status = if request {
        ImageCache::request(&resources.images, content.target, window, cx)
    } else {
        resources.images.borrow().peek(content.target)
    };
    let image = match status {
        ImageStatus::Ready(image) => image,
        ImageStatus::Loading => {
            return placeholder(
                document,
                position,
                content,
                resources,
                "Loading image...".into(),
                id,
                false,
            );
        }
        ImageStatus::Failed(error) => {
            return placeholder(
                document,
                position,
                content,
                resources,
                format!("Cannot display image: {error}"),
                id,
                true,
            );
        }
    };
    let alt_len = content
        .alt
        .iter()
        .map(|run| run.text.as_str(&document.source).len())
        .sum();
    let selected = resources
        .selection
        .borrow()
        .range_at(position, alt_len)
        .is_some();
    let (width, height) = dimensions(image.width, image.height, resources.width);
    let down = Rc::clone(&resources.selection);
    let drag = Rc::clone(&resources.selection);
    let up = Rc::clone(&resources.selection);
    let links = Rc::clone(&resources.links);
    let owner = resources.owner;
    resources.paint.wrap(
        div()
            .id(id)
            .flex_none()
            .w(px(width))
            .h(px(height))
            .max_w_full()
            .when(content.link.is_some(), |image| image.cursor_pointer())
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                down.borrow_mut().start(position, false);
                cx.notify(owner);
            })
            .on_mouse_move(move |event, _, cx| {
                if event.pressed_button == Some(MouseButton::Left)
                    && drag.borrow_mut().extend(Position {
                        byte: position.byte + alt_len,
                        ..position
                    })
                {
                    cx.notify(owner);
                }
            })
            .on_click(move |_, window, cx| {
                if let Some(target) = content.link.filter(|_| up.borrow().is_empty()) {
                    links(target, window, cx);
                }
            })
            .child(
                img(image.image)
                    .w(px(width))
                    .h(px(height))
                    .when(selected, |image| image.opacity(0.65)),
            )
            .into_any_element(),
    )
}

fn placeholder(
    document: &Document,
    position: Position,
    content: ImageContent<'_>,
    resources: &RenderContext,
    message: String,
    id: gpui::SharedString,
    permanent: bool,
) -> AnyElement {
    let ImageContent {
        alt, span_source, ..
    } = content;
    let len = alt
        .iter()
        .map(|run| run.text.as_str(&document.source).len())
        .sum();
    let large = len >= MIN_SPAN_BYTES;
    let text = if large {
        match span_source {
            Some((mother, index)) => match alt_range(position, mother, len) {
                Some(range) => {
                    WrappedText::new_span(mother, range, TextKind::Body.into(), resources, index)
                        .into_any_element()
                }
                None => div()
                    .child("Cannot display image alternative text: invalid source range.")
                    .into_any_element(),
            },
            None => WrappedText::new(position, TextKind::Body.into(), resources).into_any_element(),
        }
    } else {
        render_text::text(document, position, alt, TextKind::Body.into(), resources)
    };
    let text = if !large && len > 0 {
        resources.paint.wrap(text)
    } else {
        text
    };
    let caption = div()
        .text_sm()
        .text_color(if permanent {
            resources.palette.error
        } else {
            resources.palette.muted
        })
        .child(message)
        .into_any_element();
    div()
        .id(id)
        .min_w_0()
        .max_w_full()
        .when(large, |placeholder| placeholder.flex_none())
        .child(text)
        .child(if permanent {
            resources.paint.wrap(caption)
        } else {
            caption
        })
        .into_any_element()
}

fn alt_range(position: Position, mother: Position, len: usize) -> Option<(usize, usize)> {
    if position.block != mother.block {
        return None;
    }
    let start = position.byte.checked_sub(mother.byte)?;
    Some((start, start.checked_add(len)?))
}

fn dimensions(width: u32, height: u32, available: f32) -> (f32, f32) {
    let width = width as f32;
    let scale = (available.max(1.) / width.max(1.)).min(1.);
    (width * scale, height as f32 * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_fit_viewport_without_upscaling_or_changing_aspect() {
        assert_eq!(dimensions(1800, 240, 900.), (900., 120.));
        assert_eq!(dimensions(40, 160, 900.), (40., 160.));
    }

    #[test]
    fn mixed_alt_range_excludes_table_cell_and_prefix_offsets_once() {
        let mother = Position { block: 3, byte: 40 };
        let image = Position { block: 3, byte: 47 };
        let alt = "東京😀";
        assert_eq!(alt_range(image, mother, alt.len()), Some((7, 17)));
    }

    #[test]
    fn incoherent_alt_source_coordinates_are_rejected() {
        let mother = Position { block: 3, byte: 40 };
        assert_eq!(alt_range(Position { block: 2, byte: 47 }, mother, 10), None);
        assert_eq!(alt_range(Position { block: 3, byte: 39 }, mother, 10), None);
        assert_eq!(
            alt_range(
                Position {
                    block: 3,
                    byte: usize::MAX
                },
                mother,
                41
            ),
            None
        );
    }
}
