/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(rustdoc::private_intra_doc_links)]

//! # HTML Tables (╯°□°)╯︵ ┻━┻
//!
//! This implementation is based on the [table section of the HTML 5 Specification][1],
//! the draft [CSS Table Module Level! 3][2] and the [LayoutNG implementation of tables][3] in Blink.
//! In general, the draft specification differs greatly from what other browsers do, so we
//! generally follow LayoutNG when in question.
//!
//! [1]: https://html.spec.whatwg.org/multipage/#tables
//! [2]: https://drafts.csswg.org/css-tables
//! [3]: https://source.chromium.org/chromium/chromium/src/third_party/+/main:blink/renderer/core/layout/table
//!
//! Table layout is divided into two phases:
//!
//! 1. Box Tree Construction
//! 2. Fragment Tree Construction
//!
//! ## Box Tree Construction
//!
//! During box tree construction, table layout (`construct.rs`) will traverse the DOM and construct
//! the basic box tree representation of a table, using the structs defined in this file ([`Table`],
//! [`TableTrackGroup`], [`TableTrack`], etc). When processing the DOM, elements are handled
//! differently depending on their `display` value. For instance, an element with `display:
//! table-cell` is treated as a table cell. HTML table elements like `<table>` and `<td>` are
//! assigned the corresponding display value from the user agent stylesheet.
//!
//! Every [`Table`] holds an array of [`TableSlot`]. A [`TableSlot`] represents either a cell, a cell
//! location occupied by a cell originating from another place in the table, or is empty. In
//! addition, when there are table model errors, a slot may spanned by more than one cell.
//!
//! During processing, the box tree construction agorithm will also fix up table structure, for
//! instance, creating anonymous rows for lone table cells and putting non-table content into
//! anonymous cells. In addition, flow layout will collect table elements outside of tables and create
//! anonymous tables for them.
//!
//! After processing, box tree construction does a fix up pass on tables, converting rowspan=0 into
//! definite rowspan values and making sure that rowspan and celspan values are not larger than the
//! table itself. Finally, row groups may be reordered to enforce the fact that the first `<thead>`
//! comes before `<tbody>` which comes before the first `<tfoot>`.
//!
//! ## Fragment Tree Construction
//!
//! Fragment tree construction involves calculating the size and positioning of all table elements,
//! given their style, content, and cell and row spans. This happens both during intrinsic inline
//! size computation as well as layout into Fragments. In both of these cases, measurement and
//! layout is done by [`layout::TableLayout`], though for intrinsic size computation only a partial
//! layout is done.
//!
//! In general, we follow the following steps when laying out table content:
//!
//! 1. Compute track constrainedness and has originating cells
//! 2. Compute cell measures
//! 3. Compute column measures
//! 4. Compute intrinsic inline sizes for columns and the table
//! 5. Compute the final table inline size
//! 6. Distribute size to columns
//! 7. Do first pass cell layout
//! 8. Do row layout
//! 9. Compute table height and final row sizes
//! 10. Create fragments for table elements (columns, column groups, rows, row groups, cells)
//!
//! For intrinsic size computation this process stops at step 4.

mod construct;
mod layout;

use std::ops::Range;

pub(crate) use construct::AnonymousTableContent;
pub use construct::TableBuilder;
use euclid::{Point2D, Size2D, UnknownUnit, Vector2D};
use serde::Serialize;
use servo_arc::Arc;
use style::properties::style_structs::Font;
use style::properties::ComputedValues;
use style_traits::dom::OpaqueNode;

use super::flow::BlockFormattingContext;
use crate::cell::ArcRefCell;
use crate::flow::BlockContainer;
use crate::formatting_contexts::NonReplacedFormattingContext;
use crate::fragment_tree::BaseFragmentInfo;

pub type TableSize = Size2D<usize, UnknownUnit>;

#[derive(Debug, Serialize)]
pub struct Table {
    /// The style of this table. These are the properties that apply to the "wrapper" ie the element
    /// that contains both the grid and the captions. Not all properties are actually used on the
    /// wrapper though, such as background and borders, which apply to the grid.
    #[serde(skip_serializing)]
    style: Arc<ComputedValues>,

    /// The style of this table's grid. This is an anonymous style based on the table's style, but
    /// eliminating all the properties handled by the "wrapper."
    #[serde(skip_serializing)]
    grid_style: Arc<ComputedValues>,

    /// The [`BaseFragmentInfo`] for this table's grid. This is necessary so that when the
    /// grid has a background image, it can be associated with the table's node.
    grid_base_fragment_info: BaseFragmentInfo,

    /// The captions for this table.
    pub captions: Vec<TableCaption>,

    /// The column groups for this table.
    pub column_groups: Vec<TableTrackGroup>,

    /// The columns of this table defined by `<colgroup> | display: table-column-group`
    /// and `<col> | display: table-column` elements as well as `display: table-column`.
    pub columns: Vec<TableTrack>,

    /// The rows groups for this table defined by `<tbody>`, `<thead>`, and `<tfoot>`.
    pub row_groups: Vec<TableTrackGroup>,

    /// The rows of this table defined by `<tr>` or `display: table-row` elements.
    pub rows: Vec<TableTrack>,

    /// The content of the slots of this table.
    pub slots: Vec<Vec<TableSlot>>,

    /// The size of this table.
    pub size: TableSize,

    /// Whether or not this Table is anonymous.
    anonymous: bool,
}

impl Table {
    pub(crate) fn new(
        style: Arc<ComputedValues>,
        grid_style: Arc<ComputedValues>,
        base_fragment_info: BaseFragmentInfo,
    ) -> Self {
        Self {
            style,
            grid_style,
            grid_base_fragment_info: base_fragment_info,
            captions: Vec::new(),
            column_groups: Vec::new(),
            columns: Vec::new(),
            row_groups: Vec::new(),
            rows: Vec::new(),
            slots: Vec::new(),
            size: TableSize::zero(),
            anonymous: false,
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
            style: ComputedValues::initial_values_with_font_override(Font::initial_values())
                .to_arc(),
            base_fragment_info: BaseFragmentInfo::new_for_node(OpaqueNode(id)),
        }
    }

    /// Get the node id of this cell's [`BaseFragmentInfo`]. This is used for unit tests.
    pub fn node_id(&self) -> usize {
        self.base_fragment_info.tag.map_or(0, |tag| tag.node.0)
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

/// A row or column of a table.
#[derive(Clone, Debug, Serialize)]
pub struct TableTrack {
    /// The [`BaseFragmentInfo`] of this cell.
    base_fragment_info: BaseFragmentInfo,

    /// The style of this table column.
    #[serde(skip_serializing)]
    style: Arc<ComputedValues>,

    /// The index of the table row or column group parent in the table's list of row or column
    /// groups.
    group_index: Option<usize>,

    /// Whether or not this [`TableTrack`] was anonymous, for instance created due to
    /// a `span` attribute set on a parent `<colgroup>`.
    is_anonymous: bool,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum TableTrackGroupType {
    HeaderGroup,
    FooterGroup,
    RowGroup,
    ColumnGroup,
}

#[derive(Debug, Serialize)]
pub struct TableTrackGroup {
    /// The [`BaseFragmentInfo`] of this [`TableTrackGroup`].
    base_fragment_info: BaseFragmentInfo,

    /// The style of this [`TableTrackGroup`].
    #[serde(skip_serializing)]
    style: Arc<ComputedValues>,

    /// The type of this [`TableTrackGroup`].
    group_type: TableTrackGroupType,

    /// The range of tracks in this [`TableTrackGroup`].
    track_range: Range<usize>,
}

impl TableTrackGroup {
    pub(super) fn is_empty(&self) -> bool {
        self.track_range.is_empty()
    }
}

#[derive(Debug, Serialize)]
pub struct TableCaption {
    /// The contents of this cell, with its own layout.
    context: ArcRefCell<NonReplacedFormattingContext>,
}
