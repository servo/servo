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
use layout::parallel::{FlowParallelInfo, UnsafeFlow};
use layout::parallel;
use layout::wrapper::ThreadSafeLayoutNode;
use layout::flow_list::{FlowList, Link, Rawlink, FlowListIterator, MutFlowListIterator};

use extra::container::Deque;
use geom::point::Point2D;
use geom::Size2D;
use geom::rect::Rect;
use gfx::display_list::{ClipDisplayItemClass, DisplayListCollection, DisplayList};
use layout::display_list_builder::ToGfxColor;
use gfx::color::Color;
use servo_util::concurrentmap::{ConcurrentHashMap, ConcurrentHashMapIterator};
use servo_util::geometry::Au;
use std::cast;
use std::cell::RefCell;
use std::sync::atomics::Relaxed;
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
pub fn imm_child_iter<'a>(flow: &'a Flow) -> FlowListIterator<'a> {
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
pub fn last_child<'a>(flow: &'a mut Flow) -> Option<&'a mut Flow> {
    mut_base(flow).children.back_mut()
}

/// Iterates over the children of this flow.
pub fn child_iter<'a>(flow: &'a mut Flow) -> MutFlowListIterator<'a> {
    mut_base(flow).children.mut_iter()
}

pub trait ImmutableFlowUtils {
    // Convenience functions

    /// Returns true if this flow is a block or a float flow.
    fn is_block_like(self) -> bool;

    /// Returns true if this flow has no children.
    fn is_leaf(self) -> bool;

    /// Returns the number of children that this flow possesses.
    fn child_count(self) -> uint;

    /// Return true if this flow is a Block Container.
    fn is_block_container(self) -> bool;

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

    /// Invokes a closure with the first child of this flow.
    fn with_first_child<R>(self, f: |Option<&mut Flow>| -> R) -> R;

    /// Invokes a closure with the last child of this flow.
    fn with_last_child<R>(self, f: |Option<&mut Flow>| -> R) -> R;

    /// Computes the overflow region for this flow.
    fn store_overflow(self, _: &mut LayoutContext);

    /// builds the display lists
    fn build_display_lists<E:ExtraDisplayListData>(
                          self,
                          builder: &DisplayListBuilder,
                          container_block_size: &Size2D<Au>,
                          dirty: &Rect<Au>,
                          index: uint,
                          mut list: &RefCell<DisplayListCollection<E>>)
                          -> bool;

    /// Destroys the flow.
    fn destroy(self, leaf_set: &FlowLeafSet);
}

pub trait MutableOwnedFlowUtils {
    /// Adds a new flow as a child of this flow. Removes the flow from the given leaf set if
    /// it's present.
    fn add_new_child(&mut self, new_child: ~Flow);

    /// Marks the flow as a leaf. The flow must not have children and must not be marked as a
    /// nonleaf.
    fn mark_as_leaf(&mut self, leaf_set: &FlowLeafSet);

    /// Marks the flow as a nonleaf. The flow must not be marked as a leaf.
    fn mark_as_nonleaf(&mut self);

    /// Destroys the flow.
    fn destroy(&mut self, leaf_set: &FlowLeafSet);
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
    /// operation for this node, but continue processing the ancestors. This is called *after*
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

#[deriving(Clone)]
pub struct FlowFlagsInfo {
    flags: FlowFlags,

    /// text-decoration colors
    rare_flow_flags: Option<~RareFlowFlags>,
}

#[deriving(Clone)]
pub struct RareFlowFlags {
    underline_color: Color,
    overline_color: Color,
    line_through_color: Color,
}

/// Flags used in flows, tightly packed to save space.
#[deriving(Clone)]
pub struct FlowFlags(u8);

/// The bitmask of flags that represent text decoration fields that get propagated downward.
///
/// NB: If you update this field, you must update the bitfields below.
static TEXT_DECORATION_OVERRIDE_BITMASK: u8 = 0b0000_1110;

/// The bitmask of flags that represent the text alignment field.
///
/// NB: If you update this field, you must update the bitfields below.
static TEXT_ALIGN_BITMASK: u8 = 0b0011_0000;

/// The number of bits we must shift off to handle the text alignment field.
///
/// NB: If you update this field, you must update the bitfields below.
static TEXT_ALIGN_SHIFT: u8 = 4;

impl FlowFlagsInfo {
    /// Creates a new set of flow flags from the given style.
    pub fn new(style: &ComputedValues) -> FlowFlagsInfo {
        let text_decoration = style.Text.text_decoration;
        let mut flags = FlowFlags(0);
        flags.set_override_underline(text_decoration.underline);
        flags.set_override_overline(text_decoration.overline);
        flags.set_override_line_through(text_decoration.line_through);

        // TODO(ksh8281) compute text-decoration-color,style,line
        let rare_flow_flags = if flags.is_text_decoration_enabled() {
            Some(~RareFlowFlags {
                underline_color: style.Color.color.to_gfx_color(),
                overline_color: style.Color.color.to_gfx_color(),
                line_through_color: style.Color.color.to_gfx_color(),
            })
        } else {
            None
        };

        FlowFlagsInfo {
            flags: flags,
            rare_flow_flags: rare_flow_flags,
        }
    }

    pub fn underline_color(&self, default_color: Color) -> Color {
        match self.rare_flow_flags {
            Some(ref data) => {
                data.underline_color
            },
            None => {
                default_color
            }
        }
    }

    pub fn overline_color(&self, default_color: Color) -> Color {
        match self.rare_flow_flags {
            Some(ref data) => {
                data.overline_color
            },
            None => {
                default_color
            }
        }
    }

    pub fn line_through_color(&self, default_color: Color) -> Color {
        match self.rare_flow_flags {
            Some(ref data) => {
                data.line_through_color
            },
            None => {
                default_color
            }
        }
    }

    /// Propagates text decoration flags from an appropriate parent flow per CSS 2.1 § 16.3.1.
    pub fn propagate_text_decoration_from_parent(&mut self, parent: &FlowFlagsInfo) {
        if !parent.flags.is_text_decoration_enabled() {
            return ;
        }

        if !self.flags.is_text_decoration_enabled() && parent.flags.is_text_decoration_enabled() {
            self.rare_flow_flags = parent.rare_flow_flags.clone();
            self.flags.set_text_decoration_override(parent.flags);
            return ;
        }

        if !self.flags.override_underline() && parent.flags.override_underline() {
            match parent.rare_flow_flags {
                Some(ref parent_data) => {
                    match self.rare_flow_flags {
                        Some(ref mut data) => {
                            data.underline_color = parent_data.underline_color;
                        },
                        None => {
                            fail!("if flow has text-decoration, it must have rare_flow_flags");
                        }
                    }
                },
                None => {
                    fail!("if flow has text-decoration, it must have rare_flow_flags");
                }
            }
        }
        if !self.flags.override_overline() && parent.flags.override_overline() {
            match parent.rare_flow_flags {
                Some(ref parent_data) => {
                    match self.rare_flow_flags {
                        Some(ref mut data) => {
                            data.overline_color = parent_data.overline_color;
                        },
                        None => {
                            fail!("if flow has text-decoration, it must have rare_flow_flags");
                        }
                    }
                },
                None => {
                    fail!("if flow has text-decoration, it must have rare_flow_flags");
                }
            }
        }
        if !self.flags.override_line_through() && parent.flags.override_line_through() {
            match parent.rare_flow_flags {
                Some(ref parent_data) => {
                    match self.rare_flow_flags {
                        Some(ref mut data) => {
                            data.line_through_color = parent_data.line_through_color;
                        },
                        None => {
                            fail!("if flow has text-decoration, it must have rare_flow_flags");
                        }
                    }
                },
                None => {
                    fail!("if flow has text-decoration, it must have rare_flow_flags");
                }
            }
        }
        self.flags.set_text_decoration_override(parent.flags);
    }

    /// Propagates text alignment flags from an appropriate parent flow per CSS 2.1.
    pub fn propagate_text_alignment_from_parent(&mut self, parent: &FlowFlagsInfo) {
        self.flags.set_text_align_override(parent.flags);
    }
}

// Whether we need an in-order traversal.
bitfield!(FlowFlags, inorder, set_inorder, 0b0000_0001)

// Whether this flow forces `text-decoration: underline` on.
//
// NB: If you update this, you need to update TEXT_DECORATION_OVERRIDE_BITMASK.
bitfield!(FlowFlags, override_underline, set_override_underline, 0b0000_0010)

// Whether this flow forces `text-decoration: overline` on.
//
// NB: If you update this, you need to update TEXT_DECORATION_OVERRIDE_BITMASK.
bitfield!(FlowFlags, override_overline, set_override_overline, 0b0000_0100)

// Whether this flow forces `text-decoration: line-through` on.
//
// NB: If you update this, you need to update TEXT_DECORATION_OVERRIDE_BITMASK.
bitfield!(FlowFlags, override_line_through, set_override_line_through, 0b0000_1000)

// Whether this flow is marked as a leaf. Flows marked as leaves must not have any more kids added
// to them.
bitfield!(FlowFlags, is_leaf, set_is_leaf, 0b0100_0000)

// Whether this flow is marked as a nonleaf. Flows marked as nonleaves must have children.
bitfield!(FlowFlags, is_nonleaf, set_is_nonleaf, 0b1000_0000)

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

    #[inline]
    pub fn set_text_align_override(&mut self, parent: FlowFlags) {
        *self = FlowFlags(**self | (*parent & TEXT_ALIGN_BITMASK))
    }

    #[inline]
    pub fn set_text_decoration_override(&mut self, parent: FlowFlags) {
        *self = FlowFlags(**self | (*parent & TEXT_DECORATION_OVERRIDE_BITMASK));
    }

    #[inline]
    pub fn is_text_decoration_enabled(&self) -> bool {
        (**self & TEXT_DECORATION_OVERRIDE_BITMASK) != 0
    }
}

/// Data common to all flows.
pub struct BaseFlow {
    restyle_damage: RestyleDamage,

    /// The children of this flow.
    children: FlowList,
    next_sibling: Link,
    prev_sibling: Rawlink,

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

    /// Data used during parallel traversals.
    ///
    /// TODO(pcwalton): Group with other transient data to save space.
    parallel: FlowParallelInfo,

    floats_in: FloatContext,
    floats_out: FloatContext,
    num_floats: uint,
    abs_position: Point2D<Au>,

    /// Whether this flow has been destroyed.
    ///
    /// TODO(pcwalton): Pack this into the flags? Need to be careful because manipulation of this
    /// flag can have memory safety implications.
    priv destroyed: bool,

    /// Various flags for flows and some info
    flags_info: FlowFlagsInfo,
}

impl Drop for BaseFlow {
    fn drop(&mut self) {
        if !self.destroyed {
            fail!("Flow destroyed by going out of scope—this is unsafe! Use `destroy()` instead!")
        }
    }
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
    pub fn new(id: int, node: ThreadSafeLayoutNode) -> BaseFlow {
        let style = node.style();
        BaseFlow {
            restyle_damage: node.restyle_damage(),

            children: FlowList::new(),
            next_sibling: None,
            prev_sibling: Rawlink::none(),

            id: id,

            min_width: Au::new(0),
            pref_width: Au::new(0),
            position: Au::zero_rect(),
            overflow: Au::zero_rect(),

            parallel: FlowParallelInfo::new(),

            floats_in: Invalid,
            floats_out: Invalid,
            num_floats: 0,
            abs_position: Point2D(Au::new(0), Au::new(0)),

            destroyed: false,

            flags_info: FlowFlagsInfo::new(style.get()),
        }
    }

    pub fn child_iter<'a>(&'a mut self) -> MutFlowListIterator<'a> {
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

    /// Returns the number of children that this flow possesses.
    fn child_count(self) -> uint {
        base(self).children.len()
    }

    /// Return true if this flow is a Block Container.
    ///
    /// Except for table boxes and replaced elements, block-level boxes (`BlockFlow`) are
    /// also block container boxes.
    /// Non-replaced inline blocks and non-replaced table cells are also block
    /// containers.
    fn is_block_container(self) -> bool {
        match self.class() {
            // TODO: Change this when inline-blocks are supported.
            InlineFlowClass => false,
            BlockFlowClass => {
                // FIXME: Actually check the type of the node
                self.child_count() != 0
            }
        }
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

    /// Invokes a closure with the first child of this flow.
    fn with_first_child<R>(self, f: |Option<&mut Flow>| -> R) -> R {
        f(mut_base(self).children.front_mut())
    }

    /// Invokes a closure with the last child of this flow.
    fn with_last_child<R>(self, f: |Option<&mut Flow>| -> R) -> R {
        f(mut_base(self).children.back_mut())
    }

    fn store_overflow(self, _: &mut LayoutContext) {
        let my_position = mut_base(self).position;
        let mut overflow = my_position;
        for kid in mut_base(self).child_iter() {
            let mut kid_overflow = base(kid).overflow;
            kid_overflow = kid_overflow.translate(&my_position.origin);
            overflow = overflow.union(&kid_overflow)
        }
        mut_base(self).overflow = overflow
    }

    /// Push display items for current flow and its children onto `list`.
    ///
    /// For InlineFlow, add display items for all its boxes onto list`.
    /// For BlockFlow, add a ClipDisplayItemClass for itself and its children,
    /// plus any other display items like border.
    fn build_display_lists<E:ExtraDisplayListData>(
                          self,
                          builder: &DisplayListBuilder,
                          container_block_size: &Size2D<Au>,
                          dirty: &Rect<Au>,
                          mut index: uint,
                          lists: &RefCell<DisplayListCollection<E>>)
                          -> bool {
        debug!("Flow: building display list for f{}", base(self).id);
        index = match self.class() {
            BlockFlowClass => self.as_block().build_display_list_block(builder, container_block_size, dirty, index, lists),
            InlineFlowClass => self.as_inline().build_display_list_inline(builder, container_block_size, dirty, index, lists),
        };

        if lists.with_mut(|lists| lists.lists[index].list.len() == 0) {
            return true;
        }

        if self.is_block_container() {
            let mut child_lists = DisplayListCollection::new();
            child_lists.add_list(DisplayList::new());
            let child_lists = RefCell::new(child_lists);
            let container_block_size = match self.class() {
                BlockFlowClass => {
                    if self.as_block().box_.is_some() {
                        self.as_block().box_.get_ref().position.get().size
                    } else {
                        base(self).position.size
                    }
                },
                _ => {
                    base(self).position.size
                }
            };

            for kid in child_iter(self) {
                kid.build_display_lists(builder, &container_block_size, dirty, 0u, &child_lists);
            }

            let mut child_lists = Some(child_lists.unwrap());
            // Find parent ClipDisplayItemClass and push all child display items
            // under it
            lists.with_mut(|lists| {
                let mut child_lists = child_lists.take_unwrap();
                let result = lists.lists[index].list.mut_rev_iter().position(|item| {
                    match *item {
                        ClipDisplayItemClass(ref mut item) => {
                            item.child_list.push_all_move(child_lists.lists.shift().list);
                            true
                        },
                        _ => false,
                    }
                });

                if result.is_none() {
                    fail!("fail to find parent item");
                }

                lists.lists.push_all_move(child_lists.lists);
            });
        }
        true
    }

    /// Destroys the flow.
    fn destroy(self, leaf_set: &FlowLeafSet) {
        let is_leaf = {
            let base = mut_base(self);
            base.children.len() == 0
        };

        if is_leaf {
            leaf_set.remove(self);
        } else {
            for kid in child_iter(self) {
                kid.destroy(leaf_set)
            }
        }

        mut_base(self).destroyed = true
    }
}

impl MutableOwnedFlowUtils for ~Flow {
    /// Adds a new flow as a child of this flow. Fails if this flow is marked as a leaf.
    fn add_new_child(&mut self, mut new_child: ~Flow) {
        {
            let kid_base = mut_base(new_child);
            kid_base.parallel.parent = parallel::mut_owned_flow_to_unsafe_flow(self);
        }

        let base = mut_base(*self);
        assert!(!base.flags_info.flags.is_leaf());
        base.children.push_back(new_child);
        let _ = base.parallel.children_count.fetch_add(1, Relaxed);
    }

    /// Marks the flow as a leaf. The flow must not have children and must not be marked as a
    /// nonleaf.
    fn mark_as_leaf(&mut self, leaf_set: &FlowLeafSet) {
        {
            let base = mut_base(*self);
            if base.flags_info.flags.is_nonleaf() {
                fail!("attempted to mark a nonleaf flow as a leaf!")
            }
            if base.children.len() != 0 {
                fail!("attempted to mark a flow with children as a leaf!")
            }
            base.flags_info.flags.set_is_leaf(true)
        }
        let self_borrowed: &Flow = *self;
        leaf_set.insert(self_borrowed);
    }

    /// Marks the flow as a nonleaf. The flow must not be marked as a leaf.
    fn mark_as_nonleaf(&mut self) {
        let base = mut_base(*self);
        if base.flags_info.flags.is_leaf() {
            fail!("attempted to mark a leaf flow as a nonleaf!")
        }
        base.flags_info.flags.set_is_nonleaf(true)
        // We don't check to make sure there are no children as they might be added later.
    }

    /// Destroys the flow.
    fn destroy(&mut self, leaf_set: &FlowLeafSet) {
        let self_borrowed: &mut Flow = *self;
        self_borrowed.destroy(leaf_set);
    }
}

/// Keeps track of the leaves of the flow tree. This is used to efficiently start bottom-up
/// parallel traversals.
pub struct FlowLeafSet {
    priv set: ConcurrentHashMap<UnsafeFlow,()>,
}

impl FlowLeafSet {
    /// Creates a new flow leaf set.
    pub fn new() -> FlowLeafSet {
        FlowLeafSet {
            set: ConcurrentHashMap::with_locks_and_buckets(64, 256),
        }
    }

    /// Inserts a newly-created flow into the leaf set.
    fn insert(&self, flow: &Flow) {
        self.set.insert(parallel::borrowed_flow_to_unsafe_flow(flow), ());
    }

    /// Removes a flow from the leaf set. Asserts that the flow was indeed in the leaf set. (This
    /// invariant is needed for memory safety, as there must always be exactly one leaf set.)
    fn remove(&self, flow: &Flow) {
        if !self.contains(flow) {
            fail!("attempted to remove a flow from the leaf set that wasn't in the set!")
        }
        let flow = parallel::borrowed_flow_to_unsafe_flow(flow);
        self.set.remove(&flow);
    }

    pub fn contains(&self, flow: &Flow) -> bool {
        let flow = parallel::borrowed_flow_to_unsafe_flow(flow);
        self.set.contains_key(&flow)
    }

    pub fn clear(&self) {
        self.set.clear()
    }

    pub fn iter<'a>(&'a self) -> ConcurrentHashMapIterator<'a,UnsafeFlow,()> {
        self.set.iter()
    }
}
