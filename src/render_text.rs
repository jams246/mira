use std::{ops::Range, rc::Rc};

use gpui::{
    AnyElement, App, EntityId, FontStyle, FontWeight, SharedString, StrikethroughStyle, TextAlign,
    TextRun, UnderlineStyle, Window, font, px, rgb,
};
use mira::document::{Document, Inline};

use crate::{
    selection::{Position, SelectionHandle},
    selection_text, text_background,
    text_map::TextMap,
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
    activate_link: LinkHandler,
    selection: SelectionHandle,
    owner: EntityId,
) -> AnyElement {
    let PreparedText {
        text,
        runs,
        links,
        destinations,
        mapping,
        source_len,
    } = prepare(document, inlines, format.kind);
    let selected = selection
        .borrow()
        .range_at(position, source_len)
        .map(|range| mapping.to_display(range.start)..mapping.to_display(range.end));
    let runs = selection_text::highlight(runs, selected);
    let text: SharedString = text.into();
    let (styled, layout) = text_background::text(text.clone(), runs, format.align);
    selection_text::wrap(
        text,
        layout,
        styled,
        selection_text::Interaction {
            position,
            selection,
            owner,
            mapping,
            align: format.align,
            inline: format.inline,
            links: links.into_iter().zip(destinations).collect(),
            activate_link,
        },
    )
}

pub(crate) struct PreparedText {
    pub text: String,
    pub runs: Vec<TextRun>,
    links: Vec<Range<usize>>,
    destinations: Vec<usize>,
    mapping: TextMap,
    source_len: usize,
}

pub(crate) fn prepare(document: &Document, inlines: &[Inline], kind: TextKind) -> PreparedText {
    let mut text = String::new();
    let mut runs = Vec::with_capacity(inlines.len());
    let mut links = Vec::new();
    let mut destinations = Vec::new();
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
            color: rgb(if inline.link.is_some() {
                0x0969da
            } else {
                0x242424
            })
            .into(),
            background_color: inline.style.code.then(|| rgb(0xeeeeee).into()),
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
            links.push(start..text.len());
            destinations.push(target);
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
        for range in &mut links {
            *range = mapping.to_display(range.start)..mapping.to_display(range.end);
        }
    }
    PreparedText {
        text,
        runs,
        links,
        destinations,
        mapping,
        source_len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mira::{document::BlockKind, markdown};

    #[test]
    fn combines_native_styles_and_preserves_utf8_link_ranges() {
        let document =
            markdown::parse("***Grüße*** and [東京](https://example.com) ~~gone~~".into());
        let BlockKind::Paragraph(inlines) = &document.blocks[0].kind else {
            panic!("paragraph");
        };
        let prepared = prepare(&document, inlines, TextKind::Body);
        assert_eq!(prepared.text, "Grüße and 東京 gone");
        assert_eq!(
            prepared.runs.iter().map(|run| run.len).sum::<usize>(),
            prepared.text.len()
        );
        assert!(
            prepared
                .runs
                .iter()
                .any(|run| run.font.weight == FontWeight::BOLD
                    && run.font.style == FontStyle::Italic)
        );
        assert_eq!(&prepared.text[prepared.links[0].clone()], "東京");
        assert!(prepared.runs.iter().any(|run| run.strikethrough.is_some()));
    }

    #[test]
    fn code_preserves_indentation_blank_lines_and_native_monospace_runs() {
        let document = markdown::parse("```\nlet x = 1;\n\n\t  x\n```".into());
        let BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
            panic!("code");
        };
        let prepared = prepare(&document, runs, TextKind::Code);
        assert_eq!(prepared.text, "let x = 1;\n\n      x\n");
        assert_eq!(prepared.source_len, "let x = 1;\n\n\t  x\n".len());
        assert_eq!(
            prepared.runs.iter().map(|run| run.len).sum::<usize>(),
            prepared.text.len()
        );
        assert!(
            prepared
                .runs
                .iter()
                .all(|run| run.font.family.as_ref() == "Consolas")
        );
    }
}
