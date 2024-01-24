/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! HTML Tables (╯°□°)╯︵ ┻━┻.
//!
//! See <https://html.spec.whatwg.org/multipage/table-processing-model> and
//! <https://drafts.csswg.org/css-tables>.  This is heavily based on the latter specification, but
//! note that it is still an Editor's Draft, so there is no guarantee that what is implemented here
//! matches other browsers or the current specification.

mod construct;
mod layout;

pub(crate) use construct::AnonymousTableContent;
pub use construct::TableBuilder;
use euclid::{Point2D, Size2D, UnknownUnit, Vector2D};
use serde::Serialize;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style_traits::dom::OpaqueNode;

use super::flow::BlockFormattingContext;
use crate::flow::BlockContainer;
use crate::fragment_tree::BaseFragmentInfo;

pub type TableSize = Size2D<usize, UnknownUnit>;

#[derive(Debug, Serialize)]
pub struct Table {
    /// The style of this table.
    #[serde(skip_serializing)]
    style: Arc<ComputedValues>,

    /// The content of the slots of this table.
    pub slots: Vec<Vec<TableSlot>>,

    /// The size of this table.
    pub size: TableSize,
}

impl Table {
    pub(crate) fn new(style: Arc<ComputedValues>) -> Self {
        Self {
            style,
            slots: Vec::new(),
            size: TableSize::zero(),
        }
    }

    /// Return the slot at the given coordinates, if it exists in the table, otherwise
    /// return None.
    fn get_slot(&self, coords: TableSlotCoordinates) -> Option<&TableSlot> {
        self.slots.get(coords.y)?.get(coords.x)
    }

    fn resolve_first_cell_coords(
        &self,
        coords: TableSlotCoordinates,
    ) -> Option<TableSlotCoordinates> {
        match self.get_slot(coords) {
            Some(&TableSlot::Cell(_)) => Some(coords),
            Some(TableSlot::Spanned(offsets)) => Some(coords - offsets[0]),
            _ => None,
        }
    }

    fn resolve_first_cell(&self, coords: TableSlotCoordinates) -> Option<&TableSlotCell> {
        let resolved_coords = match self.resolve_first_cell_coords(coords) {
            Some(coords) => coords,
            None => return None,
        };

        let slot = self.get_slot(resolved_coords);
        match slot {
            Some(TableSlot::Cell(cell)) => Some(cell),
            _ => unreachable!(
                "Spanned slot should not point to an empty cell or another spanned slot."
            ),
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

    /// The style of this table cell.
    #[serde(skip_serializing)]
    style: Arc<ComputedValues>,

    /// The [`BaseFragmentInfo`] of this cell.
    base_fragment_info: BaseFragmentInfo,
}

impl TableSlotCell {
    pub fn mock_for_testing(id: usize, colspan: usize, rowspan: usize) -> Self {
        Self {
            contents: BlockFormattingContext {
                contents: BlockContainer::BlockLevelBoxes(Vec::new()),
                contains_floats: false,
            },
            colspan,
            rowspan,
            style: ComputedValues::initial_values().to_arc(),
            base_fragment_info: BaseFragmentInfo::new_for_node(OpaqueNode(id)),
        }
    }

    /// Get the node id of this cell's [`BaseFragmentInfo`]. This is used for unit tests.
    pub fn node_id(&self) -> usize {
        self.base_fragment_info.tag.node.0
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
    /// <https://html.spec.whatwg.org/multipage/#table-model-error>
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
