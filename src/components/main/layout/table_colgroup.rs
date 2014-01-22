/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::context::LayoutContext;
use layout::flow::{BaseFlow, TableColGroupFlowClass, FlowClass, Flow};
use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableColGroupFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Option<Box>,

    /// The table column boxes
    cols: ~[Box],
}

impl TableColGroupFlow {
    pub fn new(base: BaseFlow) -> TableColGroupFlow {
        TableColGroupFlow {
            base: base,
            box_: None,
            cols: ~[],
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box, boxes: ~[Box]) -> TableColGroupFlow {
        TableColGroupFlow {
            base: base,
            box_: Some(box_),
            cols: boxes,
        }
    }

    pub fn teardown(&mut self) {
        for box_ in self.box_.iter() {
            box_.teardown();
        }
        self.box_ = None;
        self.cols = ~[];
    }
}

impl Flow for TableColGroupFlow {
    fn class(&self) -> FlowClass {
        TableColGroupFlowClass
    }

    fn as_table_colgroup<'a>(&'a mut self) -> &'a mut TableColGroupFlow {
        self
    }

    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    /* TODO: absolute contexts */
    /* TODO: inline-blocks */
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    ///
    /// Dual boxes consume some width first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_widths(&mut self, _ctx: &mut LayoutContext) {
    }

    fn assign_height(&mut self, _ctx: &mut LayoutContext) {
    }

    fn collapse_margins(&mut self, _: bool, _: &mut bool, _: &mut Au,
                        _: &mut Au, _: &mut Au, _: &mut Au) {
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableColGroupFlow: ";
        txt.append(match self.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}
