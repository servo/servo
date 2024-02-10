/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

use log::warn;
use script_layout_interface::wrapper_traits::ThreadSafeLayoutNode;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::str::char_is_whitespace;
use style::values::specified::TextDecorationLine;

use super::{Table, TableSlot, TableSlotCell, TableSlotCoordinates, TableSlotOffset};
use crate::context::LayoutContext;
use crate::dom::{BoxSlot, NodeExt};
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NonReplacedContents, TraversalHandler};
use crate::flow::{BlockContainerBuilder, BlockFormattingContext};
use crate::formatting_contexts::{
    IndependentFormattingContext, NonReplacedFormattingContext,
    NonReplacedFormattingContextContents,
};
use crate::fragment_tree::{BaseFragmentInfo, FragmentFlags, Tag};
use crate::style_ext::{DisplayGeneratingBox, DisplayLayoutInternal};

/// A reference to a slot and its coordinates in the table
#[derive(Clone, Copy, Debug)]
pub(super) struct ResolvedSlotAndLocation<'a> {
    pub cell: &'a TableSlotCell,
    pub coords: TableSlotCoordinates,
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

pub(crate) enum AnonymousTableContent<'dom, Node> {
    Text(NodeAndStyleInfo<Node>, Cow<'dom, str>),
    Element {
        info: NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    },
}

impl Table {
    pub(crate) fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        contents: NonReplacedContents,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let mut traversal =
            TableBuilderTraversal::new(context, info, propagated_text_decoration_line);
        contents.traverse(context, info, &mut traversal);
        traversal.finish()
    }

    pub(crate) fn construct_anonymous<'dom, Node>(
        context: &LayoutContext,
        parent_info: &NodeAndStyleInfo<Node>,
        contents: Vec<AnonymousTableContent<'dom, Node>>,
        propagated_text_decoration_line: style::values::specified::TextDecorationLine,
    ) -> IndependentFormattingContext
    where
        Node: crate::dom::NodeExt<'dom>,
    {
        let anonymous_style = context
            .shared_context()
            .stylist
            .style_for_anonymous::<Node::ConcreteElement>(
                &context.shared_context().guards,
                // TODO: This should be updated for Layout 2020 once we've determined
                // which styles should be inherited for tables.
                &PseudoElement::ServoLegacyAnonymousTable,
                &parent_info.style,
            );
        let anonymous_info = parent_info.new_replacing_style(anonymous_style.clone());

        let mut table_builder =
            TableBuilderTraversal::new(context, &anonymous_info, propagated_text_decoration_line);

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

        IndependentFormattingContext::NonReplaced(NonReplacedFormattingContext {
            base_fragment_info: (&anonymous_info).into(),
            style: anonymous_style,
            content_sizes: None,
            contents: NonReplacedFormattingContextContents::Table(table),
        })
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
            Some(TableSlot::Cell(cell)) => vec![ResolvedSlotAndLocation { cell, coords }],
            Some(TableSlot::Spanned(ref offsets)) => offsets
                .iter()
                .flat_map(|offset| self.resolve_slot_at(coords - *offset))
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
            .filter(|slot| slot.covers_cell_at(target_coords))
            .map(|slot| target_coords - slot.coords)
            .collect();

        if coords_of_slots_that_cover_target.is_empty() {
            None
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
    pub(super) fn new(style: Arc<ComputedValues>) -> Self {
        Self {
            table: Table::new(style),
            incoming_rowspans: Vec::new(),
        }
    }

    pub fn new_for_tests() -> Self {
        Self::new(ComputedValues::initial_values().to_arc())
    }

    pub fn finish(mut self) -> Table {
        // Make sure that every row has the same number of cells.
        for row in self.table.slots.iter_mut() {
            row.resize_with(self.table.size.width, || TableSlot::Empty);
        }

        // Turn all rowspan=0 rows into the real value to avoid having to
        // make the calculation continually during layout. In addition, make
        // sure that there are no rowspans that extend past the end of the
        // table.
        for row_index in 0..self.table.size.height {
            for cell in self.table.slots[row_index].iter_mut() {
                if let TableSlot::Cell(ref mut cell) = cell {
                    let rowspan_to_end_of_table = self.table.size.height - row_index;
                    if cell.rowspan == 0 {
                        cell.rowspan = rowspan_to_end_of_table;
                    } else {
                        cell.rowspan = cell.rowspan.min(rowspan_to_end_of_table);
                    }
                }
            }
        }

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

    /// <https://html.spec.whatwg.org/multipage/#algorithm-for-processing-rows>
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

pub(crate) struct TableBuilderTraversal<'style, 'dom, Node> {
    context: &'style LayoutContext<'style>,
    info: &'style NodeAndStyleInfo<Node>,

    /// Propagated value for text-decoration-line, used to construct the block
    /// contents of table cells.
    propagated_text_decoration_line: TextDecorationLine,

    /// The [`TableBuilder`] for this [`TableBuilderTraversal`]. This is separated
    /// into another struct so that we can write unit tests against the builder.
    builder: TableBuilder,

    current_anonymous_row_content: Vec<AnonymousTableContent<'dom, Node>>,
}

impl<'style, 'dom, Node> TableBuilderTraversal<'style, 'dom, Node>
where
    Node: NodeExt<'dom>,
{
    pub(crate) fn new(
        context: &'style LayoutContext<'style>,
        info: &'style NodeAndStyleInfo<Node>,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        TableBuilderTraversal {
            context,
            info,
            propagated_text_decoration_line,
            builder: TableBuilder::new(info.style.clone()),
            current_anonymous_row_content: Vec::new(),
        }
    }

    pub(crate) fn finish(mut self) -> Table {
        self.finish_anonymous_row_if_needed();
        self.builder.finish()
    }

    fn finish_anonymous_row_if_needed(&mut self) {
        if self.current_anonymous_row_content.is_empty() {
            return;
        }

        let row_content = std::mem::take(&mut self.current_anonymous_row_content);
        let context = self.context;
        let anonymous_style = self
            .context
            .shared_context()
            .stylist
            .style_for_anonymous::<Node::ConcreteElement>(
                &context.shared_context().guards,
                &PseudoElement::ServoAnonymousTableCell,
                &self.info.style,
            );
        let anonymous_info = self.info.new_replacing_style(anonymous_style);
        let mut row_builder = TableRowBuilder::new(self, &anonymous_info);

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
    }
}

impl<'style, 'dom, Node: 'dom> TraversalHandler<'dom, Node>
    for TableBuilderTraversal<'style, 'dom, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, text: Cow<'dom, str>) {
        if text.chars().all(char_is_whitespace) {
            return;
        }
        self.current_anonymous_row_content
            .push(AnonymousTableContent::Text(info.clone(), text));
    }

    /// <https://html.spec.whatwg.org/multipage/#forming-a-table>
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
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

                    // TODO: Should we fixup `rowspan=0` to the actual resolved value and
                    // any other rowspans that have been cut short?
                    self.builder.incoming_rowspans.clear();
                    NonReplacedContents::try_from(contents).unwrap().traverse(
                        self.context,
                        info,
                        self,
                    );

                    // TODO: Handle style for row groups here.

                    // We are doing this until we have actually set a Box for this `BoxSlot`.
                    ::std::mem::forget(box_slot)
                },
                DisplayLayoutInternal::TableRow => {
                    self.finish_anonymous_row_if_needed();

                    let context = self.context;

                    let mut row_builder = TableRowBuilder::new(self, info);
                    NonReplacedContents::try_from(contents).unwrap().traverse(
                        context,
                        info,
                        &mut row_builder,
                    );
                    row_builder.finish();

                    // We are doing this until we have actually set a Box for this `BoxSlot`.
                    ::std::mem::forget(box_slot)
                },
                DisplayLayoutInternal::TableCaption |
                DisplayLayoutInternal::TableColumn |
                DisplayLayoutInternal::TableColumnGroup => {
                    // TODO: Handle these other types of table elements.

                    // We are doing this until we have actually set a Box for this `BoxSlot`.
                    ::std::mem::forget(box_slot)
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

struct TableRowBuilder<'style, 'builder, 'dom, 'a, Node> {
    table_traversal: &'builder mut TableBuilderTraversal<'style, 'dom, Node>,

    /// The [`NodeAndStyleInfo`] of this table row, which we use to
    /// construct anonymous table cells.
    info: &'a NodeAndStyleInfo<Node>,

    current_anonymous_cell_content: Vec<AnonymousTableContent<'dom, Node>>,
}

impl<'style, 'builder, 'dom, 'a, Node: 'dom> TableRowBuilder<'style, 'builder, 'dom, 'a, Node>
where
    Node: NodeExt<'dom>,
{
    fn new(
        table_traversal: &'builder mut TableBuilderTraversal<'style, 'dom, Node>,
        info: &'a NodeAndStyleInfo<Node>,
    ) -> Self {
        table_traversal.builder.start_row();

        TableRowBuilder {
            table_traversal,
            info,
            current_anonymous_cell_content: Vec::new(),
        }
    }

    fn finish(mut self) {
        self.finish_current_anonymous_cell_if_needed();
        self.table_traversal.builder.end_row();
    }

    fn finish_current_anonymous_cell_if_needed(&mut self) {
        if self.current_anonymous_cell_content.is_empty() {
            return;
        }

        let context = self.table_traversal.context;
        let anonymous_style = context
            .shared_context()
            .stylist
            .style_for_anonymous::<Node::ConcreteElement>(
                &context.shared_context().guards,
                &PseudoElement::ServoAnonymousTableCell,
                &self.info.style,
            );
        let anonymous_info = self.info.new_replacing_style(anonymous_style);
        let mut builder = BlockContainerBuilder::new(
            context,
            &anonymous_info,
            self.table_traversal.propagated_text_decoration_line,
        );

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

        let tag = Tag::new_pseudo(
            self.info.node.opaque(),
            Some(PseudoElement::ServoAnonymousTableCell),
        );
        let base_fragment_info = BaseFragmentInfo {
            tag,
            flags: FragmentFlags::empty(),
        };

        let block_container = builder.finish();
        self.table_traversal.builder.add_cell(TableSlotCell {
            contents: BlockFormattingContext::from_block_container(block_container),
            colspan: 1,
            rowspan: 1,
            style: anonymous_info.style,
            base_fragment_info,
        });
    }
}

impl<'style, 'builder, 'dom, 'a, Node: 'dom> TraversalHandler<'dom, Node>
    for TableRowBuilder<'style, 'builder, 'dom, 'a, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, text: Cow<'dom, str>) {
        if text.chars().all(char_is_whitespace) {
            return;
        }
        self.current_anonymous_cell_content
            .push(AnonymousTableContent::Text(info.clone(), text));
    }

    /// <https://html.spec.whatwg.org/multipage/#algorithm-for-processing-rows>
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
                            unreachable!("Replaced should not have a LayoutInternal display type.");
                        },
                    };

                    self.finish_current_anonymous_cell_if_needed();
                    self.table_traversal.builder.add_cell(TableSlotCell {
                        contents,
                        colspan,
                        rowspan,
                        style: info.style.clone(),
                        base_fragment_info: info.into(),
                    });

                    // We are doing this until we have actually set a Box for this `BoxSlot`.
                    ::std::mem::forget(box_slot)
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
