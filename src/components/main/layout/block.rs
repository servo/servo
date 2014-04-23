/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS block formatting contexts.
//!
//! Terminology Note:
//! As per the CSS Spec, the term 'absolute positioning' here refers to
//! elements with position = 'absolute' or 'fixed'.
//! The term 'positioned element' refers to elements with position =
//! 'relative', 'absolute', or 'fixed'.
//!
//! CB: Containing Block of the current flow.

use layout::box_::{Box, ImageBox, ScannedTextBox};
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, DisplayListBuildingInfo};
use layout::floats::{ClearBoth, ClearLeft, ClearRight, FloatKind, Floats, PlacementInfo};
use layout::flow::{BaseFlow, BlockFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow::{MutableFlowUtils, PreorderFlowTraversal, PostorderFlowTraversal, mut_base};
use layout::flow;
use layout::model::{Auto, IntrinsicWidths, MarginCollapseInfo, MarginsCollapse};
use layout::model::{MarginsCollapseThrough, MaybeAuto, NoCollapsibleMargins, Specified, specified};
use layout::model::{specified_or_none};
use layout::wrapper::ThreadSafeLayoutNode;
use style::ComputedValues;
use style::computed_values::{clear, position};

use geom::{Point2D, Rect, Size2D};
use gfx::color;
use gfx::display_list::{BackgroundAndBorderLevel, BlockLevel, RootOfStackingContextLevel};
use gfx::display_list::{StackingContext};
use gfx::render_task::RenderLayer;
use servo_msg::compositor_msg::{FixedPosition, LayerId, Scrollable};
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::smallvec::{SmallVec, SmallVec0};
use style::computed_values::{LPA_Auto, LPA_Length, LPA_Percentage, LPN_Length, LPN_None};
use style::computed_values::{LPN_Percentage, LP_Length, LP_Percentage};
use sync::Arc;

/// Information specific to floated blocks.
pub struct FloatedBlockInfo {
    containing_width: Au,

    /// Offset relative to where the parent tried to position this flow
    rel_pos: Point2D<Au>,

    /// Index into the box list for inline floats
    index: Option<uint>,

    /// Number of floated children
    floated_children: uint,

    /// Left or right?
    float_kind: FloatKind,
}

impl FloatedBlockInfo {
    pub fn new(float_kind: FloatKind) -> FloatedBlockInfo {
        FloatedBlockInfo {
            containing_width: Au(0),
            rel_pos: Point2D(Au(0), Au(0)),
            index: None,
            floated_children: 0,
            float_kind: float_kind,
        }
    }
}

/// The solutions for the heights-and-margins constraint equation.
struct HeightConstraintSolution {
    top: Au,
    bottom: Au,
    height: Au,
    margin_top: Au,
    margin_bottom: Au
}

impl HeightConstraintSolution {
    fn new(top: Au, bottom: Au, height: Au, margin_top: Au, margin_bottom: Au)
           -> HeightConstraintSolution {
        HeightConstraintSolution {
            top: top,
            bottom: bottom,
            height: height,
            margin_top: margin_top,
            margin_bottom: margin_bottom,
        }
    }

    /// Solve the vertical constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.6.4
    /// Constraint equation:
    /// top + bottom + height + margin-top + margin-bottom
    /// = absolute containing block height - (vertical padding and border)
    /// [aka available_height]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_nonreplaced(height: MaybeAuto,
                                                  top_margin: MaybeAuto,
                                                  bottom_margin: MaybeAuto,
                                                  top: MaybeAuto,
                                                  bottom: MaybeAuto,
                                                  content_height: Au,
                                                  available_height: Au,
                                                  static_y_offset: Au)
                                                  -> HeightConstraintSolution {
        // Distance from the top edge of the Absolute Containing Block to the
        // top margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_top = static_y_offset;

        let (top, bottom, height, margin_top, margin_bottom) = match (top, bottom, height) {
            (Auto, Auto, Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let top = static_position_top;
                // Now it is the same situation as top Specified and bottom
                // and height Auto.

                let height = content_height;
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
            (Specified(top), Specified(bottom), Specified(height)) => {
                match (top_margin, bottom_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = available_height - top - bottom - height;
                        (top, bottom, height,
                         total_margin_val.scale_by(0.5),
                         total_margin_val.scale_by(0.5))
                    }
                    (Specified(margin_top), Auto) => {
                        let sum = top + bottom + height + margin_top;
                        (top, bottom, height, margin_top, available_height - sum)
                    }
                    (Auto, Specified(margin_bottom)) => {
                        let sum = top + bottom + height + margin_bottom;
                        (top, bottom, height, available_height - sum, margin_bottom)
                    }
                    (Specified(margin_top), Specified(margin_bottom)) => {
                        // Values are over-constrained. Ignore value for 'bottom'.
                        let sum = top + height + margin_top + margin_bottom;
                        (top, available_height - sum, height, margin_top, margin_bottom)
                    }
                }
            }

            // For the rest of the cases, auto values for margin are set to 0

            // If only one is Auto, solve for it
            (Auto, Specified(bottom), Specified(height)) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = bottom + height + margin_top + margin_bottom;
                (available_height - sum, bottom, height, margin_top, margin_bottom)
            }
            (Specified(top), Auto, Specified(height)) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
            (Specified(top), Specified(bottom), Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = top + bottom + margin_top + margin_bottom;
                (top, bottom, available_height - sum, margin_top, margin_bottom)
            }

            // If height is auto, then height is content height. Solve for the
            // non-auto value.
            (Specified(top), Auto, Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let height = content_height;
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
            (Auto, Specified(bottom), Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let height = content_height;
                let sum = bottom + height + margin_top + margin_bottom;
                (available_height - sum, bottom, height, margin_top, margin_bottom)
            }

            (Auto, Auto, Specified(height)) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let top = static_position_top;
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
        };
        HeightConstraintSolution::new(top, bottom, height, margin_top, margin_bottom)
    }

    /// Solve the vertical constraint equation for absolute replaced elements.
    ///
    /// Assumption: The used value for height has already been calculated.
    ///
    /// CSS Section 10.6.5
    /// Constraint equation:
    /// top + bottom + height + margin-top + margin-bottom
    /// = absolute containing block height - (vertical padding and border)
    /// [aka available_height]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_replaced(height: Au,
                                               top_margin: MaybeAuto,
                                               bottom_margin: MaybeAuto,
                                               top: MaybeAuto,
                                               bottom: MaybeAuto,
                                               _: Au,
                                               available_height: Au,
                                               static_y_offset: Au)
                                               -> HeightConstraintSolution {
        // Distance from the top edge of the Absolute Containing Block to the
        // top margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_top = static_y_offset;

        let (top, bottom, height, margin_top, margin_bottom) = match (top, bottom) {
            (Auto, Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let top = static_position_top;
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
            (Specified(top), Specified(bottom)) => {
                match (top_margin, bottom_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = available_height - top - bottom - height;
                        (top, bottom, height,
                         total_margin_val.scale_by(0.5),
                         total_margin_val.scale_by(0.5))
                    }
                    (Specified(margin_top), Auto) => {
                        let sum = top + bottom + height + margin_top;
                        (top, bottom, height, margin_top, available_height - sum)
                    }
                    (Auto, Specified(margin_bottom)) => {
                        let sum = top + bottom + height + margin_bottom;
                        (top, bottom, height, available_height - sum, margin_bottom)
                    }
                    (Specified(margin_top), Specified(margin_bottom)) => {
                        // Values are over-constrained. Ignore value for 'bottom'.
                        let sum = top + height + margin_top + margin_bottom;
                        (top, available_height - sum, height, margin_top, margin_bottom)
                    }
                }
            }

            // If only one is Auto, solve for it
            (Auto, Specified(bottom)) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = bottom + height + margin_top + margin_bottom;
                (available_height - sum, bottom, height, margin_top, margin_bottom)
            }
            (Specified(top), Auto) => {
                let margin_top = top_margin.specified_or_zero();
                let margin_bottom = bottom_margin.specified_or_zero();
                let sum = top + height + margin_top + margin_bottom;
                (top, available_height - sum, height, margin_top, margin_bottom)
            }
        };
        HeightConstraintSolution::new(top, bottom, height, margin_top, margin_bottom)
    }
}

/// Performs height calculations potentially multiple times, taking `height`, `min-height`, and
/// `max-height` into account. After each call to `next()`, the caller must call `.try()` with the
/// current calculated value of `height`.
///
/// See CSS 2.1 § 10.7.
struct CandidateHeightIterator {
    height: MaybeAuto,
    max_height: Option<Au>,
    min_height: Au,
    candidate_value: Au,
    status: CandidateHeightIteratorStatus,
}

impl CandidateHeightIterator {
    /// Creates a new candidate height iterator. `block_container_height` is `None` if the height
    /// of the block container has not been determined yet. It will always be `Some` in the case of
    /// absolutely-positioned containing blocks.
    pub fn new(style: &ComputedValues, block_container_height: Option<Au>)
               -> CandidateHeightIterator {
        // Per CSS 2.1 § 10.7, percentages in `min-height` and `max-height` refer to the height of
        // the containing block. If that is not determined yet by the time we need to resolve
        // `min-height` and `max-height`, percentage values are ignored.

        let height = match (style.Box.get().height, block_container_height) {
            (LPA_Percentage(percent), Some(block_container_height)) => {
                Specified(block_container_height.scale_by(percent))
            }
            (LPA_Percentage(_), None) | (LPA_Auto, _) => Auto,
            (LPA_Length(length), _) => Specified(length),
        };
        let max_height = match (style.Box.get().max_height, block_container_height) {
            (LPN_Percentage(percent), Some(block_container_height)) => {
                Some(block_container_height.scale_by(percent))
            }
            (LPN_Percentage(_), None) | (LPN_None, _) => None,
            (LPN_Length(length), _) => Some(length),
        };
        let min_height = match (style.Box.get().min_height, block_container_height) {
            (LP_Percentage(percent), Some(block_container_height)) => {
                block_container_height.scale_by(percent)
            }
            (LP_Percentage(_), None) => Au(0),
            (LP_Length(length), _) => length,
        };

        CandidateHeightIterator {
            height: height,
            max_height: max_height,
            min_height: min_height,
            candidate_value: Au(0),
            status: InitialCandidateHeightStatus,
        }
    }

    pub fn next<'a>(&'a mut self) -> Option<(MaybeAuto, &'a mut Au)> {
        self.status = match self.status {
            InitialCandidateHeightStatus => TryingHeightCandidateHeightStatus,
            TryingHeightCandidateHeightStatus => {
                match self.max_height {
                    Some(max_height) if self.candidate_value > max_height => {
                        TryingMaxCandidateHeightStatus
                    }
                    _ if self.candidate_value < self.min_height => TryingMinCandidateHeightStatus,
                    _ => FoundCandidateHeightStatus,
                }
            }
            TryingMaxCandidateHeightStatus => {
                if self.candidate_value < self.min_height {
                    TryingMinCandidateHeightStatus
                } else {
                    FoundCandidateHeightStatus
                }
            }
            TryingMinCandidateHeightStatus | FoundCandidateHeightStatus => {
                FoundCandidateHeightStatus
            }
        };

        match self.status {
            TryingHeightCandidateHeightStatus => Some((self.height, &mut self.candidate_value)),
            TryingMaxCandidateHeightStatus => {
                Some((Specified(self.max_height.unwrap()), &mut self.candidate_value))
            }
            TryingMinCandidateHeightStatus => {
                Some((Specified(self.min_height), &mut self.candidate_value))
            }
            FoundCandidateHeightStatus => None,
            InitialCandidateHeightStatus => fail!(),
        }
    }
}

enum CandidateHeightIteratorStatus {
    InitialCandidateHeightStatus,
    TryingHeightCandidateHeightStatus,
    TryingMaxCandidateHeightStatus,
    TryingMinCandidateHeightStatus,
    FoundCandidateHeightStatus,
}

// A helper function used in height calculation.
fn translate_including_floats(cur_y: &mut Au, delta: Au, inorder: bool, floats: &mut Floats) {
    *cur_y = *cur_y + delta;

    if inorder {
        floats.translate(Point2D(Au(0), -delta));
    }
}

/// The real assign-heights traversal for flows with position 'absolute'.
///
/// This is a traversal of an Absolute Flow tree.
/// - Relatively positioned flows and the Root flow start new Absolute flow trees.
/// - The kids of a flow in this tree will be the flows for which it is the
/// absolute Containing Block.
/// - Thus, leaf nodes and inner non-root nodes are all Absolute Flows.
///
/// A Flow tree can have several Absolute Flow trees (depending on the number
/// of relatively positioned flows it has).
///
/// Note that flows with position 'fixed' just form a flat list as they all
/// have the Root flow as their CB.
struct AbsoluteAssignHeightsTraversal<'a>(&'a mut LayoutContext);

impl<'a> PreorderFlowTraversal for AbsoluteAssignHeightsTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        let block_flow = flow.as_block();

        // The root of the absolute flow tree is definitely not absolutely
        // positioned. Nothing to process here.
        if block_flow.is_root_of_absolute_flow_tree() {
            return true;
        }


        let AbsoluteAssignHeightsTraversal(ref ctx) = *self;
        block_flow.calculate_abs_height_and_margins(*ctx);
        true
    }
}

/// The store-overflow traversal particular to absolute flows.
///
/// Propagate overflow up the Absolute flow tree and update overflow up to and
/// not including the root of the Absolute flow tree.
/// After that, it is up to the normal store-overflow traversal to propagate
/// it further up.
struct AbsoluteStoreOverflowTraversal<'a>{
    layout_context: &'a mut LayoutContext,
}

impl<'a> PostorderFlowTraversal for AbsoluteStoreOverflowTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        // This will be taken care of by the normal store-overflow traversal.
        if flow.is_root_of_absolute_flow_tree() {
            return true;
        }

        flow.store_overflow(self.layout_context);
        true
    }
}

enum BlockType {
    BlockReplacedType,
    BlockNonReplacedType,
    AbsoluteReplacedType,
    AbsoluteNonReplacedType,
    FloatReplacedType,
    FloatNonReplacedType,
}

#[deriving(Clone, Eq)]
pub enum MarginsMayCollapseFlag {
    MarginsMayCollapse,
    MarginsMayNotCollapse,
}

// Propagates the `layers_needed_for_descendants` flag appropriately from a child. This is called
// as part of height assignment.
//
// If any fixed descendants of kids are present, this kid needs a layer.
//
// FIXME(#2006, pcwalton): This is too layer-happy. Like WebKit, we shouldn't do this unless
// the positioned descendants are actually on top of the fixed kids.
//
// TODO(#1244, #2007, pcwalton): Do this for CSS transforms and opacity too, at least if they're
// animating.
fn propagate_layer_flag_from_child(layers_needed_for_descendants: &mut bool, kid: &mut Flow) {
    if kid.is_absolute_containing_block() {
        let kid_base = flow::mut_base(kid);
        if kid_base.flags.needs_layer() {
            *layers_needed_for_descendants = true
        }
    } else {
        let kid_base = flow::mut_base(kid);
        if kid_base.flags.layers_needed_for_descendants() {
            *layers_needed_for_descendants = true
        }
    }
}

// A block formatting context.
pub struct BlockFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Box,

    /// TODO: is_root should be a bit field to conserve memory.
    /// Whether this block flow is the root flow.
    is_root: bool,

    /// Static y offset of an absolute flow from its CB.
    static_y_offset: Au,

    /// Additional floating flow members.
    float: Option<~FloatedBlockInfo>
}

impl BlockFlow {
    pub fn from_node(constructor: &mut FlowConstructor, node: &ThreadSafeLayoutNode) -> BlockFlow {
        BlockFlow {
            base: BaseFlow::new((*node).clone()),
            box_: Box::new(constructor, node),
            is_root: false,
            static_y_offset: Au::new(0),
            float: None
        }
    }

    pub fn from_node_and_box(node: &ThreadSafeLayoutNode, box_: Box) -> BlockFlow {
        BlockFlow {
            base: BaseFlow::new((*node).clone()),
            box_: box_,
            is_root: false,
            static_y_offset: Au::new(0),
            float: None
        }
    }

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> BlockFlow {
        BlockFlow {
            base: BaseFlow::new((*node).clone()),
            box_: Box::new(constructor, node),
            is_root: false,
            static_y_offset: Au::new(0),
            float: Some(~FloatedBlockInfo::new(float_kind))
        }
    }

    /// Return the type of this block.
    ///
    /// This determines the algorithm used to calculate width, height, and the
    /// relevant margins for this Block.
    fn block_type(&self) -> BlockType {
        if self.is_absolutely_positioned() {
            if self.is_replaced_content() {
                AbsoluteReplacedType
            } else {
                AbsoluteNonReplacedType
            }
        } else if self.is_float() {
            if self.is_replaced_content() {
                FloatReplacedType
            } else {
                FloatNonReplacedType
            }
        } else {
            if self.is_replaced_content() {
                BlockReplacedType
            } else {
                BlockNonReplacedType
            }
        }
    }

    /// Compute the used value of width for this Block.
    fn compute_used_width(&mut self, ctx: &mut LayoutContext, containing_block_width: Au) {
        let block_type = self.block_type();
        match block_type {
            AbsoluteReplacedType => {
                let width_computer = AbsoluteReplaced;
                width_computer.compute_used_width(self, ctx, containing_block_width);
            }
            AbsoluteNonReplacedType => {
                let width_computer = AbsoluteNonReplaced;
                width_computer.compute_used_width(self, ctx, containing_block_width);
            }
            FloatReplacedType => {
                let width_computer = FloatReplaced;
                width_computer.compute_used_width(self, ctx, containing_block_width);
            }
            FloatNonReplacedType => {
                let width_computer = FloatNonReplaced;
                width_computer.compute_used_width(self, ctx, containing_block_width);
            }
            BlockReplacedType => {
                let width_computer = BlockReplaced;
                width_computer.compute_used_width(self, ctx, containing_block_width);
            }
            BlockNonReplacedType => {
                let width_computer = BlockNonReplaced;
                width_computer.compute_used_width(self, ctx, containing_block_width);
            }
        }
    }

    /// Return this flow's box.
    pub fn box_<'a>(&'a mut self) -> &'a mut Box {
        &mut self.box_
    }

    /// Return the static x offset from the appropriate Containing Block for this flow.
    pub fn static_x_offset(&self) -> Au {
        if self.is_fixed() {
            self.base.fixed_static_x_offset
        } else {
            self.base.absolute_static_x_offset
        }
    }

    /// Return the size of the Containing Block for this flow.
    ///
    /// Right now, this only gets the Containing Block size for absolutely
    /// positioned elements.
    /// Note: Assume this is called in a top-down traversal, so it is ok to
    /// reference the CB.
    pub fn containing_block_size(&mut self, viewport_size: Size2D<Au>) -> Size2D<Au> {
        assert!(self.is_absolutely_positioned());
        if self.is_fixed() {
            // Initial containing block is the CB for the root
            viewport_size
        } else {
            let cb = self.base.absolute_cb.resolve().unwrap();
            cb.generated_cb_size()
        }
    }

    /// Traverse the Absolute flow tree in preorder.
    ///
    /// Traverse all your direct absolute descendants, who will then traverse
    /// their direct absolute descendants.
    /// Also, set the static y offsets for each descendant (using the value
    /// which was bubbled up during normal assign-height).
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_preorder_absolute_flows<T:PreorderFlowTraversal>(&mut self,
                                                                 traversal: &mut T)
                                                                 -> bool {
        let flow = self as &mut Flow;
        if traversal.should_prune(flow) {
            return true
        }

        if !traversal.process(flow) {
            return false
        }

        let cb_top_edge_offset = flow.generated_cb_position().y;
        let mut descendant_offset_iter = mut_base(flow).abs_descendants.iter_with_offset();
        // Pass in the respective static y offset for each descendant.
        for (ref mut descendant_link, ref y_offset) in descendant_offset_iter {
            match descendant_link.resolve() {
                Some(flow) => {
                    let block = flow.as_block();
                    // The stored y_offset is wrt to the flow box.
                    // Translate it to the CB (which is the padding box).
                    block.static_y_offset = **y_offset - cb_top_edge_offset;
                    if !block.traverse_preorder_absolute_flows(traversal) {
                        return false
                    }
                }
                None => fail!("empty Rawlink to a descendant")
            }
        }

        true
    }

    /// Traverse the Absolute flow tree in postorder.
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_postorder_absolute_flows<T:PostorderFlowTraversal>(&mut self,
                                                                   traversal: &mut T)
                                                                   -> bool {
        let flow = self as &mut Flow;
        if traversal.should_prune(flow) {
            return true
        }

        for descendant_link in mut_base(flow).abs_descendants.iter() {
            match descendant_link.resolve() {
                Some(abs_flow) => {
                    let block = abs_flow.as_block();
                    if !block.traverse_postorder_absolute_flows(traversal) {
                        return false
                    }
                }
                None => fail!("empty Rawlink to a descendant")
            }
        }

        traversal.process(flow)
    }

    /// Return true if this has a replaced box.
    ///
    /// The only two types of replaced boxes currently are text boxes and
    /// image boxes.
    fn is_replaced_content(&self) -> bool {
        match self.box_.specific {
            ScannedTextBox(_) | ImageBox(_) => true,
            _ => false,
        }
    }

    pub fn teardown(&mut self) {
        self.box_.teardown();
        self.float = None;
    }

    /// Return shrink-to-fit width.
    ///
    /// This is where we use the preferred widths and minimum widths
    /// calculated in the bubble-widths traversal.
    fn get_shrink_to_fit_width(&self, available_width: Au) -> Au {
        geometry::min(self.base.intrinsic_widths.preferred_width,
                      geometry::max(self.base.intrinsic_widths.minimum_width, available_width))
    }

    /// Collect and update static y-offsets bubbled up by kids.
    ///
    /// This would essentially give us offsets of all absolutely positioned
    /// direct descendants and all fixed descendants, in tree order.
    ///
    /// Assume that this is called in a bottom-up traversal (specifically, the
    /// assign-height traversal). So, kids have their flow origin already set.
    /// In the case of absolute flow kids, they have their hypothetical box
    /// position already set.
    fn collect_static_y_offsets_from_kids(&mut self) {
        let mut abs_descendant_y_offsets = SmallVec0::new();
        for kid in self.base.child_iter() {
            let mut gives_abs_offsets = true;
            if kid.is_block_like() {
                let kid_block = kid.as_block();
                if kid_block.is_fixed() || kid_block.is_absolutely_positioned() {
                    // It won't contribute any offsets for descendants because it
                    // would be the CB for them.
                    gives_abs_offsets = false;
                    // Give the offset for the current absolute flow alone.
                    abs_descendant_y_offsets.push(kid_block.get_hypothetical_top_edge());
                } else if kid_block.is_positioned() {
                    // It won't contribute any offsets because it would be the CB
                    // for the descendants.
                    gives_abs_offsets = false;
                }
            }

            if gives_abs_offsets {
                let kid_base = flow::mut_base(kid);
                // Consume all the static y-offsets bubbled up by kid.
                for y_offset in kid_base.abs_descendants.static_y_offsets.move_iter() {
                    // The offsets are wrt the kid flow box. Translate them to current flow.
                    abs_descendant_y_offsets.push(y_offset + kid_base.position.origin.y);
                }
            }
        }
        self.base.abs_descendants.static_y_offsets = abs_descendant_y_offsets;
    }

    /// If this is the root flow, shifts all kids down and adjusts our size to account for
    /// collapsed margins.
    ///
    /// TODO(#2017, pcwalton): This is somewhat inefficient (traverses kids twice); can we do
    /// better?
    fn adjust_boxes_for_collapsed_margins_if_root(&mut self) {
        if !self.is_root() {
            return
        }

        let (top_margin_value, bottom_margin_value) = match self.base.collapsible_margins {
            MarginsCollapseThrough(margin) => (Au(0), margin.collapse()),
            MarginsCollapse(top_margin, bottom_margin) => {
                (top_margin.collapse(), bottom_margin.collapse())
            }
            NoCollapsibleMargins(top, bottom) => (top, bottom),
        };

        // Shift all kids down (or up, if margins are negative) if necessary.
        if top_margin_value != Au(0) {
            for kid in self.base.child_iter() {
                let kid_base = flow::mut_base(kid);
                kid_base.position.origin.y = kid_base.position.origin.y + top_margin_value
            }
        }

        self.base.position.size.height = self.base.position.size.height + top_margin_value +
            bottom_margin_value;

        let mut position = self.box_.border_box.get();
        position.size.height = position.size.height + top_margin_value + bottom_margin_value;
        self.box_.border_box.set(position);
    }

    /// Assign height for current flow.
    ///
    /// * Collapse margins for flow's children and set in-flow child flows' y-coordinates now that
    ///   we know their heights.
    /// * Calculate and set the height of the current flow.
    /// * Calculate height, vertical margins, and y-coordinate for the flow's box. Ideally, this
    ///   should be calculated using CSS § 10.6.7.
    ///
    /// For absolute flows, we store the calculated content height for the flow. We defer the
    /// calculation of the other values until a later traversal.
    ///
    /// `inline(always)` because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    pub fn assign_height_block_base(&mut self,
                                    layout_context: &mut LayoutContext,
                                    inorder: bool,
                                    margins_may_collapse: MarginsMayCollapseFlag) {
        // Our current border-box position.
        let mut cur_y = Au(0);

        // Absolute positioning establishes a block formatting context. Don't propagate floats
        // in or out. (But do propagate them between kids.)
        if inorder && self.is_absolutely_positioned() {
            self.base.floats = Floats::new();
        }
        if margins_may_collapse != MarginsMayCollapse {
            self.base.floats = Floats::new();
        }

        let mut margin_collapse_info = MarginCollapseInfo::new();
        self.base.floats.translate(Point2D(-self.box_.left_offset(), Au(0)));

        // The sum of our top border and top padding.
        let top_offset = self.box_.border.get().top + self.box_.padding.get().top;
        translate_including_floats(&mut cur_y, top_offset, inorder, &mut self.base.floats);

        let can_collapse_top_margin_with_kids =
            margins_may_collapse == MarginsMayCollapse &&
            !self.is_absolutely_positioned() &&
            self.box_.border.get().top == Au(0) &&
            self.box_.padding.get().top == Au(0);
        margin_collapse_info.initialize_top_margin(&self.box_,
                                                   can_collapse_top_margin_with_kids);

        // At this point, cur_y is at the content edge of the flow's box.
        let mut floats = self.base.floats.clone();
        let mut layers_needed_for_descendants = false;
        for kid in self.base.child_iter() {
            if kid.is_absolutely_positioned() {
                // Assume that the *hypothetical box* for an absolute flow starts immediately after
                // the bottom border edge of the previous flow.
                kid.as_block().base.position.origin.y = cur_y;

                if inorder {
                    kid.assign_height_inorder(layout_context)
                }

                propagate_layer_flag_from_child(&mut layers_needed_for_descendants, kid);

                // Skip the collapsing and float processing for absolute flow kids and continue
                // with the next flow.
                continue
            }

            // Assign height now for the child if it was impacted by floats and we couldn't before.
            let mut floats_out = None;
            if inorder {
                if !kid.is_float() {
                    let kid_base = flow::mut_base(kid);
                    if kid_base.clear != clear::none {
                        // We have clearance, so assume there are no floats in and perform layout.
                        //
                        // FIXME(#2008, pcwalton): This could be wrong if we have `clear: left` or
                        // `clear: right` and there are still floats to impact, of course. But this
                        // gets complicated with margin collapse. Possibly the right thing to do is
                        // to lay out the block again in this rare case. (Note that WebKit can lay
                        // blocks out twice; this may be related, although I haven't looked into it
                        // closely.)
                        kid_base.floats = Floats::new()
                    } else {
                        kid_base.floats = floats.clone()
                    }
                } else {
                    let kid_base = flow::mut_base(kid);
                    kid_base.position.origin.y = margin_collapse_info.current_float_ceiling();
                    kid_base.floats = floats.clone()
                }

                kid.assign_height_inorder(layout_context);

                floats_out = Some(flow::mut_base(kid).floats.clone());

                // A horrible hack that has to do with the fact that `origin.y` is used for
                // something else later (containing block for float).
                if kid.is_float() {
                    flow::mut_base(kid).position.origin.y = cur_y;
                }
            }

            propagate_layer_flag_from_child(&mut layers_needed_for_descendants, kid);

            // If the child was a float, stop here.
            if kid.is_float() {
                if inorder {
                    floats = floats_out.take_unwrap();
                }
                continue
            }

            // Handle any (possibly collapsed) top margin.
            let kid_base = flow::mut_base(kid);
            let delta = margin_collapse_info.advance_top_margin(&kid_base.collapsible_margins);
            translate_including_floats(&mut cur_y, delta, inorder, &mut floats);

            // Clear past floats, if necessary.
            if inorder {
                let clearance = match kid_base.clear {
                    clear::none => Au(0),
                    clear::left => floats.clearance(ClearLeft),
                    clear::right => floats.clearance(ClearRight),
                    clear::both => floats.clearance(ClearBoth),
                };
                cur_y = cur_y + clearance
            }

            // At this point, `cur_y` is at the border edge of the child.
            assert!(kid_base.position.origin.y == Au(0));
            kid_base.position.origin.y = cur_y;

            // If this was an inorder traversal, grab the child's floats now.
            if inorder {
                floats = floats_out.take_unwrap()
            }

            // Move past the child's border box. Do not use the `translate_including_floats`
            // function here because the child has already translated floats past its border box.
            cur_y = cur_y + kid_base.position.size.height;

            // Handle any (possibly collapsed) bottom margin.
            let delta = margin_collapse_info.advance_bottom_margin(&kid_base.collapsible_margins);
            translate_including_floats(&mut cur_y, delta, inorder, &mut floats);
        }

        self.base.flags.set_layers_needed_for_descendants(layers_needed_for_descendants);

        self.collect_static_y_offsets_from_kids();

        // Add in our bottom margin and compute our collapsible margins.
        let can_collapse_bottom_margin_with_kids =
            margins_may_collapse == MarginsMayCollapse &&
            !self.is_absolutely_positioned() &&
            self.box_.border.get().bottom == Au(0) &&
            self.box_.padding.get().bottom == Au(0);
        let (collapsible_margins, delta) =
            margin_collapse_info.finish_and_compute_collapsible_margins(
            &self.box_,
            can_collapse_bottom_margin_with_kids);
        self.base.collapsible_margins = collapsible_margins;
        translate_including_floats(&mut cur_y, delta, inorder, &mut floats);

        // FIXME(#2003, pcwalton): The max is taken here so that you can scroll the page, but this
        // is not correct behavior according to CSS 2.1 § 10.5. Instead I think we should treat the
        // root element as having `overflow: scroll` and use the layers-based scrolling
        // infrastructure to make it scrollable.
        let mut height = cur_y - top_offset;
        if self.is_root() {
            height = Au::max(layout_context.screen_size.height, height)
        }

        if self.is_absolutely_positioned() {
            // The content height includes all the floats per CSS 2.1 § 10.6.7. The easiest way to
            // handle this is to just treat this as clearance.
            height = height + floats.clearance(ClearBoth);

            // Fixed position layers get layers.
            if self.is_fixed() {
                self.base.flags.set_needs_layer(true)
            }

            // Store the content height for use in calculating the absolute flow's dimensions
            // later.
            let mut temp_position = self.box_.border_box.get();
            temp_position.size.height = height;
            self.box_.border_box.set(temp_position);
            return
        }

        let mut candidate_height_iterator = CandidateHeightIterator::new(self.box_.style(),
                                                                         None);
        for (candidate_height, new_candidate_height) in candidate_height_iterator {
            *new_candidate_height = match candidate_height {
                Auto => height,
                Specified(value) => value
            }
        }

        // Adjust `cur_y` as necessary to account for the explicitly-specified height.
        height = candidate_height_iterator.candidate_value;
        let delta = height - (cur_y - top_offset);
        translate_including_floats(&mut cur_y, delta, inorder, &mut floats);

        // Compute content height and noncontent height.
        let bottom_offset = self.box_.border.get().bottom + self.box_.padding.get().bottom;
        translate_including_floats(&mut cur_y, bottom_offset, inorder, &mut floats);

        // Now that `cur_y` is at the bottom of the border box, compute the final border box
        // position.
        let mut border_box = self.box_.border_box.get();
        border_box.size.height = cur_y;
        border_box.origin.y = Au(0);
        self.box_.border_box.set(border_box);
        self.base.position.size.height = cur_y;

        self.base.floats = floats.clone();
        self.adjust_boxes_for_collapsed_margins_if_root();

        if self.is_root_of_absolute_flow_tree() {
            // Assign heights for all flows in this Absolute flow tree.
            // This is preorder because the height of an absolute flow may depend on
            // the height of its CB, which may also be an absolute flow.
            self.traverse_preorder_absolute_flows(&mut AbsoluteAssignHeightsTraversal(
                    layout_context));
            // Store overflow for all absolute descendants.
            self.traverse_postorder_absolute_flows(&mut AbsoluteStoreOverflowTraversal {
                layout_context: layout_context,
            });
        }
    }

    /// Add placement information about current float flow for use by the parent.
    ///
    /// Also, use information given by parent about other floats to find out
    /// our relative position.
    ///
    /// This does not give any information about any float descendants because
    /// they do not affect elements outside of the subtree rooted at this
    /// float.
    ///
    /// This function is called on a kid flow by a parent.
    /// Therefore, assign_height_float was already called on this kid flow by
    /// the traversal function. So, the values used are well-defined.
    pub fn assign_height_float_inorder(&mut self) {
        let height = self.box_.border_box.get().size.height;
        let clearance = match self.box_.clear() {
            None => Au(0),
            Some(clear) => self.base.floats.clearance(clear),
        };

        let noncontent_width = self.box_.padding.get().left + self.box_.padding.get().right +
            self.box_.border.get().left + self.box_.border.get().right;

        let full_noncontent_width = noncontent_width + self.box_.margin.get().left +
            self.box_.margin.get().right;
        let margin_height = self.box_.margin.get().top + self.box_.margin.get().bottom;

        let info = PlacementInfo {
            size: Size2D(self.base.position.size.width + full_noncontent_width,
                         height + margin_height),
            ceiling: clearance + self.base.position.origin.y,
            max_width: self.float.get_ref().containing_width,
            kind: self.float.get_ref().float_kind,
        };

        // Place the float and return the `Floats` back to the parent flow.
        // After, grab the position and use that to set our position.
        self.base.floats.add_float(&info);

        self.float.get_mut_ref().rel_pos = self.base.floats.last_float_pos().unwrap();
    }

    /// Assign height for current flow.
    ///
    /// + Set in-flow child flows' y-coordinates now that we know their
    /// heights. This _doesn't_ do any margin collapsing for its children.
    /// + Calculate height and y-coordinate for the flow's box. Ideally, this
    /// should be calculated using CSS Section 10.6.7
    ///
    /// It does not calculate the height of the flow itself.
    pub fn assign_height_float(&mut self, ctx: &mut LayoutContext) {
        // Now that we've determined our height, propagate that out.
        let has_inorder_children = self.base.num_floats > 0;
        if has_inorder_children {
            let mut floats = Floats::new();
            for kid in self.base.child_iter() {
                flow::mut_base(kid).floats = floats;
                kid.assign_height_inorder(ctx);
                floats = flow::mut_base(kid).floats.clone();
            }

            // Floats establish a block formatting context, so we discard the output floats here.
            drop(floats);
        }
        let mut cur_y = Au(0);

        let top_offset = self.box_.margin.get().top + self.box_.border.get().top + self.box_.padding.get().top;
        cur_y = cur_y + top_offset;

        // cur_y is now at the top content edge

        for kid in self.base.child_iter() {
            let child_base = flow::mut_base(kid);
            child_base.position.origin.y = cur_y;
            // cur_y is now at the bottom margin edge of kid
            cur_y = cur_y + child_base.position.size.height;
        }

        let content_height = cur_y - top_offset;

        let mut noncontent_height;
        let mut position = self.box_.border_box.get();

        // The associated box is the border box of this flow.
        position.origin.y = self.box_.margin.get().top;

        noncontent_height = self.box_.padding.get().top + self.box_.padding.get().bottom +
            self.box_.border.get().top + self.box_.border.get().bottom;

        // Calculate content height, taking `min-height` and `max-height` into account.

        let mut candidate_height_iterator = CandidateHeightIterator::new(self.box_.style(), None);
        for (candidate_height, new_candidate_height) in candidate_height_iterator {
            *new_candidate_height = match candidate_height {
                Auto => content_height,
                Specified(value) => value,
            }
        }

        let content_height = candidate_height_iterator.candidate_value;

        debug!("assign_height_float -- height: {}", content_height + noncontent_height);

        position.size.height = content_height + noncontent_height;
        self.box_.border_box.set(position);
    }

    fn build_display_list_block_common(&mut self,
                                       stacking_context: &mut StackingContext,
                                       builder: &mut DisplayListBuilder,
                                       info: &DisplayListBuildingInfo,
                                       offset: Point2D<Au>,
                                       background_border_level: BackgroundAndBorderLevel) {
        let mut info = *info;
        let rel_offset = self.box_.relative_position(&info.relative_containing_block_size);

        // Add the box that starts the block context.
        self.box_.build_display_list(stacking_context,
                                     builder,
                                     &info,
                                     self.base.abs_position + rel_offset + offset,
                                     background_border_level);

        // For relatively-positioned descendants, the containing block formed by a block is
        // just the content box. The containing block for absolutely-positioned descendants,
        // on the other hand, only established if we are positioned.
        info.relative_containing_block_size = self.box_.content_box_size();
        if self.is_positioned() {
            info.absolute_containing_block_position =
                self.base.abs_position +
                self.generated_cb_position() +
                self.box_.relative_position(&info.relative_containing_block_size)
        }

        let this_position = self.base.abs_position;
        for kid in self.base.child_iter() {
            {
                let kid_base = flow::mut_base(kid);
                kid_base.abs_position = this_position + kid_base.position.origin + rel_offset +
                    offset;
            }

            if kid.is_absolutely_positioned() {
                // All absolute flows will be handled by their containing block.
                continue
            }

            kid.build_display_list(stacking_context, builder, &info);
        }

        // Process absolute descendant links.
        let mut absolute_info = info;
        absolute_info.layers_needed_for_positioned_flows =
            self.base.flags.layers_needed_for_descendants();
        for abs_descendant_link in self.base.abs_descendants.iter() {
            match abs_descendant_link.resolve() {
                Some(flow) => {
                    // TODO(pradeep): Send in our absolute position directly.
                    flow.build_display_list(stacking_context, builder, &absolute_info)
                }
                None => fail!("empty Rawlink to a descendant")
            }
        }
    }

    /// Add display items for current block.
    ///
    /// Set the absolute position for children after doing any offsetting for
    /// position: relative.
    pub fn build_display_list_block(&mut self,
                                    stacking_context: &mut StackingContext,
                                    builder: &mut DisplayListBuilder,
                                    info: &DisplayListBuildingInfo) {
        if self.is_float() {
            // TODO(#2009, pcwalton): This is a pseudo-stacking context. We need to merge `z-index:
            // auto` kids into the parent stacking context, when that is supported.
            self.build_display_list_float(stacking_context, builder, info)
        } else if self.is_absolutely_positioned() {
            self.build_display_list_abs(stacking_context, builder, info)
        } else {
            self.build_display_list_block_common(stacking_context,
                                                 builder,
                                                 info,
                                                 Point2D(Au(0), Au(0)),
                                                 BlockLevel)
        }
    }

    pub fn build_display_list_float(&mut self,
                                    parent_stacking_context: &mut StackingContext,
                                    builder: &mut DisplayListBuilder,
                                    info: &DisplayListBuildingInfo) {
        let mut stacking_context = StackingContext::new();
        let float_offset = self.float.get_ref().rel_pos;
        self.build_display_list_block_common(&mut stacking_context,
                                             builder,
                                             info,
                                             float_offset,
                                             RootOfStackingContextLevel);
        parent_stacking_context.floats.push_all_move(stacking_context.flatten())
    }

    /// Calculate and set the height, offsets, etc. for absolutely positioned flow.
    ///
    /// The layout for its in-flow children has been done during normal layout.
    /// This is just the calculation of:
    /// + height for the flow
    /// + y-coordinate of the flow wrt its Containing Block.
    /// + height, vertical margins, and y-coordinate for the flow's box.
    fn calculate_abs_height_and_margins(&mut self, ctx: &LayoutContext) {
        let containing_block_height = self.containing_block_size(ctx.screen_size).height;
        let static_y_offset = self.static_y_offset;

        // This is the stored content height value from assign-height
        let content_height = self.box_.border_box.get().size.height - self.box_.noncontent_height();

        let style = self.box_.style();

        // Non-auto margin-top and margin-bottom values have already been
        // calculated during assign-width.
        let margin = self.box_.margin.get();
        let margin_top = match MaybeAuto::from_style(style.Margin.get().margin_top, Au(0)) {
            Auto => Auto,
            _ => Specified(margin.top)
        };
        let margin_bottom = match MaybeAuto::from_style(style.Margin.get().margin_bottom, Au(0)) {
            Auto => Auto,
            _ => Specified(margin.bottom)
        };

        let (top, bottom) =
            (MaybeAuto::from_style(style.PositionOffsets.get().top, containing_block_height),
             MaybeAuto::from_style(style.PositionOffsets.get().bottom, containing_block_height));
        let available_height = containing_block_height - self.box_.noncontent_height();

        let mut solution = None;
        if self.is_replaced_content() {
            // Calculate used value of height just like we do for inline replaced elements.
            // TODO: Pass in the containing block height when Box's
            // assign-height can handle it correctly.
            self.box_.assign_replaced_height_if_necessary();
            // TODO: Right now, this content height value includes the
            // margin because of erroneous height calculation in Box_.
            // Check this when that has been fixed.
            let height_used_val = self.box_.border_box.get().size.height;
            solution = Some(HeightConstraintSolution::solve_vertical_constraints_abs_replaced(
                    height_used_val,
                    margin_top,
                    margin_bottom,
                    top,
                    bottom,
                    content_height,
                    available_height,
                    static_y_offset));
        } else {
            let mut candidate_height_iterator =
                CandidateHeightIterator::new(style, Some(containing_block_height));

            for (height_used_val, new_candidate_height) in candidate_height_iterator {
                solution =
                    Some(HeightConstraintSolution::solve_vertical_constraints_abs_nonreplaced(
                        height_used_val,
                        margin_top,
                        margin_bottom,
                        top,
                        bottom,
                        content_height,
                        available_height,
                        static_y_offset));

                *new_candidate_height = solution.unwrap().height
            }
        }

        let solution = solution.unwrap();

        let mut margin = self.box_.margin.get();
        margin.top = solution.margin_top;
        margin.bottom = solution.margin_bottom;
        self.box_.margin.set(margin);

        let mut position = self.box_.border_box.get();
        position.origin.y = Au(0);
        // Border box height
        let border_and_padding = self.box_.noncontent_height();
        position.size.height = solution.height + border_and_padding;
        self.box_.border_box.set(position);

        self.base.position.origin.y = solution.top + margin.top;
        self.base.position.size.height = solution.height + border_and_padding;
    }

    /// Add display items for Absolutely Positioned flow.
    pub fn build_display_list_abs(&mut self,
                                  parent_stacking_context: &mut StackingContext,
                                  builder: &mut DisplayListBuilder,
                                  info: &DisplayListBuildingInfo) {
        let mut stacking_context = StackingContext::new();
        let mut info = *info;

        info.absolute_containing_block_position = if self.is_fixed() {
            // The viewport is initially at (0, 0).
            self.base.position.origin
        } else {
            // Absolute position of Containing Block + position of absolute flow
            // wrt Containing Block
            info.absolute_containing_block_position + self.base.position.origin
        };

        // Set the absolute position, which will be passed down later as part
        // of containing block details for absolute descendants.
        self.base.abs_position = info.absolute_containing_block_position;

        self.build_display_list_block_common(&mut stacking_context,
                                             builder,
                                             &info,
                                             Point2D(Au(0), Au(0)),
                                             RootOfStackingContextLevel);

        if !info.layers_needed_for_positioned_flows && !self.base.flags.needs_layer() {
            // We didn't need a layer.
            //
            // TODO(#781, pcwalton): `z-index`.
            parent_stacking_context.positioned_descendants.push((0, stacking_context.flatten()));
            return
        }

        // If we got here, then we need a new layer.
        let size = Size2D(self.base.position.size.width.to_nearest_px() as uint,
                          self.base.position.size.height.to_nearest_px() as uint);
        let origin = Point2D(info.absolute_containing_block_position.x.to_nearest_px() as uint,
                             info.absolute_containing_block_position.y.to_nearest_px() as uint);
        let scroll_policy = if self.is_fixed() {
            FixedPosition
        } else {
            Scrollable
        };
        let new_layer = RenderLayer {
            id: self.layer_id(0),
            display_list: Arc::new(stacking_context.flatten()),
            position: Rect(origin, size),
            background_color: color::rgba(255.0, 255.0, 255.0, 0.0),
            scroll_policy: scroll_policy,
        };
        builder.layers.push(new_layer)
    }

    /// Return the top outer edge of the Hypothetical Box for an absolute flow.
    ///
    /// This is wrt its parent flow box.
    ///
    /// During normal layout assign-height, the absolute flow's position is
    /// roughly set to its static position (the position it would have had in
    /// the normal flow).
    fn get_hypothetical_top_edge(&self) -> Au {
        self.base.position.origin.y
    }

    /// Initializes the containing width if this block flow is a float. This is done at the start
    /// of `assign_widths`.
    fn set_containing_width_if_float(&mut self, containing_block_width: Au) {
        if self.is_float() {
            self.float.get_mut_ref().containing_width = containing_block_width;

            // Parent usually sets this, but floats are never inorder
            self.base.flags.set_inorder(false)
        }
    }

    /// Assigns the computed left content edge and width to all the children of this block flow.
    pub fn propagate_assigned_width_to_children(&mut self,
                                                left_content_edge: Au,
                                                content_width: Au,
                                                opt_col_widths: Option<~[Au]>) {
        let has_inorder_children = if self.is_float() {
            self.base.num_floats > 0
        } else {
            self.base.flags.inorder() || self.base.num_floats > 0
        };

        let kid_abs_cb_x_offset;
        if self.is_positioned() {
            // Pass yourself as a new Containing Block
            // The static x offset for any immediate kid flows will be the
            // left padding
            kid_abs_cb_x_offset = self.box_.padding.get().left;
        } else {
            // For kids, the left margin edge will be at our left content edge.
            // The current static offset is at our left margin
            // edge. So move in to the left content edge.
            kid_abs_cb_x_offset = self.base.absolute_static_x_offset + left_content_edge;
        }
        let kid_fixed_cb_x_offset = self.base.fixed_static_x_offset + left_content_edge;

        // This value is used only for table cells.
        let mut kid_left_margin_edge = left_content_edge;

        let flags = self.base.flags.clone();
        for (i, kid) in self.base.child_iter().enumerate() {
            if kid.is_block_flow() {
                let kid_block = kid.as_block();
                kid_block.base.absolute_static_x_offset = kid_abs_cb_x_offset;
                kid_block.base.fixed_static_x_offset = kid_fixed_cb_x_offset;
            }

            {
                let child_base = flow::mut_base(kid);
                // Left margin edge of kid flow is at our left content edge
                child_base.position.origin.x = left_content_edge;
                // Width of kid flow is our content width
                child_base.position.size.width = content_width;
                child_base.flags.set_inorder(has_inorder_children);

                if !child_base.flags.inorder() {
                    child_base.floats = Floats::new();
                }
            }

            // Handle tables.
            match opt_col_widths {
                Some(ref col_widths) => {
                    // If kid is table_rowgroup or table_row, the column widths info should be
                    // copied from its parent.
                    let kid_width;
                    if kid.is_table() || kid.is_table_rowgroup() || kid.is_table_row() {
                        *kid.col_widths() = col_widths.clone();

                        // Width of kid flow is our content width.
                        kid_width = content_width
                    } else if kid.is_table_cell() {
                        // If kid is table_cell, the x offset and width for each cell should be
                        // calculated from parent's column widths info.
                        kid_left_margin_edge = if i == 0 {
                            Au(0)
                        } else {
                            kid_left_margin_edge + col_widths[i-1]
                        };

                        kid_width = col_widths[i]
                    } else {
                        // Width of kid flow is our content width.
                        kid_width = content_width
                    }

                    let kid_base = flow::mut_base(kid);
                    kid_base.position.origin.x = kid_left_margin_edge;
                    kid_base.position.size.width = kid_width;
                }
                None => {}
            }

            // Per CSS 2.1 § 16.3.1, text alignment propagates to all children in flow.
            //
            // TODO(#2018, pcwalton): Do this in the cascade instead.
            flow::mut_base(kid).flags.propagate_text_alignment_from_parent(flags.clone())
        }
    }
}

impl Flow for BlockFlow {
    fn class(&self) -> FlowClass {
        BlockFlowClass
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        self
    }

    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    /* TODO: inline-blocks */
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut num_floats = 0;

        // Find the maximum width from children.
        let mut intrinsic_widths = IntrinsicWidths::new();
        for child_ctx in self.base.child_iter() {
            assert!(child_ctx.is_block_flow() || child_ctx.is_inline_flow() || child_ctx.is_table_kind());

            let child_base = flow::mut_base(child_ctx);
            intrinsic_widths.minimum_width =
                geometry::max(intrinsic_widths.minimum_width,
                              child_base.intrinsic_widths.total_minimum_width());
            intrinsic_widths.preferred_width =
                geometry::max(intrinsic_widths.preferred_width,
                              child_base.intrinsic_widths.total_preferred_width());
            num_floats = num_floats + child_base.num_floats;
        }

        if self.is_float() {
            self.base.num_floats = 1;
            self.float.get_mut_ref().floated_children = num_floats;
        } else {
            self.base.num_floats = num_floats;
        }

        // Add in borders, padding, and margins.
        {
            // Can compute border width here since it doesn't depend on anything.
            self.box_.compute_borders(self.box_.style())
        }

        let box_intrinsic_widths = self.box_.intrinsic_widths();
        intrinsic_widths.minimum_width = geometry::max(intrinsic_widths.minimum_width,
                                                       box_intrinsic_widths.minimum_width);
        intrinsic_widths.preferred_width = geometry::max(intrinsic_widths.preferred_width,
                                                         box_intrinsic_widths.preferred_width);
        intrinsic_widths.surround_width = box_intrinsic_widths.surround_width;

        self.base.intrinsic_widths = intrinsic_widths
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    ///
    /// Dual boxes consume some width first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow",
               if self.is_float() {
                   "float"
               } else {
                   "block"
               });

        if self.is_root() {
            debug!("Setting root position");
            self.base.position.origin = Au::zero_point();
            self.base.position.size.width = ctx.screen_size.width;
            self.base.floats = Floats::new();
            // Root element is not floated
            self.base.flags.set_inorder(false);
        }

        // The position was set to the containing block by the flow's parent.
        let containing_block_width = self.base.position.size.width;

        self.set_containing_width_if_float(containing_block_width);

        self.compute_used_width(ctx, containing_block_width);

        // Assign `clear` now so that the assign-heights pass will have the correct value for
        // it.
        self.base.clear = self.box_.style().Box.get().clear;

        // Move in from the left border edge
        let left_content_edge = self.box_.border_box.get().origin.x
            + self.box_.padding.get().left + self.box_.border.get().left;
        let padding_and_borders = self.box_.padding.get().left + self.box_.padding.get().right +
            self.box_.border.get().left + self.box_.border.get().right;
        let content_width = self.box_.border_box.get().size.width - padding_and_borders;

        if self.is_float() {
            self.base.position.size.width = content_width;
        }

        self.propagate_assigned_width_to_children(left_content_edge, content_width, None);
    }

    /// This is called on kid flows by a parent.
    ///
    /// Hence, we can assume that assign_height has already been called on the
    /// kid (because of the bottom-up traversal).
    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        if self.is_float() {
            debug!("assign_height_inorder_float: assigning height for float");
            self.assign_height_float_inorder();
        } else {
            debug!("assign_height_inorder: assigning height for block");
            self.assign_height_block_base(ctx, true, MarginsMayCollapse);
        }
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        // Assign height for box if it is an image box.
        self.box_.assign_replaced_height_if_necessary();

        if self.is_float() {
            debug!("assign_height_float: assigning height for float");
            self.assign_height_float(ctx);
        } else {
            debug!("assign_height: assigning height for block");
            // This is the only case in which a block flow can start an inorder
            // subtraversal.
            if self.is_root() && self.base.num_floats > 0 {
                self.assign_height_inorder(ctx);
                return;
            }
            self.assign_height_block_base(ctx, false, MarginsMayCollapse);
        }
    }

    fn mark_as_root(&mut self) {
        self.is_root = true
    }

    /// Return true if store overflow is delayed for this flow.
    ///
    /// Currently happens only for absolutely positioned flows.
    fn is_store_overflow_delayed(&mut self) -> bool {
        self.is_absolutely_positioned()
    }

    fn is_root(&self) -> bool {
        self.is_root
    }

    fn is_float(&self) -> bool {
        self.float.is_some()
    }

    /// The 'position' property of this flow.
    fn positioning(&self) -> position::T {
        self.box_.style.get().Box.get().position
    }

    /// Return true if this is the root of an Absolute flow tree.
    ///
    /// It has to be either relatively positioned or the Root flow.
    fn is_root_of_absolute_flow_tree(&self) -> bool {
        self.is_relatively_positioned() || self.is_root()
    }

    /// Return the dimensions of the CB generated _by_ this flow for absolute descendants.
    ///
    /// For Blocks, this will be the padding box.
    fn generated_cb_size(&self) -> Size2D<Au> {
        self.box_.padding_box_size()
    }

    /// Return position of the CB generated by this flow from the start of this flow.
    fn generated_cb_position(&self) -> Point2D<Au> {
        // Border box y coordinate + border top
        self.box_.border_box.get().origin + Point2D(self.box_.border.get().left, self.box_.border.get().top)
    }

    fn layer_id(&self, fragment_index: uint) -> LayerId {
        // FIXME(#2010, pcwalton): This is a hack and is totally bogus in the presence of pseudo-
        // elements. But until we have incremental reflow we can't do better--we recreate the flow
        // for every DOM node so otherwise we nuke layers on every reflow.
        LayerId(self.box_.node.id(), fragment_index)
    }

    fn debug_str(&self) -> ~str {
        let txt = if self.is_float() {
            ~"FloatFlow: "
        } else if self.is_root() {
            ~"RootFlow: "
        } else {
            ~"BlockFlow: "
        };
        txt.append(self.box_.debug_str())
    }

    fn is_absolute_containing_block(&self) -> bool {
        self.is_positioned()
    }
}

/// The inputs for the widths-and-margins constraint equation.
pub struct WidthConstraintInput {
    computed_width: MaybeAuto,
    left_margin: MaybeAuto,
    right_margin: MaybeAuto,
    left: MaybeAuto,
    right: MaybeAuto,
    available_width: Au,
    static_x_offset: Au,
}

impl WidthConstraintInput {
    pub fn new(computed_width: MaybeAuto,
               left_margin: MaybeAuto,
               right_margin: MaybeAuto,
               left: MaybeAuto,
               right: MaybeAuto,
               available_width: Au,
               static_x_offset: Au)
           -> WidthConstraintInput {
        WidthConstraintInput {
            computed_width: computed_width,
            left_margin: left_margin,
            right_margin: right_margin,
            left: left,
            right: right,
            available_width: available_width,
            static_x_offset: static_x_offset,
        }
    }
}

/// The solutions for the widths-and-margins constraint equation.
pub struct WidthConstraintSolution {
    left: Au,
    right: Au,
    width: Au,
    margin_left: Au,
    margin_right: Au
}

impl WidthConstraintSolution {
    pub fn new(width: Au, margin_left: Au, margin_right: Au) -> WidthConstraintSolution {
        WidthConstraintSolution {
            left: Au(0),
            right: Au(0),
            width: width,
            margin_left: margin_left,
            margin_right: margin_right,
        }
    }

    fn for_absolute_flow(left: Au,
                         right: Au,
                         width: Au,
                         margin_left: Au,
                         margin_right: Au)
                         -> WidthConstraintSolution {
        WidthConstraintSolution {
            left: left,
            right: right,
            width: width,
            margin_left: margin_left,
            margin_right: margin_right,
        }
    }
}

// Trait to encapsulate the Width and Margin calculation.
//
// CSS Section 10.3
pub trait WidthAndMarginsComputer {
    /// Compute the inputs for the Width constraint equation.
    ///
    /// This is called only once to compute the initial inputs. For
    /// calculation involving min-width and max-width, we don't need to
    /// recompute these.
    fn compute_width_constraint_inputs(&self,
                                       block: &mut BlockFlow,
                                       parent_flow_width: Au,
                                       ctx: &mut LayoutContext)
                       -> WidthConstraintInput {
        let containing_block_width = self.containing_block_width(block, parent_flow_width, ctx);
        let computed_width = self.initial_computed_width(block, parent_flow_width, ctx);
        let style = block.box_.style();

        // The text alignment of a block flow is the text alignment of its box's style.
        block.base.flags.set_text_align(style.InheritedText.get().text_align);

        block.box_.compute_padding(style, containing_block_width);

        // We calculate and set margin-top and margin-bottom here
        // because CSS 2.1 defines % on this wrt CB *width*.
        block.box_.compute_margin_top_bottom(containing_block_width);

        let (margin_left, margin_right) =
            (MaybeAuto::from_style(style.Margin.get().margin_left, containing_block_width),
             MaybeAuto::from_style(style.Margin.get().margin_right, containing_block_width));

        let (left, right) =
            (MaybeAuto::from_style(style.PositionOffsets.get().left, containing_block_width),
             MaybeAuto::from_style(style.PositionOffsets.get().right, containing_block_width));
        let available_width = containing_block_width - block.box_.noncontent_width();
        return WidthConstraintInput::new(computed_width,
                                         margin_left,
                                         margin_right,
                                         left,
                                         right,
                                         available_width,
                                         block.static_x_offset());
    }

    /// Set the used values for width and margins got from the relevant constraint equation.
    ///
    /// This is called only once.
    ///
    /// Set:
    /// + used values for content width, left margin, and right margin for this flow's box.
    /// + x-coordinate of this flow's box.
    /// + x-coordinate of the flow wrt its Containing Block (if this is an absolute flow).
    fn set_width_constraint_solutions(&self,
                                      block: &mut BlockFlow,
                                      solution: WidthConstraintSolution) {
        let box_ = block.box_();
        let mut margin = box_.margin.get();
        margin.left = solution.margin_left;
        margin.right = solution.margin_right;
        box_.margin.set(margin);

        // The associated box is the border box of this flow.
        let mut position_ref = box_.border_box.borrow_mut();
        // Left border edge.
        position_ref.origin.x = box_.margin.borrow().left;

        // Border box width
        position_ref.size.width = solution.width + box_.noncontent_width();
    }

    /// Set the x coordinate of the given flow if it is absolutely positioned.
    fn set_flow_x_coord_if_necessary(&self, _: &mut BlockFlow, _: WidthConstraintSolution) {}

    /// Solve the width and margins constraints for this block flow.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution;

    fn initial_computed_width(&self,
                              block: &mut BlockFlow,
                              parent_flow_width: Au,
                              ctx: &mut LayoutContext)
                              -> MaybeAuto {
        MaybeAuto::from_style(block.box_().style().Box.get().width,
                              self.containing_block_width(block, parent_flow_width, ctx))
    }

    fn containing_block_width(&self,
                              _: &mut BlockFlow,
                              parent_flow_width: Au,
                              _: &mut LayoutContext)
                              -> Au {
        parent_flow_width
    }

    /// Compute the used value of width, taking care of min-width and max-width.
    ///
    /// CSS Section 10.4: Minimum and Maximum widths
    fn compute_used_width(&self,
                          block: &mut BlockFlow,
                          ctx: &mut LayoutContext,
                          parent_flow_width: Au) {
        let mut input = self.compute_width_constraint_inputs(block, parent_flow_width, ctx);

        let containing_block_width = self.containing_block_width(block, parent_flow_width, ctx);

        let mut solution = self.solve_width_constraints(block, input);

        // If the tentative used width is greater than 'max-width', width should be recalculated,
        // but this time using the computed value of 'max-width' as the computed value for 'width'.
        match specified_or_none(block.box_().style().Box.get().max_width, containing_block_width) {
            Some(max_width) if max_width < solution.width => {
                input.computed_width = Specified(max_width);
                solution = self.solve_width_constraints(block, input);
            }
            _ => {}
        }

        // If the resulting width is smaller than 'min-width', width should be recalculated,
        // but this time using the value of 'min-width' as the computed value for 'width'.
        let computed_min_width = specified(block.box_().style().Box.get().min_width,
                                           containing_block_width);
        if computed_min_width > solution.width {
            input.computed_width = Specified(computed_min_width);
            solution = self.solve_width_constraints(block, input);
        }

        self.set_width_constraint_solutions(block, solution);
        self.set_flow_x_coord_if_necessary(block, solution);
    }

    /// Computes left and right margins and width.
    ///
    /// This is used by both replaced and non-replaced Blocks.
    ///
    /// CSS 2.1 Section 10.3.3.
    /// Constraint Equation: margin-left + margin-right + width = available_width
    /// where available_width = CB width - (horizontal border + padding)
    fn solve_block_width_constraints(&self,
                                     _: &mut BlockFlow,
                                     input: WidthConstraintInput)
                                     -> WidthConstraintSolution {
        let (computed_width, left_margin, right_margin, available_width) = (input.computed_width,
                                                                            input.left_margin,
                                                                            input.right_margin,
                                                                            input.available_width);

        // If width is not 'auto', and width + margins > available_width, all
        // 'auto' margins are treated as 0.
        let (left_margin, right_margin) = match computed_width {
            Auto => (left_margin, right_margin),
            Specified(width) => {
                let left = left_margin.specified_or_zero();
                let right = right_margin.specified_or_zero();

                if (left + right + width) > available_width {
                    (Specified(left), Specified(right))
                } else {
                    (left_margin, right_margin)
                }
            }
        };

        // Invariant: left_margin + width + right_margin == available_width
        let (left_margin, width, right_margin) = match (left_margin, computed_width, right_margin) {
            // If all have a computed value other than 'auto', the system is
            // over-constrained and we need to discard a margin.
            // If direction is ltr, ignore the specified right margin and
            // solve for it.
            // If it is rtl, ignore the specified left margin.
            // FIXME(eatkinson): this assumes the direction is ltr
            (Specified(margin_l), Specified(width), Specified(_margin_r)) =>
                (margin_l, width, available_width - (margin_l + width )),

            // If exactly one value is 'auto', solve for it
            (Auto, Specified(width), Specified(margin_r)) =>
                (available_width - (width + margin_r), width, margin_r),
            (Specified(margin_l), Auto, Specified(margin_r)) =>
                (margin_l, available_width - (margin_l + margin_r), margin_r),
            (Specified(margin_l), Specified(width), Auto) =>
                (margin_l, width, available_width - (margin_l + width)),

            // If width is set to 'auto', any other 'auto' value becomes '0',
            // and width is solved for
            (Auto, Auto, Specified(margin_r)) =>
                (Au::new(0), available_width - margin_r, margin_r),
            (Specified(margin_l), Auto, Auto) =>
                (margin_l, available_width - margin_l, Au::new(0)),
            (Auto, Auto, Auto) =>
                (Au::new(0), available_width, Au::new(0)),

            // If left and right margins are auto, they become equal
            (Auto, Specified(width), Auto) => {
                let margin = (available_width - width).scale_by(0.5);
                (margin, width, margin)
            }
        };
        WidthConstraintSolution::new(width, left_margin, right_margin)
    }
}

/// The different types of Blocks.
///
/// They mainly differ in the way width and heights and margins are calculated
/// for them.
struct AbsoluteNonReplaced;
struct AbsoluteReplaced;
struct BlockNonReplaced;
struct BlockReplaced;
struct FloatNonReplaced;
struct FloatReplaced;

impl WidthAndMarginsComputer for AbsoluteNonReplaced {
    /// Solve the horizontal constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.3.7
    /// Constraint equation:
    /// left + right + width + margin-left + margin-right
    /// = absolute containing block width - (horizontal padding and border)
    /// [aka available_width]
    ///
    /// Return the solution for the equation.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        let WidthConstraintInput {
            computed_width,
            left_margin,
            right_margin,
            left,
            right,
            available_width,
            static_x_offset,
        } = input;

        // TODO: Check for direction of parent flow (NOT Containing Block)
        // when right-to-left is implemented.
        // Assume direction is 'ltr' for now

        // Distance from the left edge of the Absolute Containing Block to the
        // left margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_left = static_x_offset;

        let (left, right, width, margin_left, margin_right) = match (left, right, computed_width) {
            (Auto, Auto, Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let left = static_position_left;
                // Now it is the same situation as left Specified and right
                // and width Auto.

                // Set right to zero to calculate width
                let width = block.get_shrink_to_fit_width(
                    available_width - (left + margin_l + margin_r));
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            (Specified(left), Specified(right), Specified(width)) => {
                match (left_margin, right_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = available_width - left - right - width;
                        if total_margin_val < Au(0) {
                            // margin-left becomes 0 because direction is 'ltr'.
                            // TODO: Handle 'rtl' when it is implemented.
                            (left, right, width, Au(0), total_margin_val)
                        } else {
                            // Equal margins
                            (left, right, width,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                    }
                    (Specified(margin_l), Auto) => {
                        let sum = left + right + width + margin_l;
                        (left, right, width, margin_l, available_width - sum)
                    }
                    (Auto, Specified(margin_r)) => {
                        let sum = left + right + width + margin_r;
                        (left, right, width, available_width - sum, margin_r)
                    }
                    (Specified(margin_l), Specified(margin_r)) => {
                        // Values are over-constrained.
                        // Ignore value for 'right' cos direction is 'ltr'.
                        // TODO: Handle 'rtl' when it is implemented.
                        let sum = left + width + margin_l + margin_r;
                        (left, available_width - sum, width, margin_l, margin_r)
                    }
                }
            }
            // For the rest of the cases, auto values for margin are set to 0

            // If only one is Auto, solve for it
            (Auto, Specified(right), Specified(width)) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = right + width + margin_l + margin_r;
                (available_width - sum, right, width, margin_l, margin_r)
            }
            (Specified(left), Auto, Specified(width)) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            (Specified(left), Specified(right), Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = left + right + margin_l + margin_r;
                (left, right, available_width - sum, margin_l, margin_r)
            }

            // If width is auto, then width is shrink-to-fit. Solve for the
            // non-auto value.
            (Specified(left), Auto, Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                // Set right to zero to calculate width
                let width = block.get_shrink_to_fit_width(
                    available_width - (left + margin_l + margin_r));
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            (Auto, Specified(right), Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                // Set left to zero to calculate width
                let width = block.get_shrink_to_fit_width(
                    available_width - (right + margin_l + margin_r));
                let sum = right + width + margin_l + margin_r;
                (available_width - sum, right, width, margin_l, margin_r)
            }

            (Auto, Auto, Specified(width)) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                // Setting 'left' to static position because direction is 'ltr'.
                // TODO: Handle 'rtl' when it is implemented.
                let left = static_position_left;
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
        };
        WidthConstraintSolution::for_absolute_flow(left, right, width, margin_left, margin_right)
    }

    fn containing_block_width(&self, block: &mut BlockFlow, _: Au, ctx: &mut LayoutContext) -> Au {
        block.containing_block_size(ctx.screen_size).width
    }

    fn set_flow_x_coord_if_necessary(&self, block: &mut BlockFlow, solution: WidthConstraintSolution) {
        // Set the x-coordinate of the absolute flow wrt to its containing block.
        block.base.position.origin.x = solution.left;
    }
}

impl WidthAndMarginsComputer for AbsoluteReplaced {
    /// Solve the horizontal constraint equation for absolute replaced elements.
    ///
    /// `static_x_offset`: total offset of current flow's hypothetical
    /// position (static position) from its actual Containing Block.
    ///
    /// CSS Section 10.3.8
    /// Constraint equation:
    /// left + right + width + margin-left + margin-right
    /// = absolute containing block width - (horizontal padding and border)
    /// [aka available_width]
    ///
    /// Return the solution for the equation.
    fn solve_width_constraints(&self,
                               _: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        let WidthConstraintInput {
            computed_width,
            left_margin,
            right_margin,
            left,
            right,
            available_width,
            static_x_offset,
        } = input;
        // TODO: Check for direction of static-position Containing Block (aka
        // parent flow, _not_ the actual Containing Block) when right-to-left
        // is implemented
        // Assume direction is 'ltr' for now
        // TODO: Handle all the cases for 'rtl' direction.

        let width = match computed_width {
            Specified(w) => w,
            _ => fail!("{} {}",
                       "The used value for width for absolute replaced flow",
                       "should have already been calculated by now.")
        };

        // Distance from the left edge of the Absolute Containing Block to the
        // left margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_left = static_x_offset;

        let (left, right, width, margin_left, margin_right) = match (left, right) {
            (Auto, Auto) => {
                let left = static_position_left;
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            // If only one is Auto, solve for it
            (Auto, Specified(right)) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = right + width + margin_l + margin_r;
                (available_width - sum, right, width, margin_l, margin_r)
            }
            (Specified(left), Auto) => {
                let margin_l = left_margin.specified_or_zero();
                let margin_r = right_margin.specified_or_zero();
                let sum = left + width + margin_l + margin_r;
                (left, available_width - sum, width, margin_l, margin_r)
            }
            (Specified(left), Specified(right)) => {
                match (left_margin, right_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = available_width - left - right - width;
                        if total_margin_val < Au(0) {
                            // margin-left becomes 0 because direction is 'ltr'.
                            (left, right, width, Au(0), total_margin_val)
                        } else {
                            // Equal margins
                            (left, right, width,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                    }
                    (Specified(margin_l), Auto) => {
                        let sum = left + right + width + margin_l;
                        (left, right, width, margin_l, available_width - sum)
                    }
                    (Auto, Specified(margin_r)) => {
                        let sum = left + right + width + margin_r;
                        (left, right, width, available_width - sum, margin_r)
                    }
                    (Specified(margin_l), Specified(margin_r)) => {
                        // Values are over-constrained.
                        // Ignore value for 'right' cos direction is 'ltr'.
                        let sum = left + width + margin_l + margin_r;
                        (left, available_width - sum, width, margin_l, margin_r)
                    }
                }
            }
        };
        WidthConstraintSolution::for_absolute_flow(left, right, width, margin_left, margin_right)
    }

    /// Calculate used value of width just like we do for inline replaced elements.
    fn initial_computed_width(&self,
                              block: &mut BlockFlow,
                              _: Au,
                              ctx: &mut LayoutContext)
                              -> MaybeAuto {
        let containing_block_width = block.containing_block_size(ctx.screen_size).width;
        let box_ = block.box_();
        box_.assign_replaced_width_if_necessary(containing_block_width);
        // For replaced absolute flow, the rest of the constraint solving will
        // take width to be specified as the value computed here.
        Specified(box_.content_width())
    }

    fn containing_block_width(&self, block: &mut BlockFlow, _: Au, ctx: &mut LayoutContext) -> Au {
        block.containing_block_size(ctx.screen_size).width
    }

    fn set_flow_x_coord_if_necessary(&self, block: &mut BlockFlow, solution: WidthConstraintSolution) {
        // Set the x-coordinate of the absolute flow wrt to its containing block.
        block.base.position.origin.x = solution.left;
    }
}

impl WidthAndMarginsComputer for BlockNonReplaced {
    /// Compute left and right margins and width.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        self.solve_block_width_constraints(block, input)
    }
}

impl WidthAndMarginsComputer for BlockReplaced {
    /// Compute left and right margins and width.
    ///
    /// Width has already been calculated. We now calculate the margins just
    /// like for non-replaced blocks.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        match input.computed_width {
            Specified(_) => {},
            Auto => fail!("BlockReplaced: width should have been computed by now")
        };
        self.solve_block_width_constraints(block, input)
    }

    /// Calculate used value of width just like we do for inline replaced elements.
    fn initial_computed_width(&self,
                              block: &mut BlockFlow,
                              parent_flow_width: Au,
                              _: &mut LayoutContext)
                              -> MaybeAuto {
        let box_ = block.box_();
        box_.assign_replaced_width_if_necessary(parent_flow_width);
        // For replaced block flow, the rest of the constraint solving will
        // take width to be specified as the value computed here.
        Specified(box_.content_width())
    }

}

impl WidthAndMarginsComputer for FloatNonReplaced {
    /// CSS Section 10.3.5
    ///
    /// If width is computed as 'auto', the used value is the 'shrink-to-fit' width.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        let (computed_width, left_margin, right_margin, available_width) = (input.computed_width,
                                                                            input.left_margin,
                                                                            input.right_margin,
                                                                            input.available_width);
        let margin_left = left_margin.specified_or_zero();
        let margin_right = right_margin.specified_or_zero();
        let available_width_float = available_width - margin_left - margin_right;
        let shrink_to_fit = block.get_shrink_to_fit_width(available_width_float);
        let width = computed_width.specified_or_default(shrink_to_fit);
        debug!("assign_widths_float -- width: {}", width);
        WidthConstraintSolution::new(width, margin_left, margin_right)
    }
}

impl WidthAndMarginsComputer for FloatReplaced {
    /// CSS Section 10.3.5
    ///
    /// If width is computed as 'auto', the used value is the 'shrink-to-fit' width.
    fn solve_width_constraints(&self,
                               _: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        let (computed_width, left_margin, right_margin) = (input.computed_width,
                                                           input.left_margin,
                                                           input.right_margin);
        let margin_left = left_margin.specified_or_zero();
        let margin_right = right_margin.specified_or_zero();
        let width = match computed_width {
            Specified(w) => w,
            Auto => fail!("FloatReplaced: width should have been computed by now")
        };
        debug!("assign_widths_float -- width: {}", width);
        WidthConstraintSolution::new(width, margin_left, margin_right)
    }

    /// Calculate used value of width just like we do for inline replaced elements.
    fn initial_computed_width(&self,
                              block: &mut BlockFlow,
                              parent_flow_width: Au,
                              _: &mut LayoutContext)
                              -> MaybeAuto {
        let box_ = block.box_();
        box_.assign_replaced_width_if_necessary(parent_flow_width);
        // For replaced block flow, the rest of the constraint solving will
        // take width to be specified as the value computed here.
        Specified(box_.content_width())
    }
}
