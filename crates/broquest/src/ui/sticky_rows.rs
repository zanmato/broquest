//! Pins the folders a tree has scrolled past to the top of its list, so the
//! folder holding the rows on screen stays in reach. Both trees draw their rows
//! through `uniform_list`, whose decoration layer is told the scroll offset and
//! the row height, which is all the pinning needs.

use std::ops::Range;

use gpui::{
    AnyElement, App, Bounds, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Point, ScrollStrategy, Styled as _, UniformListDecoration,
    UniformListScrollHandle, Window, div, px,
};
use gpui_kit::component::ActiveTheme as _;

/// The most folders pinned at once, so a deep tree keeps rows to show.
const MAX_STICKY_ROWS: usize = 5;

/// A folder pinned to the top of the list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StickyRow {
    /// The index of the folder's entry.
    pub ix: usize,
    /// Where the row sits below the top of the list. It is pushed up as the
    /// folder's last child scrolls out.
    pub top: Pixels,
}

/// The folders to pin while the list is scrolled `scroll_top` down, outermost
/// first. `depth_of` gives the depth of each of the `len` entries, which are a
/// tree flattened in order with every open folder followed by its children.
///
/// Slot `d` pins the folder at depth `d` that holds the row lying under that
/// slot, once the folder's own row has scrolled under it. A folder stays
/// pinned while its children pass beneath and is pushed out by the first row
/// after them.
pub(crate) fn sticky_rows(
    depth_of: impl Fn(usize) -> usize,
    len: usize,
    scroll_top: Pixels,
    item_height: Pixels,
    max_rows: usize,
) -> Vec<StickyRow> {
    let mut rows: Vec<StickyRow> = Vec::new();
    if len == 0 || item_height <= Pixels::ZERO || scroll_top <= Pixels::ZERO {
        return rows;
    }
    let first = (scroll_top / item_height).floor() as usize;

    for slot in 0..max_rows {
        let row = first + slot;
        if row >= len || depth_of(row) < slot {
            break;
        }

        // The row's ancestors, outermost first, ending with the row itself.
        let mut chain = vec![row];
        let mut ix = row;
        while depth_of(ix) > 0 && ix > 0 {
            let depth = depth_of(ix);
            ix -= 1;
            while ix > 0 && depth_of(ix) >= depth {
                ix -= 1;
            }
            chain.push(ix);
        }
        chain.reverse();

        let Some(&folder) = chain.get(slot) else {
            break;
        };
        // Leaving the folder pinned in the slot above means its children ended.
        if rows.iter().zip(&chain).any(|(pinned, ix)| pinned.ix != *ix) {
            break;
        }
        let shows_children = folder + 1 < len && depth_of(folder + 1) > depth_of(folder);
        // A folder whose row still sits wholly in its slot needs no copy over it.
        if !shows_children || item_height * folder as f32 >= scroll_top + item_height * slot as f32
        {
            break;
        }

        // The row after the folder's last child pushes it up. Only rows near
        // the top can do that, so the search stops past them.
        let search_end = (first + max_rows + 2).min(len);
        let next_sibling = (folder + 1..search_end).find(|&ix| depth_of(ix) <= depth_of(folder));
        let slot_top = rows
            .last()
            .map_or(Pixels::ZERO, |above| above.top + item_height);
        let top = match next_sibling {
            Some(next) => slot_top.min(item_height * next as f32 - scroll_top - item_height),
            None => slot_top,
        };
        rows.push(StickyRow { ix: folder, top });
    }
    rows
}

/// How many rows to keep clear at the top when scrolling to an entry at
/// `depth`, so it does not land under its own pinned folders.
pub(crate) fn rows_hidden_above(depth: usize) -> usize {
    depth.min(MAX_STICKY_ROWS)
}

/// Scroll to an entry at `depth`, keeping it out from under the pinned folders.
pub(crate) fn scroll_to_entry(
    scroll_handle: &UniformListScrollHandle,
    ix: usize,
    depth: usize,
    strategy: ScrollStrategy,
) {
    match strategy {
        ScrollStrategy::Bottom => scroll_handle.scroll_to_item(ix, strategy),
        _ => scroll_handle.scroll_to_item_with_offset(ix, strategy, rows_hidden_above(depth)),
    }
}

/// A tree that can pin its folders.
pub(crate) trait StickyRowsHost: Sized + 'static {
    fn entry_count(&self) -> usize;

    fn entry_depth(&self, ix: usize) -> usize;

    /// The pinned copy of entry `ix`, shown in `slot` (0 at the top).
    fn render_sticky_row(
        &mut self,
        ix: usize,
        slot: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement;
}

/// The `uniform_list` decoration that draws a host's pinned folders.
pub(crate) struct StickyRows<T>(pub Entity<T>);

impl<T: StickyRowsHost> UniformListDecoration for StickyRows<T> {
    fn compute(
        &self,
        _visible_range: Range<usize>,
        bounds: Bounds<Pixels>,
        scroll_offset: Point<Pixels>,
        item_height: Pixels,
        _item_count: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.0.update(cx, |host, cx| {
            let scroll_top = -scroll_offset.y;
            // Keep at least half the list for the rows themselves.
            let fitting = if item_height > Pixels::ZERO {
                (bounds.size.height / item_height / 2.).floor() as usize
            } else {
                0
            };
            let rows = sticky_rows(
                |ix| host.entry_depth(ix),
                host.entry_count(),
                scroll_top,
                item_height,
                MAX_STICKY_ROWS.min(fitting),
            );

            let background = cx.theme().sidebar;
            let border = cx.theme().border;
            let mut layer = div().size_full();
            if let Some(last) = rows.last() {
                // The decoration is placed at the scrolled origin of the
                // list's content, so rows are offset back by the scroll.
                layer = layer.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(scroll_top + last.top + item_height)
                        .h(px(1.))
                        .bg(border),
                );
            }
            // Outermost last, so it paints and takes the mouse over a deeper
            // row pushed up beneath it.
            for (slot, row) in rows.iter().enumerate().rev() {
                let content = host.render_sticky_row(row.ix, slot, window, cx);
                layer = layer.child(
                    div()
                        .id(("sticky-row", slot))
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(scroll_top + row.top)
                        .h(item_height)
                        .bg(background)
                        .block_mouse_except_scroll()
                        .child(content),
                );
            }
            layer.into_any_element()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEIGHT: Pixels = px(10.);

    /// connection > database > Tables > t0..t19, then a second database.
    fn depths() -> Vec<usize> {
        let mut depths = vec![0, 1, 2];
        depths.extend(std::iter::repeat_n(3, 20));
        depths.extend([1, 2]);
        depths
    }

    fn pinned(scroll_top: f32) -> Vec<(usize, f32)> {
        let depths = depths();
        sticky_rows(|ix| depths[ix], depths.len(), px(scroll_top), HEIGHT, 5)
            .into_iter()
            .map(|row| (row.ix, f32::from(row.top)))
            .collect()
    }

    #[test]
    fn nothing_is_pinned_at_the_top() {
        assert_eq!(pinned(0.), vec![]);
    }

    #[test]
    fn folders_scrolled_past_are_pinned() {
        // Half a row down, the open folders at the top start sliding under
        // their slots, so all three are pinned.
        assert_eq!(pinned(5.), vec![(0, 0.), (1, 10.), (2, 20.)]);
        // Deep in the tables, every folder above them is pinned.
        assert_eq!(pinned(100.), vec![(0, 0.), (1, 10.), (2, 20.)]);
    }

    #[test]
    fn the_next_folder_pushes_a_pinned_one_out() {
        // The second database is row 23. With the list 205 down, its row's
        // top is at 25, so Tables (slot 2) moves up from 20 to 15.
        assert_eq!(pinned(200.), vec![(0, 0.), (1, 10.), (2, 20.)]);
        assert_eq!(pinned(205.), vec![(0, 0.), (1, 10.), (2, 15.)]);
        // Once the second database reaches slot 1, it is pinned instead.
        assert_eq!(pinned(225.), vec![(0, 0.), (23, 10.)]);
    }

    #[test]
    fn leaves_and_closed_folders_are_not_pinned() {
        let depths = [0, 0, 0, 0];
        assert_eq!(
            sticky_rows(|ix| depths[ix], depths.len(), px(15.), HEIGHT, 5),
            vec![]
        );
    }
}
