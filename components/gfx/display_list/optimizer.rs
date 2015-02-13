/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Transforms a display list to produce a visually-equivalent, but cheaper-to-paint, one.

use display_list::{DisplayItem, DisplayList, StackingContext};

use collections::dlist::DList;
use geom::rect::Rect;
use util::geometry::{self, Au};
use std::sync::Arc;

/// Transforms a display list to produce a visually-equivalent, but cheaper-to-paint, one.
pub struct DisplayListOptimizer {
    /// The visible rect in page coordinates.
    visible_rect: Rect<Au>,
}

impl DisplayListOptimizer {
    /// Creates a new display list optimizer object. `visible_rect` specifies the visible rect in
    /// page coordinates.
    pub fn new(visible_rect: &Rect<f32>) -> DisplayListOptimizer {
        DisplayListOptimizer {
            visible_rect: geometry::f32_rect_to_au_rect(*visible_rect),
        }
    }

    /// Optimizes the given display list, returning an equivalent, but cheaper-to-paint, one.
    pub fn optimize(self, display_list: &DisplayList) -> DisplayList {
        let mut result = DisplayList::new();
        self.add_in_bounds_display_items(&mut result.background_and_borders,
                                         display_list.background_and_borders.iter());
        self.add_in_bounds_display_items(&mut result.block_backgrounds_and_borders,
                                         display_list.block_backgrounds_and_borders.iter());
        self.add_in_bounds_display_items(&mut result.floats, display_list.floats.iter());
        self.add_in_bounds_display_items(&mut result.content, display_list.content.iter());
        self.add_in_bounds_display_items(&mut result.outlines, display_list.outlines.iter());
        self.add_in_bounds_stacking_contexts(&mut result.children, display_list.children.iter());
        result
    }

    /// Adds display items that intersect the visible rect to `result_list`.
    fn add_in_bounds_display_items<'a,I>(&self,
                                         result_list: &mut DList<DisplayItem>,
                                         display_items: I)
                                         where I: Iterator<Item=&'a DisplayItem> {
        for display_item in display_items {
            if self.visible_rect.intersects(&display_item.base().bounds) &&
                    display_item.base().clip.might_intersect_rect(&self.visible_rect) {
                result_list.push_back((*display_item).clone())
            }
        }
    }

    /// Adds child stacking contexts whose boundaries intersect the visible rect to `result_list`.
    fn add_in_bounds_stacking_contexts<'a,I>(&self,
                                             result_list: &mut DList<Arc<StackingContext>>,
                                             stacking_contexts: I)
                                             where I: Iterator<Item=&'a Arc<StackingContext>> {
        for stacking_context in stacking_contexts {
            let overflow = stacking_context.overflow.translate(&stacking_context.bounds.origin);
            if self.visible_rect.intersects(&overflow) {
                result_list.push_back((*stacking_context).clone())
            }
        }
    }
}
