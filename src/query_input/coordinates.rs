use std::{borrow::Cow, ops::Range};

pub(super) fn single_line(text: &str) -> Cow<'_, str> {
    if text.contains(['\r', '\n']) {
        Cow::Owned(text.replace("\r\n", " ").replace(['\r', '\n'], " "))
    } else {
        Cow::Borrowed(text)
    }
}

pub(super) fn from_utf16(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (index, ch) in text.char_indices() {
        if units >= offset {
            return index;
        }
        units += ch.len_utf16();
    }
    text.len()
}

pub(super) fn to_utf16(text: &str, offset: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < offset)
        .map(|(_, ch)| ch.len_utf16())
        .sum()
}

pub(super) fn from_utf16_range(text: &str, range: &Range<usize>) -> Range<usize> {
    let start = from_utf16(text, range.start);
    let end = from_utf16(text, range.end);
    start.min(end)..start.max(end)
}

pub(super) fn previous(text: &str, offset: usize) -> usize {
    text.char_indices()
        .rev()
        .find_map(|(index, _)| (index < offset).then_some(index))
        .unwrap_or(0)
}

pub(super) fn next(text: &str, offset: usize) -> usize {
    text.char_indices()
        .find_map(|(index, _)| (index > offset).then_some(index))
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_positions_preserve_utf8_boundaries() {
        let text = "a\u{1f642}é日";
        for index in text
            .char_indices()
            .map(|(index, _)| index)
            .chain([text.len()])
        {
            assert_eq!(from_utf16(text, to_utf16(text, index)), index);
        }
    }

    #[test]
    fn partial_surrogates_and_outside_ranges_stay_safe() {
        assert_eq!(from_utf16_range("a\u{1f642}z", &(2..99)), 5..6);
    }

    #[test]
    fn moving_and_deleting_unicode_does_not_split_bytes() {
        let text = "é\u{1f642}日";
        let first = next(text, 0);
        let second = next(text, first);
        assert_eq!((first, second, previous(text, second)), (2, 6, 2));
    }

    #[test]
    fn pasted_lines_become_one_search_line() {
        assert_eq!(single_line("one\r\ntwo\rthree\nfour"), "one two three four");
    }
}
