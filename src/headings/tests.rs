use crate::markdown;

use super::index;

#[test]
fn headings_preserve_levels_display_text_and_block_identity() {
    let document = markdown::parse("Intro\n\n## **Hello** &amp; 東京\n\nBody\n\n# Last".into());
    let headings = index(&document);
    assert_eq!(
        headings
            .iter()
            .map(|heading| (
                heading.block,
                heading.level,
                heading.title.as_str(),
                heading.fragment.as_str()
            ))
            .collect::<Vec<_>>(),
        [
            (1, 2, "Hello & 東京", "hello--東京"),
            (3, 1, "Last", "last")
        ]
    );
}

#[test]
fn duplicate_fragments_cannot_collide_with_numbered_titles() {
    let document = markdown::parse("# A\n# A\n# A-1\n# A\n# A-1\n".into());
    assert_eq!(
        index(&document)
            .iter()
            .map(|heading| heading.fragment.as_str())
            .collect::<Vec<_>>(),
        ["a", "a-1", "a-1-1", "a-2", "a-1-2"]
    );
}

#[test]
fn punctuation_only_headings_have_navigable_unique_fragments() {
    let document = markdown::parse("# !!!\n# section\n# ???\n".into());
    assert_eq!(
        index(&document)
            .iter()
            .map(|heading| heading.fragment.as_str())
            .collect::<Vec<_>>(),
        ["section", "section-1", "section-2"]
    );
}

#[test]
fn documents_without_headings_have_empty_index() {
    assert!(index(&markdown::parse("Ordinary text".into())).is_empty());
}
