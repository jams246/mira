use mira::document::{Document, Inline, Text};

pub(crate) enum Piece {
    Text { byte: usize, runs: Vec<Inline> },
    Image { byte: usize, index: usize },
    Break,
}

pub(crate) fn pieces(document: &Document, inlines: &[Inline]) -> Vec<Piece> {
    let mut output = Vec::new();
    let mut word = Vec::new();
    let mut word_start = 0;
    let mut byte = 0;
    for (index, inline) in inlines.iter().enumerate() {
        let text = inline.text.as_str(&document.source);
        if inline.image.is_some() {
            flush(&mut output, &mut word, word_start);
            output.push(Piece::Image { byte, index });
        } else {
            let mut start = 0;
            for (offset, ch) in text.char_indices().filter(|(_, ch)| ch.is_whitespace()) {
                let end = offset + ch.len_utf8();
                if word.is_empty() {
                    word_start = byte + start;
                }
                let text_end = if ch == '\n' { offset } else { end };
                if start < text_end {
                    word.push(fragment(inline, text, start, text_end));
                }
                flush(&mut output, &mut word, word_start);
                if ch == '\n' {
                    output.push(Piece::Break);
                }
                start = end;
            }
            if start < text.len() {
                if word.is_empty() {
                    word_start = byte + start;
                }
                word.push(fragment(inline, text, start, text.len()));
            }
        }
        byte += text.len();
    }
    flush(&mut output, &mut word, word_start);
    output
}

fn fragment(inline: &Inline, text: &str, start: usize, end: usize) -> Inline {
    Inline {
        source_range: inline.source_range.clone(),
        text: match &inline.text {
            Text::Source(range) => Text::Source(range.start + start..range.start + end),
            Text::Literal(_) => Text::Literal(text[start..end].into()),
        },
        style: inline.style,
        link: inline.link,
        image: None,
    }
}

fn flush(output: &mut Vec<Piece>, word: &mut Vec<Inline>, byte: usize) {
    if !word.is_empty() {
        output.push(Piece::Text {
            byte,
            runs: std::mem::take(word),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mira::{document::BlockKind, markdown};

    #[test]
    fn mixed_styles_keep_words_together_and_hard_breaks_preserve_image_offsets() {
        let document = markdown::parse("ab**cd**ef ![First](one.png)  \n![](two.png) 東京".into());
        let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
            panic!("paragraph");
        };
        let pieces = pieces(&document, runs);
        let Piece::Text { byte, runs } = &pieces[0] else {
            panic!("word");
        };
        assert_eq!(*byte, 0);
        assert_eq!(
            runs.iter()
                .map(|run| run.text.as_str(&document.source))
                .collect::<String>(),
            "abcdef "
        );
        assert!(runs[1].style.bold);
        assert!(matches!(pieces[1], Piece::Image { byte: 7, .. }));
        assert!(matches!(pieces[2], Piece::Break));
        assert!(matches!(pieces[3], Piece::Image { byte: 13, .. }));
    }
}
