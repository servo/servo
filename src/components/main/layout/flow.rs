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
use layout::block::BlockFlow;
use layout::box_::{Box, TableRowBox, TableCellBox};
use layout::construct::OptVector;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, DisplayListBuildingInfo, ToGfxColor};
use layout::floats::Floats;
use layout::flow_list::{FlowList, Link, Rawlink, FlowListIterator, MutFlowListIterator};
use layout::incremental::RestyleDamage;
use layout::inline::InlineFlow;
use layout::model::{CollapsibleMargins, IntrinsicWidths, MarginCollapseInfo};
use layout::parallel::FlowParallelInfo;
use layout::parallel;
use layout::table_wrapper::TableWrapperFlow;
use layout::table::TableFlow;
use layout::table_colgroup::TableColGroupFlow;
use layout::table_rowgroup::TableRowGroupFlow;
use layout::table_row::TableRowFlow;
use layout::table_caption::TableCaptionFlow;
use layout::table_cell::TableCellFlow;
use layout::wrapper::ThreadSafeLayoutNode;

use collections::Deque;
use geom::Size2D;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::color::Color;
use gfx::display_list::StackingContext;
use servo_msg::compositor_msg::LayerId;
use servo_util::geometry::Au;
use servo_util::smallvec::{SmallVec, SmallVec0};
use std::cast;
use std::iter::Zip;
use std::sync::atomics::Relaxed;
use std::slice::MutItems;
use style::ComputedValues;
use style::computed_values::{clear, position, text_align};

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

    /// If this is a table wrapper flow, returns the underlying object. Fails otherwise.
    fn as_table_wrapper<'a>(&'a mut self) -> &'a mut TableWrapperFlow {
        fail!("called as_table_wrapper() on a non-tablewrapper flow")
    }

    /// If this is a table flow, returns the underlying object. Fails otherwise.
    fn as_table<'a>(&'a mut self) -> &'a mut TableFlow {
        fail!("called as_table() on a non-table flow")
    }

    /// If this is a table colgroup flow, returns the underlying object. Fails otherwise.
    fn as_table_colgroup<'a>(&'a mut self) -> &'a mut TableColGroupFlow {
        fail!("called as_table_colgroup() on a non-tablecolgroup flow")
    }

    /// If this is a table rowgroup flow, returns the underlying object. Fails otherwise.
    fn as_table_rowgroup<'a>(&'a mut self) -> &'a mut TableRowGroupFlow {
        fail!("called as_table_rowgroup() on a non-tablerowgroup flow")
    }

    /// If this is a table row flow, returns the underlying object. Fails otherwise.
    fn as_table_row<'a>(&'a mut self) -> &'a mut TableRowFlow {
        fail!("called as_table_row() on a non-tablerow flow")
    }

    /// If this is a table cell flow, returns the underlying object. Fails otherwise.
    fn as_table_caption<'a>(&'a mut self) -> &'a mut TableCaptionFlow {
        fail!("called as_table_caption() on a non-tablecaption flow")
    }

    /// If this is a table cell flow, returns the underlying object. Fails otherwise.
    fn as_table_cell<'a>(&'a mut self) -> &'a mut TableCellFlow {
        fail!("called as_table_cell() on a non-tablecell flow")
    }

    /// If this is a table row or table rowgroup or table flow, returns column widths.
    /// Fails otherwise.
    fn col_widths<'a>(&'a mut self) -> &'a mut ~[Au] {
        fail!("called col_widths() on an other flow than table-row/table-rowgroup/table")
    }

    /// If this is a table row flow or table rowgroup flow or table flow, returns column min widths.
    /// Fails otherwise.
    fn col_min_widths<'a>(&'a self) -> &'a ~[Au] {
        fail!("called col_min_widths() on an other flow than table-row/table-rowgroup/table")
    }

    /// If this is a table row flow or table rowgroup flow or table flow, returns column min widths.
    /// Fails otherwise.
    fn col_pref_widths<'a>(&'a self) -> &'a ~[Au] {
        fail!("called col_pref_widths() on an other flow than table-row/table-rowgroup/table")
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

    fn compute_collapsible_top_margin(&mut self,
                                      _layout_context: &mut LayoutContext,
                                      _margin_collapse_info: &mut MarginCollapseInfo) {
        // The default implementation is a no-op.
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

    /// Returns true if this is an absolute containing block.
    fn is_absolute_containing_block(&self) -> bool {
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

    /// Returns a layer ID for the given fragment.
    fn layer_id(&self, fragment_id: uint) -> LayerId {
        unsafe {
            let pointer: uint = cast::transmute(self);
            LayerId(pointer, fragment_id)
        }
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
    fn generate_missing_child_flow(self, node: &ThreadSafeLayoutNode) -> ~Flow;

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

    /// Builds the display lists for this flow and its descendants.
    fn build_display_list(self,
                          stacking_context: &mut StackingContext,
                          builder: &mut DisplayListBuilder,
                          info: &DisplayListBuildingInfo);

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

    /// Destroys the flow.
    fn destroy(&mut self);
}

pub enum FlowClass {
    BlockFlowClass,
    InlineFlowClass,
    TableWrapperFlowClass,
    TableFlowClass,
    TableColGroupFlowClass,
    TableRowGroupFlowClass,
    TableRowFlowClass,
    TableCaptionFlowClass,
    TableCellFlowClass,
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
    pub flags: FlowFlags,

    /// text-decoration colors
    pub rare_flow_flags: Option<~RareFlowFlags>,
}

#[deriving(Clone)]
pub struct RareFlowFlags {
    pub underline_color: Color,
    pub overline_color: Color,
    pub line_through_color: Color,
}

/// Flags used in flows, tightly packed to save space.
#[deriving(Clone)]
pub struct FlowFlags(pub u8);

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

// Whether this flow contains a flow that has its own layer within the same absolute containing
// block.
bitfield!(FlowFlags,
          layers_needed_for_descendants,
          set_layers_needed_for_descendants,
          0b0100_0000)

// Whether this flow must have its own layer. Even if this flag is not set, it might get its own
// layer if it's deemed to be likely to overlap flows with their own layer.
bitfield!(FlowFlags, needs_layer, set_needs_layer, 0b1000_0000)

// The text alignment for this flow.
impl FlowFlags {
    #[inline]
    pub fn text_align(self) -> text_align::T {
        let FlowFlags(ff) = self;
        FromPrimitive::from_u8((ff & TEXT_ALIGN_BITMASK) >> TEXT_ALIGN_SHIFT).unwrap()
    }

    #[inline]
    pub fn set_text_align(&mut self, value: text_align::T) {
        let FlowFlags(ff) = *self;
        *self = FlowFlags((ff & !TEXT_ALIGN_BITMASK) | ((value as u8) << TEXT_ALIGN_SHIFT))
    }

    #[inline]
    pub fn set_text_align_override(&mut self, parent: FlowFlags) {
        let FlowFlags(ff) = *self;
        let FlowFlags(pff) = parent;
        *self = FlowFlags(ff | (pff & TEXT_ALIGN_BITMASK))
    }

    #[inline]
    pub fn set_text_decoration_override(&mut self, parent: FlowFlags) {
        let FlowFlags(ff) = *self;
        let FlowFlags(pff) = parent;
        *self = FlowFlags(ff | (pff & TEXT_DECORATION_OVERRIDE_BITMASK));
    }

    #[inline]
    pub fn is_text_decoration_enabled(&self) -> bool {
        let FlowFlags(ref ff) = *self;
        (*ff & TEXT_DECORATION_OVERRIDE_BITMASK) != 0
    }
}

/// The Descendants of a flow.
///
/// Also, details about their position wrt this flow.
/// FIXME: This should use @pcwalton's reference counting scheme (Coming Soon).
pub struct Descendants {
    /// Links to every Descendant.
    pub descendant_links: SmallVec0<Rawlink>,
    /// Static y offsets of all descendants from the start of this flow box.
    pub static_y_offsets: SmallVec0<Au>,
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

pub type DescendantIter<'a> = MutItems<'a, Rawlink>;

pub type DescendantOffsetIter<'a> = Zip<MutItems<'a, Rawlink>, MutItems<'a, Au>>;

/// Data common to all flows.
pub struct BaseFlow {
    pub restyle_damage: RestyleDamage,

    /// The children of this flow.
    pub children: FlowList,
    pub next_sibling: Link,
    pub prev_sibling: Rawlink,

    /* layout computations */
    // TODO: min/pref and position are used during disjoint phases of
    // layout; maybe combine into a single enum to save space.
    pub intrinsic_widths: IntrinsicWidths,

    /// The upper left corner of the box representing this flow, relative to the box representing
    /// its parent flow.
    ///
    /// For absolute flows, this represents the position with respect to its *containing block*.
    ///
    /// This does not include margins in the block flow direction, because those can collapse. So
    /// for the block direction (usually vertical), this represents the *border box*. For the
    /// inline direction (usually horizontal), this represents the *margin box*.
    pub position: Rect<Au>,

    /// The amount of overflow of this flow, relative to the containing block. Must include all the
    /// pixels of all the display list items for correct invalidation.
    pub overflow: Rect<Au>,

    /// Data used during parallel traversals.
    ///
    /// TODO(pcwalton): Group with other transient data to save space.
    pub parallel: FlowParallelInfo,

    /// The floats next to this flow.
    pub floats: Floats,

    /// The value of this flow's `clear` property, if any.
    pub clear: clear::T,

    /// For normal flows, this is the number of floated descendants that are
    /// not contained within any other floated descendant of this flow. For
    /// floats, it is 1.
    /// It is used to allocate float data if necessary and to
    /// decide whether to do an in-order traversal for assign_height.
    pub num_floats: uint,

    /// The collapsible margins for this flow, if any.
    pub collapsible_margins: CollapsibleMargins,

    /// The position of this flow in page coordinates, computed during display list construction.
    pub abs_position: Point2D<Au>,

    /// Details about descendants with position 'absolute' or 'fixed' for which we are the
    /// containing block. This is in tree order. This includes any direct children.
    pub abs_descendants: AbsDescendants,

    /// Offset wrt the nearest positioned ancestor - aka the Containing Block
    /// for any absolutely positioned elements.
    pub absolute_static_x_offset: Au,
    /// Offset wrt the Initial Containing Block.
    pub fixed_static_x_offset: Au,

    /// Reference to the Containing Block, if this flow is absolutely positioned.
    pub absolute_cb: Rawlink,

    /// Whether this flow has been destroyed.
    ///
    /// TODO(pcwalton): Pack this into the flags? Need to be careful because manipulation of this
    /// flag can have memory safety implications.
    destroyed: bool,

    /// Various flags for flows and some info
    pub flags_info: FlowFlagsInfo,
}

#[unsafe_destructor]
impl Drop for BaseFlow {
    fn drop(&mut self) {
        if !self.destroyed {
            fail!("Flow destroyed by going out of scope—this is unsafe! Use `destroy()` instead!")
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

            intrinsic_widths: IntrinsicWidths::new(),
            position: Rect::zero(),
            overflow: Rect::zero(),

            parallel: FlowParallelInfo::new(),

            floats: Floats::new(),
            num_floats: 0,
            collapsible_margins: CollapsibleMargins::new(),
            clear: clear::none,
            abs_position: Point2D(Au::new(0), Au::new(0)),
            abs_descendants: Descendants::new(),
            absolute_static_x_offset: Au::new(0),
            fixed_static_x_offset: Au::new(0),
            absolute_cb: Rawlink::none(),

            destroyed: false,

            flags_info: FlowFlagsInfo::new(&**style),
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
            _ => false,
        }
    }

    /// Returns true if this flow is a proper table child.
    /// 'Proper table child' is defined as table-row flow, table-rowgroup flow,
    /// table-column-group flow, or table-caption flow.
    fn is_proper_table_child(self) -> bool {
        match self.class() {
            TableRowFlowClass | TableRowGroupFlowClass |
                TableColGroupFlowClass | TableCaptionFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table row flow.
    fn is_table_row(self) -> bool {
        match self.class() {
            TableRowFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table cell flow.
    fn is_table_cell(self) -> bool {
        match self.class() {
            TableCellFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table colgroup flow.
    fn is_table_colgroup(self) -> bool {
        match self.class() {
            TableColGroupFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table flow.
    fn is_table(self) -> bool {
        match self.class() {
            TableFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table caption flow.
    fn is_table_caption(self) -> bool {
        match self.class() {
            TableCaptionFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if this flow is a table rowgroup flow.
    fn is_table_rowgroup(self) -> bool {
        match self.class() {
            TableRowGroupFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if this flow is one of table-related flows.
    fn is_table_kind(self) -> bool {
        match self.class() {
            TableWrapperFlowClass | TableFlowClass |
                TableColGroupFlowClass | TableRowGroupFlowClass |
                TableRowFlowClass | TableCaptionFlowClass | TableCellFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if anonymous flow is needed between this flow and child flow.
    /// Spec: http://www.w3.org/TR/CSS21/tables.html#anonymous-boxes
    fn need_anonymous_flow(self, child: &Flow) -> bool {
        match self.class() {
            TableFlowClass => !child.is_proper_table_child(),
            TableRowGroupFlowClass => !child.is_table_row(),
            TableRowFlowClass => !child.is_table_cell(),
            _ => false
        }
    }

    /// Generates missing child flow of this flow.
    fn generate_missing_child_flow(self, node: &ThreadSafeLayoutNode) -> ~Flow {
        match self.class() {
            TableFlowClass | TableRowGroupFlowClass => {
                let box_ = Box::new_anonymous_table_box(node, TableRowBox);
                ~TableRowFlow::from_node_and_box(node, box_) as ~Flow
            },
            TableRowFlowClass => {
                let box_ = Box::new_anonymous_table_box(node, TableCellBox);
                ~TableCellFlow::from_node_and_box(node, box_) as ~Flow
            },
            _ => {
                fail!("no need to generate a missing child")
            }
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
            BlockFlowClass | TableCaptionFlowClass | TableCellFlowClass => {
                // FIXME: Actually check the type of the node
                self.child_count() != 0
            }
            _ => false,
        }
    }

    /// Returns true if this flow is a block flow.
    fn is_block_flow(self) -> bool {
        match self.class() {
            BlockFlowClass => true,
            _ => false,
        }
    }

    /// Returns true if this flow is an inline flow.
    fn is_inline_flow(self) -> bool {
        match self.class() {
            InlineFlowClass => true,
            _ => false,
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

            // FIXME(#2004, pcwalton): This is wrong for `position: fixed`.
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
        }
        mut_base(self).overflow = overflow;
    }

    /// Push display items for current flow and its descendants onto the appropriate display lists
    /// of the given stacking context.
    ///
    /// Arguments:
    ///
    /// * `stacking_context`: The parent stacking context that this flow belongs to and to which
    ///   display items will be added.
    ///
    /// * `builder`: The display list builder, which contains information used during the entire
    ///   display list building pass.
    ///
    /// * `info`: Per-flow display list building information.
    fn build_display_list(self,
                          stacking_context: &mut StackingContext,
                          builder: &mut DisplayListBuilder,
                          info: &DisplayListBuildingInfo) {
        debug!("Flow: building display list");
        match self.class() {
            BlockFlowClass => {
                self.as_block().build_display_list_block(stacking_context, builder, info)
            }
            InlineFlowClass => {
                self.as_inline().build_display_list_inline(stacking_context, builder, info)
            }
            TableWrapperFlowClass => {
                self.as_table_wrapper().build_display_list_table_wrapper(stacking_context,
                                                                         builder,
                                                                         info)
            }
            TableFlowClass => {
                self.as_table().build_display_list_table(stacking_context, builder, info)
            }
            TableRowGroupFlowClass => {
                self.as_table_rowgroup().build_display_list_table_rowgroup(stacking_context,
                                                                           builder,
                                                                           info)
            }
            TableRowFlowClass => {
                self.as_table_row().build_display_list_table_row(stacking_context, builder, info)
            }
            TableCaptionFlowClass => {
                self.as_table_caption().build_display_list_table_caption(stacking_context,
                                                                         builder,
                                                                         info)
            }
            TableCellFlowClass => {
                self.as_table_cell().build_display_list_table_cell(stacking_context, builder, info)
            }
            TableColGroupFlowClass => {
                // Nothing to do here, as column groups don't render.
            }
        }
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

    /// Destroys the flow.
    fn destroy(&mut self) {
        let self_borrowed: &mut Flow = *self;
        self_borrowed.destroy();
    }
}
