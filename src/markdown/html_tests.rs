use crate::document::{BlockKind, ContextKind, InlineStyle};

use super::{parse, tests::text};

#[test]
fn html_block_forms_preserve_literal_text_and_line_endings() {
    for html in [
        "<script>\n![東京](image.png) [link](next.md) &amp;\n</script>\n",
        "<!-- ![東京](image.png) -->\n",
        "<?mira ![東京](image.png) ?>\n",
        "<!DOCTYPE html>\n",
        "<![CDATA[![東京](image.png)]]>\n",
        "<div>\n![東京](image.png)\n</div>\n",
        "<custom>\n![東京](image.png)\n</custom>\n",
    ] {
        for ending in ["\n", "\r\n"] {
            let source = html.replace('\n', ending);
            let document = parse(source.clone());
            assert_eq!(document.source, source);
            assert_eq!(document.blocks.len(), 1);
            let BlockKind::RawHtml(runs) = &document.blocks[0].kind else {
                panic!("Expected literal HTML block");
            };
            assert_eq!(text(&document, runs), html);
            assert!(runs.iter().all(|run| run.image.is_none()
                && run.link.is_none()
                && run.style == InlineStyle::default()));
            assert!(document.targets.is_empty());
        }
    }
}

#[test]
fn html_blocks_preserve_quote_and_list_contexts() {
    for (source, expected) in [
        (
            "> <div>\n> ![東京](image.png)\n> </div>\n",
            ContextKind::Quote,
        ),
        (
            "- <div>\n  ![東京](image.png)\n  </div>\n",
            ContextKind::ListItem {
                number: None,
                checked: None,
            },
        ),
    ] {
        let document = parse(source.into());
        assert_eq!(document.blocks.len(), 1);
        let block = &document.blocks[0];
        let BlockKind::RawHtml(runs) = &block.kind else {
            panic!("Expected literal HTML block");
        };
        assert_eq!(text(&document, runs), "<div>\n![東京](image.png)\n</div>\n");
        assert_eq!(document.contexts[block.context.unwrap()].kind, expected);
        assert_eq!(
            block.marker,
            matches!(expected, ContextKind::ListItem { .. })
        );
        assert!(runs.iter().all(|run| run.image.is_none()));
    }
}
