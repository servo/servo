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
use layout::debug::BoxedMutDebugMethods;
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
    AbsoluteFlow(FlowData), 
    BlockFlow(FlowData, BlockFlowData),
    FloatFlow(FlowData),
    InlineBlockFlow(FlowData),
    InlineFlow(FlowData, InlineFlowData),
    RootFlow(FlowData, RootFlowData),
    TableFlow(FlowData)
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

/* A particular kind of layout context. It manages the positioning of
   render boxes within the context.  */
pub struct FlowData {
    node: AbstractNode,

    parent: Option<@mut FlowContext>,
    first_child: Option<@mut FlowContext>,
    last_child: Option<@mut FlowContext>,
    prev_sibling: Option<@mut FlowContext>,
    next_sibling: Option<@mut FlowContext>,

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
    pub fn d(&'self mut self) -> &'self mut FlowData {
        unsafe {
            match *self {
                AbsoluteFlow(ref d)    => cast::transmute(d),
                BlockFlow(ref d, _)    => cast::transmute(d),
                FloatFlow(ref d)       => cast::transmute(d),
                InlineBlockFlow(ref d) => cast::transmute(d),
                InlineFlow(ref d, _)   => cast::transmute(d),
                RootFlow(ref d, _)     => cast::transmute(d),
                TableFlow(ref d)       => cast::transmute(d)
            }
        }
    }

    /// Iterates over the immediate children of this flow.
    ///
    /// TODO: Fold me into `util::tree`.
    pub fn each_child(@mut self, f: &fn(@mut FlowContext) -> bool) {
        let mut current_opt = self.d().first_child;
        while !current_opt.is_none() {
            let current = current_opt.get();
            if !f(current) {
                break;
            }
            current_opt = current.d().next_sibling;
        }
    }

    /// Adds the given flow to the end of the list of this flow's children. The new child must be
    /// detached from the tree before calling this method.
    ///
    /// TODO: Fold me into `util::tree`.
    pub fn add_child(@mut self, child: @mut FlowContext) {
        let self_data = self.d(), child_data = child.d();

        assert!(child_data.parent.is_none());
        assert!(child_data.prev_sibling.is_none());
        assert!(child_data.next_sibling.is_none());

        match self_data.last_child {
            None => {
                self_data.first_child = Some(child);
            }
            Some(last_child) => {
                assert!(last_child.d().next_sibling.is_none());
                last_child.d().next_sibling = Some(child);
                child_data.prev_sibling = Some(last_child);
            }
        }

        self_data.last_child = Some(child);
        child_data.parent = Some(self);
    }

    /// Removes the given flow from the tree.
    ///
    /// TODO: Fold me into `util::tree`.
    pub fn remove_child(@mut self, child: @mut FlowContext) {
        let self_data = self.d(), child_data = child.d();

        assert!(child_data.parent.is_some());
        assert!(ptr::ref_eq(&*child_data.parent.get(), self));

        match child_data.prev_sibling {
            None => self_data.first_child = child_data.next_sibling,
            Some(prev_sibling) => {
                prev_sibling.d().next_sibling = child_data.next_sibling;
                child_data.prev_sibling = None;
            }
        }

        match child_data.next_sibling {
            None => self_data.last_child = child.d().prev_sibling,
            Some(next_sibling) => {
                next_sibling.d().prev_sibling = Some(next_sibling);
                child_data.next_sibling = None;
            }
        }

        child_data.parent = None;
    }

    pub fn inline(&'self mut self) -> &'self mut InlineFlowData {
        match self {
            &InlineFlow(_, ref i) => unsafe { cast::transmute(i) },
            _ => fail!(fmt!("Tried to access inline data of non-inline: f%d", self.d().id))
        }
    }

    pub fn block(&'self mut self) -> &'self mut BlockFlowData {
        match self {
            &BlockFlow(_, ref mut b) => unsafe { cast::transmute(b) },
            _ => fail!(fmt!("Tried to access block data of non-block: f%d", self.d().id))
        }
    }

    pub fn root(&'self mut self) -> &'self mut RootFlowData {
        match self {
            &RootFlow(_, ref r) => unsafe { cast::transmute(r) },
            _ => fail!(fmt!("Tried to access root data of non-root: f%d", self.d().id))
        }
    }

    pub fn bubble_widths(@mut self, ctx: &mut LayoutContext) {
        match self {
            @BlockFlow(*)  => self.bubble_widths_block(ctx),
            @InlineFlow(*) => self.bubble_widths_inline(ctx),
            @RootFlow(*)   => self.bubble_widths_root(ctx),
            _ => fail!(fmt!("Tried to bubble_widths of flow: f%d", self.d().id))
        }
    }

    pub fn assign_widths(@mut self, ctx: &mut LayoutContext) {
        match self {
            @BlockFlow(*)  => self.assign_widths_block(ctx),
            @InlineFlow(*) => self.assign_widths_inline(ctx),
            @RootFlow(*)   => self.assign_widths_root(ctx),
            _ => fail!(fmt!("Tried to assign_widths of flow: f%d", self.d().id))
        }
    }

    pub fn assign_height(@mut self, ctx: &mut LayoutContext) {
        match self {
            @BlockFlow(*)  => self.assign_height_block(ctx),
            @InlineFlow(*) => self.assign_height_inline(ctx),
            @RootFlow(*)   => self.assign_height_root(ctx),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.d().id))
        }
    }

    pub fn build_display_list_recurse(@mut self,
                                      builder: &DisplayListBuilder,
                                      dirty: &Rect<Au>,
                                      offset: &Point2D<Au>,
                                      list: &Cell<DisplayList>) {
        let d = self.d(); // FIXME: borrow checker workaround
        debug!("FlowContext::build_display_list at %?: %s", d.position, self.debug_str());

        match self {
            @RootFlow(*) => self.build_display_list_root(builder, dirty, offset, list),
            @BlockFlow(*) => self.build_display_list_block(builder, dirty, offset, list),
            @InlineFlow(*) => self.build_display_list_inline(builder, dirty, offset, list),
            _ => fail!(fmt!("Tried to build_display_list_recurse of flow: %?", self))
        }
    }

    // Actual methods that do not require much flow-specific logic
    pub fn foldl_all_boxes<B:Copy>(&mut self,
                                   seed: B,
                                   cb: &fn(a: B, b: @mut RenderBox) -> B)
                                   -> B {
        match self {
            &RootFlow(*)   => {
                let root = self.root(); // FIXME: borrow checker workaround
                root.box.map_default(seed, |box| { cb(seed, *box) })
            }
            &BlockFlow(*)  => {
                let block = self.block(); // FIXME: borrow checker workaround
                block.box.map_default(seed, |box| { cb(seed, *box) })
            }
            &InlineFlow(*) => {
                let inline = self.inline(); // FIXME: borrow checker workaround
                inline.boxes.foldl(seed, |acc, box| { cb(*acc, *box) })
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self))
        }
    }

    pub fn foldl_boxes_for_node<B:Copy>(&mut self,
                                        node: AbstractNode,
                                        seed: B,
                                        cb: &fn(a: B, @mut RenderBox) -> B)
                                        -> B {
        do self.foldl_all_boxes(seed) |acc, box| {
            if box.d().node == node { cb(acc, box) }
            else { acc }
        }
    }

    pub fn iter_all_boxes(&mut self, cb: &fn(@mut RenderBox) -> bool) {
        match self {
            &RootFlow(*)   => {
                let root = self.root(); // FIXME: borrow checker workaround
                for root.box.each |box| {
                    if !cb(*box) {
                        break;
                    }
                }
            }
            &BlockFlow(*)  => {
                let block = self.block(); // FIXME: borrow checker workaround
                for block.box.each |box| {
                    if !cb(*box) {
                        break;
                    }
                }
            }
            &InlineFlow(*) => {
                let inline = self.inline(); // FIXME: borrow checker workaround
                for inline.boxes.each |box| {
                    if !cb(*box) {
                        break;
                    }
                }
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self))
        }
    }

    pub fn iter_boxes_for_node(&mut self, node: AbstractNode, cb: &fn(@mut RenderBox) -> bool) {
        for self.iter_all_boxes |box| {
            if box.d().node == node {
                if !cb(box) {
                    break;
                }
            }
        }
    }
}

impl BoxedMutDebugMethods for FlowContext {
    fn dump(@mut self) {
        self.dump_indent(0);
    }

    /// Dumps the flow tree, for debugging, with indentation.
    fn dump_indent(@mut self, indent: uint) {
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
    
    fn debug_str(@mut self) -> ~str {
        let repr = match *self {
            InlineFlow(*) => {
                let inline = self.inline(); // FIXME: borrow checker workaround
                let mut s = inline.boxes.foldl(~"InlineFlow(children=", |s, box| {
                    fmt!("%s b%d", *s, box.d().id)
                });
                s += ~")";
                s
            },
            BlockFlow(*) => {
                match self.block().box {
                    Some(box) => fmt!("BlockFlow(box=b%d)", box.d().id),
                    None => ~"BlockFlow",
                }
            },
            RootFlow(*) => {
                match self.root().box {
                    Some(box) => fmt!("RootFlo(box=b%d)", box.d().id),
                    None => ~"RootFlow",
                }
            },
            _ => ~"(Unknown flow)"
        };

        let d = self.d(); // FIXME: borrow checker workaround
        fmt!("f%? %?", d.id, repr)
    }
}

