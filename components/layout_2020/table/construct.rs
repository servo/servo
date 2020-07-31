use super::TableContainer;
use crate::context::LayoutContext;
use crate::dom_traversal::{
    BoxSlot, Contents, NodeAndStyleInfo, NodeExt, NonReplacedContents, TraversalHandler,
};
use crate::style_ext::{DisplayGeneratingBox, DisplayInternal};
use std::borrow::Cow;
use std::convert::TryFrom;
use style::values::specified::text::TextDecorationLine;

#[derive(Debug, Serialize, Default)]
pub(crate) struct TableSlots {
    rows: Vec<TableSlotsRow>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TableSlotsRow {
    cells: Vec<TableSlot>,
}

#[derive(Debug, Serialize)]
pub(crate) enum TableSlot {
    /// A table cell, with a width and height
    Cell {
        cell: TableCellBox,
        // the width of the cell
        width: u32,
        // the height of the cell
        height: u32,
    },
    /// This slot is spanned by the cell at offset (-x, -y)
    Spanned(u32, u32),
    /// This slot is spanned by two cells at the given negative coordinate offsets. Oops.
    DoubleSpanned((u32, u32), (u32, u32)),
}

#[derive(Debug, Serialize)]
pub(crate) struct TableCellBox {}

struct TableContainerBuilder<'a, Node> {
    context: &'a LayoutContext<'a>,
    info: &'a NodeAndStyleInfo<Node>,
    slots: TableSlots,
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
        }
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
                    let mut row_builder = TableRowBuilder::new(self);
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
}

impl<'a, 'builder, Node> TableRowBuilder<'a, 'builder, Node> {
    fn new(builder: &'builder mut TableContainerBuilder<'a, Node>) -> Self {
        TableRowBuilder { builder }
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
                DisplayInternal::TableCell => {
                    // XXXManishearth handle cells
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
