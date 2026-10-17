//! Document-order heading metadata and collision-free local fragments.

use std::collections::{HashMap, HashSet};

use crate::{document::BlockKind, document::Document, links::heading_slug};

/// One heading with a stable block identity within its parsed document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    /// Semantic block index.
    pub block: usize,
    /// CommonMark heading level, from one through six.
    pub level: u8,
    /// Display text without inline Markdown syntax.
    pub title: String,
    /// Unique fragment without its leading hash sign.
    pub fragment: String,
}

/// Builds heading metadata once without requiring layout or the window thread.
pub fn index(document: &Document) -> Vec<Heading> {
    let mut used = HashSet::new();
    let mut suffixes = HashMap::new();
    document
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(block, item)| {
            let BlockKind::Heading { level, runs } = &item.kind else {
                return None;
            };
            let title = runs
                .iter()
                .map(|run| run.text.as_str(&document.source))
                .collect::<String>();
            let mut base = heading_slug(&title);
            if base.is_empty() {
                base.push_str("section");
            }
            let suffix = suffixes.entry(base.clone()).or_insert(0);
            let fragment = loop {
                let candidate = if *suffix == 0 {
                    base.clone()
                } else {
                    format!("{base}-{suffix}")
                };
                *suffix += 1;
                if used.insert(candidate.clone()) {
                    break candidate;
                }
            };
            Some(Heading {
                block,
                level: *level,
                title,
                fragment,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
