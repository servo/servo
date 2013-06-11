/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's experimental layout system builds a tree of `FlowContext` and `RenderBox` objects and
/// solves layout constraints to obtain positions and display attributes of tree nodes. Positions
/// are computed in several tree traversals driven by the fundamental data dependencies required by
/// inline and block layout.
/// 
/// Flows are interior nodes in the layout tree and correspond closely to *flow contexts* in the
/// CSS specification. Flows are responsible for positioning their child flow contexts and render
/// boxes. Flows have purpose-specific fields, such as auxiliary line box structs, out-of-flow
/// child lists, and so on.
///
/// Currently, the important types of flows are:
/// 
/// * `BlockFlow`: A flow that establishes a block context. It has several child flows, each of
///   which are positioned according to block formatting context rules (CSS block boxes). Block
///   flows also contain a single `GenericBox` to represent their rendered borders, padding, etc.
///   (In the future, this render box may be folded into `BlockFlow` to save space.) The BlockFlow
///   at the root of the tree has special behavior: it stretches to the boundaries of the viewport.
///   
/// * `InlineFlow`: A flow that establishes an inline context. It has a flat list of child
///   boxes/flows that are subject to inline layout and line breaking and structs to represent
///   line breaks and mapping to CSS boxes, for the purpose of handling `getClientRects()` and
///   similar methods.

use layout::block::BlockFlowData;
use layout::box::RenderBox;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::inline::{InlineFlowData};

use core::cell::Cell;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use script::dom::node::{AbstractNode, LayoutView};
use servo_util::tree::{TreeNode, TreeNodeRef, TreeUtils};

/// The type of the formatting context and data specific to each context, such as line box
/// structures or float lists.
pub enum FlowContext {
    AbsoluteFlow(@mut FlowData), 
    BlockFlow(@mut BlockFlowData),
    FloatFlow(@mut FlowData),
    InlineBlockFlow(@mut FlowData),
    InlineFlow(@mut InlineFlowData),
    TableFlow(@mut FlowData),
}

pub enum FlowContextType {
    Flow_Absolute, 
    Flow_Block,
    Flow_Float,
    Flow_InlineBlock,
    Flow_Inline,
    Flow_Root,
    Flow_Table
}

impl Clone for FlowContext {
    fn clone(&self) -> FlowContext {
        *self
    }
}

impl FlowContext {
    pub fn teardown(&self) {
        match *self {
          AbsoluteFlow(data) |
          FloatFlow(data) |
          InlineBlockFlow(data) |
          TableFlow(data) => data.teardown(),
          BlockFlow(data) => data.teardown(),
          InlineFlow(data) => data.teardown()
        }
    }
}

impl FlowData {
    pub fn teardown(&mut self) {
        // Under the assumption that all flows exist in a tree,
        // we must restrict ourselves to finalizing flows that
        // are descendents and subsequent siblings to ourselves,
        // or we risk dynamic borrow failures.
        self.parent = None;

        for self.first_child.each |flow| {
            flow.teardown();
        }
        self.first_child = None;

        self.last_child = None;

        for self.next_sibling.each |flow| {
            flow.teardown();
        }
        self.next_sibling = None;

        self.prev_sibling = None;
    }
}

impl TreeNodeRef<FlowData> for FlowContext {
    fn with_base<R>(&self, callback: &fn(&FlowData) -> R) -> R {
        match *self {
            AbsoluteFlow(info) => callback(info),
            BlockFlow(info) => {
                callback(&info.common)
            }
            FloatFlow(info) => callback(info),
            InlineBlockFlow(info) => callback(info),
            InlineFlow(info) => {
                callback(&info.common)
            }
            TableFlow(info) => callback(info)
        }
    }
    fn with_mut_base<R>(&self, callback: &fn(&mut FlowData) -> R) -> R {
        match *self {
            AbsoluteFlow(info) => callback(info),
            BlockFlow(info) => {
                callback(&mut info.common)
            }
            FloatFlow(info) => callback(info),
            InlineBlockFlow(info) => callback(info),
            InlineFlow(info) => {
                callback(&mut info.common)
            }
            TableFlow(info) => callback(info),
        }
    }
}

/// Data common to all flows.
///
/// FIXME: We need a naming convention for pseudo-inheritance like this. How about
/// `CommonFlowInfo`?
pub struct FlowData {
    node: AbstractNode<LayoutView>,

    parent: Option<FlowContext>,
    first_child: Option<FlowContext>,
    last_child: Option<FlowContext>,
    prev_sibling: Option<FlowContext>,
    next_sibling: Option<FlowContext>,

    /* TODO (Issue #87): debug only */
    id: int,

    /* layout computations */
    // TODO: min/pref and position are used during disjoint phases of
    // layout; maybe combine into a single enum to save space.
    min_width: Au,
    pref_width: Au,
    position: Rect<Au>,
}

impl TreeNode<FlowContext> for FlowData {
    fn parent_node(&self) -> Option<FlowContext> {
        self.parent
    }

    fn first_child(&self) -> Option<FlowContext> {
        self.first_child
    }

    fn last_child(&self) -> Option<FlowContext> {
        self.last_child
    }

    fn prev_sibling(&self) -> Option<FlowContext> {
        self.prev_sibling
    }

    fn next_sibling(&self) -> Option<FlowContext> {
        self.next_sibling
    }

    fn set_parent_node(&mut self, new_parent_node: Option<FlowContext>) {
        self.parent = new_parent_node
    }

    fn set_first_child(&mut self, new_first_child: Option<FlowContext>) {
        self.first_child = new_first_child
    }

    fn set_last_child(&mut self, new_last_child: Option<FlowContext>) {
        self.last_child = new_last_child
    }

    fn set_prev_sibling(&mut self, new_prev_sibling: Option<FlowContext>) {
        self.prev_sibling = new_prev_sibling
    }

    fn set_next_sibling(&mut self, new_next_sibling: Option<FlowContext>) {
        self.next_sibling = new_next_sibling
    }
}

impl FlowData {
    pub fn new(id: int, node: AbstractNode<LayoutView>) -> FlowData {
        FlowData {
            node: node,

            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,

            id: id,

            min_width: Au(0),
            pref_width: Au(0),
            position: Au::zero_rect(),
        }
    }
}

impl<'self> FlowContext {
    /// A convenience method to return the position of this flow. Fails if the flow is currently
    /// being borrowed mutably.
    #[inline(always)]
    pub fn position(&self) -> Rect<Au> {
        do self.with_base |common_info| {
            common_info.position
        }
    }

    /// A convenience method to return the ID of this flow. Fails if the flow is currently being
    /// borrowed mutably.
    #[inline(always)]
    pub fn id(&self) -> int {
        do self.with_base |info| {
            info.id
        }
    }

    pub fn inline(&self) -> @mut InlineFlowData {
        match *self {
            InlineFlow(info) => info,
            _ => fail!(fmt!("Tried to access inline data of non-inline: f%d", self.id()))
        }
    }

    pub fn block(&self) -> @mut BlockFlowData {
        match *self {
            BlockFlow(info) => info,
            _ => fail!(fmt!("Tried to access block data of non-block: f%d", self.id()))
        }
    }

    pub fn root(&self) -> @mut BlockFlowData {
        match *self {
            BlockFlow(info) if info.is_root => info,
            _ => fail!(fmt!("Tried to access root block data of non-root: f%d", self.id()))
        }
    }

    pub fn bubble_widths(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(info)  => info.bubble_widths_block(ctx),
            InlineFlow(info) => info.bubble_widths_inline(ctx),
            _ => fail!(fmt!("Tried to bubble_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_widths(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(info)  => info.assign_widths_block(ctx),
            InlineFlow(info) => info.assign_widths_inline(ctx),
            _ => fail!(fmt!("Tried to assign_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_height(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(info)  => info.assign_height_block(ctx),
            InlineFlow(info) => info.assign_height_inline(ctx),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.id()))
        }
    }

    pub fn build_display_list_recurse<E:ExtraDisplayListData>(&self,
                                                              builder: &DisplayListBuilder,
                                                              dirty: &Rect<Au>,
                                                              offset: &Point2D<Au>,
                                                              list: &Cell<DisplayList<E>>) {
        do self.with_base |info| {
            debug!("FlowContext::build_display_list at %?: %s", info.position, self.debug_str());
        }

        match *self {
            BlockFlow(info)  => info.build_display_list_block(builder, dirty, offset, list),
            InlineFlow(info) => info.build_display_list_inline(builder, dirty, offset, list),
            _ => fail!(fmt!("Tried to build_display_list_recurse of flow: %?", self))
        }
    }

    // Actual methods that do not require much flow-specific logic
    pub fn foldl_all_boxes<B:Copy>(&self, seed: B, cb: &fn(a: B, b: RenderBox) -> B) -> B {
        match *self {
            BlockFlow(block) => {
                let block = &mut *block;
                do block.box.map_default(seed) |box| {
                    cb(seed, *box)
                }
            }
            InlineFlow(inline) => {
                let inline = &mut *inline;
                do inline.boxes.foldl(seed) |acc, box| {
                    cb(*acc, *box)
                }
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self)),
        }
    }

    pub fn foldl_boxes_for_node<B:Copy>(&self,
                                        node: AbstractNode<LayoutView>,
                                        seed: B,
                                        callback: &fn(a: B, RenderBox) -> B)
                                        -> B {
        do self.foldl_all_boxes(seed) |acc, box| {
            if box.node() == node {
                callback(acc, box)
            } else {
                acc
            }
        }
    }

    pub fn iter_all_boxes(&self, cb: &fn(RenderBox) -> bool) -> bool {
        match *self {
            BlockFlow(block) => {
                let block = &mut *block;
                for block.box.each |box| {
                    if !cb(*box) {
                        break;
                    }
                }
            }
            InlineFlow(inline) => {
                let inline = &mut *inline;
                for inline.boxes.each |box| {
                    if !cb(*box) {
                        break;
                    }
                }
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self))
        }

        true
    }

    pub fn iter_boxes_for_node(&self,
                               node: AbstractNode<LayoutView>,
                               callback: &fn(RenderBox) -> bool)
                               -> bool {
        for self.iter_all_boxes |box| {
            if box.node() == node {
                if !callback(box) {
                    break;
                }
            }
        }

        true
    }

    /// Dumps the flow tree for debugging.
    pub fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the flow tree, for debugging, with indentation.
    pub fn dump_indent(&self, indent: uint) {
        let mut s = ~"|";
        for uint::range(0, indent) |_i| {
            s += ~"---- ";
        }

        s += self.debug_str();
        debug!("%s", s);

        // FIXME: this should have a pure/const version?
        for self.each_child |child| {
            child.dump_indent(indent + 1)
        }
    }
    
    pub fn debug_str(&self) -> ~str {
        let repr = match *self {
            InlineFlow(inline) => {
                let mut s = inline.boxes.foldl(~"InlineFlow(children=", |s, box| {
                    fmt!("%s b%d", *s, box.id())
                });
                s += ~")";
                s
            },
            BlockFlow(block) => {
                match block.box {
                    Some(box) => fmt!("BlockFlow(box=b%d)", box.id()),
                    None => ~"BlockFlow",
                }
            },
            _ => ~"(Unknown flow)"
        };

        do self.with_base |base| {
            fmt!("f%? %? size %?", base.id, repr, base.position)
        }
    }
}

