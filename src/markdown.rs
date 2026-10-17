//! One-pass CommonMark and selected GFM parsing into native document blocks.

mod inline;
mod state;
mod table;

#[cfg(test)]
mod edge_tests;
#[cfg(test)]
mod syntax_tests;
#[cfg(test)]
mod tests;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::document::{BlockKind, ContextKind, Document, Table};

use self::state::State;

/// Parses Markdown once, preserving original source and normalized display text.
pub fn parse(source: String) -> Document {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let mut state = State::default();
    for (event, range) in Parser::new_ext(&source, options).into_offset_iter() {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => state.begin(BlockKind::Paragraph(Vec::with_capacity(1)), range),
                Tag::Heading { level, .. } => state.begin(
                    BlockKind::Heading {
                        level: level as u8,
                        runs: Vec::with_capacity(1),
                    },
                    range,
                ),
                Tag::CodeBlock(kind) => {
                    let language = match kind {
                        CodeBlockKind::Fenced(info) if !info.is_empty() => {
                            Some(inline::text(&source, info, range.clone()))
                        }
                        _ => None,
                    };
                    state.begin(
                        BlockKind::Code {
                            runs: Vec::new(),
                            language,
                        },
                        range,
                    );
                }
                Tag::HtmlBlock => state.begin(BlockKind::RawHtml(Vec::with_capacity(1)), range),
                Tag::BlockQuote(_) => {
                    state.finish();
                    state.push_context(ContextKind::Quote);
                }
                Tag::List(start) => {
                    state.finish_pending_item(range.clone());
                    state.finish();
                    state.lists.push(start);
                }
                Tag::Item => state.begin_item(),
                Tag::Table(alignments) => {
                    state.finish();
                    let index = state.tables.len();
                    state.tables.push(Table { alignments });
                    state.table = Some(table::TableState::new(index));
                }
                Tag::TableHead | Tag::TableRow => {
                    if let Some(table) = &mut state.table {
                        table.begin_row(matches!(tag, Tag::TableHead), range);
                    }
                }
                Tag::TableCell => {
                    if let Some(table) = &mut state.table {
                        table.begin_cell();
                    }
                }
                Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } => {
                    state.inline.start(tag, &mut state.targets);
                }
                Tag::Image { dest_url, .. } => {
                    state
                        .inline
                        .begin_image(dest_url, range, &mut state.targets)
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::CodeBlock | TagEnd::HtmlBlock => {
                    state.finish()
                }
                TagEnd::BlockQuote(_) => {
                    state.finish();
                    state.pop_context();
                }
                TagEnd::List(_) => {
                    state.finish();
                    state.lists.pop();
                }
                TagEnd::Item => state.end_item(range),
                TagEnd::TableCell => {
                    if let Some(table) = &mut state.table {
                        table.finish_cell();
                    }
                }
                TagEnd::TableHead | TagEnd::TableRow => state.finish_row(),
                TagEnd::Table => {
                    state.finish_row();
                    state.table = None;
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                    state.inline.end()
                }
                TagEnd::Image => {
                    if let Some(run) = state.inline.end_image() {
                        state.push_run(run);
                    }
                }
                _ => {}
            },
            Event::Text(value) | Event::Html(value) | Event::InlineHtml(value) => {
                if let Some(run) = state.inline.run(&source, value, range, false) {
                    state.push_run(run);
                }
            }
            Event::Code(value) => {
                if let Some(run) = state.inline.run(&source, value, range, true) {
                    state.push_run(run);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                let value = if matches!(event, Event::HardBreak) {
                    "\n"
                } else {
                    " "
                };
                if let Some(run) = state.inline.run(&source, value.into(), range, false) {
                    state.push_run(run);
                }
            }
            Event::Rule => {
                state.finish();
                state.emit(BlockKind::Rule, range);
            }
            Event::TaskListMarker(checked) => state.task(checked),
            _ => {}
        }
    }
    state.finish();
    Document {
        source,
        blocks: state.blocks,
        contexts: state.contexts,
        targets: state.targets,
        tables: state.tables,
    }
}
