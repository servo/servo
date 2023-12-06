/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Table layout.
//! See https://html.spec.whatwg.org/multipage/table-processing-model.

mod construct;

pub use construct::TableBuilder;
use euclid::{Point2D, UnknownUnit, Vector2D};
use serde::Serialize;
use style::values::computed::Length;

use super::flow::BlockFormattingContext;
use crate::context::LayoutContext;
use crate::flow::BlockContainer;
use crate::formatting_contexts::IndependentLayout;
use crate::positioned::PositioningContext;
use crate::sizing::ContentSizes;
use crate::ContainingBlock;

#[derive(Debug, Default, Serialize)]
pub struct Table {
    pub slots: Vec<Vec<TableSlot>>,
}

impl Table {
    pub(crate) fn inline_content_sizes(&self) -> ContentSizes {
        ContentSizes::zero()
    }

    pub(crate) fn layout(
        &self,
        _layout_context: &LayoutContext,
        _positioning_context: &mut PositioningContext,
        _containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        IndependentLayout {
            fragments: Vec::new(),
            content_block_size: Length::new(0.),
        }
    }
}

type TableSlotCoordinates = Point2D<usize, UnknownUnit>;
pub type TableSlotOffset = Vector2D<usize, UnknownUnit>;

#[derive(Debug, Serialize)]
pub struct TableSlotCell {
    /// The contents of this cell, with its own layout.
    contents: BlockFormattingContext,

    /// Number of columns that the cell is to span. Must be greater than zero.
    colspan: usize,

    /// Number of rows that the cell is to span. Zero means that the cell is to span all
    /// the remaining rows in the row group.
    rowspan: usize,

    // An id used for testing purposes.
    pub id: u8,
}

impl TableSlotCell {
    pub fn mock_for_testing(id: u8, colspan: usize, rowspan: usize) -> Self {
        Self {
            contents: BlockFormattingContext {
                contents: BlockContainer::BlockLevelBoxes(Vec::new()),
                contains_floats: false,
            },
            colspan,
            rowspan,
            id,
        }
    }
}

#[derive(Serialize)]
/// A single table slot. It may be an actual cell, or a reference
/// to a previous cell that is spanned here
///
/// In case of table model errors, it may be multiple references
pub enum TableSlot {
    /// A table cell, with a colspan and a rowspan.
    Cell(TableSlotCell),

    /// This slot is spanned by one or more multiple cells earlier in the table, which are
    /// found at the given negative coordinate offsets. The vector is in the order of most
    /// recent to earliest cell.
    ///
    /// If there is more than one cell that spans a slot, this is a table model error, but
    /// we still keep track of it. See
    /// https://html.spec.whatwg.org/multipage/#table-model-error
    Spanned(Vec<TableSlotOffset>),

    /// An empty spot in the table. This can happen when there is a gap in columns between
    /// cells that are defined and one which should exist because of cell with a rowspan
    /// from a previous row.
    Empty,
}

impl std::fmt::Debug for TableSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cell(_) => f.debug_tuple("Cell").finish(),
            Self::Spanned(spanned) => f.debug_tuple("Spanned").field(spanned).finish(),
            Self::Empty => write!(f, "Empty"),
        }
    }
}

impl TableSlot {
    fn new_spanned(offset: TableSlotOffset) -> Self {
        Self::Spanned(vec![offset])
    }
}
