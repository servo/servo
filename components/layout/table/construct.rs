/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::iter::repeat;

use atomic_refcell::AtomicRef;
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use log::warn;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::properties::style_structs::Font;
use style::selector_parser::PseudoElement;
use style::str::char_is_whitespace;

use super::{
    Table, TableCaption, TableLevelBox, TableSlot, TableSlotCell, TableSlotCoordinates,
    TableSlotOffset, TableTrack, TableTrackGroup, TableTrackGroupType,
};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::{BoxSlot, LayoutBox};
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NonReplacedContents, TraversalHandler};
use crate::flow::{BlockContainerBuilder, BlockFormattingContext};
use crate::formatting_contexts::{
    IndependentFormattingContext, IndependentFormattingContextContents,
    IndependentNonReplacedContents,
};
use crate::fragment_tree::BaseFragmentInfo;
use crate::layout_box_base::LayoutBoxBase;
use crate::style_ext::{DisplayGeneratingBox, DisplayLayoutInternal};
use crate::{PropagatedBoxTreeData, SharedStyle};

/// A reference to a slot and its coordinates in the table
#[derive(Debug)]
pub(super) struct ResolvedSlotAndLocation<'a> {
    pub cell: AtomicRef<'a, TableSlotCell>,
    pub coords: TableSlotCoordinates,
}

impl ResolvedSlotAndLocation<'_> {
    fn covers_cell_at(&self, coords: TableSlotCoordinates) -> bool {
        let covered_in_x =
            coords.x >= self.coords.x && coords.x < self.coords.x + self.cell.colspan;
        let covered_in_y = coords.y >= self.coords.y &&
            (self.cell.rowspan == 0 || coords.y < self.coords.y + self.cell.rowspan);
        covered_in_x && covered_in_y
    }
}

pub(crate) enum AnonymousTableContent<'dom> {
    Text(NodeAndStyleInfo<'dom>, Cow<'dom, str>),
    Element {
        info: NodeAndStyleInfo<'dom>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    },
}

impl AnonymousTableContent<'_> {
    fn is_whitespace_only(&self) -> bool {
        match self {
            Self::Element { .. } => false,
            Self::Text(_, text) => text.chars().all(char_is_whitespace),
        }
    }

    fn contents_are_whitespace_only(contents: &[Self]) -> bool {
        contents.iter().all(|content| content.is_whitespace_only())
    }
}

impl Table {
    pub(crate) fn construct(
        context: &LayoutContext,
        info: &NodeAndStyleInfo,
        grid_style: Arc<ComputedValues>,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        let mut traversal = TableBuilderTraversal::new(context, info, grid_style, propagated_data);
        contents.traverse(context, info, &mut traversal);
        traversal.finish()
    }

    pub(crate) fn construct_anonymous<'dom>(
        context: &LayoutContext,
        parent_info: &NodeAndStyleInfo<'dom>,
        contents: Vec<AnonymousTableContent<'dom>>,
        propagated_data: PropagatedBoxTreeData,
    ) -> (NodeAndStyleInfo<'dom>, IndependentFormattingContext) {
        let table_info = parent_info
            .pseudo(context, PseudoElement::ServoAnonymousTable)
            .expect("Should never fail to create anonymous table info.");
        let table_style = table_info.style.clone();
        let mut table_builder =
            TableBuilderTraversal::new(context, &table_info, table_style.clone(), propagated_data);

        for content in contents {
            match content {
                AnonymousTableContent::Element {
                    info,
                    display,
                    contents,
                    box_slot,
                } => {
                    table_builder.handle_element(&info, display, contents, box_slot);
                },
                AnonymousTableContent::Text(..) => {
                    // This only happens if there was whitespace between our internal table elements.
                    // We only collect that whitespace in case we need to re-emit trailing whitespace
                    // after we've added our anonymous table.
                },
            }
        }

        let mut table = table_builder.finish();
        table.anonymous = true;

        let ifc = IndependentFormattingContext {
            base: LayoutBoxBase::new((&table_info).into(), table_style),
            contents: IndependentFormattingContextContents::NonReplaced(
                IndependentNonReplacedContents::Table(table),
            ),
        };

        (table_info, ifc)
    }

    /// Push a new slot into the last row of this table.
    fn push_new_slot_to_last_row(&mut self, slot: TableSlot) {
        let last_row = match self.slots.last_mut() {
            Some(row) => row,
            None => {
                unreachable!("Should have some rows before calling `push_new_slot_to_last_row`")
            },
        };

        self.size.width = self.size.width.max(last_row.len() + 1);
        last_row.push(slot);
    }

    /// Find [`ResolvedSlotAndLocation`] of all the slots that cover the slot at the given
    /// coordinates. This recursively resolves all of the [`TableSlotCell`]s that cover
    /// the target and returns a [`ResolvedSlotAndLocation`] for each of them. If there is
    /// no slot at the given coordinates or that slot is an empty space, an empty vector
    /// is returned.
    pub(super) fn resolve_slot_at(
        &self,
        coords: TableSlotCoordinates,
    ) -> Vec<ResolvedSlotAndLocation<'_>> {
        let slot = self.get_slot(coords);
        match slot {
            Some(TableSlot::Cell(cell)) => vec![ResolvedSlotAndLocation {
                cell: cell.borrow(),
                coords,
            }],
            Some(TableSlot::Spanned(offsets)) => offsets
                .iter()
                .flat_map(|offset| self.resolve_slot_at(coords - *offset))
                .collect(),
            Some(TableSlot::Empty) | None => {
                warn!("Tried to resolve an empty or nonexistant slot!");
                vec![]
            },
        }
    }
}

impl TableSlot {
    /// Merge a TableSlot::Spanned(x, y) with this (only for model errors)
    pub fn push_spanned(&mut self, new_offset: TableSlotOffset) {
        match *self {
            TableSlot::Cell { .. } => {
                panic!(
                    "Should never have a table model error with an originating cell slot overlapping a spanned slot"
                )
            },
            TableSlot::Spanned(ref mut vec) => vec.insert(0, new_offset),
            TableSlot::Empty => {
                panic!("Should never have a table model error with an empty slot");
            },
        }
    }
}

pub struct TableBuilder {
    /// The table that we are building.
    table: Table,

    /// An incoming rowspan is a value indicating that a cell in a row above the current row,
    /// had a rowspan value other than 1. The values in this array indicate how many more
    /// rows the cell should span. For example, a value of 0 at an index before `current_x()`
    /// indicates that the cell on that column will not span into the next row, and at an index
    /// after `current_x()` it indicates that the cell will not span into the current row.
    /// A negative value means that the cell will span all remaining rows in the row group.
    ///
    /// As each column in a row is processed, the values in this vector are updated for the
    /// next row.
    pub incoming_rowspans: Vec<isize>,
}

impl TableBuilder {
    pub(super) fn new(
        style: Arc<ComputedValues>,
        grid_style: Arc<ComputedValues>,
        base_fragment_info: BaseFragmentInfo,
        percentage_columns_allowed_for_inline_content_sizes: bool,
    ) -> Self {
        Self {
            table: Table::new(
                style,
                grid_style,
                base_fragment_info,
                percentage_columns_allowed_for_inline_content_sizes,
            ),
            incoming_rowspans: Vec::new(),
        }
    }

    pub fn new_for_tests() -> Self {
        let testing_style =
            ComputedValues::initial_values_with_font_override(Font::initial_values());
        Self::new(
            testing_style.clone(),
            testing_style.clone(),
            BaseFragmentInfo::anonymous(),
            true, /* percentage_columns_allowed_for_inline_content_sizes */
        )
    }

    pub fn last_row_index_in_row_group_at_row_n(&self, n: usize) -> usize {
        // TODO: This is just a linear search, because the idea is that there are
        // generally less than or equal to three row groups, but if we notice a lot
        // of web content with more, we can consider a binary search here.
        for row_group in self.table.row_groups.iter() {
            let row_group = row_group.borrow();
            if row_group.track_range.start > n {
                return row_group.track_range.start - 1;
            }
        }
        self.table.size.height - 1
    }

    pub fn finish(mut self) -> Table {
        self.adjust_table_geometry_for_columns_and_colgroups();
        self.do_missing_cells_fixup();
        self.reorder_first_thead_and_tfoot();
        self.do_final_rowspan_calculation();
        self.table
    }

    /// Do <https://drafts.csswg.org/css-tables/#missing-cells-fixup> which ensures
    /// that every row has the same number of cells.
    fn do_missing_cells_fixup(&mut self) {
        for row in self.table.slots.iter_mut() {
            row.resize_with(self.table.size.width, || TableSlot::Empty);
        }
    }

    /// It's possible to define more table columns via `<colgroup>` and `<col>` elements
    /// than actually exist in the table. In that case, increase the size of the table.
    ///
    /// However, if the table has no row nor row group, remove the extra columns instead.
    /// This matches WebKit, and some tests require it, but Gecko and Blink don't do it.
    fn adjust_table_geometry_for_columns_and_colgroups(&mut self) {
        if self.table.rows.is_empty() && self.table.row_groups.is_empty() {
            self.table.columns.truncate(0);
            self.table.column_groups.truncate(0);
        } else {
            self.table.size.width = self.table.size.width.max(self.table.columns.len());
        }
    }

    /// Reorder the first `<thead>` and `<tbody>` to be the first and last row groups respectively.
    /// This requires fixing up all row group indices.
    /// See <https://drafts.csswg.org/css-tables/#table-header-group> and
    /// <https://drafts.csswg.org/css-tables/#table-footer-group>.
    fn reorder_first_thead_and_tfoot(&mut self) {
        let mut thead_index = None;
        let mut tfoot_index = None;
        for (row_group_index, row_group) in self.table.row_groups.iter().enumerate() {
            let row_group = row_group.borrow();
            if thead_index.is_none() && row_group.group_type == TableTrackGroupType::HeaderGroup {
                thead_index = Some(row_group_index);
            }
            if tfoot_index.is_none() && row_group.group_type == TableTrackGroupType::FooterGroup {
                tfoot_index = Some(row_group_index);
            }
            if thead_index.is_some() && tfoot_index.is_some() {
                break;
            }
        }

        if let Some(thead_index) = thead_index {
            self.move_row_group_to_front(thead_index)
        }

        if let Some(mut tfoot_index) = tfoot_index {
            // We may have moved a `<thead>` which means the original index we
            // we found for this this <tfoot>` also needs to be updated!
            if thead_index.unwrap_or(0) > tfoot_index {
                tfoot_index += 1;
            }
            self.move_row_group_to_end(tfoot_index)
        }
    }

    fn regenerate_track_ranges(&mut self) {
        // Now update all track group ranges.
        let mut current_row_group_index = None;
        for (row_index, row) in self.table.rows.iter().enumerate() {
            let row = row.borrow();
            if current_row_group_index == row.group_index {
                continue;
            }

            // Finish any row group that is currently being processed.
            if let Some(current_group_index) = current_row_group_index {
                self.table.row_groups[current_group_index]
                    .borrow_mut()
                    .track_range
                    .end = row_index;
            }

            // Start processing this new row group and update its starting index.
            current_row_group_index = row.group_index;
            if let Some(current_group_index) = current_row_group_index {
                self.table.row_groups[current_group_index]
                    .borrow_mut()
                    .track_range
                    .start = row_index;
            }
        }

        // Finish the last row group.
        if let Some(current_group_index) = current_row_group_index {
            self.table.row_groups[current_group_index]
                .borrow_mut()
                .track_range
                .end = self.table.rows.len();
        }
    }

    fn move_row_group_to_front(&mut self, index_to_move: usize) {
        // Move the group itself.
        if index_to_move > 0 {
            let removed_row_group = self.table.row_groups.remove(index_to_move);
            self.table.row_groups.insert(0, removed_row_group);

            for row in self.table.rows.iter_mut() {
                let mut row = row.borrow_mut();
                match row.group_index.as_mut() {
                    Some(group_index) if *group_index < index_to_move => *group_index += 1,
                    Some(group_index) if *group_index == index_to_move => *group_index = 0,
                    _ => {},
                }
            }
        }

        let row_range = self.table.row_groups[0].borrow().track_range.clone();
        if row_range.start > 0 {
            // Move the slots associated with the moved group.
            let removed_slots: Vec<Vec<TableSlot>> = self
                .table
                .slots
                .splice(row_range.clone(), std::iter::empty())
                .collect();
            self.table.slots.splice(0..0, removed_slots);

            // Move the rows associated with the moved group.
            let removed_rows: Vec<_> = self
                .table
                .rows
                .splice(row_range, std::iter::empty())
                .collect();
            self.table.rows.splice(0..0, removed_rows);

            // Do this now, rather than after possibly moving a `<tfoot>` row group to the end,
            // because moving row groups depends on an accurate `track_range` in every group.
            self.regenerate_track_ranges();
        }
    }

    fn move_row_group_to_end(&mut self, index_to_move: usize) {
        let last_row_group_index = self.table.row_groups.len() - 1;

        // Move the group itself.
        if index_to_move < last_row_group_index {
            let removed_row_group = self.table.row_groups.remove(index_to_move);
            self.table.row_groups.push(removed_row_group);

            for row in self.table.rows.iter_mut() {
                let mut row = row.borrow_mut();
                match row.group_index.as_mut() {
                    Some(group_index) if *group_index > index_to_move => *group_index -= 1,
                    Some(group_index) if *group_index == index_to_move => {
                        *group_index = last_row_group_index
                    },
                    _ => {},
                }
            }
        }

        let row_range = self.table.row_groups[last_row_group_index]
            .borrow()
            .track_range
            .clone();
        if row_range.end < self.table.rows.len() {
            // Move the slots associated with the moved group.
            let removed_slots: Vec<Vec<TableSlot>> = self
                .table
                .slots
                .splice(row_range.clone(), std::iter::empty())
                .collect();
            self.table.slots.extend(removed_slots);

            // Move the rows associated with the moved group.
            let removed_rows: Vec<_> = self
                .table
                .rows
                .splice(row_range, std::iter::empty())
                .collect();
            self.table.rows.extend(removed_rows);

            self.regenerate_track_ranges();
        }
    }

    /// Turn all rowspan=0 rows into the real value to avoid having to make the calculation
    /// continually during layout. In addition, make sure that there are no rowspans that extend
    /// past the end of their row group.
    fn do_final_rowspan_calculation(&mut self) {
        for row_index in 0..self.table.size.height {
            let last_row_index_in_group = self.last_row_index_in_row_group_at_row_n(row_index);
            for cell in self.table.slots[row_index].iter_mut() {
                if let TableSlot::Cell(cell) = cell {
                    let mut cell = cell.borrow_mut();
                    if cell.rowspan == 1 {
                        continue;
                    }
                    let rowspan_to_end_of_group = last_row_index_in_group - row_index + 1;
                    if cell.rowspan == 0 {
                        cell.rowspan = rowspan_to_end_of_group;
                    } else {
                        cell.rowspan = cell.rowspan.min(rowspan_to_end_of_group);
                    }
                }
            }
        }
    }

    fn current_y(&self) -> Option<usize> {
        self.table.slots.len().checked_sub(1)
    }

    fn current_x(&self) -> Option<usize> {
        Some(self.table.slots[self.current_y()?].len())
    }

    fn current_coords(&self) -> Option<TableSlotCoordinates> {
        Some(TableSlotCoordinates::new(
            self.current_x()?,
            self.current_y()?,
        ))
    }

    pub fn start_row(&mut self) {
        self.table.slots.push(Vec::new());
        self.table.size.height += 1;
        self.create_slots_for_cells_above_with_rowspan(true);
    }

    pub fn end_row(&mut self) {
        // TODO: We need to insert a cell for any leftover non-table-like
        // content in the TableRowBuilder.

        // Truncate entries that are zero at the end of [`Self::incoming_rowspans`]. This
        // prevents padding the table with empty cells when it isn't necessary.
        let current_x = self
            .current_x()
            .expect("Should have rows before calling `end_row`");
        for i in (current_x..self.incoming_rowspans.len()).rev() {
            if self.incoming_rowspans[i] == 0 {
                self.incoming_rowspans.pop();
            } else {
                break;
            }
        }

        self.create_slots_for_cells_above_with_rowspan(false);
    }

    /// Create a [`TableSlot::Spanned`] for the target cell at the given coordinates. If
    /// no slots cover the target, then this returns [`None`]. Note: This does not handle
    /// slots that cover the target using `colspan`, but instead only considers slots that
    /// cover this slot via `rowspan`. `colspan` should be handled by appending to the
    /// return value of this function.
    fn create_spanned_slot_based_on_cell_above(
        &self,
        target_coords: TableSlotCoordinates,
    ) -> Option<TableSlot> {
        let y_above = self.current_y()?.checked_sub(1)?;
        let coords_for_slot_above = TableSlotCoordinates::new(target_coords.x, y_above);
        let slots_covering_slot_above = self.table.resolve_slot_at(coords_for_slot_above);

        let coords_of_slots_that_cover_target: Vec<_> = slots_covering_slot_above
            .into_iter()
            .filter(|slot| slot.covers_cell_at(target_coords))
            .map(|slot| target_coords - slot.coords)
            .collect();

        if coords_of_slots_that_cover_target.is_empty() {
            None
        } else {
            Some(TableSlot::Spanned(coords_of_slots_that_cover_target))
        }
    }

    /// When not in the process of filling a cell, make sure any incoming rowspans are
    /// filled so that the next specified cell comes after them. Should have been called before
    /// [`Self::add_cell`]
    ///
    /// if `stop_at_cell_opportunity` is set, this will stop at the first slot with
    /// `incoming_rowspans` equal to zero. If not, it will insert [`TableSlot::Empty`] and
    /// continue to look for more incoming rowspans (which should only be done once we're
    /// finished processing the cells in a row, and after calling truncating cells with
    /// remaining rowspan from the end of `incoming_rowspans`.
    fn create_slots_for_cells_above_with_rowspan(&mut self, stop_at_cell_opportunity: bool) {
        let mut current_coords = self
            .current_coords()
            .expect("Should have rows before calling `create_slots_for_cells_above_with_rowspan`");
        while let Some(span) = self.incoming_rowspans.get_mut(current_coords.x) {
            // This column has no incoming rowspanned cells and `stop_at_zero` is true, so
            // we should stop to process new cells defined in the current row.
            if *span == 0 && stop_at_cell_opportunity {
                break;
            }

            let new_cell = if *span != 0 {
                *span -= 1;
                self.create_spanned_slot_based_on_cell_above(current_coords)
                    .expect(
                        "Nonzero incoming rowspan cannot occur without a cell spanning this slot",
                    )
            } else {
                TableSlot::Empty
            };

            self.table.push_new_slot_to_last_row(new_cell);
            current_coords.x += 1;
        }
        debug_assert_eq!(Some(current_coords), self.current_coords());
    }

    /// <https://html.spec.whatwg.org/multipage/#algorithm-for-processing-rows>
    /// Push a single cell onto the slot map, handling any colspans it may have, and
    /// setting up the outgoing rowspans.
    pub fn add_cell(&mut self, cell: ArcRefCell<TableSlotCell>) {
        // Make sure the incoming_rowspans table is large enough
        // because we will be writing to it.
        let current_coords = self
            .current_coords()
            .expect("Should have rows before calling `add_cell`");

        let (colspan, rowspan) = {
            let cell = cell.borrow();
            (cell.colspan, cell.rowspan)
        };

        if self.incoming_rowspans.len() < current_coords.x + colspan {
            self.incoming_rowspans
                .resize(current_coords.x + colspan, 0isize);
        }

        debug_assert_eq!(
            self.incoming_rowspans[current_coords.x], 0,
            "Added a cell in a position that also had an incoming rowspan!"
        );

        // If `rowspan` is zero, this is automatically negative and will stay negative.
        let outgoing_rowspan = rowspan as isize - 1;
        self.table.push_new_slot_to_last_row(TableSlot::Cell(cell));
        self.incoming_rowspans[current_coords.x] = outgoing_rowspan;

        // Draw colspanned cells
        for colspan_offset in 1..colspan {
            let current_x_plus_colspan_offset = current_coords.x + colspan_offset;
            let new_offset = TableSlotOffset::new(colspan_offset, 0);
            let incoming_rowspan = &mut self.incoming_rowspans[current_x_plus_colspan_offset];
            let new_slot = if *incoming_rowspan == 0 {
                *incoming_rowspan = outgoing_rowspan;
                TableSlot::new_spanned(new_offset)
            } else {
                // This means we have a table model error.

                // if `incoming_rowspan` is greater than zero, a cell from above is spanning
                // into our row, colliding with the cells we are creating via colspan. In
                // that case, set the incoming rowspan to the highest of two possible
                // outgoing rowspan values (the incoming rowspan minus one, OR this cell's
                // outgoing rowspan).  `spanned_slot()`` will handle filtering out
                // inapplicable spans when it needs to.
                //
                // If the `incoming_rowspan` is negative we are in `rowspan=0` mode, (i.e.
                // rowspan=infinity), so we don't have to worry about the current cell
                // making it larger. In that case, don't change the rowspan.
                if *incoming_rowspan > 0 {
                    *incoming_rowspan = std::cmp::max(*incoming_rowspan - 1, outgoing_rowspan);
                }

                // This code creates a new slot in the case that there is a table model error.
                let coords_of_spanned_cell =
                    TableSlotCoordinates::new(current_x_plus_colspan_offset, current_coords.y);
                let mut incoming_slot = self
                    .create_spanned_slot_based_on_cell_above(coords_of_spanned_cell)
                    .expect(
                        "Nonzero incoming rowspan cannot occur without a cell spanning this slot",
                    );
                incoming_slot.push_spanned(new_offset);
                incoming_slot
            };
            self.table.push_new_slot_to_last_row(new_slot);
        }

        debug_assert_eq!(
            Some(TableSlotCoordinates::new(
                current_coords.x + colspan,
                current_coords.y
            )),
            self.current_coords(),
            "Must have produced `colspan` slot entries!"
        );
        self.create_slots_for_cells_above_with_rowspan(true);
    }
}

pub(crate) struct TableBuilderTraversal<'style, 'dom> {
    context: &'style LayoutContext<'style>,
    info: &'style NodeAndStyleInfo<'dom>,

    /// The value of the [`PropagatedBoxTreeData`] to use, either for the row group
    /// if processing one or for the table itself if outside a row group.
    current_propagated_data: PropagatedBoxTreeData,

    /// The [`TableBuilder`] for this [`TableBuilderTraversal`]. This is separated
    /// into another struct so that we can write unit tests against the builder.
    builder: TableBuilder,

    current_anonymous_row_content: Vec<AnonymousTableContent<'dom>>,

    /// The index of the current row group, if there is one.
    current_row_group_index: Option<usize>,
}

impl<'style, 'dom> TableBuilderTraversal<'style, 'dom> {
    pub(crate) fn new(
        context: &'style LayoutContext<'style>,
        info: &'style NodeAndStyleInfo<'dom>,
        grid_style: Arc<ComputedValues>,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        TableBuilderTraversal {
            context,
            info,
            current_propagated_data: propagated_data,
            builder: TableBuilder::new(
                info.style.clone(),
                grid_style,
                info.into(),
                propagated_data.allow_percentage_column_in_tables,
            ),
            current_anonymous_row_content: Vec::new(),
            current_row_group_index: None,
        }
    }

    pub(crate) fn finish(mut self) -> Table {
        self.finish_anonymous_row_if_needed();
        self.builder.finish()
    }

    fn finish_anonymous_row_if_needed(&mut self) {
        if AnonymousTableContent::contents_are_whitespace_only(&self.current_anonymous_row_content)
        {
            self.current_anonymous_row_content.clear();
            return;
        }

        let row_content = std::mem::take(&mut self.current_anonymous_row_content);
        let anonymous_info = self
            .info
            .pseudo(self.context, PseudoElement::ServoAnonymousTableRow)
            .expect("Should never fail to create anonymous row info.");
        let mut row_builder =
            TableRowBuilder::new(self, &anonymous_info, self.current_propagated_data);

        for cell_content in row_content {
            match cell_content {
                AnonymousTableContent::Element {
                    info,
                    display,
                    contents,
                    box_slot,
                } => {
                    row_builder.handle_element(&info, display, contents, box_slot);
                },
                AnonymousTableContent::Text(info, text) => {
                    row_builder.handle_text(&info, text);
                },
            }
        }

        row_builder.finish();

        let style = anonymous_info.style.clone();
        self.push_table_row(ArcRefCell::new(TableTrack {
            base: LayoutBoxBase::new((&anonymous_info).into(), style.clone()),
            group_index: self.current_row_group_index,
            is_anonymous: true,
            shared_background_style: SharedStyle::new(style),
        }));
    }

    fn push_table_row(&mut self, table_track: ArcRefCell<TableTrack>) {
        self.builder.table.rows.push(table_track);

        let last_row = self.builder.table.rows.len();
        if let Some(index) = self.current_row_group_index {
            let row_group = &mut self.builder.table.row_groups[index];
            row_group.borrow_mut().track_range.end = last_row;
        }
    }
}

impl<'dom> TraversalHandler<'dom> for TableBuilderTraversal<'_, 'dom> {
    fn handle_text(&mut self, info: &NodeAndStyleInfo<'dom>, text: Cow<'dom, str>) {
        self.current_anonymous_row_content
            .push(AnonymousTableContent::Text(info.clone(), text));
    }

    /// <https://html.spec.whatwg.org/multipage/#forming-a-table>
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        match display {
            DisplayGeneratingBox::LayoutInternal(internal) => match internal {
                DisplayLayoutInternal::TableRowGroup |
                DisplayLayoutInternal::TableFooterGroup |
                DisplayLayoutInternal::TableHeaderGroup => {
                    self.finish_anonymous_row_if_needed();
                    self.builder.incoming_rowspans.clear();

                    let next_row_index = self.builder.table.rows.len();
                    let row_group = ArcRefCell::new(TableTrackGroup {
                        base: LayoutBoxBase::new(info.into(), info.style.clone()),
                        group_type: internal.into(),
                        track_range: next_row_index..next_row_index,
                        shared_background_style: SharedStyle::new(info.style.clone()),
                    });
                    self.builder.table.row_groups.push(row_group.clone());

                    let new_row_group_index = self.builder.table.row_groups.len() - 1;
                    self.current_row_group_index = Some(new_row_group_index);

                    let Contents::NonReplaced(non_replaced_contents) = contents else {
                        unreachable!("Replaced should not have a LayoutInternal display type.");
                    };
                    non_replaced_contents.traverse(self.context, info, self);
                    self.finish_anonymous_row_if_needed();

                    self.current_row_group_index = None;
                    self.builder.incoming_rowspans.clear();

                    box_slot.set(LayoutBox::TableLevelBox(TableLevelBox::TrackGroup(
                        row_group,
                    )));
                },
                DisplayLayoutInternal::TableRow => {
                    self.finish_anonymous_row_if_needed();

                    let context = self.context;
                    let mut row_builder =
                        TableRowBuilder::new(self, info, self.current_propagated_data);

                    let Contents::NonReplaced(non_replaced_contents) = contents else {
                        unreachable!("Replaced should not have a LayoutInternal display type.");
                    };
                    non_replaced_contents.traverse(context, info, &mut row_builder);
                    row_builder.finish();

                    let row = ArcRefCell::new(TableTrack {
                        base: LayoutBoxBase::new(info.into(), info.style.clone()),
                        group_index: self.current_row_group_index,
                        is_anonymous: false,
                        shared_background_style: SharedStyle::new(info.style.clone()),
                    });
                    self.push_table_row(row.clone());
                    box_slot.set(LayoutBox::TableLevelBox(TableLevelBox::Track(row)));
                },
                DisplayLayoutInternal::TableColumn => {
                    let column = add_column(
                        &mut self.builder.table.columns,
                        info,
                        None,  /* group_index */
                        false, /* is_anonymous */
                    );
                    box_slot.set(LayoutBox::TableLevelBox(TableLevelBox::Track(column)));
                },
                DisplayLayoutInternal::TableColumnGroup => {
                    let column_group_index = self.builder.table.column_groups.len();
                    let mut column_group_builder = TableColumnGroupBuilder {
                        column_group_index,
                        columns: Vec::new(),
                    };

                    let Contents::NonReplaced(non_replaced_contents) = contents else {
                        unreachable!("Replaced should not have a LayoutInternal display type.");
                    };
                    non_replaced_contents.traverse(self.context, info, &mut column_group_builder);

                    let first_column = self.builder.table.columns.len();
                    if column_group_builder.columns.is_empty() {
                        add_column(
                            &mut self.builder.table.columns,
                            info,
                            Some(column_group_index),
                            true, /* is_anonymous */
                        );
                    } else {
                        self.builder
                            .table
                            .columns
                            .extend(column_group_builder.columns);
                    }

                    let column_group = ArcRefCell::new(TableTrackGroup {
                        base: LayoutBoxBase::new(info.into(), info.style.clone()),
                        group_type: internal.into(),
                        track_range: first_column..self.builder.table.columns.len(),
                        shared_background_style: SharedStyle::new(info.style.clone()),
                    });
                    self.builder.table.column_groups.push(column_group.clone());
                    box_slot.set(LayoutBox::TableLevelBox(TableLevelBox::TrackGroup(
                        column_group,
                    )));
                },
                DisplayLayoutInternal::TableCaption => {
                    let Contents::NonReplaced(non_replaced_contents) = contents else {
                        unreachable!("Replaced should not have a LayoutInternal display type.");
                    };

                    let old_caption = (!info.damage.has_box_damage())
                        .then(|| match box_slot.take_layout_box() {
                            Some(LayoutBox::TableLevelBox(TableLevelBox::Caption(caption))) => {
                                Some(caption)
                            },
                            _ => None,
                        })
                        .flatten();

                    let caption = old_caption.unwrap_or_else(|| {
                        let contents = IndependentNonReplacedContents::Flow(
                            BlockFormattingContext::construct(
                                self.context,
                                info,
                                non_replaced_contents,
                                self.current_propagated_data,
                                false, /* is_list_item */
                            ),
                        );

                        ArcRefCell::new(TableCaption {
                            context: IndependentFormattingContext {
                                base: LayoutBoxBase::new(info.into(), info.style.clone()),
                                contents: IndependentFormattingContextContents::NonReplaced(
                                    contents,
                                ),
                            },
                        })
                    });

                    self.builder.table.captions.push(caption.clone());
                    box_slot.set(LayoutBox::TableLevelBox(TableLevelBox::Caption(caption)));
                },
                DisplayLayoutInternal::TableCell => {
                    self.current_anonymous_row_content
                        .push(AnonymousTableContent::Element {
                            info: info.clone(),
                            display,
                            contents,
                            box_slot,
                        });
                },
            },
            _ => {
                self.current_anonymous_row_content
                    .push(AnonymousTableContent::Element {
                        info: info.clone(),
                        display,
                        contents,
                        box_slot,
                    });
            },
        }
    }
}

struct TableRowBuilder<'style, 'builder, 'dom, 'a> {
    table_traversal: &'builder mut TableBuilderTraversal<'style, 'dom>,

    /// The [`NodeAndStyleInfo`] of this table row, which we use to
    /// construct anonymous table cells.
    info: &'a NodeAndStyleInfo<'dom>,

    current_anonymous_cell_content: Vec<AnonymousTableContent<'dom>>,

    /// The [`PropagatedBoxTreeData`] to use for all children of this row.
    propagated_data: PropagatedBoxTreeData,
}

impl<'style, 'builder, 'dom, 'a> TableRowBuilder<'style, 'builder, 'dom, 'a> {
    fn new(
        table_traversal: &'builder mut TableBuilderTraversal<'style, 'dom>,
        info: &'a NodeAndStyleInfo<'dom>,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        table_traversal.builder.start_row();

        TableRowBuilder {
            table_traversal,
            info,
            current_anonymous_cell_content: Vec::new(),
            propagated_data,
        }
    }

    fn finish(mut self) {
        self.finish_current_anonymous_cell_if_needed();
        self.table_traversal.builder.end_row();
    }

    fn finish_current_anonymous_cell_if_needed(&mut self) {
        if AnonymousTableContent::contents_are_whitespace_only(&self.current_anonymous_cell_content)
        {
            self.current_anonymous_cell_content.clear();
            return;
        }

        let context = self.table_traversal.context;
        let anonymous_info = self
            .info
            .pseudo(context, PseudoElement::ServoAnonymousTableCell)
            .expect("Should never fail to create anonymous table cell info");
        let propagated_data = self.propagated_data.disallowing_percentage_table_columns();
        let mut builder = BlockContainerBuilder::new(context, &anonymous_info, propagated_data);

        for cell_content in self.current_anonymous_cell_content.drain(..) {
            match cell_content {
                AnonymousTableContent::Element {
                    info,
                    display,
                    contents,
                    box_slot,
                } => {
                    builder.handle_element(&info, display, contents, box_slot);
                },
                AnonymousTableContent::Text(info, text) => {
                    builder.handle_text(&info, text);
                },
            }
        }

        let block_container = builder.finish();
        self.table_traversal
            .builder
            .add_cell(ArcRefCell::new(TableSlotCell {
                base: LayoutBoxBase::new(BaseFragmentInfo::anonymous(), anonymous_info.style),
                contents: BlockFormattingContext::from_block_container(block_container),
                colspan: 1,
                rowspan: 1,
            }));
    }
}

impl<'dom> TraversalHandler<'dom> for TableRowBuilder<'_, '_, 'dom, '_> {
    fn handle_text(&mut self, info: &NodeAndStyleInfo<'dom>, text: Cow<'dom, str>) {
        self.current_anonymous_cell_content
            .push(AnonymousTableContent::Text(info.clone(), text));
    }

    /// <https://html.spec.whatwg.org/multipage/#algorithm-for-processing-rows>
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        #[allow(clippy::collapsible_match)] //// TODO: Remove once the other cases are handled
        match display {
            DisplayGeneratingBox::LayoutInternal(internal) => match internal {
                DisplayLayoutInternal::TableCell => {
                    // This value will already have filtered out rowspan=0
                    // in quirks mode, so we don't have to worry about that.
                    let (rowspan, colspan) = if info.pseudo_element_type.is_none() {
                        let node = info.node.to_threadsafe();
                        let rowspan = node.get_rowspan().unwrap_or(1) as usize;
                        let colspan = node.get_colspan().unwrap_or(1) as usize;

                        // The HTML specification clamps value of `rowspan` to [0, 65534] and
                        // `colspan` to [1, 1000].
                        assert!((1..=1000).contains(&colspan));
                        assert!((0..=65534).contains(&rowspan));

                        (rowspan, colspan)
                    } else {
                        (1, 1)
                    };

                    let propagated_data =
                        self.propagated_data.disallowing_percentage_table_columns();
                    let Contents::NonReplaced(non_replaced_contents) = contents else {
                        unreachable!("Replaced should not have a LayoutInternal display type.");
                    };

                    let contents = BlockFormattingContext::construct(
                        self.table_traversal.context,
                        info,
                        non_replaced_contents,
                        propagated_data,
                        false, /* is_list_item */
                    );

                    self.finish_current_anonymous_cell_if_needed();

                    let cell = ArcRefCell::new(TableSlotCell {
                        base: LayoutBoxBase::new(info.into(), info.style.clone()),
                        contents,
                        colspan,
                        rowspan,
                    });
                    self.table_traversal.builder.add_cell(cell.clone());
                    box_slot.set(LayoutBox::TableLevelBox(TableLevelBox::Cell(cell)));
                },
                _ => {
                    //// TODO: Properly handle other table-like elements in the middle of a row.
                    self.current_anonymous_cell_content
                        .push(AnonymousTableContent::Element {
                            info: info.clone(),
                            display,
                            contents,
                            box_slot,
                        });
                },
            },
            _ => {
                self.current_anonymous_cell_content
                    .push(AnonymousTableContent::Element {
                        info: info.clone(),
                        display,
                        contents,
                        box_slot,
                    });
            },
        }
    }
}

struct TableColumnGroupBuilder {
    column_group_index: usize,
    columns: Vec<ArcRefCell<TableTrack>>,
}

impl<'dom> TraversalHandler<'dom> for TableColumnGroupBuilder {
    fn handle_text(&mut self, _info: &NodeAndStyleInfo<'dom>, _text: Cow<'dom, str>) {}
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display: DisplayGeneratingBox,
        _contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        if !matches!(
            display,
            DisplayGeneratingBox::LayoutInternal(DisplayLayoutInternal::TableColumn)
        ) {
            // The BoxSlot destructor will check to ensure that it isn't empty but in this case, the
            // DOM node doesn't produce any box, so explicitly skip the destructor here.
            ::std::mem::forget(box_slot);
            return;
        }
        let column = add_column(
            &mut self.columns,
            info,
            Some(self.column_group_index),
            false, /* is_anonymous */
        );
        box_slot.set(LayoutBox::TableLevelBox(TableLevelBox::Track(column)));
    }
}

impl From<DisplayLayoutInternal> for TableTrackGroupType {
    fn from(value: DisplayLayoutInternal) -> Self {
        match value {
            DisplayLayoutInternal::TableColumnGroup => TableTrackGroupType::ColumnGroup,
            DisplayLayoutInternal::TableFooterGroup => TableTrackGroupType::FooterGroup,
            DisplayLayoutInternal::TableHeaderGroup => TableTrackGroupType::HeaderGroup,
            DisplayLayoutInternal::TableRowGroup => TableTrackGroupType::RowGroup,
            _ => unreachable!(),
        }
    }
}

fn add_column(
    collection: &mut Vec<ArcRefCell<TableTrack>>,
    column_info: &NodeAndStyleInfo,
    group_index: Option<usize>,
    is_anonymous: bool,
) -> ArcRefCell<TableTrack> {
    let span = if column_info.pseudo_element_type.is_none() {
        column_info.node.to_threadsafe().get_span().unwrap_or(1)
    } else {
        1
    };

    // The HTML specification clamps value of `span` for `<col>` to [1, 1000].
    assert!((1..=1000).contains(&span));

    let column = ArcRefCell::new(TableTrack {
        base: LayoutBoxBase::new(column_info.into(), column_info.style.clone()),
        group_index,
        is_anonymous,
        shared_background_style: SharedStyle::new(column_info.style.clone()),
    });
    collection.extend(repeat(column.clone()).take(span as usize));
    column
}
