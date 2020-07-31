use super::TableContainer;
use crate::context::LayoutContext;
use crate::dom_traversal::{
    BoxSlot, Contents, NodeAndStyleInfo, NodeExt, NonReplacedContents, TraversalHandler,
};
use crate::style_ext::{DisplayInside, DisplayGeneratingBox};
use std::borrow::Cow;
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
    slots: TableSlots
}

impl TableContainer {
    pub fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        contents: NonReplacedContents,
        // XXXManishearth is this useful?
        _propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let mut builder = TableContainerBuilder { context, info, slots: TableSlots::default() };
        contents.traverse(context, info, &mut builder);
        TableContainer {}
    }
}

impl<'a, 'dom, Node: 'dom> TraversalHandler<'dom, Node> for TableContainerBuilder<'a, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, text: Cow<'dom, str>) {
        println!("text {:?}", text);
        // TODO: this might need to be wrapped in something
    }

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {

        ::std::mem::forget(box_slot)
        // do something?
    }
}
