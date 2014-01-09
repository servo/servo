/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's experimental layout system builds a tree of `Flow` and `Box` objects and solves
//! layout constraints to obtain positions and display attributes of tree nodes. Positions are
//! computed in several tree traversals driven by the fundamental data dependencies required by
/// inline and block layout.
/// 
/// Flows are interior nodes in the layout tree and correspond closely to *flow contexts* in the
/// CSS specification. Flows are responsible for positioning their child flow contexts and boxes.
/// Flows have purpose-specific fields, such as auxiliary line box structs, out-of-flow child
/// lists, and so on.
///
/// Currently, the important types of flows are:
/// 
/// * `BlockFlow`: A flow that establishes a block context. It has several child flows, each of
///   which are positioned according to block formatting context rules (CSS block boxes). Block
///   flows also contain a single `GenericBox` to represent their rendered borders, padding, etc.
///   The BlockFlow at the root of the tree has special behavior: it stretches to the boundaries of
///   the viewport.
///   
/// * `InlineFlow`: A flow that establishes an inline context. It has a flat list of child
///   boxes/flows that are subject to inline layout and line breaking and structs to represent
///   line breaks and mapping to CSS boxes, for the purpose of handling `getClientRects()` and
///   similar methods.

use css::node_style::StyledNode;
use layout::block::BlockFlow;
use layout::box_::Box;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::float_context::{FloatContext, Invalid};
use layout::incremental::RestyleDamage;
use layout::inline::InlineFlow;
use layout::wrapper::LayoutNode;

use extra::dlist::{DList, DListIterator, MutDListIterator};
use extra::container::Deque;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::{ClipDisplayItemClass, DisplayList};
use servo_util::geometry::Au;
use std::cast;
use std::cell::RefCell;
use style::ComputedValues;
use style::computed_values::text_align;

/// Virtual methods that make up a float context.
///
/// Note that virtual methods have a cost; we should not overuse them in Servo. Consider adding
/// methods to `ImmutableFlowUtils` or `MutableFlowUtils` before adding more methods here.
pub trait Flow {
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

    /// Marks this flow as the root flow. The default implementation is a no-op.
    fn mark_as_root(&mut self) {}

    /// Returns a debugging string describing this flow.
    fn debug_str(&self) -> ~str {
        ~"???"
    }
}

// Base access

#[inline(always)]
pub fn base<'a>(this: &'a Flow) -> &'a BaseFlow {
    unsafe {
        let (_, ptr): (uint, &BaseFlow) = cast::transmute(this);
        ptr
    }
}

/// Iterates over the children of this immutable flow.
pub fn imm_child_iter<'a>(flow: &'a Flow) -> DListIterator<'a,~Flow> {
    base(flow).children.iter()
}

#[inline(always)]
pub fn mut_base<'a>(this: &'a mut Flow) -> &'a mut BaseFlow {
    unsafe {
        let (_, ptr): (uint, &mut BaseFlow) = cast::transmute(this);
        ptr
    }
}

/// Returns the last child of this flow.
pub fn last_child<'a>(flow: &'a mut Flow) -> Option<&'a mut ~Flow> {
    mut_base(flow).children.back_mut()
}

/// Iterates over the children of this flow.
pub fn child_iter<'a>(flow: &'a mut Flow) -> MutDListIterator<'a,~Flow> {
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

    /// Dumps the flow tree for debugging, with a prefix to indicate that we're at the given level.
    fn dump_with_level(self, level: uint);
}

pub trait MutableFlowUtils {
    // Traversals

    /// Traverses the tree in preorder.
    fn traverse_preorder<T:PreorderFlowTraversal>(self, traversal: &mut T) -> bool;

    /// Traverses the tree in postorder.
    fn traverse_postorder<T:PostorderFlowTraversal>(self, traversal: &mut T) -> bool;

    // Mutators

    /// Adds a new flow as a child of this flow.
    fn add_new_child(self, new_child: ~Flow);

    /// Invokes a closure with the first child of this flow.
    fn with_first_child<R>(self, f: |Option<&mut ~Flow>| -> R) -> R;

    /// Invokes a closure with the last child of this flow.
    fn with_last_child<R>(self, f: |Option<&mut ~Flow>| -> R) -> R;

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
                          list: &RefCell<DisplayList<E>>)
                          -> bool;
}

pub enum FlowClass {
    BlockFlowClass,
    InlineFlowClass,
}

/// A top-down traversal.
pub trait PreorderFlowTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&mut self, flow: &mut Flow) -> bool;

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune(&mut self, _flow: &mut Flow) -> bool {
        false
    }
}

/// A bottom-up traversal, with a optional in-order pass.
pub trait PostorderFlowTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&mut self, flow: &mut Flow) -> bool;

    /// Returns false if this node must be processed in-order. If this returns false, we skip the
    /// operation for this node, but continue processing the descendants. This is called *after*
    /// child nodes are visited.
    fn should_process(&mut self, _flow: &mut Flow) -> bool {
        true
    }

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune(&mut self, _flow: &mut Flow) -> bool {
        false
    }
}

/// Flags used in flows, tightly packed to save space.
pub struct FlowFlags(u8);

/// The bitmask of flags that represent text decoration fields that get propagated downward.
///
/// NB: If you update this field, you must update the bitfields below.
static TEXT_DECORATION_OVERRIDE_BITMASK: u8 = 0b00001110;

/// The bitmask of flags that represent the text alignment field.
///
/// NB: If you update this field, you must update the bitfields below.
static TEXT_ALIGN_BITMASK: u8 = 0b00110000;

/// The number of bits we must shift off to handle the text alignment field.
///
/// NB: If you update this field, you must update the bitfields below.
static TEXT_ALIGN_SHIFT: u8 = 4;

impl FlowFlags {
    /// Creates a new set of flow flags from the given style.
    fn new(style: &ComputedValues) -> FlowFlags {
        let text_decoration = style.Text.text_decoration;
        let mut flags = FlowFlags(0);
        flags.set_override_underline(text_decoration.underline);
        flags.set_override_overline(text_decoration.overline);
        flags.set_override_line_through(text_decoration.line_through);
        flags
    }

    /// Propagates text decoration flags from an appropriate parent flow per CSS 2.1 § 16.3.1.
    pub fn propagate_text_decoration_from_parent(&mut self, parent: FlowFlags) {
        *self = FlowFlags(**self | (*parent & TEXT_DECORATION_OVERRIDE_BITMASK))
    }

    /// Propagates text alignment flags from an appropriate parent flow per CSS 2.1.
    pub fn propagate_text_alignment_from_parent(&mut self, parent: FlowFlags) {
        *self = FlowFlags(**self | (*parent & TEXT_ALIGN_BITMASK))
    }
}

// Whether we need an in-order traversal.
bitfield!(FlowFlags, inorder, set_inorder, 0x01)

// Whether this flow forces `text-decoration: underline` on.
//
// NB: If you update this, you need to update TEXT_DECORATION_OVERRIDE_BITMASK.
bitfield!(FlowFlags, override_underline, set_override_underline, 0x02)

// Whether this flow forces `text-decoration: overline` on.
//
// NB: If you update this, you need to update TEXT_DECORATION_OVERRIDE_BITMASK.
bitfield!(FlowFlags, override_overline, set_override_overline, 0x04)

// Whether this flow forces `text-decoration: line-through` on.
//
// NB: If you update this, you need to update TEXT_DECORATION_OVERRIDE_BITMASK.
bitfield!(FlowFlags, override_line_through, set_override_line_through, 0x08)

// The text alignment for this flow.
impl FlowFlags {
    #[inline]
    pub fn text_align(self) -> text_align::T {
        FromPrimitive::from_u8((*self & TEXT_ALIGN_BITMASK) >> TEXT_ALIGN_SHIFT).unwrap()
    }

    #[inline]
    pub fn set_text_align(&mut self, value: text_align::T) {
        *self = FlowFlags((**self & !TEXT_ALIGN_BITMASK) | ((value as u8) << TEXT_ALIGN_SHIFT))
    }
}

/// Data common to all flows.
pub struct BaseFlow {
    restyle_damage: RestyleDamage,

    /// The children of this flow.
    children: DList<~Flow>,

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

    /// Various flags for flows, tightly packed to save space.
    flags: FlowFlags,
}

pub struct BoxIterator {
    priv boxes: ~[@Box],
    priv index: uint,
}

impl Iterator<@Box> for BoxIterator {
    fn next(&mut self) -> Option<@Box> {
        if self.index >= self.boxes.len() {
            None
        } else {
            let v = self.boxes[self.index].clone();
            self.index += 1;
            Some(v)
        }
    }
}

impl BaseFlow {
    #[inline]
    pub fn new(id: int, node: LayoutNode) -> BaseFlow {
        let style = node.style();
        BaseFlow {
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

            flags: FlowFlags::new(style.get()),
        }
    }

    pub fn child_iter<'a>(&'a mut self) -> MutDListIterator<'a,~Flow> {
        self.children.mut_iter()
    }
}

impl<'a> ImmutableFlowUtils for &'a Flow {
    /// Returns true if this flow is a block or a float flow.
    fn is_block_like(self) -> bool {
        match self.class() {
            BlockFlowClass => true,
            InlineFlowClass => false,
        }
    }

    /// Returns true if this flow has no children.
    fn is_leaf(self) -> bool {
        base(self).children.len() == 0
    }

    /// Returns true if this flow is a block flow, an inline-block flow, or a float flow.
    fn starts_block_flow(self) -> bool {
        match self.class() {
            BlockFlowClass => true,
            InlineFlowClass => false,
        }
    }

    /// Returns true if this flow is a block flow, an inline flow, or a float flow.
    fn starts_inline_flow(self) -> bool {
        match self.class() {
            InlineFlowClass => true,
            BlockFlowClass => false,
        }
    }

    /// Dumps the flow tree for debugging.
    fn dump(self) {
        self.dump_with_level(0)
    }

    /// Dumps the flow tree for debugging, with a prefix to indicate that we're at the given level.
    fn dump_with_level(self, level: uint) {
        let mut indent = ~"";
        for _ in range(0, level) {
            indent.push_str("| ")
        }
        debug!("{}+ {}", indent, self.debug_str());
        for kid in imm_child_iter(self) {
            kid.dump_with_level(level + 1)
        }
    }
}

impl<'a> MutableFlowUtils for &'a mut Flow {
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
    fn add_new_child(self, new_child: ~Flow) {
        mut_base(self).children.push_back(new_child)
    }

    /// Invokes a closure with the first child of this flow.
    fn with_first_child<R>(self, f: |Option<&mut ~Flow>| -> R) -> R {
        f(mut_base(self).children.front_mut())
    }

    /// Invokes a closure with the last child of this flow.
    fn with_last_child<R>(self, f: |Option<&mut ~Flow>| -> R) -> R {
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
                          list: &RefCell<DisplayList<E>>)
                          -> bool {
        debug!("Flow: building display list for f{}", base(self).id);
        match self.class() {
            BlockFlowClass => self.as_block().build_display_list_block(builder, dirty, list),
            InlineFlowClass => self.as_inline().build_display_list_inline(builder, dirty, list),
        };

        if list.with_mut(|list| list.list.len() == 0) {
            return true;
        }

        let child_list = ~RefCell::new(DisplayList::new());
        for kid in child_iter(self) {
            kid.build_display_list(builder,dirty,child_list);
        }

        let mut child_list = Some(child_list.unwrap());
        list.with_mut(|list| {
            let result = list.list.mut_rev_iter().position(|item| {
                match *item {
                    ClipDisplayItemClass(ref mut item) => {
                        item.child_list.push_all_move(child_list.take_unwrap().list);
                        true
                    },
                    _ => false,
                }
            });

            if result.is_none() {
                fail!("fail to find parent item");
            }

        });
        true
    }
}

