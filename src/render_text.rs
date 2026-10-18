use std::{ops::Range, rc::Rc};

use gpui::{
    AnyElement, App, FontStyle, FontWeight, SharedString, StrikethroughStyle, TextAlign, TextRun,
    UnderlineStyle, Window, font, px,
};
use mira::document::{Document, Inline};

use crate::{
    appearance::Palette, render_context::RenderContext, selection::Position, selection_text,
    text_background, text_map::TextMap,
};

pub(crate) type LinkHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextKind {
    Body,
    Heading,
    Code,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TextFormat {
    pub kind: TextKind,
    pub align: TextAlign,
    pub inline: bool,
}

impl From<TextKind> for TextFormat {
    fn from(kind: TextKind) -> Self {
        Self {
            kind,
            align: TextAlign::Left,
            inline: false,
        }
    }
}

pub(crate) fn text(
    document: &Document,
    position: Position,
    inlines: &[Inline],
    format: TextFormat,
    resources: &RenderContext,
) -> AnyElement {
    let PreparedText {
        text,
        runs,
        links,
        mapping,
        source_len,
    } = prepare(document, inlines, format.kind, resources.palette);
    let runs = crate::find_highlight::highlight(
        runs,
        position,
        resources.cell,
        source_len,
        &mapping,
        &resources.highlights,
        resources.palette,
    );
    let selected = resources
        .selection
        .borrow()
        .range_at(position, source_len)
        .map(|range| mapping.to_display(range.start)..mapping.to_display(range.end));
    let runs = selection_text::highlight(runs, selected, resources.palette);
    let text: SharedString = text.into();
    let (styled, layout) = text_background::text(text.clone(), runs, format.align);
    selection_text::wrap(
        text,
        layout,
        styled,
        selection_text::Interaction {
            position,
            selection: Rc::clone(&resources.selection),
            owner: resources.owner,
            mapping,
            align: format.align,
            inline: format.inline,
            links,
            activate_link: Rc::clone(&resources.links),
            resources: resources.clone(),
            source_len,
        },
    )
}

pub(crate) struct PreparedText {
    pub text: String,
    pub runs: Vec<TextRun>,
    links: Vec<(Range<usize>, usize)>,
    mapping: TextMap,
    source_len: usize,
}

pub(crate) fn prepare(
    document: &Document,
    inlines: &[Inline],
    kind: TextKind,
    palette: Palette,
) -> PreparedText {
    let mut text = String::new();
    let mut runs = Vec::with_capacity(inlines.len());
    let mut links = Vec::new();
    for inline in inlines {
        let start = text.len();
        text.push_str(inline.text.as_str(&document.source));
        let mut font = font(if inline.style.code || kind == TextKind::Code {
            "Consolas"
        } else {
            "Segoe UI"
        });
        if kind == TextKind::Heading || inline.style.bold {
            font.weight = FontWeight::BOLD;
        }
        if inline.style.italic {
            font.style = FontStyle::Italic;
        }
        runs.push(TextRun {
            len: text.len() - start,
            font,
            color: if inline.link.is_some() {
                palette.link
            } else {
                palette.text
            },
            background_color: inline.style.code.then_some(palette.panel),
            underline: inline.link.map(|_| UnderlineStyle {
                thickness: px(1.),
                color: None,
                wavy: false,
            }),
            strikethrough: inline.style.strike.then_some(StrikethroughStyle {
                thickness: px(1.),
                color: None,
            }),
        });
        if let Some(target) = inline.link {
            links.push((start..text.len(), target));
        }
    }
    let source_len = text.len();
    let mut mapping = TextMap::default();
    if kind == TextKind::Code {
        (text, mapping) = TextMap::expand(&text);
        let mut byte = 0;
        for run in &mut runs {
            let end = byte + run.len;
            run.len = mapping.to_display(end) - mapping.to_display(byte);
            byte = end;
        }
        for (range, _) in &mut links {
            *range = mapping.to_display(range.start)..mapping.to_display(range.end);
        }
    }
    PreparedText {
        text,
        runs,
        links,
        mapping,
        source_len,
    }
}

#[cfg(test)]
mod tests;
