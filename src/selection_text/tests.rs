use super::*;
use gpui::{FontStyle, FontWeight, font};

#[test]
fn selection_splits_runs_without_losing_nested_styles() {
    let mut font = font("Segoe UI");
    font.style = FontStyle::Italic;
    font.weight = FontWeight::BOLD;
    let run = TextRun {
        len: 8,
        font,
        color: Palette::light().text,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let selected = highlight(vec![run.clone()], Some(2..6), Palette::light());
    assert_eq!(
        selected.iter().map(|run| run.len).collect::<Vec<_>>(),
        vec![2, 4, 2]
    );
    assert!(selected[1].background_color.is_some());
    assert_eq!(selected[1].font, run.font);
    assert!(selected[0].background_color.is_none());
}

#[test]
fn appearance_changes_preserve_selected_utf8_bytes_and_link_styles() {
    let text = "a東京😀z";
    for palette in [Palette::light(), Palette::dark()] {
        let run = TextRun {
            len: text.len(),
            font: font("Segoe UI"),
            color: palette.link,
            background_color: None,
            underline: Some(gpui::UnderlineStyle::default()),
            strikethrough: None,
        };
        let runs = highlight(vec![run.clone()], Some(1..11), palette);
        assert_eq!(
            runs.iter().map(|run| run.len).collect::<Vec<_>>(),
            [1, 10, 1]
        );
        assert_eq!(&text[1..1 + runs[1].len], "東京😀");
        assert_eq!(runs[1].color, palette.text);
        assert_eq!(runs[1].background_color, Some(palette.selection));
        assert_eq!(runs[1].underline, run.underline);
    }
}
