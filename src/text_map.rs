use std::ops::Range;

#[derive(Clone)]
struct Tab {
    source: usize,
    display: Range<usize>,
}

#[derive(Clone, Default)]
pub(crate) struct TextMap {
    pub preformatted: bool,
    tabs: Vec<Tab>,
}

impl TextMap {
    pub(crate) fn expand(source: &str) -> (String, Self) {
        let mut text = String::with_capacity(source.len());
        let mut mapping = Self {
            preformatted: true,
            tabs: Vec::new(),
        };
        let mut column = 0;
        for (byte, character) in source.char_indices() {
            if character == '\t' {
                let spaces = 4 - column % 4;
                let start = text.len();
                text.extend(std::iter::repeat_n(' ', spaces));
                mapping.tabs.push(Tab {
                    source: byte,
                    display: start..text.len(),
                });
                column += spaces;
            } else {
                text.push(character);
                column = if character == '\n' { 0 } else { column + 1 };
            }
        }
        (text, mapping)
    }

    pub(crate) fn to_display(&self, source: usize) -> usize {
        let end = self.tabs.partition_point(|tab| tab.source < source);
        source
            + self.tabs[..end]
                .last()
                .map_or(0, |tab| tab.display.end - tab.source - 1)
    }

    pub(crate) fn to_source(&self, display: usize) -> usize {
        let end = self
            .tabs
            .partition_point(|tab| tab.display.start <= display);
        let Some(tab) = self.tabs[..end].last() else {
            return display;
        };
        if display < tab.display.end {
            tab.source
        } else {
            display - (tab.display.end - tab.source - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_tabs_use_four_column_stops_and_reset_on_newlines() {
        let (display, mapping) = TextMap::expand("a\tX\n\t東京\tY");
        assert_eq!(display, "a   X\n    東京  Y");
        assert_eq!(mapping.to_source(2), 1);
        assert_eq!(mapping.to_source(4), 2);
        assert_eq!(mapping.to_display(2), 4);
    }

    #[test]
    fn source_boundaries_round_trip_after_unicode_and_adjacent_tabs() {
        let source = "é\t\t😀\n\tx";
        let (display, mapping) = TextMap::expand(source);
        for byte in source
            .char_indices()
            .map(|(byte, _)| byte)
            .chain([source.len()])
        {
            assert_eq!(mapping.to_source(mapping.to_display(byte)), byte);
            assert!(display.is_char_boundary(mapping.to_display(byte)));
        }
    }

    #[test]
    fn exact_tab_starts_ends_and_trailing_expansion_map_to_original_bytes() {
        let (display, mapping) = TextMap::expand("ab\tc\t");
        assert_eq!(display, "ab  c   ");
        assert_eq!(
            (0..=5)
                .map(|byte| mapping.to_display(byte))
                .collect::<Vec<_>>(),
            [0, 1, 2, 4, 5, 8]
        );
        assert_eq!(
            (0..=8)
                .map(|byte| mapping.to_source(byte))
                .collect::<Vec<_>>(),
            [0, 1, 2, 2, 3, 4, 4, 4, 5]
        );
    }

    #[test]
    fn plain_unicode_and_empty_text_have_identity_mapping() {
        for source in ["", "é 東京 😀\nplain"] {
            let (display, mapping) = TextMap::expand(source);
            assert_eq!(display, source);
            assert!((0..=source.len()).all(|byte| mapping.to_display(byte) == byte && mapping.to_source(byte) == byte));
        }
    }

    #[test]
    fn many_indented_code_runs_preserve_display_lengths_and_source_endpoints() {
        let document = mira::markdown::parse("    \tvalue\t日本語\n".repeat(10_000));
        let mira::document::BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
            panic!("Expected code");
        };
        let plain: String = runs
            .iter()
            .map(|run| run.text.as_str(&document.source))
            .collect();
        let (display, mapping) = TextMap::expand(&plain);
        let mut source_byte = 0;
        let mut display_bytes = 0;
        for run in runs {
            let end = source_byte + run.text.as_str(&document.source).len();
            display_bytes += mapping.to_display(end) - mapping.to_display(source_byte);
            assert_eq!(mapping.to_source(mapping.to_display(end)), end);
            source_byte = end;
        }
        assert_eq!((runs.len(), display_bytes), (10_000, display.len()));
    }
}
