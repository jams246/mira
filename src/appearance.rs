use gpui::{Hsla, Window, WindowAppearance, rgb};

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub background: Hsla,
    pub text: Hsla,
    pub link: Hsla,
    pub panel: Hsla,
    pub selection: Hsla,
    pub border: Hsla,
    pub quote: Hsla,
    pub rule: Hsla,
    pub muted: Hsla,
    pub error: Hsla,
}

impl Palette {
    pub(crate) fn for_window(window: &Window) -> Self {
        match window.appearance() {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
        }
    }

    pub(crate) fn light() -> Self {
        Self {
            background: rgb(0xfafafa).into(),
            text: rgb(0x242424).into(),
            link: rgb(0x075fc7).into(),
            panel: rgb(0xeeeeee).into(),
            selection: rgb(0xb6d7ff).into(),
            border: rgb(0xc4c8cc).into(),
            quote: rgb(0xb0b7bf).into(),
            rule: rgb(0xd8d8d8).into(),
            muted: rgb(0x62666a).into(),
            error: rgb(0xa02020).into(),
        }
    }

    pub(crate) fn dark() -> Self {
        Self {
            background: rgb(0x1e1f22).into(),
            text: rgb(0xe3e5e8).into(),
            link: rgb(0x79b8ff).into(),
            panel: rgb(0x2b2d31).into(),
            selection: rgb(0x264f78).into(),
            border: rgb(0x50545c).into(),
            quote: rgb(0x676c76).into(),
            rule: rgb(0x50545c).into(),
            muted: rgb(0xa9adb5).into(),
            error: rgb(0xff9292).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Rgba;

    fn luminance(color: Hsla) -> f32 {
        let color = Rgba::from(color);
        let linear = |channel: f32| {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }

    #[test]
    fn both_native_palettes_keep_body_links_status_and_selected_text_readable() {
        for palette in [Palette::light(), Palette::dark()] {
            for (text, background) in [
                (palette.text, palette.background),
                (palette.text, palette.panel),
                (palette.text, palette.selection),
                (palette.link, palette.background),
                (palette.link, palette.panel),
                (palette.muted, palette.background),
                (palette.error, palette.background),
            ] {
                let first = luminance(text);
                let second = luminance(background);
                let contrast = (first.max(second) + 0.05) / (first.min(second) + 0.05);
                assert!(contrast >= 4.5, "text contrast {contrast}");
            }
        }
    }
}
