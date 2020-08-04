use super::TableContainer;
use crate::context::LayoutContext;
use crate::dom_traversal::{
    BoxSlot, Contents, NodeAndStyleInfo, NodeExt, NonReplacedContents, TraversalHandler,
};
use crate::style_ext::{DisplayGeneratingBox, DisplayInternal};
use script_layout_interface::wrapper_traits::ThreadSafeLayoutNode;
use std::borrow::Cow;
use std::cmp;
use std::convert::TryFrom;
use std::fmt;
use style::values::specified::text::TextDecorationLine;

#[derive(Debug, Default, Serialize)]
/// A map of table slots to cells
pub(crate) struct TableSlots {
    rows: Vec<TableSlotsRow>,
}

#[derive(Debug, Default, Serialize)]
/// A row in the table slot map
pub(crate) struct TableSlotsRow {
    cells: Vec<TableSlot>,
}

impl TableSlots {
    /// Get the slot at (x, y)
    pub fn get(&self, x: usize, y: usize) -> Option<&TableSlot> {
        self.rows.get(y)?.cells.get(x)
    }

    /// Inserts a new slot into the last row
    fn push(&mut self, slot: TableSlot) {
        let y = self.rows.len() - 1;
        self.rows[y].cells.push(slot)
    }

    /// Convenience method for get() that returns a SlotAndLocation
    fn get_loc(&self, x: usize, y: usize) -> Option<SlotAndLocation> {
        self.rows
            .get(y)?
            .cells
            .get(x)
            .map(|slot| SlotAndLocation { slot, x, y })
    }

    /// Get the slot from the previous row at x
    fn get_above(&self, x: usize) -> Option<SlotAndLocation> {
        if self.rows.len() > 1 {
            self.get_loc(x, self.rows.len() - 2)
        } else {
            None
        }
    }

    /// Given a slot and location, find the originating TableSlot::Cell.
    /// In the case of a table model error, there may be multiple, returning
    /// the oldest cell first
    fn resolve_slot<'a>(
        &'a self,
        location: SlotAndLocation<'a>,
    ) -> (SlotAndLocation<'a>, Vec<SlotAndLocation<'a>>) {
        match *location.slot {
            TableSlot::Cell { .. } => (location, Vec::new()),
            TableSlot::Spanned(x, y) => (
                self.get_loc(location.x - x, location.y - y)
                    .expect("Spanned slot reference must resolve"),
                Vec::new(),
            ),
            TableSlot::MultiSpanned(ref vec) => {
                let mut v: Vec<_> = vec
                    .iter()
                    .map(|(x, y)| {
                        self.get_loc(location.x - x, location.y - y)
                            .expect("Spanned slot reference must resolve")
                    })
                    .collect();
                (v.pop().unwrap(), v)
            },
            TableSlot::Empty => {
                panic!("Should never attempt to resolve an empty slot");
            },
        }
    }

    /// If (x, y) is spanned by an already resolved `spanner`, return offsets for a TableSlot::Spanned() for it
    fn spanned_slot_single(
        &self,
        x: usize,
        y: usize,
        spanner: SlotAndLocation,
    ) -> Option<(usize, usize)> {
        if let TableSlot::Cell { width, height, .. } = spanner.slot {
            if x >= spanner.x && y >= spanner.y && x < spanner.x + *width {
                if *height == 0 || y < spanner.y + *height {
                    Some((x - spanner.x, y - spanner.y))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            panic!("spanned_slot_single cannot be called with a non-resolved slot")
        }
    }

    /// If (x, y) is spanned by the resolution of `spanner`, return a
    /// TableSlot::Spanned() or TableSlot::MultiSpanned() for it
    fn spanned_slot(&self, x: usize, y: usize, spanner: SlotAndLocation) -> Option<TableSlot> {
        let (first, mut rest) = self.resolve_slot(spanner);
        if rest.is_empty() {
            self.spanned_slot_single(x, y, first)
                .map(|spanned| TableSlot::Spanned(spanned.0, spanned.1))
        } else {
            rest.push(first);

            let coordinates: Vec<_> = rest
                .into_iter()
                .filter_map(|slot| self.spanned_slot_single(x, y, slot))
                .collect();
            if coordinates.is_empty() {
                return None;
            }

            if coordinates.len() == 1 {
                Some(TableSlot::Spanned(coordinates[0].0, coordinates[0].1))
            } else {
                Some(TableSlot::MultiSpanned(coordinates))
            }
        }
    }
}

// A reference to a slot and its coordinates in the table
#[derive(Clone, Copy, Debug)]
struct SlotAndLocation<'a> {
    slot: &'a TableSlot,
    x: usize,
    y: usize,
}

#[derive(Serialize)]
/// A single table slot. It may be an actual cell, or a reference
/// to a previous cell that is spanned here
///
/// In case of table model errors, it may be multiple references
pub(crate) enum TableSlot {
    /// A table cell, with a width and height
    Cell {
        cell: TableCellBox,
        /// the width of the cell
        width: usize,
        /// the height of the cell (0 for rowspan=0)
        // XXXManishearth should we fixup rowspan=0 later?
        height: usize,
    },
    /// This slot is spanned by the cell at offset (-x, -y)
    Spanned(usize, usize),
    /// This slot is spanned by multiple cells at the given negative coordinate offsets. Oops.
    /// This is a table model error, but we still keep track of it
    /// https://html.spec.whatwg.org/multipage/tables.html#table-model-error
    ///
    /// The Vec is in the order of newest to oldest cell
    MultiSpanned(Vec<(usize, usize)>),

    /// There's nothing here
    /// Only exists when there's a rowspan coming up
    Empty,
}

impl TableSlot {
    /// Assuming this is a TableSlot::Spanned, get the coordinates
    pub fn as_spanned(&self) -> (usize, usize) {
        if let TableSlot::Spanned(x, y) = *self {
            (x, y)
        } else {
            panic!("TableSlot::as_spanned called with a non-Spanned TableSlot")
        }
    }

    /// Merge a TableSlot::Spanned(x, y) with this (only for model errors)
    pub fn push_spanned(&mut self, x: usize, y: usize) {
        match *self {
            TableSlot::Cell {..} => {
                panic!("Should never have a table model error with an originating cell slot overlapping a spanned slot")
            }
            TableSlot::Spanned(x1, y1) => {
                *self = TableSlot::MultiSpanned(vec![(x, y), (x1, y1)])
            }
            TableSlot::MultiSpanned(ref mut vec) => vec.insert(0, (x, y)),
            TableSlot::Empty => {
                panic!("Should never have a table model error with an empty slot");
            }
        }
    }
}

impl fmt::Debug for TableSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TableSlot::Cell { width, height, .. } => write!(f, "Cell({}, {})", width, height),
            TableSlot::Spanned(x, y) => write!(f, "Spanned({}, {})", x, y),
            TableSlot::MultiSpanned(ref vec) => write!(f, "MultiSpanned({:?})", vec),
            TableSlot::Empty => write!(f, "Empty"),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct TableCellBox {}

struct TableContainerBuilder<'a, Node> {
    context: &'a LayoutContext<'a>,
    info: &'a NodeAndStyleInfo<Node>,
    slots: TableSlots,
    /// If there is an incoming rowspanned cell in this column,
    /// this value will be nonzero. Positive values indicate the number of
    /// rows that still need to be spanned. Negative values indicate rowspan=0
    ///
    /// This vector is reused for the outgoing rowspans, if there is already a cell
    /// in the cell map the value in this array represents the incoming rowspan for the *next* row
    incoming_rowspans: Vec<isize>,
}

impl TableContainer {
    pub fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        contents: NonReplacedContents,
        // XXXManishearth is this useful?
        _propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let mut builder = TableContainerBuilder::new(context, info);
        contents.traverse(context, info, &mut builder);
        TableContainer {}
    }
}

impl<'a, Node> TableContainerBuilder<'a, Node> {
    fn new(context: &'a LayoutContext, info: &'a NodeAndStyleInfo<Node>) -> Self {
        TableContainerBuilder {
            context,
            info,
            slots: TableSlots::default(),
            incoming_rowspans: Vec::new(),
        }
    }

    fn current_y(&self) -> usize {
        self.slots.rows.len() - 1
    }
}

impl<'a, 'dom, Node: 'dom> TraversalHandler<'dom, Node> for TableContainerBuilder<'a, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, text: Cow<'dom, str>) {
        // TODO: this might need to be wrapped in something
    }

    /// https://html.spec.whatwg.org/multipage/tables.html#forming-a-table
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        match display {
            DisplayGeneratingBox::Internal(i) => match i {
                DisplayInternal::TableRowGroup => {
                    // XXXManishearth maybe fixup `width=0` to the actual resolved value
                    // and any other rowspans that have been cut short
                    self.incoming_rowspans.clear();
                    NonReplacedContents::try_from(contents).unwrap().traverse(
                        self.context,
                        info,
                        self,
                    );

                    // XXXManishearth push some kind of row group box somewhere
                },
                DisplayInternal::TableRow => {
                    let context = self.context;
                    // XXXManishearth use with_capacity
                    self.slots.rows.push(TableSlotsRow::default());
                    let mut row_builder = TableRowBuilder::new(self);
                    row_builder.consume_rowspans(true);
                    NonReplacedContents::try_from(contents).unwrap().traverse(
                        context,
                        info,
                        &mut row_builder,
                    );
                    row_builder.truncate_incoming_rowspans();
                    row_builder.consume_rowspans(false);

                    // XXXManishearth push some kind of row box somewhere
                },
                _ => (),
                // XXXManishearth handle colgroups/etc
                // XXXManishearth handle unparented cells ?
                // XXXManishearth handle captions
            },
            _ => {
                // TODO this might need to be wrapped
            },
        }
        ::std::mem::forget(box_slot)
        // do something?
    }
}

struct TableRowBuilder<'a, 'builder, Node> {
    builder: &'builder mut TableContainerBuilder<'a, Node>,
}

impl<'a, 'builder, Node> TableRowBuilder<'a, 'builder, Node> {
    fn new(builder: &'builder mut TableContainerBuilder<'a, Node>) -> Self {
        TableRowBuilder { builder }
    }

    fn current_x(&self) -> usize {
        self.builder.slots.rows[self.builder.current_y()]
            .cells
            .len()
    }
}

impl<'a, 'builder, 'dom, Node: 'dom> TraversalHandler<'dom, Node>
    for TableRowBuilder<'a, 'builder, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, text: Cow<'dom, str>) {
        // TODO: this might need to be wrapped in something
    }

    /// https://html.spec.whatwg.org/multipage/tables.html#algorithm-for-processing-rows
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        match display {
            DisplayGeneratingBox::Internal(i) => match i {
                DisplayInternal::TableCell => {
                    self.handle_cell(&info);
                    self.consume_rowspans(true);
                    // XXXManishearth this will not handle any leftover incoming rowspans
                    // after all cells are processed, we need to introduce TableSlot::None
                },
                _ => (), // XXXManishearth handle unparented row groups/etc ?
            },
            _ => {
                // TODO this might need to be wrapped
            },
        }
        ::std::mem::forget(box_slot)
        // do something?
    }
}

impl<'a, 'builder, 'dom, Node> TableRowBuilder<'a, 'builder, Node>
where
    Node: NodeExt<'dom>,
{
    /// When not in the process of filling a cell, make sure any incoming rowspans are
    /// filled so that the next specified cell comes after them. Should have been called before
    /// handle_cell
    ///
    /// if stop_at_zero is set, this will stop at the first slot with incoming_rowspans equal
    /// to zero. If not, it will insert Empty TableSlots and continue to look for more incoming
    /// rowspans (which should only be done once we're finished processing the cells in a row,
    /// and after calling truncate_incoming_rowspans() )
    fn consume_rowspans(&mut self, stop_at_zero: bool) {
        loop {
            let current_x = self.current_x();
            if let Some(span) = self.builder.incoming_rowspans.get_mut(current_x) {
                if *span != 0 {
                    *span -= 1;
                    let previous = self
                        .builder
                        .slots
                        .get_above(current_x)
                        .expect("Cannot have nonzero incoming rowspan with no slot above");
                    let new_slot = self.builder
                        .slots.spanned_slot(current_x, self.builder.current_y(), previous)
                        .expect("Nonzero incoming rowspan cannot occur without a cell spanning this slot");
                    self.builder.slots.push(new_slot);
                } else {
                    if stop_at_zero {
                        // We have at least one free slot here, exit so that cells can be filled in
                        break;
                    } else {
                        // Push an empty slot so that the incoming span is in the right place
                        self.builder.slots.push(TableSlot::Empty);
                    }
                }
            } else {
                // No more incoming rowspans, exit
                break;
            }
        }
    }

    /// Before calling consume_rowspans() with stop_at_zero=true, make sure there are no extraneous incoming rowspans
    /// at the tail
    fn truncate_incoming_rowspans(&mut self) {
        let current_x = self.current_x();
        for i in (current_x..self.builder.incoming_rowspans.len()).rev() {
            if self.builder.incoming_rowspans[i] == 0 {
                self.builder.incoming_rowspans.pop();
            } else {
                break;
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/tables.html#algorithm-for-processing-rows
    /// Push a single cell onto the cell slot map, handling any colspans it may have, and
    /// setting up the outgoing rowspans
    fn handle_cell(&mut self, info: &NodeAndStyleInfo<Node>) {
        let current_x = self.current_x();
        let node = info.node.to_threadsafe();
        // This value will already have filtered out rowspan=0
        // in quirks mode, so we don't have to worry about that
        let rowspan = cmp::min(node.get_rowspan() as usize, 1000);
        let colspan = cmp::min(node.get_colspan() as usize, 1000);

        let me = TableSlot::Cell {
            cell: TableCellBox {},
            width: colspan,
            height: rowspan,
        };

        if self.builder.incoming_rowspans.len() < current_x + colspan {
            // make sure the incoming_rowspans table is large enough
            // because we will be writing to it
            self.builder
                .incoming_rowspans
                .resize(current_x + colspan, 0isize);
        }

        debug_assert_eq!(
            self.builder.incoming_rowspans[current_x], 0,
            "consume_rowspans must have been called before this!"
        );

        // if rowspan is zero, this is automatically negative and will stay negative
        let outgoing_rowspan = rowspan as isize - 1;
        self.builder.slots.push(me);
        self.builder.incoming_rowspans[current_x] = outgoing_rowspan;

        // Draw colspanned cells
        for offset in 1..colspan {
            let offset_x = current_x + offset;
            let new_slot = TableSlot::Spanned(offset, 0);
            let incoming_rowspan = &mut self.builder.incoming_rowspans[offset_x];
            if *incoming_rowspan == 0 {
                *incoming_rowspan = outgoing_rowspan;
                self.builder.slots.push(new_slot);

                // No model error, skip the remaining stuff
                continue;
            } else if *incoming_rowspan > 0 {
                // Set the incoming rowspan to the highest of two possible outgoing rowspan values
                // (the incoming rowspan minus one, OR this cell's outgoing rowspan)
                // spanned_slot() will handle filtering out inapplicable spans when it needs to
                *incoming_rowspan = cmp::max(*incoming_rowspan - 1, outgoing_rowspan);
            } else {
                // Don't change the rowspan, if it's negative we are in `rowspan=0` mode,
                // i.e. rowspan=infinity, so we don't have to worry about the current cell making
                // it larger

                // do nothing
            }

            // Code for handling model errors
            let previous = self
                .builder
                .slots
                .get_above(offset_x)
                .expect("Cannot have nonzero incoming rowspan with no slot above");
            let incoming_slot =
                self.builder
                    .slots
                    .spanned_slot(offset_x, self.builder.current_y(), previous);
            let new_slot = incoming_slot
                .map(|mut s| {
                    s.push_spanned(offset, 0);
                    s
                })
                .unwrap_or(new_slot);
            self.builder.slots.push(new_slot)
        }

        debug_assert_eq!(
            current_x + colspan,
            self.current_x(),
            "Must have produced `colspan` slot entries!"
        );
    }
}
