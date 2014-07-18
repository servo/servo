/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_block)]

use context::LayoutContext;
use flow::{BaseFlow, TableColGroupFlowClass, FlowClass, Flow};
use fragment::{Fragment, TableColumnFragment};
use model::{MaybeAuto};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use std::fmt;

/// A table formatting context.
pub struct TableColGroupFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// The associated fragment.
    pub fragment: Option<Fragment>,

    /// The table column fragments
    pub cols: Vec<Fragment>,

    /// The specified inline-sizes of table columns
    pub inline_sizes: Vec<Au>,
}

impl TableColGroupFlow {
    pub fn from_node_and_fragments(node: &ThreadSafeLayoutNode,
                                   fragment: Fragment,
                                   fragments: Vec<Fragment>) -> TableColGroupFlow {
        TableColGroupFlow {
            base: BaseFlow::new((*node).clone()),
            fragment: Some(fragment),
            cols: fragments,
            inline_sizes: vec!(),
        }
    }
}

impl Flow for TableColGroupFlow {
    fn class(&self) -> FlowClass {
        TableColGroupFlowClass
    }

    fn as_table_colgroup<'a>(&'a mut self) -> &'a mut TableColGroupFlow {
        self
    }

    fn bubble_inline_sizes(&mut self, _: &mut LayoutContext) {
        for fragment in self.cols.iter() {
            // get the specified value from inline-size property
            let inline_size = MaybeAuto::from_style(fragment.style().content_inline_size(),
                                              Au::new(0)).specified_or_zero();

            let span: int = match fragment.specific {
                TableColumnFragment(col_fragment) => col_fragment.span.unwrap_or(1),
                _ => fail!("Other fragment come out in TableColGroupFlow. {:?}", fragment.specific)
            };
            for _ in range(0, span) {
                self.inline_sizes.push(inline_size);
            }
        }
    }

    /// Table column inline-sizes are assigned in table flow and propagated to table row or rowgroup flow.
    /// Therefore, table colgroup flow does not need to assign its inline-size.
    fn assign_inline_sizes(&mut self, _ctx: &mut LayoutContext) {
    }

    /// Table column do not have block-size.
    fn assign_block_size(&mut self, _ctx: &mut LayoutContext) {
    }
}

impl fmt::Show for TableColGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.fragment {
            Some(ref rb) => write!(f, "TableColGroupFlow: {}", rb),
            None => write!(f, "TableColGroupFlow"),
        }
    }
}
