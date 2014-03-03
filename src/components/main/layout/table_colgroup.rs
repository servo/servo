/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::{Box, TableColumnBox};
use layout::context::LayoutContext;
use layout::flow::{BaseFlow, TableColGroupFlowClass, FlowClass, Flow};
use layout::model::{MaybeAuto};
use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableColGroupFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Option<Box>,

    /// The table column boxes
    cols: ~[Box],

    /// The specified widths of table columns
    widths: ~[Au],
}

impl TableColGroupFlow {
    pub fn new(base: BaseFlow) -> TableColGroupFlow {
        TableColGroupFlow {
            base: base,
            box_: None,
            cols: ~[],
            widths: ~[],
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box, boxes: ~[Box]) -> TableColGroupFlow {
        TableColGroupFlow {
            base: base,
            box_: Some(box_),
            cols: boxes,
            widths: ~[],
        }
    }

    pub fn teardown(&mut self) {
        for box_ in self.box_.iter() {
            box_.teardown();
        }
        self.box_ = None;
        self.cols = ~[];
        self.widths = ~[];
    }
}

impl Flow for TableColGroupFlow {
    fn class(&self) -> FlowClass {
        TableColGroupFlowClass
    }

    fn as_table_colgroup<'a>(&'a mut self) -> &'a mut TableColGroupFlow {
        self
    }

    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        for box_ in self.cols.iter() {
            // get the specified value from width property
            let width = MaybeAuto::from_style(box_.style().Box.width, Au::new(0)).specified_or_zero();

            let span: int = match box_.specific {
                TableColumnBox(col_box) => col_box.span.unwrap_or(1),
                _ => fail!("Other box come out in TableColGroupFlow. {:?}", box_.specific)
            };
            for _ in range(0, span) {
                self.widths.push(width);
            }
        }
    }

    /// Table column widths are assigned in table flow and propagated to table row or rowgroup flow.
    /// Therefore, table colgroup flow does not need to assign its width.
    fn assign_widths(&mut self, _ctx: &mut LayoutContext) {
    }

    /// Table column do not have height.
    fn assign_height(&mut self, _ctx: &mut LayoutContext) {
    }

    /// TableColumnBox and their parents(TableBox) do not have margins.
    /// Therefore, margins to be collapsed do not exist.
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
