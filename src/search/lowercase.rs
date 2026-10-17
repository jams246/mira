use std::{
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};

struct Change {
    source: Range<usize>,
    lowercase: Range<usize>,
}

pub(super) fn query(text: &str, cancel: &AtomicBool) -> Option<String> {
    let mut output = String::with_capacity(text.len());
    let mut check_at = 0;
    for (byte, ch) in text.char_indices() {
        if byte >= check_at {
            if cancel.load(Ordering::Relaxed) {
                return None;
            }
            check_at = byte + 8192;
        }
        output.extend(ch.to_lowercase());
    }
    Some(output)
}

pub(super) struct Lowercase {
    text: String,
    changes: Vec<Change>,
}

impl Lowercase {
    pub(super) fn new(mut text: String, match_case: bool, cancel: &AtomicBool) -> Option<Self> {
        if match_case || text.is_ascii() {
            if !match_case {
                text.make_ascii_lowercase();
            }
            return Some(Self {
                text,
                changes: Vec::new(),
            });
        }
        let mut output = String::with_capacity(text.len());
        let mut changes = Vec::new();
        let mut check_at = 0;
        for (byte, ch) in text.char_indices() {
            if byte >= check_at {
                if cancel.load(Ordering::Relaxed) {
                    return None;
                }
                check_at = byte + 8192;
            }
            let start = output.len();
            let lowercase = ch.to_lowercase();
            let expansion = lowercase.clone().count() > 1;
            output.extend(lowercase);
            if output.len() - start != ch.len_utf8() || expansion {
                changes.push(Change {
                    source: byte..byte + ch.len_utf8(),
                    lowercase: start..output.len(),
                });
            }
        }
        Some(Self {
            text: output,
            changes,
        })
    }

    pub(super) fn as_str(&self) -> &str {
        &self.text
    }

    pub(super) fn source_boundary(&self, byte: usize) -> Option<usize> {
        let next = self
            .changes
            .partition_point(|change| change.lowercase.end <= byte);
        if self
            .changes
            .get(next)
            .is_some_and(|change| change.lowercase.start < byte)
        {
            return None;
        }
        Some(next.checked_sub(1).map_or(byte, |previous| {
            let change = &self.changes[previous];
            change.source.end + (byte - change.lowercase.end)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_normalization_uses_scalar_lowercase_and_honors_cancellation() {
        let cancelled = AtomicBool::new(true);
        assert_eq!(query("ÉKİΟΣ", &cancelled), None);
        cancelled.store(false, Ordering::Relaxed);
        assert_eq!(query("ÉKİΟΣ", &cancelled).as_deref(), Some("éki\u{307}οσ"));
    }
}
