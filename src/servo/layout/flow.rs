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
/// * `BlockFlow`: a flow that establishes a block context. It has several child flows, each of
///   which are positioned according to block formatting context rules (CSS block boxes). Block
///   flows also contain a single `GenericBox` to represent their rendered borders, padding, etc.
///   (In the future, this render box may be folded into `BlockFlow` to save space.)
///
/// * `InlineFlow`: a flow that establishes an inline context. It has a flat list of child
///   boxes/flows that are subject to inline layout and line breaking and structs to represent
///   line breaks and mapping to CSS boxes, for the purpose of handling `getClientRects()` and
///   similar methods.

use dom::node::AbstractNode;
use layout::block::{BlockFlowData, BlockLayout};
use layout::box::RenderBox;
use layout::context::LayoutContext;
use layout::debug::DebugMethods;
use layout::display_list_builder::DisplayListBuilder;
use layout::inline::{InlineFlowData, InlineLayout};
use layout::root::{RootFlowData, RootLayout};

use core::cell::Cell;
use core::ptr;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;

/// The type of the formatting context and data specific to each context, such as line box
/// structures or float lists.
pub enum FlowContext {
    AbsoluteFlow(@mut FlowData), 
    BlockFlow(@mut BlockFlowData),
    FloatFlow(@mut FlowData),
    InlineBlockFlow(@mut FlowData),
    InlineFlow(@mut InlineFlowData),
    RootFlow(@mut RootFlowData),
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

/// Data common to all flows.
///
/// FIXME: We need a naming convention for pseudo-inheritance like this. How about
/// `CommonFlowInfo`?
pub struct FlowData {
    node: AbstractNode,

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

impl FlowData {
    pub fn new(id: int, node: AbstractNode) -> FlowData {
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
    #[inline(always)]
    pub fn with_common_info<R>(&self, block: &fn(&mut FlowData) -> R) -> R {
        match *self {
            AbsoluteFlow(info) => block(info),
            BlockFlow(info) => {
                let info = &mut *info;  // FIXME: Borrow check workaround.
                block(&mut info.common)
            }
            FloatFlow(info) => block(info),
            InlineBlockFlow(info) => block(info),
            InlineFlow(info) => {
                let info = &mut *info;  // FIXME: Borrow check workaround.
                block(&mut info.common)
            }
            RootFlow(info) => {
                let info = &mut *info;  // FIXME: Borrow check workaround.
                block(&mut info.common)
            }
            TableFlow(info) => block(info),
        }
    }

    pub fn position(&self) -> Rect<Au> {
        do self.with_common_info |common_info| {
            common_info.position
        }
    }

    /// Returns the ID of this flow.
    #[inline(always)]
    pub fn id(&self) -> int {
        do self.with_common_info |info| {
            info.id
        }
    }

    /// Iterates over the immediate children of this flow.
    ///
    /// TODO: Fold me into `util::tree`.
    pub fn each_child(&self, f: &fn(FlowContext) -> bool) {
        let mut current_opt = self.with_common_info(|info| info.first_child);
        while !current_opt.is_none() {
            let current = current_opt.get();
            if !f(current) {
                break;
            }
            current_opt = current.with_common_info(|info| info.next_sibling);
        }
    }

    /// Adds the given flow to the end of the list of this flow's children. The new child must be
    /// detached from the tree before calling this method.
    ///
    /// TODO: Fold me into `util::tree`.
    pub fn add_child(&self, child: FlowContext) {
        do self.with_common_info |self_info| {
            do child.with_common_info |child_info| {
                assert!(child_info.parent.is_none());
                assert!(child_info.prev_sibling.is_none());
                assert!(child_info.next_sibling.is_none());

                match self_info.last_child {
                    None => {
                        self_info.first_child = Some(child);
                    }
                    Some(last_child) => {
                        do last_child.with_common_info |last_child_info| {
                            assert!(last_child_info.next_sibling.is_none());
                            last_child_info.next_sibling = Some(child);
                            child_info.prev_sibling = Some(last_child);
                        }
                    }
                }

                self_info.last_child = Some(child);
                child_info.parent = Some(*self);
            }
        }
    }

    /// Removes the given flow from the tree.
    ///
    /// TODO: Fold me into `util::tree`.
    pub fn remove_child(&self, child: FlowContext) {
        do self.with_common_info |self_info| {
            do child.with_common_info |child_info| {
                assert!(child_info.parent.is_some());

                match child_info.prev_sibling {
                    None => self_info.first_child = child_info.next_sibling,
                    Some(prev_sibling) => {
                        do prev_sibling.with_common_info |prev_sibling_info| {
                            prev_sibling_info.next_sibling = child_info.next_sibling;
                            child_info.prev_sibling = None;
                        }
                    }
                }

                match child_info.next_sibling {
                    None => {
                        do child.with_common_info |child_info| {
                            self_info.last_child = child_info.prev_sibling;
                        }
                    }
                    Some(next_sibling) => {
                        do next_sibling.with_common_info |next_sibling_info| {
                            next_sibling_info.prev_sibling = Some(next_sibling);
                            child_info.next_sibling = None;
                        }
                    }
                }

                child_info.parent = None;
            }
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

    pub fn root(&self) -> @mut RootFlowData {
        match *self {
            RootFlow(info) => info,
            _ => fail!(fmt!("Tried to access root data of non-root: f%d", self.id()))
        }
    }

    pub fn bubble_widths(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(*)     => self.bubble_widths_block(ctx),
            InlineFlow(info) => info.bubble_widths_inline(ctx),
            RootFlow(info)   => info.bubble_widths_root(ctx),
            _ => fail!(fmt!("Tried to bubble_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_widths(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(*)     => self.assign_widths_block(ctx),
            InlineFlow(info) => info.assign_widths_inline(ctx),
            RootFlow(info)   => info.assign_widths_root(ctx),
            _ => fail!(fmt!("Tried to assign_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_height(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(*)     => self.assign_height_block(ctx),
            InlineFlow(info) => info.assign_height_inline(ctx),
            RootFlow(info)   => info.assign_height_root(ctx),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.id()))
        }
    }

    pub fn build_display_list_recurse(&self,
                                      builder: &DisplayListBuilder,
                                      dirty: &Rect<Au>,
                                      offset: &Point2D<Au>,
                                      list: &Cell<DisplayList>) {
        do self.with_common_info |info| {
            debug!("FlowContext::build_display_list at %?: %s", info.position, self.debug_str());
        }

        match *self {
            RootFlow(info) => info.build_display_list_root(builder, dirty, offset, list),
            BlockFlow(*) => self.build_display_list_block(builder, dirty, offset, list),
            InlineFlow(info) => info.build_display_list_inline(builder, dirty, offset, list),
            _ => fail!(fmt!("Tried to build_display_list_recurse of flow: %?", self))
        }
    }

    // Actual methods that do not require much flow-specific logic
    pub fn foldl_all_boxes<B:Copy>(&self, seed: B, cb: &fn(a: B, b: @mut RenderBox) -> B) -> B {
        match *self {
            RootFlow(root) => {
                let root = &mut *root;
                root.box.map_default(seed, |box| { cb(seed, *box) })
            }
            BlockFlow(block) => {
                let block = &mut *block;
                block.box.map_default(seed, |box| { cb(seed, *box) })
            }
            InlineFlow(inline) => {
                let inline = &mut *inline;
                inline.boxes.foldl(seed, |acc, box| { cb(*acc, *box) })
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self))
        }
    }

    pub fn foldl_boxes_for_node<B:Copy>(&self,
                                        node: AbstractNode,
                                        seed: B,
                                        callback: &fn(a: B, @mut RenderBox) -> B)
                                        -> B {
        do self.foldl_all_boxes(seed) |acc, box| {
            if box.d().node == node {
                callback(acc, box)
            } else {
                acc
            }
        }
    }

    pub fn iter_all_boxes(&self, cb: &fn(@mut RenderBox) -> bool) {
        match *self {
            RootFlow(root) => {
                let root = &mut *root;
                for root.box.each |box| {
                    if !cb(*box) {
                        break;
                    }
                }
            }
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
    }

    pub fn iter_boxes_for_node(&self, node: AbstractNode, callback: &fn(@mut RenderBox) -> bool) {
        for self.iter_all_boxes |box| {
            if box.d().node == node {
                if !callback(box) {
                    break;
                }
            }
        }
    }
}

impl DebugMethods for FlowContext {
    fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the flow tree, for debugging, with indentation.
    fn dump_indent(&self, indent: uint) {
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
    
    fn debug_str(&self) -> ~str {
        let repr = match *self {
            InlineFlow(inline) => {
                let inline = &mut *inline;
                let mut s = inline.boxes.foldl(~"InlineFlow(children=", |s, box| {
                    fmt!("%s b%d", *s, box.d().id)
                });
                s += ~")";
                s
            },
            BlockFlow(block) => {
                let block = &mut *block;
                match block.box {
                    Some(box) => fmt!("BlockFlow(box=b%d)", box.d().id),
                    None => ~"BlockFlow",
                }
            },
            RootFlow(root) => {
                let root = &mut *root;
                match root.box {
                    Some(box) => fmt!("RootFlo(box=b%d)", box.d().id),
                    None => ~"RootFlow",
                }
            },
            _ => ~"(Unknown flow)"
        };

        do self.with_common_info |info| {
            fmt!("f%? %?", info.id, repr)
        }
    }
}

