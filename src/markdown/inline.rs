use std::ops::Range;

use pulldown_cmark::{CowStr, Tag};

use crate::document::{Inline, InlineStyle, Text};

#[derive(Default)]
pub(super) struct InlineState {
    style: InlineStyle,
    link: Option<usize>,
    frames: Vec<(InlineStyle, Option<usize>)>,
    images: Vec<Image>,
}

struct Image {
    target: usize,
    alt: String,
    style: InlineStyle,
    link: Option<usize>,
}

impl InlineState {
    pub(super) fn start(&mut self, tag: Tag<'_>, targets: &mut Vec<String>) {
        self.frames.push((self.style, self.link));
        match tag {
            Tag::Emphasis => self.style.italic = true,
            Tag::Strong => self.style.bold = true,
            Tag::Strikethrough => self.style.strike = true,
            Tag::Link { dest_url, .. } => {
                self.link = Some(targets.len());
                targets.push(dest_url.into_string());
            }
            _ => {}
        }
    }

    pub(super) fn end(&mut self) {
        if let Some((style, link)) = self.frames.pop() {
            self.style = style;
            self.link = link;
        }
    }

    pub(super) fn begin_image(&mut self, target: CowStr<'_>, targets: &mut Vec<String>) {
        self.images.push(Image {
            target: targets.len(),
            alt: String::new(),
            style: self.style,
            link: self.link,
        });
        targets.push(target.into_string());
    }

    pub(super) fn end_image(&mut self) -> Option<Inline> {
        let image = self.images.pop()?;
        if !self.images.is_empty() {
            return None;
        }
        Some(Inline {
            text: Text::Literal(image.alt.into_boxed_str()),
            style: image.style,
            link: image.link,
            image: Some(image.target),
        })
    }

    pub(super) fn run(
        &mut self,
        source: &str,
        value: CowStr<'_>,
        range: Range<usize>,
        code: bool,
    ) -> Option<Inline> {
        if let Some(image) = self.images.first_mut() {
            image.alt.push_str(&value);
            return None;
        }
        let mut style = self.style;
        style.code |= code;
        Some(Inline {
            text: text(source, value, range),
            style,
            link: self.link,
            image: None,
        })
    }
}

pub(super) fn text(source: &str, value: CowStr<'_>, range: Range<usize>) -> Text {
    if source.get(range.clone()) == Some(value.as_ref()) {
        Text::Source(range)
    } else if let CowStr::Borrowed(value) = &value
        && let Some(start) = (value.as_ptr() as usize).checked_sub(source.as_ptr() as usize)
        && let Some(end) = start.checked_add(value.len())
        && source.get(start..end) == Some(*value)
    {
        Text::Source(start..end)
    } else {
        Text::Literal(value.into_string().into_boxed_str())
    }
}
