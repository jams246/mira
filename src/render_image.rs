use std::rc::Rc;

use gpui::{AnyElement, App, IntoElement, MouseButton, Window, div, img, prelude::*, px, rgb};
use mira::document::{Document, Inline};

use crate::{
    image_cache::{ImageCache, ImageStatus},
    render_context::RenderContext,
    render_text::{self, TextKind},
    selection::Position,
};

pub(crate) struct ImageContent<'a> {
    pub target: usize,
    pub alt: &'a [Inline],
    pub link: Option<usize>,
    pub occurrence: usize,
}

pub(crate) fn image(
    document: &Document,
    position: Position,
    content: ImageContent<'_>,
    resources: &RenderContext,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = gpui::SharedString::from(format!(
        "image-{}-{}-{}",
        position.block, position.byte, content.occurrence
    ));
    let image = match ImageCache::request(&resources.images, content.target, window, cx) {
        ImageStatus::Ready(image) => image,
        ImageStatus::Loading => {
            return placeholder(
                document,
                position,
                content.alt,
                resources,
                "Loading image...".into(),
                id,
            );
        }
        ImageStatus::Failed(error) => {
            return resources.paint.wrap(placeholder(
                document,
                position,
                content.alt,
                resources,
                format!("Cannot display image: {error}"),
                id,
            ));
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
    alt: &[Inline],
    resources: &RenderContext,
    message: String,
    id: gpui::SharedString,
) -> AnyElement {
    div()
        .id(id)
        .min_w_0()
        .max_w_full()
        .child(render_text::text(
            document,
            position,
            alt,
            TextKind::Body.into(),
            Rc::clone(&resources.links),
            Rc::clone(&resources.selection),
            resources.owner,
        ))
        .child(div().text_sm().text_color(rgb(0x666666)).child(message))
        .into_any_element()
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
}
