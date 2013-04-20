/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core;
use core::cell::Cell;
use dom::node::AbstractNode;
use layout::block::{BlockFlowData, BlockLayout};
use layout::box::RenderBox;
use layout::context::LayoutContext;
use layout::debug::BoxedMutDebugMethods;
use layout::display_list_builder::DisplayListBuilder;
use layout::inline::{InlineFlowData, InlineLayout};
use layout::root::{RootFlowData, RootLayout};
use util::tree;
use geom::rect::Rect;
use geom::point::Point2D;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;

/** Servo's experimental layout system builds a tree of FlowContexts
and RenderBoxes, and figures out positions and display attributes of
tree nodes. Positions are computed in several tree traversals driven
by fundamental data dependencies of inline and block layout.

Flows are interior nodes in the layout tree, and correspond closely to
flow contexts in the CSS specification. Flows are responsible for
positioning their child flow contexts and render boxes. Flows have
purpose-specific fields, such as auxilliary line box structs,
out-of-flow child lists, and so on.

Currently, the important types of flows are:

 * BlockFlow: a flow that establishes a block context. It has several
   child flows, each of which are positioned according to block
   formatting context rules (as if child flows CSS block boxes). Block
   flows also contain a single GenericBox to represent their rendered
   borders, padding, etc. (In the future, this render box may be
   folded into BlockFlow to save space.)

 * InlineFlow: a flow that establishes an inline context. It has a
   flat list of child boxes/flows that are subject to inline layout
   and line breaking, and structs to represent line breaks and mapping
   to CSS boxes, for the purpose of handling `getClientRects()`.

*/

/* The type of the formatting context, and data specific to each
context, such as linebox structures or float lists */ 
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
    node: Option<AbstractNode>,
    /* reference to parent, children flow contexts */
    tree: tree::Tree<@mut FlowContext>,
    /* TODO (Issue #87): debug only */
    id: int,

    /* layout computations */
    // TODO: min/pref and position are used during disjoint phases of
    // layout; maybe combine into a single enum to save space.
    min_width: Au,
    pref_width: Au,
    position: Rect<Au>,
}

pub fn FlowData(id: int) -> FlowData {
    FlowData {
        node: None,
        tree: tree::empty(),
        id: id,

        min_width: Au(0),
        pref_width: Au(0),
        position: Au::zero_rect()
    }
}

pub impl<'self> FlowContext {
    fn d(&'self mut self) -> &'self mut FlowData {
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

    fn inline(&'self mut self) -> &'self mut InlineFlowData {
        match self {
            &InlineFlow(_, ref i) => unsafe { cast::transmute(i) },
            _ => fail!(fmt!("Tried to access inline data of non-inline: f%d", self.d().id))
        }
    }

    fn block(&'self mut self) -> &'self mut BlockFlowData {
        match self {
            &BlockFlow(_, ref mut b) => unsafe { cast::transmute(b) },
            _ => fail!(fmt!("Tried to access block data of non-block: f%d", self.d().id))
        }
    }

    fn root(&'self mut self) -> &'self mut RootFlowData {
        match self {
            &RootFlow(_, ref r) => unsafe { cast::transmute(r) },
            _ => fail!(fmt!("Tried to access root data of non-root: f%d", self.d().id))
        }
    }

    fn bubble_widths(@mut self, ctx: &mut LayoutContext) {
        match self {
            @BlockFlow(*)  => self.bubble_widths_block(ctx),
            @InlineFlow(*) => self.bubble_widths_inline(ctx),
            @RootFlow(*)   => self.bubble_widths_root(ctx),
            _ => fail!(fmt!("Tried to bubble_widths of flow: f%d", self.d().id))
        }
    }

    fn assign_widths(@mut self, ctx: &mut LayoutContext) {
        match self {
            @BlockFlow(*)  => self.assign_widths_block(ctx),
            @InlineFlow(*) => self.assign_widths_inline(ctx),
            @RootFlow(*)   => self.assign_widths_root(ctx),
            _ => fail!(fmt!("Tried to assign_widths of flow: f%d", self.d().id))
        }
    }

    fn assign_height(@mut self, ctx: &mut LayoutContext) {
        match self {
            @BlockFlow(*)  => self.assign_height_block(ctx),
            @InlineFlow(*) => self.assign_height_inline(ctx),
            @RootFlow(*)   => self.assign_height_root(ctx),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.d().id))
        }
    }

    fn build_display_list_recurse(@mut self, builder: &DisplayListBuilder, dirty: &Rect<Au>,
                                  offset: &Point2D<Au>, list: &Cell<DisplayList>) {
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
    fn foldl_all_boxes<B: Copy>(&mut self,
                                seed: B,
                                cb: &fn(a: B, b: @mut RenderBox) -> B) -> B {
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

    fn foldl_boxes_for_node<B: Copy>(&mut self,
                                     node: AbstractNode,
                                     seed: B,
                                     cb: &fn(a: B,@mut RenderBox) -> B)
            -> B {
        do self.foldl_all_boxes(seed) |acc, box| {
            if box.d().node == node { cb(acc, box) }
            else { acc }
        }
    }

    fn iter_all_boxes<T>(&mut self, cb: &fn(@mut RenderBox) -> T) {
        match self {
            &RootFlow(*)   => {
                let root = self.root(); // FIXME: borrow checker workaround
                for root.box.each |box| { cb(*box); }
            }
            &BlockFlow(*)  => {
                let block = self.block(); // FIXME: borrow checker workaround
                for block.box.each |box| { cb(*box); }
            }
            &InlineFlow(*) => {
                let inline = self.inline(); // FIXME: borrow checker workaround
                for inline.boxes.each |box| { cb(*box); }
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self))
        }
    }

    fn iter_boxes_for_node<T>(&mut self,
                              node: AbstractNode,
                              cb: &fn(@mut RenderBox) -> T) {
        do self.iter_all_boxes |box| {
            if box.d().node == node { cb(box); }
        }
    }
}

/* The tree holding FlowContexts */
pub enum FlowTree { FlowTree }

impl FlowTree {
    fn each_child(&self, ctx: @mut FlowContext, f: &fn(box: @mut FlowContext) -> bool) {
        tree::each_child(self, &ctx, |box| f(*box) )
    }
}

impl tree::ReadMethods<@mut FlowContext> for FlowTree {
    fn with_tree_fields<R>(&self, box: &@mut FlowContext, f: &fn(&mut tree::Tree<@mut FlowContext>) -> R) -> R {
        let tree = &mut box.d().tree;
        f(tree)
    }
}

impl FlowTree {
    fn add_child(self, parent: @mut FlowContext, child: @mut FlowContext) {
        tree::add_child(&self, parent, child)
    }
}

impl tree::WriteMethods<@mut FlowContext> for FlowTree {
    fn tree_eq(&self, a: &@mut FlowContext, b: &@mut FlowContext) -> bool { core::managed::mut_ptr_eq(*a, *b) }

    fn with_tree_fields<R>(&self, box: &@mut FlowContext, f: &fn(&mut tree::Tree<@mut FlowContext>) -> R) -> R {
        let tree = &mut box.d().tree;
        f(tree)
    }
}


impl BoxedMutDebugMethods for FlowContext {
    fn dump(@mut self) {
        self.dump_indent(0u);
    }

    /** Dumps the flow tree, for debugging, with indentation. */
    fn dump_indent(@mut self, indent: uint) {
        let mut s = ~"|";
        for uint::range(0u, indent) |_i| {
            s += ~"---- ";
        }

        s += self.debug_str();
        debug!("%s", s);

        // FIXME: this should have a pure/const version?
        unsafe {
            for FlowTree.each_child(self) |child| {
                child.dump_indent(indent + 1u) 
            }
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
