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
        let extra: usize = self
            .tabs
            .iter()
            .take_while(|tab| tab.source < source)
            .map(|tab| tab.display.len() - 1)
            .sum();
        source + extra
    }

    pub(crate) fn to_source(&self, display: usize) -> usize {
        let mut extra = 0;
        for tab in &self.tabs {
            if display < tab.display.start {
                break;
            }
            if display < tab.display.end {
                return tab.source;
            }
            extra += tab.display.len() - 1;
        }
        display - extra
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
}
