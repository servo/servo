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
///   flows also contain a single box to represent their rendered borders, padding, etc.
///   The BlockFlow at the root of the tree has special behavior: it stretches to the boundaries of
///   the viewport.
///
/// * `InlineFlow`: A flow that establishes an inline context. It has a flat list of child
///   boxes/flows that are subject to inline layout and line breaking and structs to represent
///   line breaks and mapping to CSS boxes, for the purpose of handling `getClientRects()` and
///   similar methods.

use css::node_style::StyledNode;
use layout::block::{BlockFlow};
use layout::box_::Box;
use layout::context::LayoutContext;
use layout::construct::OptVector;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::floats::Floats;
use layout::incremental::RestyleDamage;
use layout::inline::InlineFlow;
use layout::parallel::FlowParallelInfo;
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
use servo_util::smallvec::{SmallVec, SmallVec0};
use servo_util::geometry::Au;
use std::cast;
use std::cell::RefCell;
use std::sync::atomics::Relaxed;
use std::vec::VecMutIterator;
use std::iter::Zip;
use style::ComputedValues;
use style::computed_values::{text_align, position};

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

    fn is_float(&self) -> bool {
        false
    }

    /// The 'position' property of this flow.
    fn positioning(&self) -> position::T {
        position::static_
    }

    /// Return true if this flow has position 'fixed'.
    fn is_fixed(&self) -> bool {
        self.positioning() == position::fixed
    }

    fn is_positioned(&self) -> bool {
        self.is_relatively_positioned() || self.is_absolutely_positioned()
    }

    fn is_relatively_positioned(&self) -> bool {
        self.positioning() == position::relative
    }

    fn is_absolutely_positioned(&self) -> bool {
        self.positioning() == position::absolute || self.is_fixed()
    }

    /// Return true if this is the root of an Absolute flow tree.
    fn is_root_of_absolute_flow_tree(&self) -> bool {
        false
    }

    /// Return the dimensions of the CB generated _by_ this flow for absolute descendants.
    fn generated_cb_size(&self) -> Size2D<Au> {
        fail!("generated_cb_size not yet implemented")
    }

    /// Return position of the CB generated by this flow from the start of this flow.
    fn generated_cb_position(&self) -> Point2D<Au> {
        fail!("this is not the CB-generating flow you're looking for")
    }

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

    /// Returns true if this flow is a block flow.
    fn is_block_flow(self) -> bool;

    /// Returns true if this flow is an inline flow.
    fn is_inline_flow(self) -> bool;

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
                          absolute_cb_abs_position: Point2D<Au>,
                          dirty: &Rect<Au>,
                          index: uint,
                          mut list: &RefCell<DisplayListCollection<E>>)
                          -> bool;

    /// Destroys the flow.
    fn destroy(self);
}

pub trait MutableOwnedFlowUtils {
    /// Adds a new flow as a child of this flow. Removes the flow from the given leaf set if
    /// it's present.
    fn add_new_child(&mut self, new_child: ~Flow);

    /// Finishes a flow. Once a flow is finished, no more child flows or boxes may be added to it.
    /// This will normally run the bubble-widths (minimum and preferred -- i.e. intrinsic -- width)
    /// calculation, unless the global `bubble_widths_separately` flag is on.
    ///
    /// All flows must be finished at some point, or they will not have their intrinsic widths
    /// properly computed. (This is not, however, a memory safety problem.)
    fn finish(&mut self, context: &mut LayoutContext);

    /// Set absolute descendants for this flow.
    ///
    /// Set this flow as the Containing Block for all the absolute descendants.
    fn set_abs_descendants(&mut self, abs_descendants: AbsDescendants);

    /// Set fixed descendants for this flow.
    ///
    /// Set yourself as the Containing Block for all the fixed descendants.
    fn set_fixed_descendants(&mut self, fixed_descendants: AbsDescendants);

    /// Destroys the flow.
    fn destroy(&mut self);
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
        let text_decoration = style.Text.get().text_decoration;
        let mut flags = FlowFlags(0);
        flags.set_override_underline(text_decoration.underline);
        flags.set_override_overline(text_decoration.overline);
        flags.set_override_line_through(text_decoration.line_through);

        // TODO(ksh8281) compute text-decoration-color,style,line
        let rare_flow_flags = if flags.is_text_decoration_enabled() {
            Some(~RareFlowFlags {
                underline_color: style.Color.get().color.to_gfx_color(),
                overline_color: style.Color.get().color.to_gfx_color(),
                line_through_color: style.Color.get().color.to_gfx_color(),
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

/// The Descendants of a flow.
///
/// Also, details about their position wrt this flow.
/// FIXME: This should use @pcwalton's reference counting scheme (Coming Soon).
pub struct Descendants {
    /// Links to every Descendant.
    descendant_links: SmallVec0<Rawlink>,
    /// Static y offsets of all descendants from the start of this flow box.
    static_y_offsets: SmallVec0<Au>,
}

impl Descendants {
    pub fn new() -> Descendants {
        Descendants {
            descendant_links: SmallVec0::new(),
            static_y_offsets: SmallVec0::new(),
        }
    }

    pub fn len(&self) -> uint {
        self.descendant_links.len()
    }

    pub fn push(&mut self, given_descendant: Rawlink) {
        self.descendant_links.push(given_descendant);
    }

    /// Push the given descendants on to the existing descendants.
    ///
    /// Ignore any static y offsets, because they are None before layout.
    pub fn push_descendants(&mut self, mut given_descendants: Descendants) {
        for elem in given_descendants.descendant_links.move_iter() {
            self.descendant_links.push(elem);
        }
    }

    /// Return an iterator over the descendant flows.
    pub fn iter<'a>(&'a mut self) -> DescendantIter<'a> {
        self.descendant_links.mut_slice_from(0).mut_iter()
    }

    /// Return an iterator over (descendant, static y offset).
    pub fn iter_with_offset<'a>(&'a mut self) -> DescendantOffsetIter<'a> {
        self.descendant_links.mut_slice_from(0).mut_iter().zip(
            self.static_y_offsets.mut_slice_from(0).mut_iter())
    }
}

pub type AbsDescendants = Descendants;
pub type FixedDescendants = Descendants;

type DescendantIter<'a> = VecMutIterator<'a, Rawlink>;

type DescendantOffsetIter<'a> = Zip<VecMutIterator<'a, Rawlink>, VecMutIterator<'a, Au>>;

/// Data common to all flows.
pub struct BaseFlow {
    restyle_damage: RestyleDamage,

    /// The children of this flow.
    children: FlowList,
    next_sibling: Link,
    prev_sibling: Rawlink,

    /* layout computations */
    // TODO: min/pref and position are used during disjoint phases of
    // layout; maybe combine into a single enum to save space.
    min_width: Au,
    pref_width: Au,

    /// The upper left corner of the box representing this flow, relative to
    /// the box representing its parent flow.
    /// For absolute flows, this represents the position wrt to its Containing Block.
    position: Rect<Au>,

    /// The amount of overflow of this flow, relative to the containing block. Must include all the
    /// pixels of all the display list items for correct invalidation.
    overflow: Rect<Au>,

    /// Data used during parallel traversals.
    ///
    /// TODO(pcwalton): Group with other transient data to save space.
    parallel: FlowParallelInfo,

    /// The floats next to this flow.
    floats: Floats,

    /// For normal flows, this is the number of floated descendants that are
    /// not contained within any other floated descendant of this flow. For
    /// floats, it is 1.
    /// It is used to allocate float data if necessary and to
    /// decide whether to do an in-order traversal for assign_height.
    num_floats: uint,

    /// The position of this flow in page coordinates, computed during display list construction.
    abs_position: Point2D<Au>,

    /// Details about descendants with position 'absolute' for which we are
    /// the CB. This is in tree order. This includes any direct children.
    abs_descendants: AbsDescendants,
    /// Details about descendants with position 'fixed'.
    /// TODO: Optimize this, because this will be set only for the root.
    fixed_descendants: FixedDescendants,

    /// Offset wrt the nearest positioned ancestor - aka the Containing Block
    /// for any absolutely positioned elements.
    absolute_static_x_offset: Au,
    /// Offset wrt the Initial Containing Block.
    fixed_static_x_offset: Au,

    /// Reference to the Containing Block, if this flow is absolutely positioned.
    absolute_cb: Rawlink,

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
    pub fn new(node: ThreadSafeLayoutNode) -> BaseFlow {
        let style = node.style();
        BaseFlow {
            restyle_damage: node.restyle_damage(),

            children: FlowList::new(),
            next_sibling: None,
            prev_sibling: Rawlink::none(),

            min_width: Au::new(0),
            pref_width: Au::new(0),
            position: Au::zero_rect(),
            overflow: Au::zero_rect(),

            parallel: FlowParallelInfo::new(),

            floats: Floats::new(),
            num_floats: 0,
            abs_position: Point2D(Au::new(0), Au::new(0)),
            abs_descendants: Descendants::new(),
            fixed_descendants: Descendants::new(),
            absolute_static_x_offset: Au::new(0),
            fixed_static_x_offset: Au::new(0),
            absolute_cb: Rawlink::none(),

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

    /// Returns true if this flow is a block flow.
    fn is_block_flow(self) -> bool {
        match self.class() {
            BlockFlowClass => true,
            InlineFlowClass => false,
        }
    }

    /// Returns true if this flow is an inline flow.
    fn is_inline_flow(self) -> bool {
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

    /// Calculate and set overflow for current flow.
    ///
    /// CSS Section 11.1
    /// This is the union of rectangles of the flows for which we define the
    /// Containing Block.
    ///
    /// Assumption: This is called in a bottom-up traversal, so kids' overflows have
    /// already been set.
    /// Assumption: Absolute descendants have had their overflow calculated.
    fn store_overflow(self, _: &mut LayoutContext) {
        let my_position = mut_base(self).position;
        let mut overflow = my_position;

        if self.is_block_container() {
            for kid in child_iter(self) {
                if kid.is_store_overflow_delayed() {
                    // Absolute flows will be handled by their CB. If we are
                    // their CB, they will show up in `abs_descendants`.
                    continue;
                }
                let mut kid_overflow = base(kid).overflow;
                kid_overflow = kid_overflow.translate(&my_position.origin);
                overflow = overflow.union(&kid_overflow)
            }

            for descendant_link in mut_base(self).abs_descendants.iter() {
                match descendant_link.resolve() {
                    Some(flow) => {
                        let mut kid_overflow = base(flow).overflow;
                        kid_overflow = kid_overflow.translate(&my_position.origin);
                        overflow = overflow.union(&kid_overflow)
                    }
                    None => fail!("empty Rawlink to a descendant")
                }
            }

            if self.is_root() {
                for fixed_descendant_link in mut_base(self).fixed_descendants.iter() {
                    match fixed_descendant_link.resolve() {
                        Some(flow) => {
                            let mut kid_overflow = base(flow).overflow;
                            kid_overflow = kid_overflow.translate(&my_position.origin);
                            overflow = overflow.union(&kid_overflow)
                        }
                        None => fail!("empty Rawlink to a descendant")
                    }
                }
            }
        }
        mut_base(self).overflow = overflow;
    }

    /// Push display items for current flow and its children onto `list`.
    ///
    /// For InlineFlow, add display items for all its boxes onto list`.
    /// For BlockFlow, add a ClipDisplayItemClass for itself and its children,
    /// plus any other display items like border.
    ///
    /// `container_block_size`: Size of the Containing Block for the current
    /// flow. This is used for relative positioning (which resolves percentage
    /// values for 'top', etc. after all Containing Block heights have been computed.)
    /// `absolute_cb_abs_position`: Absolute position of the Containing Block
    /// for the flow if it is absolutely positioned.
    fn build_display_lists<E:ExtraDisplayListData>(
                          self,
                          builder: &DisplayListBuilder,
                          container_block_size: &Size2D<Au>,
                          absolute_cb_abs_position: Point2D<Au>,
                          dirty: &Rect<Au>,
                          mut index: uint,
                          lists: &RefCell<DisplayListCollection<E>>)
                          -> bool {
        debug!("Flow: building display list");
        index = match self.class() {
            BlockFlowClass => self.as_block().build_display_list_block(builder,
                                                                       container_block_size,
                                                                       absolute_cb_abs_position,
                                                                       dirty,
                                                                       index,
                                                                       lists),
            InlineFlowClass => self.as_inline().build_display_list_inline(builder, container_block_size, dirty, index, lists),
        };

        if lists.with_mut(|lists| lists.lists[index].list.len() == 0) {
            return true;
        }

        if self.is_block_container() {
            let block = self.as_block();
            let mut child_lists = DisplayListCollection::new();
            child_lists.add_list(DisplayList::new());
            let child_lists = RefCell::new(child_lists);
            let container_block_size;
            let abs_cb_position;
            // TODO(pradeep): Move this into a generated CB function and stuff in Flow.
            match block.box_ {
                Some(ref box_) => {
                    // The Containing Block formed by a Block for relatively
                    // positioned descendants is the content box.
                    container_block_size = box_.content_box_size();

                    abs_cb_position = if block.is_positioned() {
                        block.base.abs_position + block.generated_cb_position()
                    } else {
                        absolute_cb_abs_position
                    };
                }
                None => fail!("Flow: block container should have a box_")
            }

            for kid in block.base.child_iter() {
                if kid.is_absolutely_positioned() {
                    // All absolute flows will be handled by their CB.
                    continue;
                }
                kid.build_display_lists(builder, &container_block_size,
                                        abs_cb_position,
                                        dirty, 0u, &child_lists);
            }

            // TODO: Maybe we should handle position 'absolute' and 'fixed'
            // descendants before normal descendants just in case there is a
            // problem when display-list building is parallel and both the
            // original parent and this flow access the same absolute flow.
            // Note that this can only be done once we have paint order
            // working cos currently the later boxes paint over the absolute
            // and fixed boxes :|
            for abs_descendant_link in block.base.abs_descendants.iter() {
                match abs_descendant_link.resolve() {
                    Some(flow) => {
                        // TODO(pradeep): Send in your abs_position directly.
                        flow.build_display_lists(builder, &container_block_size,
                                                 abs_cb_position,
                                                 dirty, 0u, &child_lists);
                    }
                    None => fail!("empty Rawlink to a descendant")
                }
            }

            if block.is_root() {
                for fixed_descendant_link in block.base.fixed_descendants.iter() {
                    match fixed_descendant_link.resolve() {
                        Some(flow) => {
                            flow.build_display_lists(builder, &container_block_size,
                                                     abs_cb_position,
                                                     dirty, 0u, &child_lists);
                        }
                        None => fail!("empty Rawlink to a descendant")
                    }
                }
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
    fn destroy(self) {
        for kid in child_iter(self) {
            kid.destroy()
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
        base.children.push_back(new_child);
        let _ = base.parallel.children_count.fetch_add(1, Relaxed);
    }

    /// Finishes a flow. Once a flow is finished, no more child flows or boxes may be added to it.
    /// This will normally run the bubble-widths (minimum and preferred -- i.e. intrinsic -- width)
    /// calculation, unless the global `bubble_widths_separately` flag is on.
    ///
    /// All flows must be finished at some point, or they will not have their intrinsic widths
    /// properly computed. (This is not, however, a memory safety problem.)
    fn finish(&mut self, context: &mut LayoutContext) {
        if !context.opts.bubble_widths_separately {
            self.bubble_widths(context)
        }
    }

    /// Set absolute descendants for this flow.
    ///
    /// Set yourself as the Containing Block for all the absolute descendants.
    ///
    /// Assumption: This is called in a bottom-up traversal, so that nothing
    /// else is accessing the descendant flows.
    fn set_abs_descendants(&mut self, abs_descendants: AbsDescendants) {
        let self_link = Rawlink::some(*self);
        let block = self.as_block();
        block.base.abs_descendants = abs_descendants;

        for descendant_link in block.base.abs_descendants.iter() {
            match descendant_link.resolve() {
                Some(flow) => {
                    let base = mut_base(flow);
                    base.absolute_cb = self_link.clone();
                }
                None => fail!("empty Rawlink to a descendant")
            }
        }
    }

    /// Set fixed descendants for this flow.
    ///
    /// Set yourself as the Containing Block for all the fixed descendants.
    ///
    /// Assumption: This is called in a bottom-up traversal, so that nothing
    /// else is accessing the descendant flows.
    /// Assumption: This is the root flow.
    fn set_fixed_descendants(&mut self, fixed_descendants: FixedDescendants) {
        let self_link = Rawlink::some(*self);
        let block = self.as_block();
        block.base.fixed_descendants = fixed_descendants;

        for descendant_link in block.base.fixed_descendants.iter() {
            match descendant_link.resolve() {
                Some(flow) => {
                    let base = mut_base(flow);
                    base.absolute_cb = self_link.clone();
                }
                None => fail!("empty Rawlink to a descendant")
            }
        }
    }

    /// Destroys the flow.
    fn destroy(&mut self) {
        let self_borrowed: &mut Flow = *self;
        self_borrowed.destroy();
    }
}
