/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

use log::warn;
use script_layout_interface::wrapper_traits::ThreadSafeLayoutNode;
use style::values::specified::TextDecorationLine;

use super::{Table, TableSlot, TableSlotCell, TableSlotCoordinates, TableSlotOffset};
use crate::context::LayoutContext;
use crate::dom::{BoxSlot, NodeExt};
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NonReplacedContents, TraversalHandler};
use crate::flow::BlockFormattingContext;
use crate::style_ext::{DisplayGeneratingBox, DisplayLayoutInternal};

/// A reference to a slot and its coordinates in the table
#[derive(Clone, Copy, Debug)]
struct ResolvedSlotAndLocation<'a> {
    cell: &'a TableSlotCell,
    coords: TableSlotCoordinates,
}

impl<'a> ResolvedSlotAndLocation<'a> {
    fn covers_cell_at(&self, coords: TableSlotCoordinates) -> bool {
        let covered_in_x =
            coords.x >= self.coords.x && coords.x < self.coords.x + self.cell.colspan;
        let covered_in_y = coords.y >= self.coords.y &&
            (self.cell.rowspan == 0 || coords.y < self.coords.y + self.cell.rowspan);
        covered_in_x && covered_in_y
    }
}

impl Table {
    pub(crate) fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        contents: NonReplacedContents,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let mut traversal = TableBuilderTraversal {
            context,
            _info: info,
            propagated_text_decoration_line,
            builder: Default::default(),
        };
        contents.traverse(context, info, &mut traversal);
        traversal.builder.finish()
    }

    /// Push a new slot into the last row of this table.
    fn push_new_slot_to_last_row(&mut self, slot: TableSlot) {
        self.slots.last_mut().expect("Should have rows").push(slot)
    }

    /// Convenience method for get() that returns a SlotAndLocation
    fn get_slot<'a>(&'a self, coords: TableSlotCoordinates) -> Option<&'a TableSlot> {
        self.slots.get(coords.y)?.get(coords.x)
    }

    /// Find [`ResolvedSlotAndLocation`] of all the slots that cover the slot at the given
    /// coordinates. This recursively resolves all of the [`TableSlotCell`]s that cover
    /// the target and returns a [`ResolvedSlotAndLocation`] for each of them. If there is
    /// no slot at the given coordinates or that slot is an empty space, an empty vector
    /// is returned.
    fn resolve_slot_at<'a>(
        &'a self,
        coords: TableSlotCoordinates,
    ) -> Vec<ResolvedSlotAndLocation<'a>> {
        let slot = self.get_slot(coords);
        match slot {
            Some(TableSlot::Cell(cell)) => vec![ResolvedSlotAndLocation {
                cell: &cell,
                coords,
            }],
            Some(TableSlot::Spanned(ref offsets)) => offsets
                .iter()
                .map(|offset| self.resolve_slot_at(coords - *offset))
                .flatten()
                .collect(),
            Some(TableSlot::Empty) | None => {
                warn!("Tried to resolve an empty or nonexistant slot!");
                vec![]
            },
        }
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
        let coords_for_slot_above =
            TableSlotCoordinates::new(target_coords.x, self.slots.len() - 2);
        let slots_covering_slot_above = self.resolve_slot_at(coords_for_slot_above);

        let coords_of_slots_that_cover_target: Vec<_> = slots_covering_slot_above
            .into_iter()
            .filter(|ref slot| slot.covers_cell_at(target_coords))
            .map(|slot| target_coords - slot.coords)
            .collect();

        if coords_of_slots_that_cover_target.is_empty() {
            return None;
        } else {
            Some(TableSlot::Spanned(coords_of_slots_that_cover_target))
        }
    }
}

impl TableSlot {
    /// Merge a TableSlot::Spanned(x, y) with this (only for model errors)
    pub fn push_spanned(&mut self, new_offset: TableSlotOffset) {
        match *self {
            TableSlot::Cell { .. } => {
                panic!("Should never have a table model error with an originating cell slot overlapping a spanned slot")
            },
            TableSlot::Spanned(ref mut vec) => vec.insert(0, new_offset),
            TableSlot::Empty => {
                panic!("Should never have a table model error with an empty slot");
            },
        }
    }
}

#[derive(Default)]
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
    pub fn finish(self) -> Table {
        self.table
    }

    fn current_y(&self) -> usize {
        self.table.slots.len() - 1
    }

    fn current_x(&self) -> usize {
        self.table.slots[self.current_y()].len()
    }

    fn current_coords(&self) -> TableSlotCoordinates {
        TableSlotCoordinates::new(self.current_x(), self.current_y())
    }

    pub fn start_row<'builder>(&'builder mut self) {
        self.table.slots.push(Vec::new());
        self.create_slots_for_cells_above_with_rowspan(true);
    }

    pub fn end_row(&mut self) {
        // TODO: We need to insert a cell for any leftover non-table-like
        // content in the TableRowBuilder.

        // Truncate entries that are zero at the end of [`Self::incoming_rowspans`]. This
        // prevents padding the table with empty cells when it isn't necessary.
        let current_x = self.current_x();
        for i in (current_x..self.incoming_rowspans.len()).rev() {
            if self.incoming_rowspans[i] == 0 {
                self.incoming_rowspans.pop();
            } else {
                break;
            }
        }

        self.create_slots_for_cells_above_with_rowspan(false);
    }

    /// When not in the process of filling a cell, make sure any incoming rowspans are
    /// filled so that the next specified cell comes after them. Should have been called before
    /// [`Self::handle_cell`].
    ///
    /// if `stop_at_cell_opportunity` is set, this will stop at the first slot with
    /// `incoming_rowspans` equal to zero. If not, it will insert [`TableSlot::Empty`] and
    /// continue to look for more incoming rowspans (which should only be done once we're
    /// finished processing the cells in a row, and after calling truncating cells with
    /// remaining rowspan from the end of `incoming_rowspans`.
    fn create_slots_for_cells_above_with_rowspan(&mut self, stop_at_cell_opportunity: bool) {
        let mut current_x = self.current_x();
        while let Some(span) = self.incoming_rowspans.get_mut(current_x) {
            // This column has no incoming rowspanned cells and `stop_at_zero` is true, so
            // we should stop to process new cells defined in the current row.
            if *span == 0 && stop_at_cell_opportunity {
                break;
            }

            let new_cell = if *span != 0 {
                *span -= 1;
                self.table
                    .create_spanned_slot_based_on_cell_above(self.current_coords())
                    .expect(
                        "Nonzero incoming rowspan cannot occur without a cell spanning this slot",
                    )
            } else {
                TableSlot::Empty
            };

            self.table.push_new_slot_to_last_row(new_cell);
            current_x = self.current_x();
        }
    }

    /// https://html.spec.whatwg.org/multipage/#algorithm-for-processing-rows
    /// Push a single cell onto the slot map, handling any colspans it may have, and
    /// setting up the outgoing rowspans.
    pub fn add_cell(&mut self, cell: TableSlotCell) {
        // Make sure the incoming_rowspans table is large enough
        // because we will be writing to it.
        let current_x = self.current_x();
        let colspan = cell.colspan;
        let rowspan = cell.rowspan;

        if self.incoming_rowspans.len() < current_x + colspan {
            self.incoming_rowspans.resize(current_x + colspan, 0isize);
        }

        debug_assert_eq!(
            self.incoming_rowspans[current_x], 0,
            "Added a cell in a position that also had an incoming rowspan!"
        );

        // If `rowspan` is zero, this is automatically negative and will stay negative.
        let outgoing_rowspan = rowspan as isize - 1;
        self.table.push_new_slot_to_last_row(TableSlot::Cell(cell));
        self.incoming_rowspans[current_x] = outgoing_rowspan;

        // Draw colspanned cells
        for colspan_offset in 1..colspan {
            let current_x_plus_colspan_offset = current_x + colspan_offset;
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
                    TableSlotCoordinates::new(current_x_plus_colspan_offset, self.current_y());
                match self
                    .table
                    .create_spanned_slot_based_on_cell_above(coords_of_spanned_cell)
                {
                    Some(mut incoming_slot) => {
                        incoming_slot.push_spanned(new_offset);
                        incoming_slot
                    },
                    None => TableSlot::new_spanned(new_offset),
                }
            };
            self.table.push_new_slot_to_last_row(new_slot);
        }

        debug_assert_eq!(
            current_x + colspan,
            self.current_x(),
            "Must have produced `colspan` slot entries!"
        );
        self.create_slots_for_cells_above_with_rowspan(true);
    }
}

struct TableBuilderTraversal<'a, Node> {
    context: &'a LayoutContext<'a>,
    _info: &'a NodeAndStyleInfo<Node>,

    /// Propagated value for text-decoration-line, used to construct the block
    /// contents of table cells.
    propagated_text_decoration_line: TextDecorationLine,

    /// The [`TableBuilder`] for this [`TableBuilderTraversal`]. This is separated
    /// into another struct so that we can write unit tests against the builder.
    builder: TableBuilder,
}

impl<'a, 'dom, Node: 'dom> TraversalHandler<'dom, Node> for TableBuilderTraversal<'a, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(&mut self, _info: &NodeAndStyleInfo<Node>, _text: Cow<'dom, str>) {
        // TODO: We should collect these contents into a new table cell.
    }

    /// https://html.spec.whatwg.org/multipage/#forming-a-table
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        match display {
            DisplayGeneratingBox::LayoutInternal(internal) => match internal {
                DisplayLayoutInternal::TableRowGroup => {
                    // TODO: Should we fixup `rowspan=0` to the actual resolved value and
                    // any other rowspans that have been cut short?
                    self.builder.incoming_rowspans.clear();
                    NonReplacedContents::try_from(contents).unwrap().traverse(
                        self.context,
                        info,
                        self,
                    );

                    // TODO: Handle style for row groups here.
                },
                DisplayLayoutInternal::TableRow => {
                    self.builder.start_row();
                    NonReplacedContents::try_from(contents).unwrap().traverse(
                        self.context,
                        info,
                        &mut TableRowBuilder::new(self),
                    );
                    self.builder.end_row();
                },
                _ => {
                    // TODO: Handle other types of unparented table content, colgroups, and captions.
                },
            },
            _ => {
                // TODO: Create an anonymous row and cell for other unwrapped content.
            },
        }

        // We are doing this until we have actually set a Box for this `BoxSlot`.
        ::std::mem::forget(box_slot)
    }
}

struct TableRowBuilder<'a, 'builder, Node> {
    table_traversal: &'builder mut TableBuilderTraversal<'a, Node>,
}

impl<'a, 'builder, Node> TableRowBuilder<'a, 'builder, Node> {
    fn new(table_traversal: &'builder mut TableBuilderTraversal<'a, Node>) -> Self {
        TableRowBuilder { table_traversal }
    }
}

impl<'a, 'builder, 'dom, Node: 'dom> TraversalHandler<'dom, Node>
    for TableRowBuilder<'a, 'builder, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(&mut self, _info: &NodeAndStyleInfo<Node>, _text: Cow<'dom, str>) {
        // TODO: We should collect these contents into a new table cell.
    }

    /// https://html.spec.whatwg.org/multipage/#algorithm-for-processing-rows
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        match display {
            DisplayGeneratingBox::LayoutInternal(internal) => match internal {
                DisplayLayoutInternal::TableCell => {
                    // This value will already have filtered out rowspan=0
                    // in quirks mode, so we don't have to worry about that.
                    //
                    // The HTML specification limits the parsed value of `rowspan` to
                    // 65534 and `colspan` to 1000, so we also enforce the same limits
                    // when dealing with arbitrary DOM elements (perhaps created via
                    // script).
                    let node = info.node.to_threadsafe();
                    let rowspan = std::cmp::min(node.get_rowspan() as usize, 65534);
                    let colspan = std::cmp::min(node.get_colspan() as usize, 1000);

                    let contents = match contents.try_into() {
                        Ok(non_replaced_contents) => {
                            BlockFormattingContext::construct(
                                self.table_traversal.context,
                                info,
                                non_replaced_contents,
                                self.table_traversal.propagated_text_decoration_line,
                                false, /* is_list_item */
                            )
                        },
                        Err(_replaced) => {
                            panic!("We don't handle this yet.");
                        },
                    };

                    self.table_traversal.builder.add_cell(TableSlotCell {
                        contents,
                        colspan,
                        rowspan,
                        id: 0, // This is just an id used for testing purposes.
                    });
                },
                _ => {
                    // TODO: Properly handle other table-like elements in the middle of a row.
                },
            },
            _ => {
                // TODO: We should collect these contents into a new table cell.
            },
        }

        // We are doing this until we have actually set a Box for this `BoxSlot`.
        ::std::mem::forget(box_slot)
    }
}
