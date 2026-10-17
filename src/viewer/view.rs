use super::*;
use crate::{
    appearance::Palette, code_cache::CodeCache, flow_cache::FlowCache, image_cache::ImageCache,
    reader, render_context::RenderContext, text_cache::TextCache,
};
use gpui::{MouseButton, div, prelude::*};

impl Render for Viewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::for_window(window);
        if self.follow_bottom {
            self.list.scroll_to(gpui::ListOffset {
                item_ix: self.list.item_count(),
                offset_in_item: px(0.),
            });
        }
        let root = div()
            .relative()
            .key_context("Mira")
            .track_focus(&self.focus)
            .on_scroll_wheel(cx.listener(|this, _, _, _| {
                this.reveal.borrow_mut().target = None;
            }))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::top))
            .on_action(cx.listener(Self::bottom))
            .on_action(cx.listener(Self::page_up))
            .on_action(cx.listener(Self::page_down))
            .on_action(cx.listener(|this, _: &Find, window, cx| this.open_find(window, cx)))
            .on_action(cx.listener(|this, _: &FindNext, window, cx| this.find_next(window, cx)))
            .on_action(
                cx.listener(|this, _: &FindPrevious, window, cx| this.find_previous(window, cx)),
            )
            .on_action(cx.listener(|this, _: &CloseFind, window, cx| this.close_find(window, cx)))
            .on_action(
                cx.listener(|this, _: &Contents, window, cx| this.toggle_contents(window, cx)),
            )
            .on_action(cx.listener(|this, _: &Back, window, cx| this.back(window, cx)))
            .on_action(cx.listener(|this, _: &Forward, window, cx| this.forward(window, cx)))
            .on_action(cx.listener(|this, _: &Reload, window, cx| this.reload(window, cx)))
            .on_action(cx.listener(|this, _: &ZoomIn, window, cx| this.zoom_in(window, cx)))
            .on_action(cx.listener(|this, _: &ZoomOut, window, cx| this.zoom_out(window, cx)))
            .on_action(cx.listener(|this, _: &ZoomReset, window, cx| this.zoom_reset(window, cx)))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.selection.borrow_mut().finish()),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.selection.borrow_mut().finish()),
            )
            .size_full()
            .flex()
            .flex_col()
            .bg(palette.background)
            .text_color(palette.text)
            .font_family("Segoe UI");
        let Some(loaded) = self.loaded.clone() else {
            return self.empty(root, palette, window, cx);
        };
        let images = self
            .images
            .get_or_insert_with(|| ImageCache::new(Arc::clone(&loaded), cx.entity_id()));
        images.borrow_mut().begin_frame();
        let texts = self
            .texts
            .get_or_insert_with(|| TextCache::new(Arc::clone(&loaded), cx.entity_id()));
        let codes = self
            .codes
            .get_or_insert_with(|| CodeCache::new(Arc::clone(&loaded), cx.entity_id()));
        let flows = self
            .flows
            .get_or_insert_with(|| FlowCache::new(Arc::clone(&loaded), cx.entity_id()));
        let width = f32::from(window.viewport_size().width);
        let sidebar = if self.contents {
            240_f32.min(width * 0.4)
        } else {
            0.
        };
        let resources = RenderContext {
            loaded: Arc::clone(&loaded),
            links: Rc::new(cx.processor(Self::activate_link)),
            selection: Rc::clone(&self.selection),
            tables: Rc::clone(&self.table_scrolls),
            images: Rc::clone(images),
            texts: Rc::clone(texts),
            codes: Rc::clone(codes),
            flows: Rc::clone(flows),
            cell: None,
            paint: self.content_paint.clone(),
            owner: cx.entity_id(),
            palette,
            width: (width - sidebar - 64.).max(1.),
            scale: f32::from(self.zoom) / 100.,
            highlights: Rc::clone(&self.find.highlights),
            reveal: Rc::clone(&self.reveal),
            code_scrolls: Rc::clone(&self.code_scrolls),
        };
        root.when_some(self.link_error.clone(), |root, message| {
            root.child(div().p_2().text_color(palette.error).child(message))
        })
        .when(self.find.visible, |root| {
            root.child(self.find_bar(palette, cx))
        })
        .child(
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .w_full()
                .when(self.contents, |body| {
                    body.child(self.outline(sidebar, palette, cx))
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .flex()
                        .flex_col()
                        .when(loaded.document.blocks.is_empty(), |body| {
                            body.child(
                                self.content_paint.wrap(
                                    div()
                                        .p_8()
                                        .child("This Markdown file is empty.")
                                        .into_any_element(),
                                ),
                            )
                        })
                        .child(reader::viewport(loaded, self.list.clone(), resources)),
                ),
        )
    }
}
