use gpui::EntityId;

use crate::{
    image_cache::CacheHandle, render_table::TableScrolls, render_text::LinkHandler,
    selection::SelectionHandle,
};

#[derive(Clone)]
pub(crate) struct RenderContext {
    pub links: LinkHandler,
    pub selection: SelectionHandle,
    pub tables: TableScrolls,
    pub images: CacheHandle,
    pub owner: EntityId,
    pub width: f32,
}
