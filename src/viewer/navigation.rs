use gpui::{ClipboardItem, Context, ListOffset, Pixels, Window, px};

use super::{Bottom, Copy, PageDown, PageUp, SelectAll, Top, Viewer};

impl Viewer {
    pub(crate) fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(loaded) = &self.loaded {
            let text = self.selection.borrow().text(&loaded.document);
            if !text.is_empty() {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
            }
        }
    }

    pub(crate) fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(loaded) = &self.loaded {
            self.selection.borrow_mut().select_all(&loaded.document);
            cx.notify();
        }
    }

    pub(crate) fn top(&mut self, _: &Top, _: &mut Window, cx: &mut Context<Self>) {
        self.reveal.borrow_mut().target = None;
        self.follow_bottom = false;
        self.list.scroll_to(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        cx.notify();
    }

    pub(crate) fn bottom(&mut self, _: &Bottom, _: &mut Window, cx: &mut Context<Self>) {
        self.reveal.borrow_mut().target = None;
        self.follow_bottom = true;
        self.list.scroll_to(ListOffset {
            item_ix: self.list.item_count(),
            offset_in_item: px(0.),
        });
        cx.notify();
    }

    pub(crate) fn page_up(&mut self, _: &PageUp, _: &mut Window, cx: &mut Context<Self>) {
        self.reveal.borrow_mut().target = None;
        self.follow_bottom = false;
        let top = self.list.logical_scroll_top();
        let viewport = self.list.viewport_bounds();
        let visible = (top.item_ix..self.list.item_count())
            .take_while(|index| {
                self.list
                    .bounds_for_item(*index)
                    .is_some_and(|bounds| bounds.top() < viewport.bottom())
            })
            .count();
        self.list
            .scroll_to(previous_page(top, visible, viewport.size.height * 0.9));
        cx.notify();
    }

    pub(crate) fn page_down(&mut self, _: &PageDown, _: &mut Window, cx: &mut Context<Self>) {
        self.reveal.borrow_mut().target = None;
        self.follow_bottom = false;
        let top = self.list.logical_scroll_top();
        let viewport = self.list.viewport_bounds();
        let target = viewport.top() + viewport.size.height * 0.9;
        for index in top.item_ix..self.list.item_count() {
            let Some(bounds) = self.list.bounds_for_item(index) else {
                break;
            };
            if target < bounds.bottom() {
                self.list.scroll_to(ListOffset {
                    item_ix: index,
                    offset_in_item: (target - bounds.top()).max(px(0.)),
                });
                break;
            }
        }
        cx.notify();
    }
}

fn previous_page(top: ListOffset, visible: usize, height: Pixels) -> ListOffset {
    if top.offset_in_item >= height || (visible <= 1 && top.offset_in_item > px(0.)) {
        return ListOffset {
            item_ix: top.item_ix,
            offset_in_item: (top.offset_in_item - height).max(px(0.)),
        };
    }
    // shortcut: Estimate prior rows from the visible count, refine if variable-height paging needs pixel precision.
    ListOffset {
        item_ix: top
            .item_ix
            .saturating_sub(((visible as f32 * 0.9) as usize).max(1)),
        offset_in_item: px(0.),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_up_uses_logical_rows_across_unmeasured_gaps() {
        let previous = previous_page(
            ListOffset {
                item_ix: 283,
                offset_in_item: px(17.),
            },
            19,
            px(650.),
        );
        assert_eq!(previous.item_ix, 266);
        assert_eq!(previous.offset_in_item, px(0.));
    }

    #[test]
    fn page_up_preserves_paging_within_a_tall_block() {
        let previous = previous_page(
            ListOffset {
                item_ix: 4,
                offset_in_item: px(900.),
            },
            1,
            px(650.),
        );
        assert_eq!(previous.item_ix, 4);
        assert_eq!(previous.offset_in_item, px(250.));
        assert_eq!(previous_page(previous, 1, px(650.)).offset_in_item, px(0.));
    }
}
