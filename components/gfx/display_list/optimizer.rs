/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use display_list::{DisplayItem, DisplayList};

use collections::dlist::DList;
use geom::rect::Rect;
use servo_util::geometry::Au;
use sync::Arc;

pub struct DisplayListOptimizer {
    display_list: Arc<DisplayList>,
    /// The visible rect in page coordinates.
    visible_rect: Rect<Au>,
}

impl DisplayListOptimizer {
    /// `visible_rect` specifies the visible rect in page coordinates.
    pub fn new(display_list: Arc<DisplayList>, visible_rect: Rect<Au>) -> DisplayListOptimizer {
        DisplayListOptimizer {
            display_list: display_list,
            visible_rect: visible_rect,
        }
    }

    pub fn optimize(self) -> DisplayList {
        self.process_display_list(&*self.display_list)
    }

    fn process_display_list(&self, display_list: &DisplayList) -> DisplayList {
        let mut result = DList::new();
        for item in display_list.iter() {
            match self.process_display_item(item) {
                None => {}
                Some(display_item) => result.push(display_item),
            }
        }
        DisplayList {
            list: result,
        }
    }

    fn process_display_item(&self, display_item: &DisplayItem) -> Option<DisplayItem> {
        // Eliminate display items outside the visible region.
        if !self.visible_rect.intersects(&display_item.base().bounds) ||
                !self.visible_rect.intersects(&display_item.base().clip_rect) {
            None
        } else {
            Some((*display_item).clone())
        }
    }
}

