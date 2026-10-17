use std::sync::{Arc, LazyLock};

use gpui::{Image, ImageFormat, Img, Styled, img, px};

pub(crate) fn logo() -> Img {
    static LOGO: LazyLock<Arc<Image>> = LazyLock::new(|| {
        Arc::new(Image::from_bytes(
            ImageFormat::Png,
            include_bytes!("../assets/mira.png").to_vec(),
        ))
    });
    img(Arc::clone(&LOGO)).size(px(72.))
}
