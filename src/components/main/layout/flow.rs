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

use css::node_style::StyledNode;
use layout::block::BlockFlow;
use layout::box::RenderBox;
use layout::context::LayoutContext;
use layout::float::FloatFlow;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::float_context::{FloatContext, Invalid};
use layout::incremental::RestyleDamage;
use layout::inline::InlineFlow;

use extra::dlist::{DList,MutDListIterator};
use extra::container::Deque;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use script::dom::node::{AbstractNode, LayoutView};
use std::cast;
use std::cell::Cell;

/// Virtual methods that make up a float context.
///
/// Note that virtual methods have a cost; we should not overuse them in Servo. Consider adding
/// methods to `ImmutableFlowUtils` or `MutableFlowUtils` before adding more methods here.
pub trait FlowContext {
    // RTTI
    //
    // TODO(pcwalton): Use Rust's RTTI, once that works.

    /// Returns the class of flow that this is.
    fn class(&self) -> FlowClass;

    /// If this is a block flow, returns the underlying object. Fails otherwise.
    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        fail!("called as_block() on a non-block flow")
    }

    /// If this is an inline flow, returns the underlying object, borrowed immutably. Fails
    /// otherwise.
    fn as_immutable_inline<'a>(&'a self) -> &'a InlineFlow {
        fail!("called as_immutable_inline() on a non-inline flow")
    }

    /// If this is an inline flow, returns the underlying object. Fails otherwise.
    fn as_inline<'a>(&'a mut self) -> &'a mut InlineFlow {
        fail!("called as_inline() on a non-inline flow")
    }

    /// If this is a float flow, returns the underlying object. Fails otherwise.
    fn as_float<'a>(&'a mut self) -> &'a mut FloatFlow {
        fail!("called as_float() on a non-float flow")
    }

    // Main methods

    /// Pass 1 of reflow: computes minimum and preferred widths.
    fn bubble_widths(&mut self, _ctx: &mut LayoutContext) {
        fail!("bubble_widths not yet implemented")
    }

    /// Pass 2 of reflow: computes width.
    fn assign_widths(&mut self, _ctx: &mut LayoutContext) {
        fail!("assign_widths not yet implemented")
    }

    /// Pass 3a of reflow: computes height.
    fn assign_height(&mut self, _ctx: &mut LayoutContext) {
        fail!("assign_height not yet implemented")
    }

    /// In-order version of pass 3a of reflow: computes heights with floats present.
    fn assign_height_inorder(&mut self, _ctx: &mut LayoutContext) {
        fail!("assign_height_inorder not yet implemented")
    }

    /// Collapses margins with the parent flow. This runs as part of assign-heights.
    fn collapse_margins(&mut self,
                        _top_margin_collapsible: bool,
                        _first_in_flow: &mut bool,
                        _margin_top: &mut Au,
                        _top_offset: &mut Au,
                        _collapsing: &mut Au,
                        _collapsible: &mut Au) {
        fail!("collapse_margins not yet implemented")
    }

    /// Returns a debugging string describing this flow.
    fn debug_str(&self) -> ~str {
        ~"???"
    }
}

// Base access

#[inline(always)]
pub fn base<'a>(this: &'a FlowContext) -> &'a FlowData {
    unsafe {
        let (_, ptr): (uint, &FlowData) = cast::transmute(this);
        ptr
    }
}

#[inline(always)]
pub fn mut_base<'a>(this: &'a mut FlowContext) -> &'a mut FlowData {
    unsafe {
        let (_, ptr): (uint, &mut FlowData) = cast::transmute(this);
        ptr
    }
}

/// Returns the last child of this flow.
pub fn last_child<'a>(flow: &'a mut FlowContext) -> Option<&'a mut ~FlowContext:> {
    mut_base(flow).children.back_mut()
}

/// Iterates over the children of this flow.
pub fn child_iter<'a>(flow: &'a mut FlowContext) -> MutDListIterator<'a,~FlowContext:> {
    mut_base(flow).children.mut_iter()
}

pub trait ImmutableFlowUtils {
    // Convenience functions

    /// Returns true if this flow is a block or a float flow.
    fn is_block_like(self) -> bool;

    /// Returns true if this flow has no children.
    fn is_leaf(self) -> bool;

    /// Returns true if this flow is a block flow, an inline flow, or a float flow.
    fn starts_block_flow(self) -> bool;

    /// Returns true if this flow is an inline flow.
    fn starts_inline_flow(self) -> bool;

    /// Dumps the flow tree for debugging.
    fn dump(self);
}

pub trait MutableFlowUtils {
    // Traversals

    /// Traverses the tree in preorder.
    fn traverse_preorder<T:PreorderFlowTraversal>(self, traversal: &mut T) -> bool;

    /// Traverses the tree in postorder.
    fn traverse_postorder<T:PostorderFlowTraversal>(self, traversal: &mut T) -> bool;

    // Mutators

    /// Adds a new flow as a child of this flow.
    fn add_new_child(self, new_child: ~FlowContext:);

    /// Invokes a closure with the first child of this flow.
    fn with_first_child<R>(self, f: &fn(Option<&mut ~FlowContext:>) -> R) -> R;

    /// Invokes a closure with the last child of this flow.
    fn with_last_child<R>(self, f: &fn(Option<&mut ~FlowContext:>) -> R) -> R;

    /// Removes the first child of this flow and destroys it.
    fn remove_first(self);

    /// Removes the last child of this flow and destroys it.
    fn remove_last(self);

    /// Computes the overflow region for this flow.
    fn store_overflow(self, _: &mut LayoutContext);

    /// Builds a display list for this flow and its children.
    fn build_display_list<E:ExtraDisplayListData>(
                          self,
                          builder: &DisplayListBuilder,
                          dirty: &Rect<Au>,
                          list: &Cell<DisplayList<E>>)
                          -> bool;
}

pub enum FlowClass {
    AbsoluteFlowClass,
    BlockFlowClass,
    FloatFlowClass,
    InlineBlockFlowClass,
    InlineFlowClass,
    TableFlowClass,
}

// Miscellaneous flows that are not yet implemented.

pub struct AbsoluteFlow {
    base: FlowData,
}

impl AbsoluteFlow {
    pub fn new(base: FlowData) -> AbsoluteFlow {
        AbsoluteFlow {
            base: base,
        }
    }
}

impl FlowContext for AbsoluteFlow {
    fn class(&self) -> FlowClass {
        AbsoluteFlowClass
    }
}

pub struct InlineBlockFlow {
    base: FlowData,
}

impl InlineBlockFlow {
    pub fn new(base: FlowData) -> InlineBlockFlow {
        InlineBlockFlow {
            base: base,
        }
    }
}

impl FlowContext for InlineBlockFlow {
    fn class(&self) -> FlowClass {
        InlineBlockFlowClass
    }
}

pub struct TableFlow {
    base: FlowData,
}

impl TableFlow {
    pub fn new(base: FlowData) -> TableFlow {
        TableFlow {
            base: base,
        }
    }
}

impl FlowContext for TableFlow {
    fn class(&self) -> FlowClass {
        TableFlowClass
    }
}

/// A top-down traversal.
pub trait PreorderFlowTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&mut self, flow: &mut FlowContext) -> bool;

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune(&mut self, _flow: &mut FlowContext) -> bool {
        false
    }
}

/// A bottom-up traversal, with a optional in-order pass.
pub trait PostorderFlowTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&mut self, flow: &mut FlowContext) -> bool;

    /// Returns false if this node must be processed in-order. If this returns false, we skip the
    /// operation for this node, but continue processing the descendants. This is called *after*
    /// child nodes are visited.
    fn should_process(&mut self, _flow: &mut FlowContext) -> bool {
        true
    }

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune(&mut self, _flow: &mut FlowContext) -> bool {
        false
    }
}

/// Data common to all flows.
///
/// FIXME: We need a naming convention for pseudo-inheritance like this. How about
/// `CommonFlowInfo`?
pub struct FlowData {
    node: AbstractNode<LayoutView>,
    restyle_damage: RestyleDamage,

    children: DList<~FlowContext:>,

    /* TODO (Issue #87): debug only */
    id: int,

    /* layout computations */
    // TODO: min/pref and position are used during disjoint phases of
    // layout; maybe combine into a single enum to save space.
    min_width: Au,
    pref_width: Au,

    /// The position of the upper left corner of the border box of this flow, relative to the
    /// containing block.
    position: Rect<Au>,

    /// The amount of overflow of this flow, relative to the containing block. Must include all the
    /// pixels of all the display list items for correct invalidation.
    overflow: Rect<Au>,

    floats_in: FloatContext,
    floats_out: FloatContext,
    num_floats: uint,
    abs_position: Point2D<Au>,
    is_inorder: bool,
}

pub struct BoxIterator {
    priv boxes: ~[@RenderBox],
    priv index: uint,
}

impl Iterator<@RenderBox> for BoxIterator {
    fn next(&mut self) -> Option<@RenderBox> {
        if self.index >= self.boxes.len() {
            None
        } else {
            let v = self.boxes[self.index].clone();
            self.index += 1;
            Some(v)
        }
    }
}

impl FlowData {
    pub fn new(id: int, node: AbstractNode<LayoutView>) -> FlowData {
        FlowData {
            node: node,
            restyle_damage: node.restyle_damage(),

            children: DList::new(),

            id: id,

            min_width: Au::new(0),
            pref_width: Au::new(0),
            position: Au::zero_rect(),
            overflow: Au::zero_rect(),
            floats_in: Invalid,
            floats_out: Invalid,
            num_floats: 0,
            abs_position: Point2D(Au::new(0), Au::new(0)),
            is_inorder: false
        }
    }

    pub fn child_iter<'a>(&'a mut self) -> MutDListIterator<'a,~FlowContext:> {
        self.children.mut_iter()
    }
}

impl<'self> ImmutableFlowUtils for &'self FlowContext {
    /// Returns true if this flow is a block or a float flow.
    fn is_block_like(self) -> bool {
        match self.class() {
            BlockFlowClass | FloatFlowClass => true,
            AbsoluteFlowClass | InlineBlockFlowClass | InlineFlowClass | TableFlowClass => false,
        }
    }

    /// Returns true if this flow has no children.
    fn is_leaf(self) -> bool {
        base(self).children.len() == 0
    }

    /// Returns true if this flow is a block flow, an inline-block flow, or a float flow.
    fn starts_block_flow(self) -> bool {
        match self.class() {
            BlockFlowClass | InlineBlockFlowClass | FloatFlowClass => true,
            AbsoluteFlowClass | InlineFlowClass | TableFlowClass => false,
        }
    }

    /// Returns true if this flow is a block flow, an inline flow, or a float flow.
    fn starts_inline_flow(self) -> bool {
        match self.class() {
            InlineFlowClass => true,
            AbsoluteFlowClass | BlockFlowClass | FloatFlowClass | InlineBlockFlowClass |
            TableFlowClass => false,
        }
    }

    /// Dumps the flow tree for debugging.
    fn dump(self) {
        // TODO(pcwalton): Fill this in.
    }
}

impl<'self> MutableFlowUtils for &'self mut FlowContext {
    /// Traverses the tree in preorder.
    fn traverse_preorder<T:PreorderFlowTraversal>(self, traversal: &mut T) -> bool {
        if traversal.should_prune(self) {
            return true
        }

        if !traversal.process(self) {
            return false
        }

        for kid in child_iter(self) {
            if !kid.traverse_preorder(traversal) {
                return false
            }
        }

        true
    }

    /// Traverses the tree in postorder.
    fn traverse_postorder<T:PostorderFlowTraversal>(self, traversal: &mut T) -> bool {
        if traversal.should_prune(self) {
            return true
        }

        for kid in child_iter(self) {
            if !kid.traverse_postorder(traversal) {
                return false
            }
        }

        if !traversal.should_process(self) {
            return true
        }

        traversal.process(self)
    }

    /// Adds a new flow as a child of this flow.
    fn add_new_child(self, new_child: ~FlowContext:) {
        mut_base(self).children.push_back(new_child)
    }

    /// Invokes a closure with the first child of this flow.
    fn with_first_child<R>(self, f: &fn(Option<&mut ~FlowContext:>) -> R) -> R {
        f(mut_base(self).children.front_mut())
    }

    /// Invokes a closure with the last child of this flow.
    fn with_last_child<R>(self, f: &fn(Option<&mut ~FlowContext:>) -> R) -> R {
        f(mut_base(self).children.back_mut())
    }

    /// Removes the first child of this flow and destroys it.
    fn remove_first(self) {
        let _ = mut_base(self).children.pop_front();
    }

    /// Removes the last child of this flow and destroys it.
    fn remove_last(self) {
        let _ = mut_base(self).children.pop_back();
    }

    fn store_overflow(self, _: &mut LayoutContext) {
        let my_position = mut_base(self).position;
        let mut overflow = my_position;
        for kid in mut_base(self).child_iter() {
            let mut kid_overflow = base(*kid).overflow;
            kid_overflow = kid_overflow.translate(&my_position.origin);
            overflow = overflow.union(&kid_overflow)
        }
        mut_base(self).overflow = overflow
    }

    fn build_display_list<E:ExtraDisplayListData>(
                          self,
                          builder: &DisplayListBuilder,
                          dirty: &Rect<Au>,
                          list: &Cell<DisplayList<E>>)
                          -> bool {
        debug!("FlowContext: building display list for f%?", base(self).id);
        match self.class() {
            BlockFlowClass => self.as_block().build_display_list_block(builder, dirty, list),
            InlineFlowClass => self.as_inline().build_display_list_inline(builder, dirty, list),
            FloatFlowClass => self.as_float().build_display_list_float(builder, dirty, list),
            _ => fail!("Tried to build_display_list_recurse of flow: %?", self),
        }
    }
}

