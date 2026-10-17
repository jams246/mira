use gpui::{AnyElement, App, IntoElement, Window};
use mira::document::{Document, Inline};
use mira::flow_index::FlowIndex;

use crate::{
    flow_cache::Input,
    flow_view::{self, FlowView},
    render_context::RenderContext,
    render_text::{self, TextFormat},
    selection::Position,
    text_cache::MIN_ASYNC_BYTES,
    wrapped_text::WrappedText,
};

pub(crate) fn content(
    document: &Document,
    position: Position,
    inlines: &[Inline],
    format: TextFormat,
    resources: &RenderContext,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    if !inlines.iter().any(|inline| inline.image.is_some()) {
        if format.kind != crate::render_text::TextKind::Code
            && inlines
                .iter()
                .map(|run| run.text.as_str(&document.source).len())
                .sum::<usize>()
                >= MIN_ASYNC_BYTES
        {
            return WrappedText::new(position, format, resources).into_any_element();
        }
        return text(document, position, inlines, format, resources);
    }
    if needs_flow(document, inlines) {
        return FlowView::new(position, format, resources).into_any_element();
    }
    let index = FlowIndex::new(document, inlines);
    flow_view::row::build(
        &index,
        0..index.pieces().len(),
        &Input {
            resources,
            position,
            format,
            width: resources.width,
        },
        false,
        None,
        window,
        cx,
    )
}

fn needs_flow(document: &Document, inlines: &[Inline]) -> bool {
    inlines.len() > 256
        || inlines
            .iter()
            .map(|run| run.text.as_str(&document.source).len())
            .sum::<usize>()
            >= MIN_ASYNC_BYTES
}

pub(crate) fn text(
    document: &Document,
    position: Position,
    runs: &[Inline],
    format: TextFormat,
    resources: &RenderContext,
) -> AnyElement {
    resources.paint.wrap(render_text::text(
        document, position, runs, format, resources,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mira::{document::BlockKind, markdown};

    #[test]
    fn empty_alt_occurrences_use_virtual_flow_even_without_semantic_bytes() {
        let document = markdown::parse("![](one.png)".repeat(10_000));
        let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
            panic!("paragraph")
        };
        assert_eq!(runs.len(), 10_000);
        assert!(
            runs.iter()
                .all(|run| run.text.as_str(&document.source).is_empty())
        );
        assert!(needs_flow(&document, runs));
    }

    #[test]
    fn combining_text_below_the_old_cutoff_uses_incremental_flow() {
        let document = markdown::parse(format!(
            "![left](left.png) a{} after",
            "\u{301}".repeat(15000)
        ));
        let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
            panic!("paragraph")
        };
        assert!(needs_flow(&document, runs));
    }
}
