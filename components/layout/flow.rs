/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's experimental layout system builds a tree of `Flow` and `Fragment` objects and solves
//! layout constraints to obtain positions and display attributes of tree nodes. Positions are
//! computed in several tree traversals driven by the fundamental data dependencies required by
//! inline and block layout.
//!
//! Flows are interior nodes in the layout tree and correspond closely to *flow contexts* in the
//! CSS specification. Flows are responsible for positioning their child flow contexts and
//! fragments. Flows have purpose-specific fields, such as auxiliary line structs, out-of-flow
//! child lists, and so on.
//!
//! Currently, the important types of flows are:
//!
//! * `BlockFlow`: A flow that establishes a block context. It has several child flows, each of
//!   which are positioned according to block formatting context rules (CSS block boxes). Block
//!   flows also contain a single box to represent their rendered borders, padding, etc.
//!   The BlockFlow at the root of the tree has special behavior: it stretches to the boundaries of
//!   the viewport.
//!
//! * `InlineFlow`: A flow that establishes an inline context. It has a flat list of child
//!   fragments/flows that are subject to inline layout and line breaking and structs to represent
//!   line breaks and mapping to CSS boxes, for the purpose of handling `getClientRects()` and
//!   similar methods.

use block::BlockFlow;
use context::LayoutContext;
use display_list_builder::DisplayListBuildingResult;
use floats::Floats;
use flow_list::{FlowList, FlowListIterator, MutFlowListIterator};
use flow_ref::{FlowRef, WeakFlowRef};
use fragment::{Fragment, FragmentBorderBoxIterator, SpecificFragmentInfo};
use incremental::{self, RECONSTRUCT_FLOW, REFLOW, REFLOW_OUT_OF_FLOW, RestyleDamage};
use inline::InlineFlow;
use model::{CollapsibleMargins, IntrinsicISizes, MarginCollapseInfo};
use multicol::MulticolFlow;
use parallel::FlowParallelInfo;
use table::{ColumnComputedInlineSize, ColumnIntrinsicInlineSize, TableFlow};
use table_caption::TableCaptionFlow;
use table_cell::TableCellFlow;
use table_colgroup::TableColGroupFlow;
use table_row::TableRowFlow;
use table_rowgroup::TableRowGroupFlow;
use table_wrapper::TableWrapperFlow;
use wrapper::{PseudoElementType, ThreadSafeLayoutNode};

use euclid::{Point2D, Rect, Size2D};
use gfx::display_list::ClippingRegion;
use msg::compositor_msg::LayerId;
use msg::constellation_msg::ConstellationChan;
use rustc_serialize::{Encoder, Encodable};
use std::fmt;
use std::iter::Zip;
use std::mem;
use std::raw;
use std::slice::IterMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use style::computed_values::{clear, display, empty_cells, float, position, text_align};
use style::properties::{self, ComputedValues};
use style::values::computed::LengthOrPercentageOrAuto;
use util::geometry::{Au, ZERO_RECT};
use util::logical_geometry::{LogicalRect, LogicalSize, WritingMode};

/// Virtual methods that make up a float context.
///
/// Note that virtual methods have a cost; we should not overuse them in Servo. Consider adding
/// methods to `ImmutableFlowUtils` or `MutableFlowUtils` before adding more methods here.
pub trait Flow: fmt::Debug + Sync {
    // RTTI
    //
    // TODO(pcwalton): Use Rust's RTTI, once that works.

    /// Returns the class of flow that this is.
    fn class(&self) -> FlowClass;

    /// If this is a block flow, returns the underlying object. Fails otherwise.
    fn as_block<'a>(&'a self) -> &'a BlockFlow {
        panic!("called as_block() on a non-block flow")
    }

    /// If this is a block flow, returns the underlying object, borrowed mutably. Fails otherwise.
    fn as_mut_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        debug!("called as_mut_block() on a flow of type {:?}", self.class());
        panic!("called as_mut_block() on a non-block flow")
    }

    /// If this is an inline flow, returns the underlying object. Fails otherwise.
    fn as_inline<'a>(&'a self) -> &'a InlineFlow {
        panic!("called as_inline() on a non-inline flow")
    }

    /// If this is an inline flow, returns the underlying object, borrowed mutably. Fails
    /// otherwise.
    fn as_mut_inline<'a>(&'a mut self) -> &'a mut InlineFlow {
        panic!("called as_mut_inline() on a non-inline flow")
    }

    /// If this is a table wrapper flow, returns the underlying object, borrowed mutably. Fails
    /// otherwise.
    fn as_mut_table_wrapper<'a>(&'a mut self) -> &'a mut TableWrapperFlow {
        panic!("called as_mut_table_wrapper() on a non-tablewrapper flow")
    }

    /// If this is a table wrapper flow, returns the underlying object. Fails otherwise.
    fn as_table_wrapper<'a>(&'a self) -> &'a TableWrapperFlow {
        panic!("called as_table_wrapper() on a non-tablewrapper flow")
    }

    /// If this is a table flow, returns the underlying object, borrowed mutably. Fails otherwise.
    fn as_mut_table<'a>(&'a mut self) -> &'a mut TableFlow {
        panic!("called as_mut_table() on a non-table flow")
    }

    /// If this is a table flow, returns the underlying object. Fails otherwise.
    fn as_table<'a>(&'a self) -> &'a TableFlow {
        panic!("called as_table() on a non-table flow")
    }

    /// If this is a table colgroup flow, returns the underlying object, borrowed mutably. Fails
    /// otherwise.
    fn as_mut_table_colgroup<'a>(&'a mut self) -> &'a mut TableColGroupFlow {
        panic!("called as_mut_table_colgroup() on a non-tablecolgroup flow")
    }

    /// If this is a table rowgroup flow, returns the underlying object, borrowed mutably. Fails
    /// otherwise.
    fn as_mut_table_rowgroup<'a>(&'a mut self) -> &'a mut TableRowGroupFlow {
        panic!("called as_mut_table_rowgroup() on a non-tablerowgroup flow")
    }

    /// If this is a table rowgroup flow, returns the underlying object. Fails otherwise.
    fn as_table_rowgroup<'a>(&'a self) -> &'a TableRowGroupFlow {
        panic!("called as_table_rowgroup() on a non-tablerowgroup flow")
    }

    /// If this is a table row flow, returns the underlying object, borrowed mutably. Fails
    /// otherwise.
    fn as_mut_table_row<'a>(&'a mut self) -> &'a mut TableRowFlow {
        panic!("called as_mut_table_row() on a non-tablerow flow")
    }

    /// If this is a table row flow, returns the underlying object. Fails otherwise.
    fn as_table_row<'a>(&'a self) -> &'a TableRowFlow {
        panic!("called as_table_row() on a non-tablerow flow")
    }

    /// If this is a table cell flow, returns the underlying object, borrowed mutably. Fails
    /// otherwise.
    fn as_mut_table_caption<'a>(&'a mut self) -> &'a mut TableCaptionFlow {
        panic!("called as_mut_table_caption() on a non-tablecaption flow")
    }

    /// If this is a table cell flow, returns the underlying object, borrowed mutably. Fails
    /// otherwise.
    fn as_mut_table_cell<'a>(&'a mut self) -> &'a mut TableCellFlow {
        panic!("called as_mut_table_cell() on a non-tablecell flow")
    }

    /// If this is a multicol flow, returns the underlying object, borrowed mutably. Fails
    /// otherwise.
    fn as_mut_multicol<'a>(&'a mut self) -> &'a mut MulticolFlow {
        panic!("called as_mut_multicol() on a non-multicol flow")
    }

    /// If this is a table cell flow, returns the underlying object. Fails otherwise.
    fn as_table_cell<'a>(&'a self) -> &'a TableCellFlow {
        panic!("called as_table_cell() on a non-tablecell flow")
    }

    /// If this is a table row, table rowgroup, or table flow, returns column intrinsic
    /// inline-sizes. Fails otherwise.
    fn column_intrinsic_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnIntrinsicInlineSize> {
        panic!("called column_intrinsic_inline_sizes() on non-table flow")
    }

    /// If this is a table row, table rowgroup, or table flow, returns column computed
    /// inline-sizes. Fails otherwise.
    fn column_computed_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnComputedInlineSize> {
        panic!("called column_intrinsic_inline_sizes() on non-table flow")
    }

    // Main methods

    /// Pass 1 of reflow: computes minimum and preferred inline-sizes.
    ///
    /// Recursively (bottom-up) determine the flow's minimum and preferred inline-sizes. When
    /// called on this flow, all child flows have had their minimum and preferred inline-sizes set.
    /// This function must decide minimum/preferred inline-sizes based on its children's inline-
    /// sizes and the dimensions of any boxes it is responsible for flowing.
    fn bubble_inline_sizes(&mut self) {
        panic!("bubble_inline_sizes not yet implemented")
    }

    /// Pass 2 of reflow: computes inline-size.
    fn assign_inline_sizes(&mut self, _ctx: &LayoutContext) {
        panic!("assign_inline_sizes not yet implemented")
    }

    /// Pass 3a of reflow: computes block-size.
    fn assign_block_size<'a>(&mut self, _ctx: &'a LayoutContext<'a>) {
        panic!("assign_block_size not yet implemented")
    }

    /// If this is a float, places it. The default implementation does nothing.
    fn place_float_if_applicable<'a>(&mut self, _: &'a LayoutContext<'a>) {}

    /// Assigns block-sizes in-order; or, if this is a float, places the float. The default
    /// implementation simply assigns block-sizes if this flow is impacted by floats. Returns true
    /// if this child was impacted by floats or false otherwise.
    ///
    /// `parent_thread_id` is the thread ID of the parent. This is used for the layout tinting
    /// debug mode; if the block size of this flow was determined by its parent, we should treat
    /// it as laid out by its parent.
    fn assign_block_size_for_inorder_child_if_necessary<'a>(&mut self,
                                                            layout_context: &'a LayoutContext<'a>,
                                                            parent_thread_id: u8)
                                                            -> bool {
        let impacted = base(self).flags.impacted_by_floats();
        if impacted {
            mut_base(self).thread_id = parent_thread_id;
            self.assign_block_size(layout_context);
            self.store_overflow(layout_context);
            mut_base(self).restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
        }
        impacted
    }

    /// Calculate and set overflow for current flow.
    ///
    /// CSS Section 11.1
    /// This is the union of rectangles of the flows for which we define the
    /// Containing Block.
    ///
    /// FIXME(pcwalton): This should not be a virtual method, but currently is due to a compiler
    /// bug ("the trait `Sized` is not implemented for `self`").
    ///
    /// Assumption: This is called in a bottom-up traversal, so kids' overflows have
    /// already been set.
    /// Assumption: Absolute descendants have had their overflow calculated.
    fn store_overflow(&mut self, _: &LayoutContext) {
        // Calculate overflow on a per-fragment basis.
        let mut overflow = self.compute_overflow();
        match self.class() {
            FlowClass::Block |
            FlowClass::TableCaption |
            FlowClass::TableCell if !base(self).children.is_empty() => {
                // FIXME(#2795): Get the real container size.
                let container_size = Size2D::zero();
                for kid in mut_base(self).children.iter_mut() {
                    if base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED) {
                        continue
                    }
                    let kid_overflow = base(kid).overflow;
                    let kid_position = base(kid).position.to_physical(base(kid).writing_mode,
                                                                      container_size);
                    overflow = overflow.union(&kid_overflow.translate(&kid_position.origin))
                }

                for kid in mut_base(self).abs_descendants.iter() {
                    let kid_overflow = base(kid).overflow;
                    let kid_position = base(kid).position.to_physical(base(kid).writing_mode,
                                                                      container_size);
                    overflow = overflow.union(&kid_overflow.translate(&kid_position.origin))
                }
            }
            _ => {}
        }
        mut_base(self).overflow = overflow;
    }

    /// Phase 4 of reflow: computes absolute positions.
    fn compute_absolute_position(&mut self, _: &LayoutContext) {
        // The default implementation is a no-op.
    }

    /// Phase 5 of reflow: builds display lists.
    fn build_display_list(&mut self, layout_context: &LayoutContext);

    /// Returns the union of all overflow rects of all of this flow's fragments.
    fn compute_overflow(&self) -> Rect<Au>;

    /// Iterates through border boxes of all of this flow's fragments.
    /// Level provides a zero based index indicating the current
    /// depth of the flow tree during fragment iteration.
    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             level: i32,
                                             stacking_context_position: &Point2D<Au>);

    /// Mutably iterates through fragments in this flow.
    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment));

    fn compute_collapsible_block_start_margin(&mut self,
                                              _layout_context: &mut LayoutContext,
                                              _margin_collapse_info: &mut MarginCollapseInfo) {
        // The default implementation is a no-op.
    }

    /// Marks this flow as the root flow. The default implementation is a no-op.
    fn mark_as_root(&mut self) {
        debug!("called mark_as_root() on a flow of type {:?}", self.class());
        panic!("called mark_as_root() on an unhandled flow");
    }

    // Note that the following functions are mostly called using static method
    // dispatch, so it's ok to have them in this trait. Plus, they have
    // different behaviour for different types of Flow, so they can't go into
    // the Immutable / Mutable Flow Utils traits without additional casts.

    /// Return true if store overflow is delayed for this flow.
    ///
    /// Currently happens only for absolutely positioned flows.
    fn is_store_overflow_delayed(&mut self) -> bool {
        false
    }

    fn is_root(&self) -> bool {
        false
    }

    /// The 'position' property of this flow.
    fn positioning(&self) -> position::T {
        position::T::static_
    }

    /// Return true if this flow has position 'fixed'.
    fn is_fixed(&self) -> bool {
        self.positioning() == position::T::fixed
    }

    fn contains_positioned_fragments(&self) -> bool {
        self.contains_relatively_positioned_fragments() ||
            base(self).flags.contains(IS_ABSOLUTELY_POSITIONED)
    }

    fn contains_relatively_positioned_fragments(&self) -> bool {
        self.positioning() == position::T::relative
    }

    /// Returns true if this is an absolute containing block.
    fn is_absolute_containing_block(&self) -> bool {
        false
    }

    /// Updates the inline position of a child flow during the assign-height traversal. At present,
    /// this is only used for absolutely-positioned inline-blocks.
    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au);

    /// Updates the block position of a child flow during the assign-height traversal. At present,
    /// this is only used for absolutely-positioned inline-blocks.
    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au);

    /// Return the size of the containing block generated by this flow for the absolutely-
    /// positioned descendant referenced by `for_flow`. For block flows, this is the padding box.
    ///
    /// NB: Do not change this `&self` to `&mut self` under any circumstances! It has security
    /// implications because this can be called on parents concurrently from descendants!
    fn generated_containing_block_size(&self, _: OpaqueFlow) -> LogicalSize<Au>;

    /// Returns a layer ID for the given fragment.
    #[allow(unsafe_code)]
    fn layer_id(&self, fragment_id: u32) -> LayerId {
        let obj = unsafe { mem::transmute::<&&Self, &raw::TraitObject>(&self) };
        LayerId(obj.data as usize, fragment_id)
    }

    /// Attempts to perform incremental fixup of this flow by replacing its fragment's style with
    /// the new style. This can only succeed if the flow has exactly one fragment.
    fn repair_style(&mut self, new_style: &Arc<ComputedValues>);

    /// Remove any compositor layers associated with this flow
    fn remove_compositor_layers(&self, _: ConstellationChan) {}
}

// Base access

#[inline(always)]
#[allow(unsafe_code)]
pub fn base<'a, T: ?Sized + Flow>(this: &'a T) -> &'a BaseFlow {
    unsafe {
        let obj = mem::transmute::<&&'a T, &'a raw::TraitObject>(&this);
        mem::transmute::<*mut (), &'a BaseFlow>(obj.data)
    }
}

/// Iterates over the children of this immutable flow.
pub fn imm_child_iter<'a>(flow: &'a Flow) -> FlowListIterator<'a> {
    base(flow).children.iter()
}

#[inline(always)]
#[allow(unsafe_code)]
pub fn mut_base<'a, T: ?Sized + Flow>(this: &'a mut T) -> &'a mut BaseFlow {
    unsafe {
        let obj = mem::transmute::<&&'a mut T, &'a raw::TraitObject>(&this);
        mem::transmute::<*mut (), &'a mut BaseFlow>(obj.data)
    }
}

/// Iterates over the children of this flow.
pub fn child_iter<'a>(flow: &'a mut Flow) -> MutFlowListIterator<'a> {
    mut_base(flow).children.iter_mut()
}

pub trait ImmutableFlowUtils {
    // Convenience functions

    /// Returns true if this flow is a block flow or subclass thereof.
    fn is_block_like(self) -> bool;

    /// Returns true if this flow is a table flow.
    fn is_table(self) -> bool;

    /// Returns true if this flow is a table caption flow.
    fn is_table_caption(self) -> bool;

    /// Returns true if this flow is a proper table child.
    fn is_proper_table_child(self) -> bool;

    /// Returns true if this flow is a table row flow.
    fn is_table_row(self) -> bool;

    /// Returns true if this flow is a table cell flow.
    fn is_table_cell(self) -> bool;

    /// Returns true if this flow is a table colgroup flow.
    fn is_table_colgroup(self) -> bool;

    /// Returns true if this flow is a table rowgroup flow.
    fn is_table_rowgroup(self) -> bool;

    /// Returns true if this flow is one of table-related flows.
    fn is_table_kind(self) -> bool;

    /// Returns true if anonymous flow is needed between this flow and child flow.
    fn need_anonymous_flow(self, child: &Flow) -> bool;

    /// Generates missing child flow of this flow.
    fn generate_missing_child_flow(self, node: &ThreadSafeLayoutNode) -> FlowRef;

    /// Returns true if this flow contains fragments that are roots of an absolute flow tree.
    fn contains_roots_of_absolute_flow_tree(&self) -> bool;

    /// Returns true if this flow has no children.
    fn is_leaf(self) -> bool;

    /// Returns the number of children that this flow possesses.
    fn child_count(self) -> usize;

    /// Return true if this flow is a Block Container.
    fn is_block_container(self) -> bool;

    /// Returns true if this flow is a block flow.
    fn is_block_flow(self) -> bool;

    /// Returns true if this flow is an inline flow.
    fn is_inline_flow(self) -> bool;

    /// Dumps the flow tree for debugging.
    fn dump(self);

    /// Dumps the flow tree for debugging, with a prefix to indicate that we're at the given level.
    fn dump_with_level(self, level: u32);
}

pub trait MutableFlowUtils {
    // Traversals

    /// Traverses the tree in preorder.
    fn traverse_preorder<T: PreorderFlowTraversal>(self, traversal: &T);

    /// Traverses the tree in postorder.
    fn traverse_postorder<T: PostorderFlowTraversal>(self, traversal: &T);

    /// Traverse the Absolute flow tree in preorder.
    ///
    /// Traverse all your direct absolute descendants, who will then traverse
    /// their direct absolute descendants.
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_preorder_absolute_flows<T>(&mut self, traversal: &mut T)
                                           where T: PreorderFlowTraversal;

    /// Traverse the Absolute flow tree in postorder.
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_postorder_absolute_flows<T>(&mut self, traversal: &mut T)
                                            where T: PostorderFlowTraversal;

    // Mutators

    /// Calls `repair_style` and `bubble_inline_sizes`. You should use this method instead of
    /// calling them individually, since there is no reason not to perform both operations.
    fn repair_style_and_bubble_inline_sizes(self, style: &Arc<ComputedValues>);
}

pub trait MutableOwnedFlowUtils {
    /// Set absolute descendants for this flow.
    ///
    /// Set this flow as the Containing Block for all the absolute descendants.
    fn set_absolute_descendants(&mut self, abs_descendants: AbsoluteDescendants);
}

#[derive(RustcEncodable, PartialEq, Debug)]
pub enum FlowClass {
    Block,
    Inline,
    ListItem,
    TableWrapper,
    Table,
    TableColGroup,
    TableRowGroup,
    TableRow,
    TableCaption,
    TableCell,
    Multicol,
}

/// A top-down traversal.
pub trait PreorderFlowTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&self, flow: &mut Flow);

    /// Returns true if this node must be processed in-order. If this returns false,
    /// we skip the operation for this node, but continue processing the descendants.
    /// This is called *after* parent nodes are visited.
    fn should_process(&self, _flow: &mut Flow) -> bool {
        true
    }
}

/// A bottom-up traversal, with a optional in-order pass.
pub trait PostorderFlowTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&self, flow: &mut Flow);

    /// Returns false if this node must be processed in-order. If this returns false, we skip the
    /// operation for this node, but continue processing the ancestors. This is called *after*
    /// child nodes are visited.
    fn should_process(&self, _flow: &mut Flow) -> bool {
        true
    }
}

/// An in-order (sequential only) traversal.
pub trait InorderFlowTraversal {
    /// The operation to perform. Returns the level of the tree we're at.
    fn process(&mut self, flow: &mut Flow, level: u32);

    /// Returns true if this node should be processed and false if neither this node nor its
    /// descendants should be processed.
    fn should_process(&mut self, flow: &mut Flow) -> bool;
}

bitflags! {
    #[doc = "Flags used in flows."]
    flags FlowFlags: u32 {
        // floated descendants flags
        #[doc = "Whether this flow has descendants that float left in the same block formatting"]
        #[doc = "context."]
        const HAS_LEFT_FLOATED_DESCENDANTS = 0b0000_0000_0000_0000_0001,
        #[doc = "Whether this flow has descendants that float right in the same block formatting"]
        #[doc = "context."]
        const HAS_RIGHT_FLOATED_DESCENDANTS = 0b0000_0000_0000_0000_0010,
        #[doc = "Whether this flow is impacted by floats to the left in the same block formatting"]
        #[doc = "context (i.e. its height depends on some prior flows with `float: left`)."]
        const IMPACTED_BY_LEFT_FLOATS = 0b0000_0000_0000_0000_0100,
        #[doc = "Whether this flow is impacted by floats to the right in the same block"]
        #[doc = "formatting context (i.e. its height depends on some prior flows with `float:"]
        #[doc = "right`)."]
        const IMPACTED_BY_RIGHT_FLOATS = 0b0000_0000_0000_0000_1000,

        // text align flags
        #[doc = "Whether this flow contains a flow that has its own layer within the same absolute"]
        #[doc = "containing block."]
        const LAYERS_NEEDED_FOR_DESCENDANTS = 0b0000_0000_0000_0001_0000,
        #[doc = "Whether this flow must have its own layer. Even if this flag is not set, it might"]
        #[doc = "get its own layer if it's deemed to be likely to overlap flows with their own"]
        #[doc = "layer."]
        const NEEDS_LAYER = 0b0000_0000_0000_0010_0000,
        #[doc = "Whether this flow is absolutely positioned. This is checked all over layout, so a"]
        #[doc = "virtual call is too expensive."]
        const IS_ABSOLUTELY_POSITIONED = 0b0000_0000_0000_0100_0000,
        #[doc = "Whether this flow clears to the left. This is checked all over layout, so a"]
        #[doc = "virtual call is too expensive."]
        const CLEARS_LEFT = 0b0000_0000_0000_1000_0000,
        #[doc = "Whether this flow clears to the right. This is checked all over layout, so a"]
        #[doc = "virtual call is too expensive."]
        const CLEARS_RIGHT = 0b0000_0000_0001_0000_0000,
        #[doc = "Whether this flow is left-floated. This is checked all over layout, so a"]
        #[doc = "virtual call is too expensive."]
        const FLOATS_LEFT = 0b0000_0000_0010_0000_0000,
        #[doc = "Whether this flow is right-floated. This is checked all over layout, so a"]
        #[doc = "virtual call is too expensive."]
        const FLOATS_RIGHT = 0b0000_0000_0100_0000_0000,
        #[doc = "Text alignment. \

                 NB: If you update this, update `TEXT_ALIGN_SHIFT` below."]
        const TEXT_ALIGN = 0b0000_0111_1000_0000_0000,
        #[doc = "Whether this flow has a fragment with `counter-reset` or `counter-increment` \
                 styles."]
        const AFFECTS_COUNTERS = 0b0000_1000_0000_0000_0000,
        #[doc = "Whether this flow's descendants have fragments that affect `counter-reset` or \
                 `counter-increment` styles."]
        const HAS_COUNTER_AFFECTING_CHILDREN = 0b0001_0000_0000_0000_0000,
        #[doc = "Whether this flow behaves as though it had `position: static` for the purposes \
                 of positioning in the inline direction. This is set for flows with `position: \
                 static` and `position: relative` as well as absolutely-positioned flows with \
                 unconstrained positions in the inline direction."]
        const INLINE_POSITION_IS_STATIC = 0b0010_0000_0000_0000_0000,
        #[doc = "Whether this flow behaves as though it had `position: static` for the purposes \
                 of positioning in the block direction. This is set for flows with `position: \
                 static` and `position: relative` as well as absolutely-positioned flows with \
                 unconstrained positions in the block direction."]
        const BLOCK_POSITION_IS_STATIC = 0b0100_0000_0000_0000_0000,
    }
}

// NB: If you update this field, you must update the the floated descendants flags.
/// The bitmask of flags that represent the `has_left_floated_descendants` and
/// `has_right_floated_descendants` fields.

static HAS_FLOATED_DESCENDANTS_BITMASK: FlowFlags = FlowFlags { bits: 0b0000_0011 };

/// The number of bits we must shift off to handle the text alignment field.
///
/// NB: If you update this, update `TEXT_ALIGN` above.
static TEXT_ALIGN_SHIFT: usize = 11;

impl FlowFlags {
    /// Propagates text alignment flags from an appropriate parent flow per CSS 2.1.
    ///
    /// FIXME(#2265, pcwalton): It would be cleaner and faster to make this a derived CSS property
    /// `-servo-text-align-in-effect`.
    pub fn propagate_text_alignment_from_parent(&mut self, parent_flags: FlowFlags) {
        self.set_text_align_override(parent_flags);
    }

    #[inline]
    pub fn text_align(self) -> text_align::T {
        text_align::T::from_u32((self & TEXT_ALIGN).bits() >> TEXT_ALIGN_SHIFT).unwrap()
    }

    #[inline]
    pub fn set_text_align(&mut self, value: text_align::T) {
        *self = (*self & !TEXT_ALIGN) |
                FlowFlags::from_bits(value.to_u32() << TEXT_ALIGN_SHIFT).unwrap();
    }

    #[inline]
    pub fn set_text_align_override(&mut self, parent: FlowFlags) {
        self.insert(parent & TEXT_ALIGN);
    }

    #[inline]
    pub fn union_floated_descendants_flags(&mut self, other: FlowFlags) {
        self.insert(other & HAS_FLOATED_DESCENDANTS_BITMASK);
    }

    #[inline]
    pub fn impacted_by_floats(&self) -> bool {
        self.contains(IMPACTED_BY_LEFT_FLOATS) || self.contains(IMPACTED_BY_RIGHT_FLOATS)
    }

    #[inline]
    pub fn set(&mut self, flags: FlowFlags, value: bool) {
        if value {
            self.insert(flags);
        } else {
            self.remove(flags);
        }
    }

    #[inline]
    pub fn float_kind(&self) -> float::T {
        if self.contains(FLOATS_LEFT) {
            float::T::left
        } else if self.contains(FLOATS_RIGHT) {
            float::T::right
        } else {
            float::T::none
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        self.contains(FLOATS_LEFT) || self.contains(FLOATS_RIGHT)
    }

    #[inline]
    pub fn clears_floats(&self) -> bool {
        self.contains(CLEARS_LEFT) || self.contains(CLEARS_RIGHT)
    }
}

/// Absolutely-positioned descendants of this flow.
#[derive(Clone)]
pub struct AbsoluteDescendants {
    /// Links to every descendant. This must be private because it is unsafe to leak `FlowRef`s to
    /// layout.
    descendant_links: Vec<AbsoluteDescendantInfo>,
}

impl AbsoluteDescendants {
    pub fn new() -> AbsoluteDescendants {
        AbsoluteDescendants {
            descendant_links: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.descendant_links.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descendant_links.is_empty()
    }

    pub fn push(&mut self, given_descendant: FlowRef) {
        self.descendant_links.push(AbsoluteDescendantInfo {
            flow: given_descendant,
        });
    }

    /// Push the given descendants on to the existing descendants.
    ///
    /// Ignore any static y offsets, because they are None before layout.
    pub fn push_descendants(&mut self, given_descendants: AbsoluteDescendants) {
        for elem in given_descendants.descendant_links {
            self.descendant_links.push(elem);
        }
    }

    /// Return an iterator over the descendant flows.
    pub fn iter<'a>(&'a mut self) -> AbsoluteDescendantIter<'a> {
        AbsoluteDescendantIter {
            iter: self.descendant_links.iter_mut(),
        }
    }
}

/// TODO(pcwalton): This structure is going to need a flag to record whether the absolute
/// descendants have reached their containing block yet. The reason is so that we can handle cases
/// like the following:
///
///     <div>
///         <span id=a style="position: absolute; ...">foo</span>
///         <span style="position: relative">
///             <span id=b style="position: absolute; ...">bar</span>
///         </span>
///     </div>
///
/// When we go to create the `InlineFlow` for the outer `div`, our absolute descendants will
/// be `a` and `b`. At this point, we need a way to distinguish between the two, because the
/// containing block for `a` will be different from the containing block for `b`. Specifically,
/// the latter's containing block is the inline flow itself, while the former's containing
/// block is going to be some parent of the outer `div`. Hence we need this flag as a way to
/// distinguish the two; it will be false for `a` and true for `b`.
#[derive(Clone)]
pub struct AbsoluteDescendantInfo {
    /// The absolute descendant flow in question.
    flow: FlowRef,
}

pub struct AbsoluteDescendantIter<'a> {
    iter: IterMut<'a, AbsoluteDescendantInfo>,
}

impl<'a> Iterator for AbsoluteDescendantIter<'a> {
    type Item = &'a mut (Flow + 'a);
    fn next(&mut self) -> Option<&'a mut (Flow + 'a)> {
        self.iter.next().map(|info| &mut *info.flow)
    }
}

pub type AbsoluteDescendantOffsetIter<'a> = Zip<AbsoluteDescendantIter<'a>, IterMut<'a, Au>>;

/// Information needed to compute absolute (i.e. viewport-relative) flow positions (not to be
/// confused with absolutely-positioned flows).
#[derive(RustcEncodable, Copy, Clone)]
pub struct AbsolutePositionInfo {
    /// The size of the containing block for relatively-positioned descendants.
    pub relative_containing_block_size: LogicalSize<Au>,

    /// The writing mode for `relative_containing_block_size`.
    pub relative_containing_block_mode: WritingMode,

    /// The position of the absolute containing block relative to the nearest ancestor stacking
    /// context. If the absolute containing block establishes the stacking context for this flow,
    /// and this flow is not itself absolutely-positioned, then this is (0, 0).
    pub stacking_relative_position_of_absolute_containing_block: Point2D<Au>,

    /// Whether the absolute containing block forces positioned descendants to be layerized.
    ///
    /// FIXME(pcwalton): Move into `FlowFlags`.
    pub layers_needed_for_positioned_flows: bool,
}

impl AbsolutePositionInfo {
    pub fn new(writing_mode: WritingMode) -> AbsolutePositionInfo {
        // FIXME(pcwalton): The initial relative containing block-size should be equal to the size
        // of the root layer.
        AbsolutePositionInfo {
            relative_containing_block_size: LogicalSize::zero(writing_mode),
            relative_containing_block_mode: writing_mode,
            stacking_relative_position_of_absolute_containing_block: Point2D::zero(),
            layers_needed_for_positioned_flows: false,
        }
    }
}

/// Data common to all flows.
pub struct BaseFlow {
    /// NB: Must be the first element.
    ///
    /// The necessity of this will disappear once we have dynamically-sized types.
    strong_ref_count: AtomicUsize,

    weak_ref_count: AtomicUsize,

    pub restyle_damage: RestyleDamage,

    /// The children of this flow.
    pub children: FlowList,

    /// Intrinsic inline sizes for this flow.
    pub intrinsic_inline_sizes: IntrinsicISizes,

    /// The upper left corner of the box representing this flow, relative to the box representing
    /// its parent flow.
    ///
    /// For absolute flows, this represents the position with respect to its *containing block*.
    ///
    /// This does not include margins in the block flow direction, because those can collapse. So
    /// for the block direction (usually vertical), this represents the *border box*. For the
    /// inline direction (usually horizontal), this represents the *margin box*.
    pub position: LogicalRect<Au>,

    /// The amount of overflow of this flow, relative to the containing block. Must include all the
    /// pixels of all the display list items for correct invalidation.
    pub overflow: Rect<Au>,

    /// Data used during parallel traversals.
    ///
    /// TODO(pcwalton): Group with other transient data to save space.
    pub parallel: FlowParallelInfo,

    /// The floats next to this flow.
    pub floats: Floats,

    /// The collapsible margins for this flow, if any.
    pub collapsible_margins: CollapsibleMargins,

    /// The position of this flow relative to the start of the nearest ancestor stacking context.
    /// This is computed during the top-down pass of display list construction.
    pub stacking_relative_position: Point2D<Au>,

    /// Details about descendants with position 'absolute' or 'fixed' for which we are the
    /// containing block. This is in tree order. This includes any direct children.
    pub abs_descendants: AbsoluteDescendants,

    /// The inline-size of the block container of this flow. Used for computing percentage and
    /// automatic values for `width`.
    pub block_container_inline_size: Au,

    /// The writing mode of the block container of this flow.
    ///
    /// FIXME (mbrubeck): Combine this and block_container_inline_size and maybe
    /// block_container_explicit_block_size into a struct, to guarantee they are set at the same
    /// time?  Or just store a link to the containing block flow.
    pub block_container_writing_mode: WritingMode,

    /// The block-size of the block container of this flow, if it is an explicit size (does not
    /// depend on content heights).  Used for computing percentage values for `height`.
    pub block_container_explicit_block_size: Option<Au>,

    /// Reference to the Containing Block, if this flow is absolutely positioned.
    pub absolute_cb: ContainingBlockLink,

    /// Information needed to compute absolute (i.e. viewport-relative) flow positions (not to be
    /// confused with absolutely-positioned flows).
    pub absolute_position_info: AbsolutePositionInfo,

    /// The clipping region for this flow and its descendants, in layer coordinates.
    pub clip: ClippingRegion,

    /// The stacking-relative position of the display port.
    ///
    /// FIXME(pcwalton): This might be faster as an Arc, since this varies only
    /// per-stacking-context.
    pub stacking_relative_position_of_display_port: Rect<Au>,

    /// The results of display list building for this flow.
    pub display_list_building_result: DisplayListBuildingResult,

    /// The writing mode for this flow.
    pub writing_mode: WritingMode,

    /// For debugging and profiling, the identifier of the thread that laid out this fragment.
    pub thread_id: u8,

    /// Various flags for flows, tightly packed to save space.
    pub flags: FlowFlags,
}

#[allow(unsafe_code)]
unsafe impl Send for BaseFlow {}
#[allow(unsafe_code)]
unsafe impl Sync for BaseFlow {}

impl fmt::Debug for BaseFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "@ {:?}, CC {}, ADC {}",
               self.position,
               self.parallel.children_count.load(Ordering::SeqCst),
               self.abs_descendants.len())
    }
}

impl Encodable for BaseFlow {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_struct("base", 0, |e| {
            try!(e.emit_struct_field("id", 0, |e| self.debug_id().encode(e)));
            try!(e.emit_struct_field("stacking_relative_position",
                                     1,
                                     |e| self.stacking_relative_position.encode(e)));
            try!(e.emit_struct_field("intrinsic_inline_sizes",
                                     2,
                                     |e| self.intrinsic_inline_sizes.encode(e)));
            try!(e.emit_struct_field("position", 3, |e| self.position.encode(e)));
            e.emit_struct_field("children", 4, |e| {
                e.emit_seq(self.children.len(), |e| {
                    for (i, c) in self.children.iter().enumerate() {
                        try!(e.emit_seq_elt(i, |e| {
                            try!(e.emit_struct("flow", 0, |e| {
                                try!(e.emit_struct_field("class", 0, |e| c.class().encode(e)));
                                e.emit_struct_field("data", 1, |e| {
                                    match c.class() {
                                        FlowClass::Block => c.as_block().encode(e),
                                        FlowClass::Inline => c.as_inline().encode(e),
                                        FlowClass::Table => c.as_table().encode(e),
                                        FlowClass::TableWrapper => c.as_table_wrapper().encode(e),
                                        FlowClass::TableRowGroup => c.as_table_rowgroup().encode(e),
                                        FlowClass::TableRow => c.as_table_row().encode(e),
                                        FlowClass::TableCell => c.as_table_cell().encode(e),
                                        _ => { Ok(()) }     // TODO: Support captions
                                    }
                                })
                            }));
                            Ok(())
                        }));
                    }
                    Ok(())
                })

            })
        })
    }
}

impl Drop for BaseFlow {
    fn drop(&mut self) {
        if self.strong_ref_count.load(Ordering::SeqCst) != 0 &&
           self.weak_ref_count.load(Ordering::SeqCst) != 0 {
            panic!("Flow destroyed before its ref count hit zero—this is unsafe!")
        }
    }
}

/// Whether a base flow should be forced to be nonfloated. This can affect e.g. `TableFlow`, which
/// is never floated because the table wrapper flow is the floated one.
#[derive(Clone, PartialEq)]
pub enum ForceNonfloatedFlag {
    /// The flow should be floated if the node has a `float` property.
    FloatIfNecessary,
    /// The flow should be forced to be nonfloated.
    ForceNonfloated,
}

impl BaseFlow {
    #[inline]
    pub fn new(style: Option<&ComputedValues>,
               writing_mode: WritingMode,
               force_nonfloated: ForceNonfloatedFlag)
               -> BaseFlow {
        let mut flags = FlowFlags::empty();
        match style {
            Some(style) => {
                match style.get_box().position {
                    position::T::absolute | position::T::fixed => {
                        flags.insert(IS_ABSOLUTELY_POSITIONED);

                        let logical_position = style.logical_position();
                        if logical_position.inline_start == LengthOrPercentageOrAuto::Auto &&
                                logical_position.inline_end == LengthOrPercentageOrAuto::Auto {
                            flags.insert(INLINE_POSITION_IS_STATIC);
                        }
                        if logical_position.block_start == LengthOrPercentageOrAuto::Auto &&
                                logical_position.block_end == LengthOrPercentageOrAuto::Auto {
                            flags.insert(BLOCK_POSITION_IS_STATIC);
                        }
                    }
                    _ => flags.insert(BLOCK_POSITION_IS_STATIC | INLINE_POSITION_IS_STATIC),
                }

                if force_nonfloated == ForceNonfloatedFlag::FloatIfNecessary {
                    match style.get_box().float {
                        float::T::none => {}
                        float::T::left => flags.insert(FLOATS_LEFT),
                        float::T::right => flags.insert(FLOATS_RIGHT),
                    }
                }

                match style.get_box().clear {
                    clear::T::none => {}
                    clear::T::left => flags.insert(CLEARS_LEFT),
                    clear::T::right => flags.insert(CLEARS_RIGHT),
                    clear::T::both => {
                        flags.insert(CLEARS_LEFT);
                        flags.insert(CLEARS_RIGHT);
                    }
                }

                if !style.get_counters().counter_reset.0.is_empty() ||
                        !style.get_counters().counter_increment.0.is_empty() {
                    flags.insert(AFFECTS_COUNTERS)
                }
            }
            None => flags.insert(BLOCK_POSITION_IS_STATIC | INLINE_POSITION_IS_STATIC),
        }

        // New flows start out as fully damaged.
        let mut damage = incremental::rebuild_and_reflow();
        damage.remove(RECONSTRUCT_FLOW);

        BaseFlow {
            strong_ref_count: AtomicUsize::new(1),
            weak_ref_count: AtomicUsize::new(1),
            restyle_damage: damage,
            children: FlowList::new(),
            intrinsic_inline_sizes: IntrinsicISizes::new(),
            position: LogicalRect::zero(writing_mode),
            overflow: ZERO_RECT,
            parallel: FlowParallelInfo::new(),
            floats: Floats::new(writing_mode),
            collapsible_margins: CollapsibleMargins::new(),
            stacking_relative_position: Point2D::zero(),
            abs_descendants: AbsoluteDescendants::new(),
            block_container_inline_size: Au(0),
            block_container_writing_mode: writing_mode,
            block_container_explicit_block_size: None,
            absolute_cb: ContainingBlockLink::new(),
            display_list_building_result: DisplayListBuildingResult::None,
            absolute_position_info: AbsolutePositionInfo::new(writing_mode),
            clip: ClippingRegion::max(),
            stacking_relative_position_of_display_port: Rect::zero(),
            flags: flags,
            writing_mode: writing_mode,
            thread_id: 0,
        }
    }

    pub fn child_iter<'a>(&'a mut self) -> MutFlowListIterator<'a> {
        self.children.iter_mut()
    }

    #[allow(unsafe_code)]
    pub unsafe fn strong_ref_count<'a>(&'a self) -> &'a AtomicUsize {
        &self.strong_ref_count
    }

    #[allow(unsafe_code)]
    pub unsafe fn weak_ref_count<'a>(&'a self) -> &'a AtomicUsize {
        &self.weak_ref_count
    }

    pub fn debug_id(&self) -> usize {
        let p = self as *const _;
        p as usize
    }

    /// Ensures that all display list items generated by this flow are within the flow's overflow
    /// rect. This should only be used for debugging.
    pub fn validate_display_list_geometry(&self) {
        // FIXME(pcwalton, #2795): Get the real container size.
        let container_size = Size2D::zero();
        let position_with_overflow = self.position
                                         .to_physical(self.writing_mode, container_size)
                                         .union(&self.overflow);
        let bounds = Rect::new(self.stacking_relative_position, position_with_overflow.size);

        let all_items = match self.display_list_building_result {
            DisplayListBuildingResult::None => Vec::new(),
            DisplayListBuildingResult::StackingContext(ref stacking_context) => {
                stacking_context.display_list.all_display_items()
            }
            DisplayListBuildingResult::Normal(ref display_list) => display_list.all_display_items(),
        };

        for item in &all_items {
            let paint_bounds = item.base().clip.clone().intersect_rect(&item.base().bounds);
            if !paint_bounds.might_be_nonempty() {
                continue;
            }

            if bounds.union(&paint_bounds.bounding_rect()) != bounds {
                error!("DisplayList item {:?} outside of Flow overflow ({:?})", item, paint_bounds);
            }
        }
    }
}

impl<'a> ImmutableFlowUtils for &'a (Flow + 'a) {
    /// Returns true if this flow is a block flow or subclass thereof.
    fn is_block_like(self) -> bool {
        match self.class() {
            FlowClass::Block | FlowClass::ListItem | FlowClass::Table | FlowClass::TableRowGroup |
            FlowClass::TableRow | FlowClass::TableCaption | FlowClass::TableCell |
            FlowClass::TableWrapper => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a proper table child.
    /// 'Proper table child' is defined as table-row flow, table-rowgroup flow,
    /// table-column-group flow, or table-caption flow.
    fn is_proper_table_child(self) -> bool {
        match self.class() {
            FlowClass::TableRow | FlowClass::TableRowGroup |
                FlowClass::TableColGroup | FlowClass::TableCaption => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table row flow.
    fn is_table_row(self) -> bool {
        match self.class() {
            FlowClass::TableRow => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table cell flow.
    fn is_table_cell(self) -> bool {
        match self.class() {
            FlowClass::TableCell => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table colgroup flow.
    fn is_table_colgroup(self) -> bool {
        match self.class() {
            FlowClass::TableColGroup => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table flow.
    fn is_table(self) -> bool {
        match self.class() {
            FlowClass::Table => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table caption flow.
    fn is_table_caption(self) -> bool {
        match self.class() {
            FlowClass::TableCaption => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table rowgroup flow.
    fn is_table_rowgroup(self) -> bool {
        match self.class() {
            FlowClass::TableRowGroup => true,
            _ => false,
        }
    }

    /// Returns true if this flow is one of table-related flows.
    fn is_table_kind(self) -> bool {
        match self.class() {
            FlowClass::TableWrapper | FlowClass::Table |
                FlowClass::TableColGroup | FlowClass::TableRowGroup |
                FlowClass::TableRow | FlowClass::TableCaption | FlowClass::TableCell => true,
            _ => false,
        }
    }

    /// Returns true if anonymous flow is needed between this flow and child flow.
    /// Spec: http://www.w3.org/TR/CSS21/tables.html#anonymous-boxes
    fn need_anonymous_flow(self, child: &Flow) -> bool {
        match self.class() {
            FlowClass::Table => !child.is_proper_table_child(),
            FlowClass::TableRowGroup => !child.is_table_row(),
            FlowClass::TableRow => !child.is_table_cell(),
            _ => false
        }
    }

    /// Generates missing child flow of this flow.
    ///
    /// FIXME(pcwalton): This duplicates some logic in
    /// `generate_anonymous_table_flows_if_necessary()`. We should remove this function eventually,
    /// as it's harder to understand.
    fn generate_missing_child_flow(self, node: &ThreadSafeLayoutNode) -> FlowRef {
        let mut style = node.style().clone();
        let flow = match self.class() {
            FlowClass::Table | FlowClass::TableRowGroup => {
                properties::modify_style_for_anonymous_table_object(
                    &mut style,
                    display::T::table_row);
                let fragment = Fragment::from_opaque_node_and_style(
                    node.opaque(),
                    PseudoElementType::Normal,
                    style,
                    node.restyle_damage(),
                    SpecificFragmentInfo::TableRow);
                box TableRowFlow::from_fragment(fragment) as Box<Flow>
            },
            FlowClass::TableRow => {
                properties::modify_style_for_anonymous_table_object(
                    &mut style,
                    display::T::table_cell);
                let fragment = Fragment::from_opaque_node_and_style(
                    node.opaque(),
                    PseudoElementType::Normal,
                    style,
                    node.restyle_damage(),
                    SpecificFragmentInfo::TableCell);
                let hide = node.style().get_inheritedtable().empty_cells == empty_cells::T::hide;
                box TableCellFlow::from_node_fragment_and_visibility_flag(node, fragment, !hide) as
                    Box<Flow>
            },
            _ => {
                panic!("no need to generate a missing child")
            }
        };
        FlowRef::new(flow)
    }

    /// Returns true if this flow contains fragments that are roots of an absolute flow tree.
    fn contains_roots_of_absolute_flow_tree(&self) -> bool {
        self.contains_relatively_positioned_fragments() || self.is_root()
    }

    /// Returns true if this flow has no children.
    fn is_leaf(self) -> bool {
        base(self).children.is_empty()
    }

    /// Returns the number of children that this flow possesses.
    fn child_count(self) -> usize {
        base(self).children.len()
    }

    /// Return true if this flow is a Block Container.
    ///
    /// Except for table fragments and replaced elements, block-level fragments (`BlockFlow`) are
    /// also block container fragments.
    /// Non-replaced inline blocks and non-replaced table cells are also block
    /// containers.
    fn is_block_container(self) -> bool {
        match self.class() {
            // TODO: Change this when inline-blocks are supported.
            FlowClass::Block | FlowClass::TableCaption | FlowClass::TableCell => {
                // FIXME: Actually check the type of the node
                self.child_count() != 0
            }
            _ => false,
        }
    }

    /// Returns true if this flow is a block flow.
    fn is_block_flow(self) -> bool {
        match self.class() {
            FlowClass::Block => true,
            _ => false,
        }
    }

    /// Returns true if this flow is an inline flow.
    fn is_inline_flow(self) -> bool {
        match self.class() {
            FlowClass::Inline => true,
            _ => false,
        }
    }

    /// Dumps the flow tree for debugging.
    fn dump(self) {
        self.dump_with_level(0)
    }

    /// Dumps the flow tree for debugging, with a prefix to indicate that we're at the given level.
    fn dump_with_level(self, level: u32) {
        let mut indent = String::new();
        for _ in 0..level {
            indent.push_str("| ")
        }

        println!("{}+ {:?}", indent, self);

        for kid in imm_child_iter(self) {
            kid.dump_with_level(level + 1)
        }
    }
}

impl<'a> MutableFlowUtils for &'a mut (Flow + 'a) {
    /// Traverses the tree in preorder.
    fn traverse_preorder<T: PreorderFlowTraversal>(self, traversal: &T) {
        if traversal.should_process(self) {
            traversal.process(self);
        }

        for kid in child_iter(self) {
            kid.traverse_preorder(traversal);
        }
    }

    /// Traverses the tree in postorder.
    fn traverse_postorder<T: PostorderFlowTraversal>(self, traversal: &T) {
        for kid in child_iter(self) {
            kid.traverse_postorder(traversal);
        }

        if traversal.should_process(self) {
            traversal.process(self)
        }
    }


    /// Calls `repair_style` and `bubble_inline_sizes`. You should use this method instead of
    /// calling them individually, since there is no reason not to perform both operations.
    fn repair_style_and_bubble_inline_sizes(self, style: &Arc<ComputedValues>) {
        self.repair_style(style);
        self.bubble_inline_sizes();
    }

    /// Traverse the Absolute flow tree in preorder.
    ///
    /// Traverse all your direct absolute descendants, who will then traverse
    /// their direct absolute descendants.
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_preorder_absolute_flows<T>(&mut self, traversal: &mut T)
                                           where T: PreorderFlowTraversal {
        traversal.process(*self);

        let descendant_offset_iter = mut_base(*self).abs_descendants.iter();
        for ref mut descendant_link in descendant_offset_iter {
            descendant_link.traverse_preorder_absolute_flows(traversal)
        }
    }

    /// Traverse the Absolute flow tree in postorder.
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_postorder_absolute_flows<T>(&mut self, traversal: &mut T)
                                            where T: PostorderFlowTraversal {
        for mut descendant_link in mut_base(*self).abs_descendants.iter() {
            descendant_link.traverse_postorder_absolute_flows(traversal);
        }

        traversal.process(*self)
    }
}

impl MutableOwnedFlowUtils for FlowRef {
    /// Set absolute descendants for this flow.
    ///
    /// Set yourself as the Containing Block for all the absolute descendants.
    ///
    /// This is called during flow construction, so nothing else can be accessing the descendant
    /// flows. This is enforced by the fact that we have a mutable `FlowRef`, which only flow
    /// construction is allowed to possess.
    fn set_absolute_descendants(&mut self, abs_descendants: AbsoluteDescendants) {
        let this = self.clone();
        let base = mut_base(&mut **self);
        base.abs_descendants = abs_descendants;
        for descendant_link in base.abs_descendants.iter() {
            let descendant_base = mut_base(descendant_link);
            descendant_base.absolute_cb.set(this.clone());
        }
    }
}

/// A link to a flow's containing block.
///
/// This cannot safely be a `Flow` pointer because this is a pointer *up* the tree, not *down* the
/// tree. A pointer up the tree is unsafe during layout because it can be used to access a node
/// with an immutable reference while that same node is being laid out, causing possible iterator
/// invalidation and use-after-free.
///
/// FIXME(pcwalton): I think this would be better with a borrow flag instead of `unsafe`.
pub struct ContainingBlockLink {
    /// The pointer up to the containing block.
    link: Option<WeakFlowRef>,
}

impl ContainingBlockLink {
    fn new() -> ContainingBlockLink {
        ContainingBlockLink {
            link: None,
        }
    }

    fn set(&mut self, link: FlowRef) {
        self.link = Some(link.downgrade())
    }

    #[allow(unsafe_code)]
    pub unsafe fn get<'a>(&'a mut self) -> &'a mut Option<WeakFlowRef> {
        &mut self.link
    }

    #[inline]
    pub fn generated_containing_block_size(&self, for_flow: OpaqueFlow) -> LogicalSize<Au> {
        match self.link {
            None => {
                panic!("Link to containing block not established; perhaps you forgot to call \
                        `set_absolute_descendants`?")
            }
            Some(ref link) => {
                let flow = link.upgrade().unwrap();
                flow.generated_containing_block_size(for_flow)
            }
        }
    }

    #[inline]
    pub fn explicit_block_containing_size(&self, layout_context: &LayoutContext) -> Option<Au> {
        match self.link {
            None => {
                panic!("Link to containing block not established; perhaps you forgot to call \
                        `set_absolute_descendants`?")
            }
            Some(ref link) => {
                let flow = link.upgrade().unwrap();
                if flow.is_block_like() {
                    flow.as_block().explicit_block_containing_size(layout_context)
                } else if flow.is_inline_flow() {
                    Some(flow.as_inline().minimum_block_size_above_baseline)
                } else {
                    None
                }
            }
        }
    }
}

/// A wrapper for the pointer address of a flow. These pointer addresses may only be compared for
/// equality with other such pointer addresses, never dereferenced.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct OpaqueFlow(pub usize);

impl OpaqueFlow {
    #[allow(unsafe_code)]
    pub fn from_flow(flow: &Flow) -> OpaqueFlow {
        unsafe {
            let object = mem::transmute::<&Flow, raw::TraitObject>(flow);
            OpaqueFlow(object.data as usize)
        }
    }

    pub fn from_base_flow(base_flow: &BaseFlow) -> OpaqueFlow {
        OpaqueFlow(base_flow as *const BaseFlow as usize)
    }
}
