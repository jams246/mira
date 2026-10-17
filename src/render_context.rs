use gpui::EntityId;
use std::sync::Arc;

use crate::{
    content_paint::ContentPaint, file::LoadedDocument, flow_cache, image_cache::CacheHandle,
    render_table::TableScrolls, render_text::LinkHandler, selection::SelectionHandle, text_cache,
};

#[derive(Clone)]
pub(crate) struct RenderContext {
    pub loaded: Arc<LoadedDocument>,
    pub links: LinkHandler,
    pub selection: SelectionHandle,
    pub tables: TableScrolls,
    pub images: CacheHandle,
    pub texts: text_cache::CacheHandle,
    pub flows: flow_cache::CacheHandle,
    pub paint: ContentPaint,
    pub cell: Option<usize>,
    pub owner: EntityId,
    pub width: f32,
}
