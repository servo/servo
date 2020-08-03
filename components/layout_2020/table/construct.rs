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
use style::values::specified::text::TextDecorationLine;

#[derive(Debug, Serialize, Default)]
/// A map of table slots to cells
pub(crate) struct TableSlots {
    rows: Vec<TableSlotsRow>,
}

impl TableSlots {
    /// Get the slot at (x, y)
    pub fn get(&self, x: usize, y: usize) -> Option<&TableSlot> {
        self.rows.get(y)?.cells.get(x)
    }

    fn insert(&mut self, slot: TableSlot) {
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
#[derive(Copy, Clone, Debug)]
struct SlotAndLocation<'a> {
    slot: &'a TableSlot,
    x: usize,
    y: usize,
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct TableSlotsRow {
    cells: Vec<TableSlot>,
}

#[derive(Debug, Serialize)]
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
}

impl TableSlot {
    pub fn as_spanned(&self) -> (usize, usize) {
        if let TableSlot::Spanned(x, y) = *self {
            (x, y)
        } else {
            panic!("TableSlot::as_spanned called with a non-Spanned TableSlot")
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
    incoming_rowspans: Vec<i32>,
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
                    // XXXManishearth we need to cap downward growing rows
                    // https://html.spec.whatwg.org/multipage/tables.html#algorithm-for-ending-a-row-group
                    NonReplacedContents::try_from(contents).unwrap().traverse(
                        self.context,
                        info,
                        self,
                    );
                },
                DisplayInternal::TableRow => {
                    let context = self.context;
                    // XXXManishearth use with_capacity
                    self.slots.rows.push(TableSlotsRow::default());
                    let mut row_builder = TableRowBuilder::new(self);
                    row_builder.consume_rowspans();
                    NonReplacedContents::try_from(contents).unwrap().traverse(
                        context,
                        info,
                        &mut row_builder,
                    );
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
    current_x: usize,
}

impl<'a, 'builder, Node> TableRowBuilder<'a, 'builder, Node> {
    fn new(builder: &'builder mut TableContainerBuilder<'a, Node>) -> Self {
        TableRowBuilder {
            builder,
            current_x: 0,
        }
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
                DisplayInternal::TableCell => self.handle_cell(&info),
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
    fn consume_rowspans(&mut self) {
        loop {
            if let Some(span) = self.builder.incoming_rowspans.get_mut(self.current_x) {
                if *span != 0 {
                    *span -= 1;
                    let previous = self
                        .builder
                        .slots
                        .get_above(self.current_x)
                        .expect("Cannot have nonzero incoming rowspan with no slot above");
                    let new_slot = self.builder
                        .slots.spanned_slot(self.current_x, self.builder.current_y(), previous)
                        .expect("Nonzero incoming rowspan cannot occur without a cell spannign this slot");
                    self.builder.slots.insert(new_slot);
                    self.current_x += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/tables.html#algorithm-for-processing-rows
    fn handle_cell(&mut self, info: &NodeAndStyleInfo<Node>) {
        let node = info.node.to_threadsafe();
        // This value will already have filtered out rowspan=0
        // in quirks mode, so we don't have to worry about that
        let rowspan = cmp::min(node.get_rowspan(), 1000);
        let colspan = cmp::min(node.get_colspan(), 1000);
    }
}
