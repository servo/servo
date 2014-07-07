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

#![deny(unsafe_block)]

use construct::FlowConstructor;
use context::LayoutContext;
use floats::{ClearBoth, ClearLeft, ClearRight, FloatKind, Floats, PlacementInfo};
use flow::{BaseFlow, BlockFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use flow::{MutableFlowUtils, PreorderFlowTraversal, PostorderFlowTraversal, mut_base};
use flow;
use fragment::{Fragment, ImageFragment, ScannedTextFragment};
use model::{Auto, IntrinsicISizes, MarginCollapseInfo, MarginsCollapse};
use model::{MarginsCollapseThrough, MaybeAuto, NoCollapsibleMargins, Specified, specified};
use model::{specified_or_none};
use wrapper::ThreadSafeLayoutNode;
use style::ComputedValues;
use style::computed_values::{clear, position};

use collections::Deque;
use collections::dlist::DList;
use geom::{Size2D, Point2D, Rect};
use gfx::color;
use gfx::display_list::{BackgroundAndBorderLevel, BlockLevel, ContentStackingLevel, DisplayList};
use gfx::display_list::{FloatStackingLevel, PositionedDescendantStackingLevel};
use gfx::display_list::{RootOfStackingContextLevel};
use gfx::render_task::RenderLayer;
use servo_msg::compositor_msg::{FixedPosition, LayerId, Scrollable};
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::logical_geometry::WritingMode;
use servo_util::logical_geometry::{LogicalPoint, LogicalRect, LogicalSize};
use std::fmt;
use std::mem;
use style::computed_values::{LPA_Auto, LPA_Length, LPA_Percentage, LPN_Length, LPN_None};
use style::computed_values::{LPN_Percentage, LP_Length, LP_Percentage};
use style::computed_values::{display, float, overflow};
use sync::Arc;

/// Information specific to floated blocks.
pub struct FloatedBlockInfo {
    pub containing_isize: Au,

    /// Offset relative to where the parent tried to position this flow
    pub rel_pos: LogicalPoint<Au>,

    /// Index into the fragment list for inline floats
    pub index: Option<uint>,

    /// Left or right?
    pub float_kind: FloatKind,
}

impl FloatedBlockInfo {
    pub fn new(float_kind: FloatKind, writing_mode: WritingMode) -> FloatedBlockInfo {
        FloatedBlockInfo {
            containing_isize: Au(0),
            rel_pos: LogicalPoint::new(writing_mode, Au(0), Au(0)),
            index: None,
            float_kind: float_kind,
        }
    }
}

/// The solutions for the bsizes-and-margins constraint equation.
struct BSizeConstraintSolution {
    bstart: Au,
    _bend: Au,
    bsize: Au,
    margin_bstart: Au,
    margin_bend: Au
}

impl BSizeConstraintSolution {
    fn new(bstart: Au, bend: Au, bsize: Au, margin_bstart: Au, margin_bend: Au)
           -> BSizeConstraintSolution {
        BSizeConstraintSolution {
            bstart: bstart,
            _bend: bend,
            bsize: bsize,
            margin_bstart: margin_bstart,
            margin_bend: margin_bend,
        }
    }

    /// Solve the vertical constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.6.4
    /// Constraint equation:
    /// bstart + bend + bsize + margin-bstart + margin-bend
    /// = absolute containing block bsize - (vertical padding and border)
    /// [aka available_bsize]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_nonreplaced(bsize: MaybeAuto,
                                                  bstart_margin: MaybeAuto,
                                                  bend_margin: MaybeAuto,
                                                  bstart: MaybeAuto,
                                                  bend: MaybeAuto,
                                                  content_bsize: Au,
                                                  available_bsize: Au,
                                                  static_b_offset: Au)
                                                  -> BSizeConstraintSolution {
        // Distance from the bstart edge of the Absolute Containing Block to the
        // bstart margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_bstart = static_b_offset;

        let (bstart, bend, bsize, margin_bstart, margin_bend) = match (bstart, bend, bsize) {
            (Auto, Auto, Auto) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let bstart = static_position_bstart;
                // Now it is the same situation as bstart Specified and bend
                // and bsize Auto.

                let bsize = content_bsize;
                let sum = bstart + bsize + margin_bstart + margin_bend;
                (bstart, available_bsize - sum, bsize, margin_bstart, margin_bend)
            }
            (Specified(bstart), Specified(bend), Specified(bsize)) => {
                match (bstart_margin, bend_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = available_bsize - bstart - bend - bsize;
                        (bstart, bend, bsize,
                         total_margin_val.scale_by(0.5),
                         total_margin_val.scale_by(0.5))
                    }
                    (Specified(margin_bstart), Auto) => {
                        let sum = bstart + bend + bsize + margin_bstart;
                        (bstart, bend, bsize, margin_bstart, available_bsize - sum)
                    }
                    (Auto, Specified(margin_bend)) => {
                        let sum = bstart + bend + bsize + margin_bend;
                        (bstart, bend, bsize, available_bsize - sum, margin_bend)
                    }
                    (Specified(margin_bstart), Specified(margin_bend)) => {
                        // Values are over-constrained. Ignore value for 'bend'.
                        let sum = bstart + bsize + margin_bstart + margin_bend;
                        (bstart, available_bsize - sum, bsize, margin_bstart, margin_bend)
                    }
                }
            }

            // For the rest of the cases, auto values for margin are set to 0

            // If only one is Auto, solve for it
            (Auto, Specified(bend), Specified(bsize)) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let sum = bend + bsize + margin_bstart + margin_bend;
                (available_bsize - sum, bend, bsize, margin_bstart, margin_bend)
            }
            (Specified(bstart), Auto, Specified(bsize)) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let sum = bstart + bsize + margin_bstart + margin_bend;
                (bstart, available_bsize - sum, bsize, margin_bstart, margin_bend)
            }
            (Specified(bstart), Specified(bend), Auto) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let sum = bstart + bend + margin_bstart + margin_bend;
                (bstart, bend, available_bsize - sum, margin_bstart, margin_bend)
            }

            // If bsize is auto, then bsize is content bsize. Solve for the
            // non-auto value.
            (Specified(bstart), Auto, Auto) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let bsize = content_bsize;
                let sum = bstart + bsize + margin_bstart + margin_bend;
                (bstart, available_bsize - sum, bsize, margin_bstart, margin_bend)
            }
            (Auto, Specified(bend), Auto) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let bsize = content_bsize;
                let sum = bend + bsize + margin_bstart + margin_bend;
                (available_bsize - sum, bend, bsize, margin_bstart, margin_bend)
            }

            (Auto, Auto, Specified(bsize)) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let bstart = static_position_bstart;
                let sum = bstart + bsize + margin_bstart + margin_bend;
                (bstart, available_bsize - sum, bsize, margin_bstart, margin_bend)
            }
        };
        BSizeConstraintSolution::new(bstart, bend, bsize, margin_bstart, margin_bend)
    }

    /// Solve the vertical constraint equation for absolute replaced elements.
    ///
    /// Assumption: The used value for bsize has already been calculated.
    ///
    /// CSS Section 10.6.5
    /// Constraint equation:
    /// bstart + bend + bsize + margin-bstart + margin-bend
    /// = absolute containing block bsize - (vertical padding and border)
    /// [aka available_bsize]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_replaced(bsize: Au,
                                               bstart_margin: MaybeAuto,
                                               bend_margin: MaybeAuto,
                                               bstart: MaybeAuto,
                                               bend: MaybeAuto,
                                               _: Au,
                                               available_bsize: Au,
                                               static_b_offset: Au)
                                               -> BSizeConstraintSolution {
        // Distance from the bstart edge of the Absolute Containing Block to the
        // bstart margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_bstart = static_b_offset;

        let (bstart, bend, bsize, margin_bstart, margin_bend) = match (bstart, bend) {
            (Auto, Auto) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let bstart = static_position_bstart;
                let sum = bstart + bsize + margin_bstart + margin_bend;
                (bstart, available_bsize - sum, bsize, margin_bstart, margin_bend)
            }
            (Specified(bstart), Specified(bend)) => {
                match (bstart_margin, bend_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = available_bsize - bstart - bend - bsize;
                        (bstart, bend, bsize,
                         total_margin_val.scale_by(0.5),
                         total_margin_val.scale_by(0.5))
                    }
                    (Specified(margin_bstart), Auto) => {
                        let sum = bstart + bend + bsize + margin_bstart;
                        (bstart, bend, bsize, margin_bstart, available_bsize - sum)
                    }
                    (Auto, Specified(margin_bend)) => {
                        let sum = bstart + bend + bsize + margin_bend;
                        (bstart, bend, bsize, available_bsize - sum, margin_bend)
                    }
                    (Specified(margin_bstart), Specified(margin_bend)) => {
                        // Values are over-constrained. Ignore value for 'bend'.
                        let sum = bstart + bsize + margin_bstart + margin_bend;
                        (bstart, available_bsize - sum, bsize, margin_bstart, margin_bend)
                    }
                }
            }

            // If only one is Auto, solve for it
            (Auto, Specified(bend)) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let sum = bend + bsize + margin_bstart + margin_bend;
                (available_bsize - sum, bend, bsize, margin_bstart, margin_bend)
            }
            (Specified(bstart), Auto) => {
                let margin_bstart = bstart_margin.specified_or_zero();
                let margin_bend = bend_margin.specified_or_zero();
                let sum = bstart + bsize + margin_bstart + margin_bend;
                (bstart, available_bsize - sum, bsize, margin_bstart, margin_bend)
            }
        };
        BSizeConstraintSolution::new(bstart, bend, bsize, margin_bstart, margin_bend)
    }
}

/// Performs bsize calculations potentially multiple times, taking `bsize`, `min-bsize`, and
/// `max-bsize` into account. After each call to `next()`, the caller must call `.try()` with the
/// current calculated value of `bsize`.
///
/// See CSS 2.1 § 10.7.
struct CandidateBSizeIterator {
    bsize: MaybeAuto,
    max_bsize: Option<Au>,
    min_bsize: Au,
    candidate_value: Au,
    status: CandidateBSizeIteratorStatus,
}

impl CandidateBSizeIterator {
    /// Creates a new candidate bsize iterator. `block_container_bsize` is `None` if the bsize
    /// of the block container has not been determined yet. It will always be `Some` in the case of
    /// absolutely-positioned containing blocks.
    pub fn new(style: &ComputedValues, block_container_bsize: Option<Au>)
               -> CandidateBSizeIterator {
        // Per CSS 2.1 § 10.7, percentages in `min-bsize` and `max-bsize` refer to the bsize of
        // the containing block. If that is not determined yet by the time we need to resolve
        // `min-bsize` and `max-bsize`, percentage values are ignored.

        let bsize = match (style.content_bsize(), block_container_bsize) {
            (LPA_Percentage(percent), Some(block_container_bsize)) => {
                Specified(block_container_bsize.scale_by(percent))
            }
            (LPA_Percentage(_), None) | (LPA_Auto, _) => Auto,
            (LPA_Length(length), _) => Specified(length),
        };
        let max_bsize = match (style.max_bsize(), block_container_bsize) {
            (LPN_Percentage(percent), Some(block_container_bsize)) => {
                Some(block_container_bsize.scale_by(percent))
            }
            (LPN_Percentage(_), None) | (LPN_None, _) => None,
            (LPN_Length(length), _) => Some(length),
        };
        let min_bsize = match (style.min_bsize(), block_container_bsize) {
            (LP_Percentage(percent), Some(block_container_bsize)) => {
                block_container_bsize.scale_by(percent)
            }
            (LP_Percentage(_), None) => Au(0),
            (LP_Length(length), _) => length,
        };

        CandidateBSizeIterator {
            bsize: bsize,
            max_bsize: max_bsize,
            min_bsize: min_bsize,
            candidate_value: Au(0),
            status: InitialCandidateBSizeStatus,
        }
    }

    pub fn next<'a>(&'a mut self) -> Option<(MaybeAuto, &'a mut Au)> {
        self.status = match self.status {
            InitialCandidateBSizeStatus => TryingBSizeCandidateBSizeStatus,
            TryingBSizeCandidateBSizeStatus => {
                match self.max_bsize {
                    Some(max_bsize) if self.candidate_value > max_bsize => {
                        TryingMaxCandidateBSizeStatus
                    }
                    _ if self.candidate_value < self.min_bsize => TryingMinCandidateBSizeStatus,
                    _ => FoundCandidateBSizeStatus,
                }
            }
            TryingMaxCandidateBSizeStatus => {
                if self.candidate_value < self.min_bsize {
                    TryingMinCandidateBSizeStatus
                } else {
                    FoundCandidateBSizeStatus
                }
            }
            TryingMinCandidateBSizeStatus | FoundCandidateBSizeStatus => {
                FoundCandidateBSizeStatus
            }
        };

        match self.status {
            TryingBSizeCandidateBSizeStatus => Some((self.bsize, &mut self.candidate_value)),
            TryingMaxCandidateBSizeStatus => {
                Some((Specified(self.max_bsize.unwrap()), &mut self.candidate_value))
            }
            TryingMinCandidateBSizeStatus => {
                Some((Specified(self.min_bsize), &mut self.candidate_value))
            }
            FoundCandidateBSizeStatus => None,
            InitialCandidateBSizeStatus => fail!(),
        }
    }
}

enum CandidateBSizeIteratorStatus {
    InitialCandidateBSizeStatus,
    TryingBSizeCandidateBSizeStatus,
    TryingMaxCandidateBSizeStatus,
    TryingMinCandidateBSizeStatus,
    FoundCandidateBSizeStatus,
}

// A helper function used in bsize calculation.
fn translate_including_floats(cur_b: &mut Au, delta: Au, floats: &mut Floats) {
    *cur_b = *cur_b + delta;
    let writing_mode = floats.writing_mode;
    floats.translate(LogicalSize::new(writing_mode, Au(0), -delta));
}

/// The real assign-bsizes traversal for flows with position 'absolute'.
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
struct AbsoluteAssignBSizesTraversal<'a>(&'a mut LayoutContext);

impl<'a> PreorderFlowTraversal for AbsoluteAssignBSizesTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        let block_flow = flow.as_block();

        // The root of the absolute flow tree is definitely not absolutely
        // positioned. Nothing to process here.
        if block_flow.is_root_of_absolute_flow_tree() {
            return true;
        }


        let AbsoluteAssignBSizesTraversal(ref ctx) = *self;
        block_flow.calculate_abs_bsize_and_margins(*ctx);
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

#[deriving(Clone, PartialEq)]
pub enum MarginsMayCollapseFlag {
    MarginsMayCollapse,
    MarginsMayNotCollapse,
}

#[deriving(PartialEq)]
enum FormattingContextType {
    NonformattingContext,
    BlockFormattingContext,
    OtherFormattingContext,
}

// Propagates the `layers_needed_for_descendants` flag appropriately from a child. This is called
// as part of bsize assignment.
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
    pub base: BaseFlow,

    /// The associated fragment.
    pub fragment: Fragment,

    /// TODO: is_root should be a bit field to conserve memory.
    /// Whether this block flow is the root flow.
    pub is_root: bool,

    /// Static y offset of an absolute flow from its CB.
    pub static_b_offset: Au,

    /// The isize of the last float prior to this block. This is used to speculatively lay out
    /// block formatting contexts.
    previous_float_isize: Option<Au>,

    /// Additional floating flow members.
    pub float: Option<Box<FloatedBlockInfo>>
}

impl BlockFlow {
    pub fn from_node(constructor: &mut FlowConstructor, node: &ThreadSafeLayoutNode) -> BlockFlow {
        BlockFlow {
            base: BaseFlow::new((*node).clone()),
            fragment: Fragment::new(constructor, node),
            is_root: false,
            static_b_offset: Au::new(0),
            previous_float_isize: None,
            float: None
        }
    }

    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode, fragment: Fragment) -> BlockFlow {
        BlockFlow {
            base: BaseFlow::new((*node).clone()),
            fragment: fragment,
            is_root: false,
            static_b_offset: Au::new(0),
            previous_float_isize: None,
            float: None
        }
    }

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> BlockFlow {
        let base = BaseFlow::new((*node).clone());
        BlockFlow {
            fragment: Fragment::new(constructor, node),
            is_root: false,
            static_b_offset: Au::new(0),
            previous_float_isize: None,
            float: Some(box FloatedBlockInfo::new(float_kind, base.writing_mode)),
            base: base,
        }
    }

    /// Return the type of this block.
    ///
    /// This determines the algorithm used to calculate isize, bsize, and the
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

    /// Compute the used value of isize for this Block.
    fn compute_used_isize(&mut self, ctx: &mut LayoutContext, containing_block_isize: Au) {
        let block_type = self.block_type();
        match block_type {
            AbsoluteReplacedType => {
                let isize_computer = AbsoluteReplaced;
                isize_computer.compute_used_isize(self, ctx, containing_block_isize);
            }
            AbsoluteNonReplacedType => {
                let isize_computer = AbsoluteNonReplaced;
                isize_computer.compute_used_isize(self, ctx, containing_block_isize);
            }
            FloatReplacedType => {
                let isize_computer = FloatReplaced;
                isize_computer.compute_used_isize(self, ctx, containing_block_isize);
            }
            FloatNonReplacedType => {
                let isize_computer = FloatNonReplaced;
                isize_computer.compute_used_isize(self, ctx, containing_block_isize);
            }
            BlockReplacedType => {
                let isize_computer = BlockReplaced;
                isize_computer.compute_used_isize(self, ctx, containing_block_isize);
            }
            BlockNonReplacedType => {
                let isize_computer = BlockNonReplaced;
                isize_computer.compute_used_isize(self, ctx, containing_block_isize);
            }
        }
    }

    /// Return this flow's fragment.
    pub fn fragment<'a>(&'a mut self) -> &'a mut Fragment {
        &mut self.fragment
    }

    /// Return the static x offset from the appropriate Containing Block for this flow.
    pub fn static_i_offset(&self) -> Au {
        if self.is_fixed() {
            self.base.fixed_static_i_offset
        } else {
            self.base.absolute_static_i_offset
        }
    }

    /// Return the size of the Containing Block for this flow.
    ///
    /// Right now, this only gets the Containing Block size for absolutely
    /// positioned elements.
    /// Note: Assume this is called in a top-down traversal, so it is ok to
    /// reference the CB.
    #[inline]
    pub fn containing_block_size(&mut self, viewport_size: Size2D<Au>) -> LogicalSize<Au> {
        assert!(self.is_absolutely_positioned());
        if self.is_fixed() {
            // Initial containing block is the CB for the root
            LogicalSize::from_physical(self.base.writing_mode, viewport_size)
        } else {
            self.base.absolute_cb.generated_containing_block_rect().size
        }
    }

    /// Traverse the Absolute flow tree in preorder.
    ///
    /// Traverse all your direct absolute descendants, who will then traverse
    /// their direct absolute descendants.
    /// Also, set the static y offsets for each descendant (using the value
    /// which was bubbled up during normal assign-bsize).
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

        let cb_bstart_edge_offset = flow.generated_containing_block_rect().start.b;
        let mut descendant_offset_iter = mut_base(flow).abs_descendants.iter_with_offset();
        // Pass in the respective static y offset for each descendant.
        for (ref mut descendant_link, ref y_offset) in descendant_offset_iter {
            let block = descendant_link.as_block();
            // The stored y_offset is wrt to the flow box.
            // Translate it to the CB (which is the padding box).
            block.static_b_offset = **y_offset - cb_bstart_edge_offset;
            if !block.traverse_preorder_absolute_flows(traversal) {
                return false
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
            let block = descendant_link.as_block();
            if !block.traverse_postorder_absolute_flows(traversal) {
                return false
            }
        }

        traversal.process(flow)
    }

    /// Return true if this has a replaced fragment.
    ///
    /// The only two types of replaced fragments currently are text fragments
    /// and image fragments.
    fn is_replaced_content(&self) -> bool {
        match self.fragment.specific {
            ScannedTextFragment(_) | ImageFragment(_) => true,
            _ => false,
        }
    }

    /// Return shrink-to-fit isize.
    ///
    /// This is where we use the preferred isizes and minimum isizes
    /// calculated in the bubble-isizes traversal.
    fn get_shrink_to_fit_isize(&self, available_isize: Au) -> Au {
        geometry::min(self.base.intrinsic_isizes.preferred_isize,
                      geometry::max(self.base.intrinsic_isizes.minimum_isize, available_isize))
    }

    /// Collect and update static y-offsets bubbled up by kids.
    ///
    /// This would essentially give us offsets of all absolutely positioned
    /// direct descendants and all fixed descendants, in tree order.
    ///
    /// Assume that this is called in a bottom-up traversal (specifically, the
    /// assign-bsize traversal). So, kids have their flow origin already set.
    /// In the case of absolute flow kids, they have their hypothetical box
    /// position already set.
    fn collect_static_b_offsets_from_kids(&mut self) {
        let mut abs_descendant_y_offsets = Vec::new();
        for kid in self.base.child_iter() {
            let mut gives_abs_offsets = true;
            if kid.is_block_like() {
                let kid_block = kid.as_block();
                if kid_block.is_fixed() || kid_block.is_absolutely_positioned() {
                    // It won't contribute any offsets for descendants because it
                    // would be the CB for them.
                    gives_abs_offsets = false;
                    // Give the offset for the current absolute flow alone.
                    abs_descendant_y_offsets.push(kid_block.get_hypothetical_bstart_edge());
                } else if kid_block.is_positioned() {
                    // It won't contribute any offsets because it would be the CB
                    // for the descendants.
                    gives_abs_offsets = false;
                }
            }

            if gives_abs_offsets {
                let kid_base = flow::mut_base(kid);
                // Avoid copying the offset vector.
                let offsets = mem::replace(&mut kid_base.abs_descendants.static_b_offsets, Vec::new());
                // Consume all the static y-offsets bubbled up by kid.
                for y_offset in offsets.move_iter() {
                    // The offsets are wrt the kid flow box. Translate them to current flow.
                    abs_descendant_y_offsets.push(y_offset + kid_base.position.start.b);
                }
            }
        }
        self.base.abs_descendants.static_b_offsets = abs_descendant_y_offsets;
    }

    /// If this is the root flow, shifts all kids down and adjusts our size to account for
    /// collapsed margins.
    ///
    /// TODO(#2017, pcwalton): This is somewhat inefficient (traverses kids twice); can we do
    /// better?
    fn adjust_fragments_for_collapsed_margins_if_root(&mut self) {
        if !self.is_root() {
            return
        }

        let (bstart_margin_value, bend_margin_value) = match self.base.collapsible_margins {
            MarginsCollapseThrough(margin) => (Au(0), margin.collapse()),
            MarginsCollapse(bstart_margin, bend_margin) => {
                (bstart_margin.collapse(), bend_margin.collapse())
            }
            NoCollapsibleMargins(bstart, bend) => (bstart, bend),
        };

        // Shift all kids down (or up, if margins are negative) if necessary.
        if bstart_margin_value != Au(0) {
            for kid in self.base.child_iter() {
                let kid_base = flow::mut_base(kid);
                kid_base.position.start.b = kid_base.position.start.b + bstart_margin_value
            }
        }

        self.base.position.size.bsize = self.base.position.size.bsize + bstart_margin_value +
            bend_margin_value;
        self.fragment.border_box.size.bsize = self.fragment.border_box.size.bsize + bstart_margin_value +
            bend_margin_value;
    }

    /// Assign bsize for current flow.
    ///
    /// * Collapse margins for flow's children and set in-flow child flows' y-coordinates now that
    ///   we know their bsizes.
    /// * Calculate and set the bsize of the current flow.
    /// * Calculate bsize, vertical margins, and y-coordinate for the flow's box. Ideally, this
    ///   should be calculated using CSS § 10.6.7.
    ///
    /// For absolute flows, we store the calculated content bsize for the flow. We defer the
    /// calculation of the other values until a later traversal.
    ///
    /// `inline(always)` because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    pub fn assign_bsize_block_base(&mut self,
                                    layout_context: &mut LayoutContext,
                                    margins_may_collapse: MarginsMayCollapseFlag) {
        // Our current border-box position.
        let mut cur_b = Au(0);

        // Absolute positioning establishes a block formatting context. Don't propagate floats
        // in or out. (But do propagate them between kids.)
        if self.is_absolutely_positioned() {
            self.base.floats = Floats::new(self.fragment.style.writing_mode);
        }
        if margins_may_collapse != MarginsMayCollapse {
            self.base.floats = Floats::new(self.fragment.style.writing_mode);
        }

        let mut margin_collapse_info = MarginCollapseInfo::new();
        self.base.floats.translate(LogicalSize::new(
            self.fragment.style.writing_mode, -self.fragment.istart_offset(), Au(0)));

        // The sum of our bstart border and bstart padding.
        let bstart_offset = self.fragment.border_padding.bstart;
        translate_including_floats(&mut cur_b, bstart_offset, &mut self.base.floats);

        let can_collapse_bstart_margin_with_kids =
            margins_may_collapse == MarginsMayCollapse &&
            !self.is_absolutely_positioned() &&
            self.fragment.border_padding.bstart == Au(0);
        margin_collapse_info.initialize_bstart_margin(&self.fragment,
                                                   can_collapse_bstart_margin_with_kids);

        // At this point, `cur_b` is at the content edge of our box. Now iterate over children.
        let mut floats = self.base.floats.clone();
        let mut layers_needed_for_descendants = false;
        for kid in self.base.child_iter() {
            if kid.is_absolutely_positioned() {
                // Assume that the *hypothetical box* for an absolute flow starts immediately after
                // the bend border edge of the previous flow.
                flow::mut_base(kid).position.start.b = cur_b;
                kid.assign_bsize_for_inorder_child_if_necessary(layout_context);
                propagate_layer_flag_from_child(&mut layers_needed_for_descendants, kid);

                // Skip the collapsing and float processing for absolute flow kids and continue
                // with the next flow.
                continue
            }

            // Assign bsize now for the child if it was impacted by floats and we couldn't before.
            flow::mut_base(kid).floats = floats.clone();
            if kid.is_float() {
                // FIXME(pcwalton): Using `position.start.b` to mean the float ceiling is a
                // bit of a hack.
                flow::mut_base(kid).position.start.b =
                    margin_collapse_info.current_float_ceiling();
                propagate_layer_flag_from_child(&mut layers_needed_for_descendants, kid);

                let kid_was_impacted_by_floats =
                    kid.assign_bsize_for_inorder_child_if_necessary(layout_context);
                assert!(kid_was_impacted_by_floats);    // As it was a float itself...

                let kid_base = flow::mut_base(kid);
                kid_base.position.start.b = cur_b;
                floats = kid_base.floats.clone();
                continue
            }


            // If we have clearance, assume there are no floats in.
            //
            // FIXME(#2008, pcwalton): This could be wrong if we have `clear: left` or `clear:
            // right` and there are still floats to impact, of course. But this gets complicated
            // with margin collapse. Possibly the right thing to do is to lay out the block again
            // in this rare case. (Note that WebKit can lay blocks out twice; this may be related,
            // although I haven't looked into it closely.)
            if kid.float_clearance() != clear::none {
                flow::mut_base(kid).floats = Floats::new(self.fragment.style.writing_mode)
            }

            // Lay the child out if this was an in-order traversal.
            let kid_was_impacted_by_floats =
                kid.assign_bsize_for_inorder_child_if_necessary(layout_context);

            // Mark flows for layerization if necessary to handle painting order correctly.
            propagate_layer_flag_from_child(&mut layers_needed_for_descendants, kid);

            // Handle any (possibly collapsed) top margin.
            let delta = margin_collapse_info.advance_bstart_margin(
                &flow::base(kid).collapsible_margins);
            translate_including_floats(&mut cur_b, delta, &mut floats);

            // Clear past the floats that came in, if necessary.
            let clearance = match kid.float_clearance() {
                clear::none => Au(0),
                clear::left => floats.clearance(ClearLeft),
                clear::right => floats.clearance(ClearRight),
                clear::both => floats.clearance(ClearBoth),
            };
            cur_b = cur_b + clearance;

            // At this point, `cur_b` is at the border edge of the child.
            flow::mut_base(kid).position.start.b = cur_b;

            // Now pull out the child's outgoing floats. We didn't do this immediately after the
            // `assign_bsize_for_inorder_child_if_necessary` call because clearance on a block
            // operates on the floats that come *in*, not the floats that go *out*.
            if kid_was_impacted_by_floats {
                floats = flow::mut_base(kid).floats.clone()
            }

            // Move past the child's border box. Do not use the `translate_including_floats`
            // function here because the child has already translated floats past its border box.
            let kid_base = flow::mut_base(kid);
            cur_b = cur_b + kid_base.position.size.bsize;

            // Handle any (possibly collapsed) bend margin.
            let delta = margin_collapse_info.advance_bend_margin(&kid_base.collapsible_margins);
            translate_including_floats(&mut cur_b, delta, &mut floats);
        }

        // Mark ourselves for layerization if that will be necessary to paint in the proper order
        // (CSS 2.1, Appendix E).
        self.base.flags.set_layers_needed_for_descendants(layers_needed_for_descendants);

        // Collect various offsets needed by absolutely positioned descendants.
        self.collect_static_b_offsets_from_kids();

        // Add in our bend margin and compute our collapsible margins.
        let can_collapse_bend_margin_with_kids =
            margins_may_collapse == MarginsMayCollapse &&
            !self.is_absolutely_positioned() &&
            self.fragment.border_padding.bend == Au(0);
        let (collapsible_margins, delta) =
            margin_collapse_info.finish_and_compute_collapsible_margins(
            &self.fragment,
            can_collapse_bend_margin_with_kids);
        self.base.collapsible_margins = collapsible_margins;
        translate_including_floats(&mut cur_b, delta, &mut floats);

        // FIXME(#2003, pcwalton): The max is taken here so that you can scroll the page, but this
        // is not correct behavior according to CSS 2.1 § 10.5. Instead I think we should treat the
        // root element as having `overflow: scroll` and use the layers-based scrolling
        // infrastructure to make it scrollable.
        let mut bsize = cur_b - bstart_offset;
        if self.is_root() {
            let screen_size = LogicalSize::from_physical(
                self.fragment.style.writing_mode, layout_context.screen_size);
            bsize = Au::max(screen_size.bsize, bsize)
        }

        if self.is_absolutely_positioned() {
            // The content bsize includes all the floats per CSS 2.1 § 10.6.7. The easiest way to
            // handle this is to just treat this as clearance.
            bsize = bsize + floats.clearance(ClearBoth);

            // Fixed position layers get layers.
            if self.is_fixed() {
                self.base.flags.set_needs_layer(true)
            }

            // Store the content bsize for use in calculating the absolute flow's dimensions
            // later.
            self.fragment.border_box.size.bsize = bsize;
            return
        }

        let mut candidate_bsize_iterator = CandidateBSizeIterator::new(self.fragment.style(),
                                                                         None);
        for (candidate_bsize, new_candidate_bsize) in candidate_bsize_iterator {
            *new_candidate_bsize = match candidate_bsize {
                Auto => bsize,
                Specified(value) => value
            }
        }

        // Adjust `cur_b` as necessary to account for the explicitly-specified bsize.
        bsize = candidate_bsize_iterator.candidate_value;
        let delta = bsize - (cur_b - bstart_offset);
        translate_including_floats(&mut cur_b, delta, &mut floats);

        // Compute content bsize and noncontent bsize.
        let bend_offset = self.fragment.border_padding.bend;
        translate_including_floats(&mut cur_b, bend_offset, &mut floats);

        // Now that `cur_b` is at the bend of the border box, compute the final border box
        // position.
        self.fragment.border_box.size.bsize = cur_b;
        self.fragment.border_box.start.b = Au(0);
        self.base.position.size.bsize = cur_b;

        self.base.floats = floats.clone();
        self.adjust_fragments_for_collapsed_margins_if_root();

        if self.is_root_of_absolute_flow_tree() {
            // Assign bsizes for all flows in this Absolute flow tree.
            // This is preorder because the bsize of an absolute flow may depend on
            // the bsize of its CB, which may also be an absolute flow.
            self.traverse_preorder_absolute_flows(&mut AbsoluteAssignBSizesTraversal(
                    layout_context));
            // Store overflow for all absolute descendants.
            self.traverse_postorder_absolute_flows(&mut AbsoluteStoreOverflowTraversal {
                layout_context: layout_context,
            });
        }
    }

    /// Add placement information about current float flow for use by the parent.
    ///
    /// Also, use information given by parent about other floats to find out our relative position.
    ///
    /// This does not give any information about any float descendants because they do not affect
    /// elements outside of the subtree rooted at this float.
    ///
    /// This function is called on a kid flow by a parent. Therefore, `assign_bsize_float` was
    /// already called on this kid flow by the traversal function. So, the values used are
    /// well-defined.
    pub fn place_float(&mut self) {
        let bsize = self.fragment.border_box.size.bsize;
        let clearance = match self.fragment.clear() {
            None => Au(0),
            Some(clear) => self.base.floats.clearance(clear),
        };

        let margin_bsize = self.fragment.margin.bstart_end();
        let info = PlacementInfo {
            size: LogicalSize::new(
                self.fragment.style.writing_mode,
                self.base.position.size.isize + self.fragment.margin.istart_end() +
                    self.fragment.border_padding.istart_end(),
                bsize + margin_bsize),
            ceiling: clearance + self.base.position.start.b,
            max_isize: self.float.get_ref().containing_isize,
            kind: self.float.get_ref().float_kind,
        };

        // Place the float and return the `Floats` back to the parent flow.
        // After, grab the position and use that to set our position.
        self.base.floats.add_float(&info);

        self.float.get_mut_ref().rel_pos = self.base.floats.last_float_pos().unwrap();
    }

    /// Assign bsize for current flow.
    ///
    /// + Set in-flow child flows' y-coordinates now that we know their
    /// bsizes. This _doesn't_ do any margin collapsing for its children.
    /// + Calculate bsize and y-coordinate for the flow's box. Ideally, this
    /// should be calculated using CSS Section 10.6.7
    ///
    /// It does not calculate the bsize of the flow itself.
    pub fn assign_bsize_float(&mut self, ctx: &mut LayoutContext) {
        let mut floats = Floats::new(self.fragment.style.writing_mode);
        for kid in self.base.child_iter() {
            flow::mut_base(kid).floats = floats.clone();
            kid.assign_bsize_for_inorder_child_if_necessary(ctx);
            floats = flow::mut_base(kid).floats.clone();
        }

        // Floats establish a block formatting context, so we discard the output floats here.
        drop(floats);

        let bstart_offset = self.fragment.margin.bstart + self.fragment.border_padding.bstart;
        let mut cur_b = bstart_offset;

        // cur_b is now at the bstart content edge

        for kid in self.base.child_iter() {
            let child_base = flow::mut_base(kid);
            child_base.position.start.b = cur_b;
            // cur_b is now at the bend margin edge of kid
            cur_b = cur_b + child_base.position.size.bsize;
        }

        let content_bsize = cur_b - bstart_offset;

        // The associated fragment has the border box of this flow.
        self.fragment.border_box.start.b = self.fragment.margin.bstart;

        // Calculate content bsize, taking `min-bsize` and `max-bsize` into account.
        let mut candidate_bsize_iterator = CandidateBSizeIterator::new(self.fragment.style(), None);
        for (candidate_bsize, new_candidate_bsize) in candidate_bsize_iterator {
            *new_candidate_bsize = match candidate_bsize {
                Auto => content_bsize,
                Specified(value) => value,
            }
        }

        let content_bsize = candidate_bsize_iterator.candidate_value;
        let noncontent_bsize = self.fragment.border_padding.bstart_end();
        debug!("assign_bsize_float -- bsize: {}", content_bsize + noncontent_bsize);
        self.fragment.border_box.size.bsize = content_bsize + noncontent_bsize;
    }

    fn build_display_list_block_common(&mut self,
                                       layout_context: &LayoutContext,
                                       offset: LogicalPoint<Au>,
                                       background_border_level: BackgroundAndBorderLevel) {
        let rel_offset =
            self.fragment.relative_position(&self.base
                                             .absolute_position_info
                                             .relative_containing_block_size,
                                        None);

        // Add the box that starts the block context.
        let mut display_list = DisplayList::new();
        let mut accumulator =
            self.fragment.build_display_list(&mut display_list,
                                             layout_context,
                                             self.base.abs_position
                                                .add_point(&offset)
                                                + rel_offset,
                                             background_border_level,
                                             None);

        let mut child_layers = DList::new();
        for kid in self.base.child_iter() {
            if kid.is_absolutely_positioned() {
                // All absolute flows will be handled by their containing block.
                continue
            }

            accumulator.push_child(&mut display_list, kid);
            child_layers.append(mem::replace(&mut flow::mut_base(kid).layers, DList::new()))
        }

        // Process absolute descendant links.
        for abs_descendant_link in self.base.abs_descendants.iter() {
            // TODO(pradeep): Send in our absolute position directly.
            accumulator.push_child(&mut display_list, abs_descendant_link);
            child_layers.append(mem::replace(&mut flow::mut_base(abs_descendant_link).layers,
                                             DList::new()));
        }

        accumulator.finish(&mut *self, display_list);
        self.base.layers = child_layers
    }

    /// Add display items for current block.
    ///
    /// Set the absolute position for children after doing any offsetting for
    /// position: relative.
    pub fn build_display_list_block(&mut self, layout_context: &LayoutContext) {
        if self.is_float() {
            // TODO(#2009, pcwalton): This is a pseudo-stacking context. We need to merge `z-index:
            // auto` kids into the parent stacking context, when that is supported.
            self.build_display_list_float(layout_context)
        } else if self.is_absolutely_positioned() {
            self.build_display_list_abs(layout_context)
        } else {
            let writing_mode = self.base.writing_mode;
            self.build_display_list_block_common(
                layout_context, LogicalPoint::zero(writing_mode), BlockLevel)
        }
    }

    pub fn build_display_list_float(&mut self, layout_context: &LayoutContext) {
        let float_offset = self.float.get_ref().rel_pos;
        self.build_display_list_block_common(layout_context,
                                             float_offset,
                                             RootOfStackingContextLevel);
        self.base.display_list = mem::replace(&mut self.base.display_list,
                                              DisplayList::new()).flatten(FloatStackingLevel)
    }

    /// Calculate and set the bsize, offsets, etc. for absolutely positioned flow.
    ///
    /// The layout for its in-flow children has been done during normal layout.
    /// This is just the calculation of:
    /// + bsize for the flow
    /// + y-coordinate of the flow wrt its Containing Block.
    /// + bsize, vertical margins, and y-coordinate for the flow's box.
    fn calculate_abs_bsize_and_margins(&mut self, ctx: &LayoutContext) {
        let containing_block_bsize = self.containing_block_size(ctx.screen_size).bsize;
        let static_b_offset = self.static_b_offset;

        // This is the stored content bsize value from assign-bsize
        let content_bsize = self.fragment.content_box().size.bsize;

        let mut solution = None;
        {
            // Non-auto margin-bstart and margin-bend values have already been
            // calculated during assign-isize.
            let margin = self.fragment.style().logical_margin();
            let margin_bstart = match margin.bstart {
                LPA_Auto => Auto,
                _ => Specified(self.fragment.margin.bstart)
            };
            let margin_bend = match margin.bend {
                LPA_Auto => Auto,
                _ => Specified(self.fragment.margin.bend)
            };

            let bstart;
            let bend;
            {
                let position = self.fragment.style().logical_position();
                bstart = MaybeAuto::from_style(position.bstart, containing_block_bsize);
                bend = MaybeAuto::from_style(position.bend, containing_block_bsize);
            }

            let available_bsize = containing_block_bsize - self.fragment.border_padding.bstart_end();
            if self.is_replaced_content() {
                // Calculate used value of bsize just like we do for inline replaced elements.
                // TODO: Pass in the containing block bsize when Fragment's
                // assign-bsize can handle it correctly.
                self.fragment.assign_replaced_bsize_if_necessary();
                // TODO: Right now, this content bsize value includes the
                // margin because of erroneous bsize calculation in fragment.
                // Check this when that has been fixed.
                let bsize_used_val = self.fragment.border_box.size.bsize;
                solution = Some(BSizeConstraintSolution::solve_vertical_constraints_abs_replaced(
                        bsize_used_val,
                        margin_bstart,
                        margin_bend,
                        bstart,
                        bend,
                        content_bsize,
                        available_bsize,
                        static_b_offset));
            } else {
                let style = self.fragment.style();
                let mut candidate_bsize_iterator =
                    CandidateBSizeIterator::new(style, Some(containing_block_bsize));

                for (bsize_used_val, new_candidate_bsize) in candidate_bsize_iterator {
                    solution =
                        Some(BSizeConstraintSolution::solve_vertical_constraints_abs_nonreplaced(
                            bsize_used_val,
                            margin_bstart,
                            margin_bend,
                            bstart,
                            bend,
                            content_bsize,
                            available_bsize,
                            static_b_offset));

                    *new_candidate_bsize = solution.unwrap().bsize
                }
            }
        }

        let solution = solution.unwrap();
        self.fragment.margin.bstart = solution.margin_bstart;
        self.fragment.margin.bend = solution.margin_bend;
        self.fragment.border_box.start.b = Au(0);
        self.fragment.border_box.size.bsize = solution.bsize + self.fragment.border_padding.bstart_end();

        self.base.position.start.b = solution.bstart + self.fragment.margin.bstart;
        self.base.position.size.bsize = solution.bsize + self.fragment.border_padding.bstart_end();
    }

    /// Add display items for Absolutely Positioned flow.
    fn build_display_list_abs(&mut self, layout_context: &LayoutContext) {
        let writing_mode = self.base.writing_mode;
        self.build_display_list_block_common(layout_context,
                                             LogicalPoint::zero(writing_mode),
                                             RootOfStackingContextLevel);

        if !self.base.absolute_position_info.layers_needed_for_positioned_flows &&
                !self.base.flags.needs_layer() {
            // We didn't need a layer.
            //
            // TODO(#781, pcwalton): `z-index`.
            self.base.display_list =
                mem::replace(&mut self.base.display_list,
                             DisplayList::new()).flatten(PositionedDescendantStackingLevel(0));
            return
        }

        // If we got here, then we need a new layer.
        let layer_rect = self.base.position.union(&self.base.overflow);
        let size = Size2D(layer_rect.size.isize.to_nearest_px() as uint,
                          layer_rect.size.bsize.to_nearest_px() as uint);
        let origin = Point2D(layer_rect.start.i.to_nearest_px() as uint,
                             layer_rect.start.b.to_nearest_px() as uint);
        let scroll_policy = if self.is_fixed() {
            FixedPosition
        } else {
            Scrollable
        };
        let display_list = mem::replace(&mut self.base.display_list, DisplayList::new());
        let new_layer = RenderLayer {
            id: self.layer_id(0),
            display_list: Arc::new(display_list.flatten(ContentStackingLevel)),
            position: Rect(origin, size),
            background_color: color::rgba(255.0, 255.0, 255.0, 0.0),
            scroll_policy: scroll_policy,
        };
        self.base.layers.push_back(new_layer)
    }

    /// Return the bstart outer edge of the hypothetical box for an absolute flow.
    ///
    /// This is wrt its parent flow box.
    ///
    /// During normal layout assign-bsize, the absolute flow's position is
    /// roughly set to its static position (the position it would have had in
    /// the normal flow).
    fn get_hypothetical_bstart_edge(&self) -> Au {
        self.base.position.start.b
    }

    /// Assigns the computed istart content edge and isize to all the children of this block flow.
    /// Also computes whether each child will be impacted by floats.
    ///
    /// `#[inline(always)]` because this is called only from block or table isize assignment and
    /// the code for block layout is significantly simpler.
    #[inline(always)]
    pub fn propagate_assigned_isize_to_children(&mut self,
                                                istart_content_edge: Au,
                                                content_isize: Au,
                                                opt_col_isizes: Option<Vec<Au>>) {
        // Keep track of whether floats could impact each child.
        let mut istart_floats_impact_child = self.base.flags.impacted_by_left_floats();
        let mut iend_floats_impact_child = self.base.flags.impacted_by_right_floats();

        let absolute_static_i_offset = if self.is_positioned() {
            // This flow is the containing block. The static X offset will be the istart padding
            // edge.
            self.fragment.border_padding.istart
                - self.fragment.style().logical_border_width().istart
        } else {
            // For kids, the istart margin edge will be at our istart content edge. The current static
            // offset is at our istart margin edge. So move in to the istart content edge.
            self.base.absolute_static_i_offset + istart_content_edge
        };

        let fixed_static_i_offset = self.base.fixed_static_i_offset + istart_content_edge;
        let flags = self.base.flags.clone();

        // This value is used only for table cells.
        let mut istart_margin_edge = istart_content_edge;

        // The isize of the last float, if there was one. This is used for estimating the isizes of
        // block formatting contexts. (We estimate that the isize of any block formatting context
        // that we see will be based on the isize of the containing block as well as the last float
        // seen before it.)
        let mut last_float_isize = None;

        for (i, kid) in self.base.child_iter().enumerate() {
            if kid.is_block_flow() {
                let kid_block = kid.as_block();
                kid_block.base.absolute_static_i_offset = absolute_static_i_offset;
                kid_block.base.fixed_static_i_offset = fixed_static_i_offset;

                if kid_block.is_float() {
                    last_float_isize = Some(kid_block.base.intrinsic_isizes.preferred_isize)
                } else {
                    kid_block.previous_float_isize = last_float_isize
                }
            }

            // The istart margin edge of the child flow is at our istart content edge, and its isize
            // is our content isize.
            flow::mut_base(kid).position.start.i = istart_content_edge;
            flow::mut_base(kid).position.size.isize = content_isize;

            // Determine float impaction.
            match kid.float_clearance() {
                clear::none => {}
                clear::left => istart_floats_impact_child = false,
                clear::right => iend_floats_impact_child = false,
                clear::both => {
                    istart_floats_impact_child = false;
                    iend_floats_impact_child = false;
                }
            }
            {
                let kid_base = flow::mut_base(kid);
                istart_floats_impact_child = istart_floats_impact_child ||
                    kid_base.flags.has_left_floated_descendants();
                iend_floats_impact_child = iend_floats_impact_child ||
                    kid_base.flags.has_right_floated_descendants();
                kid_base.flags.set_impacted_by_left_floats(istart_floats_impact_child);
                kid_base.flags.set_impacted_by_right_floats(iend_floats_impact_child);
            }

            // Handle tables.
            match opt_col_isizes {
                Some(ref col_isizes) => {
                    propagate_column_isizes_to_child(kid,
                                                     i,
                                                     content_isize,
                                                     col_isizes.as_slice(),
                                                     &mut istart_margin_edge)
                }
                None => {}
            }

            // Per CSS 2.1 § 16.3.1, text alignment propagates to all children in flow.
            //
            // TODO(#2018, pcwalton): Do this in the cascade instead.
            flow::mut_base(kid).flags.propagate_text_alignment_from_parent(flags.clone())
        }
    }

    /// Determines the type of formatting context this is. See the definition of
    /// `FormattingContextType`.
    fn formatting_context_type(&self) -> FormattingContextType {
        let style = self.fragment.style();
        if style.get_box().float != float::none {
            return OtherFormattingContext
        }
        match style.get_box().display {
            display::table_cell | display::table_caption | display::inline_block => {
                OtherFormattingContext
            }
            _ if style.get_box().position == position::static_ &&
                    style.get_box().overflow != overflow::visible => {
                BlockFormattingContext
            }
            _ => NonformattingContext,
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

    /// Returns the direction that this flow clears floats in, if any.
    fn float_clearance(&self) -> clear::T {
        self.fragment.style().get_box().clear
    }

    /// Pass 1 of reflow: computes minimum and preferred isizes.
    ///
    /// Recursively (bottom-up) determine the flow's minimum and preferred isizes. When called on
    /// this flow, all child flows have had their minimum and preferred isizes set. This function
    /// must decide minimum/preferred isizes based on its children's isizes and the dimensions of
    /// any fragments it is responsible for flowing.
    ///
    /// TODO(pcwalton): Inline blocks.
    fn bubble_isizes(&mut self, _: &mut LayoutContext) {
        let mut flags = self.base.flags;
        flags.set_has_left_floated_descendants(false);
        flags.set_has_right_floated_descendants(false);

        // Find the maximum isize from children.
        let mut intrinsic_isizes = IntrinsicISizes::new();
        for child_ctx in self.base.child_iter() {
            assert!(child_ctx.is_block_flow() ||
                    child_ctx.is_inline_flow() ||
                    child_ctx.is_table_kind());

            let child_base = flow::mut_base(child_ctx);
            intrinsic_isizes.minimum_isize =
                geometry::max(intrinsic_isizes.minimum_isize,
                              child_base.intrinsic_isizes.total_minimum_isize());
            intrinsic_isizes.preferred_isize =
                geometry::max(intrinsic_isizes.preferred_isize,
                              child_base.intrinsic_isizes.total_preferred_isize());

            flags.union_floated_descendants_flags(child_base.flags);
        }

        let fragment_intrinsic_isizes = self.fragment.intrinsic_isizes(None);
        intrinsic_isizes.minimum_isize = geometry::max(intrinsic_isizes.minimum_isize,
                                                       fragment_intrinsic_isizes.minimum_isize);
        intrinsic_isizes.preferred_isize = geometry::max(intrinsic_isizes.preferred_isize,
                                                         fragment_intrinsic_isizes.preferred_isize);
        intrinsic_isizes.surround_isize = fragment_intrinsic_isizes.surround_isize;
        self.base.intrinsic_isizes = intrinsic_isizes;

        match self.fragment.style().get_box().float {
            float::none => {}
            float::left => flags.set_has_left_floated_descendants(true),
            float::right => flags.set_has_right_floated_descendants(true),
        }
        self.base.flags = flags
    }

    /// Recursively (top-down) determines the actual isize of child contexts and fragments. When
    /// called on this context, the context has had its isize set by the parent context.
    ///
    /// Dual fragments consume some isize first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_isizes(&mut self, layout_context: &mut LayoutContext) {
        debug!("assign_isizes({}): assigning isize for flow",
               if self.is_float() {
                   "float"
               } else {
                   "block"
               });

        if self.is_root() {
            debug!("Setting root position");
            self.base.position.start = LogicalPoint::zero(self.base.writing_mode);
            self.base.position.size.isize = LogicalSize::from_physical(
                self.base.writing_mode, layout_context.screen_size).isize;
            self.base.floats = Floats::new(self.base.writing_mode);

            // The root element is never impacted by floats.
            self.base.flags.set_impacted_by_left_floats(false);
            self.base.flags.set_impacted_by_right_floats(false);
        }

        // Our isize was set to the isize of the containing block by the flow's parent. Now compute
        // the real value.
        let containing_block_isize = self.base.position.size.isize;
        self.compute_used_isize(layout_context, containing_block_isize);
        if self.is_float() {
            self.float.get_mut_ref().containing_isize = containing_block_isize;
        }

        // Formatting contexts are never impacted by floats.
        match self.formatting_context_type() {
            NonformattingContext => {}
            BlockFormattingContext => {
                self.base.flags.set_impacted_by_left_floats(false);
                self.base.flags.set_impacted_by_right_floats(false);

                // We can't actually compute the isize of this block now, because floats might
                // affect it. Speculate that its isize is equal to the isize computed above minus
                // the isize of the previous float.
                match self.previous_float_isize {
                    None => {}
                    Some(previous_float_isize) => {
                        self.fragment.border_box.size.isize =
                            self.fragment.border_box.size.isize - previous_float_isize
                    }
                }
            }
            OtherFormattingContext => {
                self.base.flags.set_impacted_by_left_floats(false);
                self.base.flags.set_impacted_by_right_floats(false);
            }
        }

        // Move in from the istart border edge
        let istart_content_edge = self.fragment.border_box.start.i + self.fragment.border_padding.istart;
        let padding_and_borders = self.fragment.border_padding.istart_end();
        let content_isize = self.fragment.border_box.size.isize - padding_and_borders;

        if self.is_float() {
            self.base.position.size.isize = content_isize;
        }

        self.propagate_assigned_isize_to_children(istart_content_edge, content_isize, None);
    }

    /// Assigns bsizes in-order; or, if this is a float, places the float. The default
    /// implementation simply assigns bsizes if this flow is impacted by floats. Returns true if
    /// this child was impacted by floats or false otherwise.
    ///
    /// This is called on child flows by the parent. Hence, we can assume that `assign_bsize` has
    /// already been called on the child (because of the bottom-up traversal).
    fn assign_bsize_for_inorder_child_if_necessary(&mut self, layout_context: &mut LayoutContext)
                                                    -> bool {
        if self.is_float() {
            self.place_float();
            return true
        }

        let impacted = self.base.flags.impacted_by_floats();
        if impacted {
            self.assign_bsize(layout_context);
        }
        impacted
    }

    fn assign_bsize(&mut self, ctx: &mut LayoutContext) {
        // Assign bsize for fragment if it is an image fragment.
        self.fragment.assign_replaced_bsize_if_necessary();

        if self.is_float() {
            debug!("assign_bsize_float: assigning bsize for float");
            self.assign_bsize_float(ctx);
        } else {
            debug!("assign_bsize: assigning bsize for block");
            self.assign_bsize_block_base(ctx, MarginsMayCollapse);
        }
    }

    fn compute_absolute_position(&mut self) {
        if self.is_absolutely_positioned() {
            self.base
                .absolute_position_info
                .absolute_containing_block_position = if self.is_fixed() {
                // The viewport is initially at (0, 0).
                self.base.position.start
            } else {
                // Absolute position of the containing block + position of absolute flow w/r/t the
                // containing block.
                self.base.absolute_position_info.absolute_containing_block_position
                    .add_point(&self.base.position.start)
            };

            // Set the absolute position, which will be passed down later as part
            // of containing block details for absolute descendants.
            self.base.abs_position =
                self.base.absolute_position_info.absolute_containing_block_position;
        }

        // For relatively-positioned descendants, the containing block formed by a block is just
        // the content box. The containing block for absolutely-positioned descendants, on the
        // other hand, is only established if we are positioned.
        let relative_offset =
            self.fragment.relative_position(&self.base
                                                 .absolute_position_info
                                                 .relative_containing_block_size,
                                            None);
        if self.is_positioned() {
            self.base.absolute_position_info.absolute_containing_block_position =
                self.base.abs_position
                .add_point(&self.generated_containing_block_rect().start)
                + relative_offset
        }

        let float_offset = if self.is_float() {
            self.float.get_ref().rel_pos
        } else {
            LogicalPoint::zero(self.base.writing_mode)
        };

        // Compute absolute position info for children.
        let mut absolute_position_info = self.base.absolute_position_info;
        absolute_position_info.relative_containing_block_size = self.fragment.content_box().size;
        absolute_position_info.layers_needed_for_positioned_flows =
            self.base.flags.layers_needed_for_descendants();

        // Process children.
        let this_position = self.base.abs_position;
        for kid in self.base.child_iter() {
            if !kid.is_absolutely_positioned() {
                let kid_base = flow::mut_base(kid);
                kid_base.abs_position =
                    this_position
                    .add_point(&kid_base.position.start)
                    .add_point(&float_offset)
                    + relative_offset;
                kid_base.absolute_position_info = absolute_position_info
            }
        }

        // Process absolute descendant links.
        for absolute_descendant in self.base.abs_descendants.iter() {
            flow::mut_base(absolute_descendant).absolute_position_info = absolute_position_info
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
        self.fragment.style.get_box().position
    }

    /// Return true if this is the root of an Absolute flow tree.
    ///
    /// It has to be either relatively positioned or the Root flow.
    fn is_root_of_absolute_flow_tree(&self) -> bool {
        self.is_relatively_positioned() || self.is_root()
    }

    /// Return the dimensions of the containing block generated by this flow for absolutely-
    /// positioned descendants. For block flows, this is the padding box.
    fn generated_containing_block_rect(&self) -> LogicalRect<Au> {
        self.fragment.border_box - self.fragment.style().logical_border_width()
    }

    fn layer_id(&self, fragment_index: uint) -> LayerId {
        // FIXME(#2010, pcwalton): This is a hack and is totally bogus in the presence of pseudo-
        // elements. But until we have incremental reflow we can't do better--we recreate the flow
        // for every DOM node so otherwise we nuke layers on every reflow.
        LayerId(self.fragment.node.id(), fragment_index)
    }

    fn is_absolute_containing_block(&self) -> bool {
        self.is_positioned()
    }
}

impl fmt::Show for BlockFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_float() {
            write!(f, "FloatFlow: {}", self.fragment)
        } else if self.is_root() {
            write!(f, "RootFlow: {}", self.fragment)
        } else {
            write!(f, "BlockFlow: {}", self.fragment)
        }
    }
}

/// The inputs for the isizes-and-margins constraint equation.
pub struct ISizeConstraintInput {
    pub computed_isize: MaybeAuto,
    pub istart_margin: MaybeAuto,
    pub iend_margin: MaybeAuto,
    pub istart: MaybeAuto,
    pub iend: MaybeAuto,
    pub available_isize: Au,
    pub static_i_offset: Au,
}

impl ISizeConstraintInput {
    pub fn new(computed_isize: MaybeAuto,
               istart_margin: MaybeAuto,
               iend_margin: MaybeAuto,
               istart: MaybeAuto,
               iend: MaybeAuto,
               available_isize: Au,
               static_i_offset: Au)
           -> ISizeConstraintInput {
        ISizeConstraintInput {
            computed_isize: computed_isize,
            istart_margin: istart_margin,
            iend_margin: iend_margin,
            istart: istart,
            iend: iend,
            available_isize: available_isize,
            static_i_offset: static_i_offset,
        }
    }
}

/// The solutions for the isizes-and-margins constraint equation.
pub struct ISizeConstraintSolution {
    pub istart: Au,
    pub iend: Au,
    pub isize: Au,
    pub margin_istart: Au,
    pub margin_iend: Au
}

impl ISizeConstraintSolution {
    pub fn new(isize: Au, margin_istart: Au, margin_iend: Au) -> ISizeConstraintSolution {
        ISizeConstraintSolution {
            istart: Au(0),
            iend: Au(0),
            isize: isize,
            margin_istart: margin_istart,
            margin_iend: margin_iend,
        }
    }

    fn for_absolute_flow(istart: Au,
                         iend: Au,
                         isize: Au,
                         margin_istart: Au,
                         margin_iend: Au)
                         -> ISizeConstraintSolution {
        ISizeConstraintSolution {
            istart: istart,
            iend: iend,
            isize: isize,
            margin_istart: margin_istart,
            margin_iend: margin_iend,
        }
    }
}

// Trait to encapsulate the ISize and Margin calculation.
//
// CSS Section 10.3
pub trait ISizeAndMarginsComputer {
    /// Compute the inputs for the ISize constraint equation.
    ///
    /// This is called only once to compute the initial inputs. For
    /// calculation involving min-isize and max-isize, we don't need to
    /// recompute these.
    fn compute_isize_constraint_inputs(&self,
                                       block: &mut BlockFlow,
                                       parent_flow_isize: Au,
                                       ctx: &mut LayoutContext)
                                       -> ISizeConstraintInput {
        let containing_block_isize = self.containing_block_isize(block, parent_flow_isize, ctx);
        let computed_isize = self.initial_computed_isize(block, parent_flow_isize, ctx);

        block.fragment.compute_border_padding_margins(containing_block_isize, None);

        let style = block.fragment.style();

        // The text alignment of a block flow is the text alignment of its box's style.
        block.base.flags.set_text_align(style.get_inheritedtext().text_align);

        let margin = style.logical_margin();
        let position = style.logical_position();

        let available_isize = containing_block_isize - block.fragment.border_padding.istart_end();
        return ISizeConstraintInput::new(
            computed_isize,
            MaybeAuto::from_style(margin.istart, containing_block_isize),
            MaybeAuto::from_style(margin.iend, containing_block_isize),
            MaybeAuto::from_style(position.istart, containing_block_isize),
            MaybeAuto::from_style(position.iend, containing_block_isize),
            available_isize,
            block.static_i_offset());
    }

    /// Set the used values for isize and margins got from the relevant constraint equation.
    ///
    /// This is called only once.
    ///
    /// Set:
    /// + used values for content isize, istart margin, and iend margin for this flow's box.
    /// + x-coordinate of this flow's box.
    /// + x-coordinate of the flow wrt its Containing Block (if this is an absolute flow).
    fn set_isize_constraint_solutions(&self,
                                      block: &mut BlockFlow,
                                      solution: ISizeConstraintSolution) {
        let isize;
        {
            let fragment = block.fragment();
            fragment.margin.istart = solution.margin_istart;
            fragment.margin.iend = solution.margin_iend;

            // The associated fragment has the border box of this flow.
            // Left border edge.
            fragment.border_box.start.i = fragment.margin.istart;
            // Border box isize.
            isize = solution.isize + fragment.border_padding.istart_end();
            fragment.border_box.size.isize = isize;
        }

        // We also resize the block itself, to ensure that overflow is not calculated
        // as the isize of our parent. We might be smaller and we might be larger if we
        // overflow.
        let flow = flow::mut_base(block);
        flow.position.size.isize = isize;
    }

    /// Set the x coordinate of the given flow if it is absolutely positioned.
    fn set_flow_x_coord_if_necessary(&self, _: &mut BlockFlow, _: ISizeConstraintSolution) {}

    /// Solve the isize and margins constraints for this block flow.
    fn solve_isize_constraints(&self,
                               block: &mut BlockFlow,
                               input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution;

    fn initial_computed_isize(&self,
                              block: &mut BlockFlow,
                              parent_flow_isize: Au,
                              ctx: &mut LayoutContext)
                              -> MaybeAuto {
        MaybeAuto::from_style(block.fragment().style().content_isize(),
                              self.containing_block_isize(block, parent_flow_isize, ctx))
    }

    fn containing_block_isize(&self,
                              _: &mut BlockFlow,
                              parent_flow_isize: Au,
                              _: &mut LayoutContext)
                              -> Au {
        parent_flow_isize
    }

    /// Compute the used value of isize, taking care of min-isize and max-isize.
    ///
    /// CSS Section 10.4: Minimum and Maximum isizes
    fn compute_used_isize(&self,
                          block: &mut BlockFlow,
                          ctx: &mut LayoutContext,
                          parent_flow_isize: Au) {
        let mut input = self.compute_isize_constraint_inputs(block, parent_flow_isize, ctx);

        let containing_block_isize = self.containing_block_isize(block, parent_flow_isize, ctx);

        let mut solution = self.solve_isize_constraints(block, &input);

        // If the tentative used isize is greater than 'max-isize', isize should be recalculated,
        // but this time using the computed value of 'max-isize' as the computed value for 'isize'.
        match specified_or_none(block.fragment().style().max_isize(), containing_block_isize) {
            Some(max_isize) if max_isize < solution.isize => {
                input.computed_isize = Specified(max_isize);
                solution = self.solve_isize_constraints(block, &input);
            }
            _ => {}
        }

        // If the resulting isize is smaller than 'min-isize', isize should be recalculated,
        // but this time using the value of 'min-isize' as the computed value for 'isize'.
        let computed_min_isize = specified(block.fragment().style().min_isize(),
                                           containing_block_isize);
        if computed_min_isize > solution.isize {
            input.computed_isize = Specified(computed_min_isize);
            solution = self.solve_isize_constraints(block, &input);
        }

        self.set_isize_constraint_solutions(block, solution);
        self.set_flow_x_coord_if_necessary(block, solution);
    }

    /// Computes istart and iend margins and isize.
    ///
    /// This is used by both replaced and non-replaced Blocks.
    ///
    /// CSS 2.1 Section 10.3.3.
    /// Constraint Equation: margin-istart + margin-iend + isize = available_isize
    /// where available_isize = CB isize - (horizontal border + padding)
    fn solve_block_isize_constraints(&self,
                                     _: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        let (computed_isize, istart_margin, iend_margin, available_isize) = (input.computed_isize,
                                                                            input.istart_margin,
                                                                            input.iend_margin,
                                                                            input.available_isize);

        // If isize is not 'auto', and isize + margins > available_isize, all
        // 'auto' margins are treated as 0.
        let (istart_margin, iend_margin) = match computed_isize {
            Auto => (istart_margin, iend_margin),
            Specified(isize) => {
                let istart = istart_margin.specified_or_zero();
                let iend = iend_margin.specified_or_zero();

                if (istart + iend + isize) > available_isize {
                    (Specified(istart), Specified(iend))
                } else {
                    (istart_margin, iend_margin)
                }
            }
        };

        // Invariant: istart_margin + isize + iend_margin == available_isize
        let (istart_margin, isize, iend_margin) = match (istart_margin, computed_isize, iend_margin) {
            // If all have a computed value other than 'auto', the system is
            // over-constrained so we discard the end margin.
            (Specified(margin_start), Specified(isize), Specified(_margin_end)) =>
                (margin_start, isize, available_isize - (margin_start + isize)),

            // If exactly one value is 'auto', solve for it
            (Auto, Specified(isize), Specified(margin_end)) =>
                (available_isize - (isize + margin_end), isize, margin_end),
            (Specified(margin_start), Auto, Specified(margin_end)) =>
                (margin_start, available_isize - (margin_start + margin_end), margin_end),
            (Specified(margin_start), Specified(isize), Auto) =>
                (margin_start, isize, available_isize - (margin_start + isize)),

            // If isize is set to 'auto', any other 'auto' value becomes '0',
            // and isize is solved for
            (Auto, Auto, Specified(margin_end)) =>
                (Au::new(0), available_isize - margin_end, margin_end),
            (Specified(margin_start), Auto, Auto) =>
                (margin_start, available_isize - margin_start, Au::new(0)),
            (Auto, Auto, Auto) =>
                (Au::new(0), available_isize, Au::new(0)),

            // If istart and iend margins are auto, they become equal
            (Auto, Specified(isize), Auto) => {
                let margin = (available_isize - isize).scale_by(0.5);
                (margin, isize, margin)
            }
        };
        ISizeConstraintSolution::new(isize, istart_margin, iend_margin)
    }
}

/// The different types of Blocks.
///
/// They mainly differ in the way isize and bsizes and margins are calculated
/// for them.
struct AbsoluteNonReplaced;
struct AbsoluteReplaced;
struct BlockNonReplaced;
struct BlockReplaced;
struct FloatNonReplaced;
struct FloatReplaced;

impl ISizeAndMarginsComputer for AbsoluteNonReplaced {
    /// Solve the horizontal constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.3.7
    /// Constraint equation:
    /// istart + iend + isize + margin-istart + margin-iend
    /// = absolute containing block isize - (horizontal padding and border)
    /// [aka available_isize]
    ///
    /// Return the solution for the equation.
    fn solve_isize_constraints(&self,
                               block: &mut BlockFlow,
                               input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        let &ISizeConstraintInput {
            computed_isize,
            istart_margin,
            iend_margin,
            istart,
            iend,
            available_isize,
            static_i_offset,
            ..
        } = input;

        // TODO: Check for direction of parent flow (NOT Containing Block)
        // when right-to-left is implemented.
        // Assume direction is 'ltr' for now

        // Distance from the istart edge of the Absolute Containing Block to the
        // istart margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_istart = static_i_offset;

        let (istart, iend, isize, margin_istart, margin_iend) = match (istart, iend, computed_isize) {
            (Auto, Auto, Auto) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                let istart = static_position_istart;
                // Now it is the same situation as istart Specified and iend
                // and isize Auto.

                // Set iend to zero to calculate isize
                let isize = block.get_shrink_to_fit_isize(
                    available_isize - (istart + margin_start + margin_end));
                let sum = istart + isize + margin_start + margin_end;
                (istart, available_isize - sum, isize, margin_start, margin_end)
            }
            (Specified(istart), Specified(iend), Specified(isize)) => {
                match (istart_margin, iend_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = available_isize - istart - iend - isize;
                        if total_margin_val < Au(0) {
                            // margin-istart becomes 0 because direction is 'ltr'.
                            // TODO: Handle 'rtl' when it is implemented.
                            (istart, iend, isize, Au(0), total_margin_val)
                        } else {
                            // Equal margins
                            (istart, iend, isize,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                    }
                    (Specified(margin_start), Auto) => {
                        let sum = istart + iend + isize + margin_start;
                        (istart, iend, isize, margin_start, available_isize - sum)
                    }
                    (Auto, Specified(margin_end)) => {
                        let sum = istart + iend + isize + margin_end;
                        (istart, iend, isize, available_isize - sum, margin_end)
                    }
                    (Specified(margin_start), Specified(margin_end)) => {
                        // Values are over-constrained.
                        // Ignore value for 'iend' cos direction is 'ltr'.
                        // TODO: Handle 'rtl' when it is implemented.
                        let sum = istart + isize + margin_start + margin_end;
                        (istart, available_isize - sum, isize, margin_start, margin_end)
                    }
                }
            }
            // For the rest of the cases, auto values for margin are set to 0

            // If only one is Auto, solve for it
            (Auto, Specified(iend), Specified(isize)) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                let sum = iend + isize + margin_start + margin_end;
                (available_isize - sum, iend, isize, margin_start, margin_end)
            }
            (Specified(istart), Auto, Specified(isize)) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                let sum = istart + isize + margin_start + margin_end;
                (istart, available_isize - sum, isize, margin_start, margin_end)
            }
            (Specified(istart), Specified(iend), Auto) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                let sum = istart + iend + margin_start + margin_end;
                (istart, iend, available_isize - sum, margin_start, margin_end)
            }

            // If isize is auto, then isize is shrink-to-fit. Solve for the
            // non-auto value.
            (Specified(istart), Auto, Auto) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                // Set iend to zero to calculate isize
                let isize = block.get_shrink_to_fit_isize(
                    available_isize - (istart + margin_start + margin_end));
                let sum = istart + isize + margin_start + margin_end;
                (istart, available_isize - sum, isize, margin_start, margin_end)
            }
            (Auto, Specified(iend), Auto) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                // Set istart to zero to calculate isize
                let isize = block.get_shrink_to_fit_isize(
                    available_isize - (iend + margin_start + margin_end));
                let sum = iend + isize + margin_start + margin_end;
                (available_isize - sum, iend, isize, margin_start, margin_end)
            }

            (Auto, Auto, Specified(isize)) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                // Setting 'istart' to static position because direction is 'ltr'.
                // TODO: Handle 'rtl' when it is implemented.
                let istart = static_position_istart;
                let sum = istart + isize + margin_start + margin_end;
                (istart, available_isize - sum, isize, margin_start, margin_end)
            }
        };
        ISizeConstraintSolution::for_absolute_flow(istart, iend, isize, margin_istart, margin_iend)
    }

    fn containing_block_isize(&self, block: &mut BlockFlow, _: Au, ctx: &mut LayoutContext) -> Au {
        block.containing_block_size(ctx.screen_size).isize
    }

    fn set_flow_x_coord_if_necessary(&self,
                                     block: &mut BlockFlow,
                                     solution: ISizeConstraintSolution) {
        // Set the x-coordinate of the absolute flow wrt to its containing block.
        block.base.position.start.i = solution.istart;
    }
}

impl ISizeAndMarginsComputer for AbsoluteReplaced {
    /// Solve the horizontal constraint equation for absolute replaced elements.
    ///
    /// `static_i_offset`: total offset of current flow's hypothetical
    /// position (static position) from its actual Containing Block.
    ///
    /// CSS Section 10.3.8
    /// Constraint equation:
    /// istart + iend + isize + margin-istart + margin-iend
    /// = absolute containing block isize - (horizontal padding and border)
    /// [aka available_isize]
    ///
    /// Return the solution for the equation.
    fn solve_isize_constraints(&self, _: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        let &ISizeConstraintInput {
            computed_isize,
            istart_margin,
            iend_margin,
            istart,
            iend,
            available_isize,
            static_i_offset,
            ..
        } = input;
        // TODO: Check for direction of static-position Containing Block (aka
        // parent flow, _not_ the actual Containing Block) when right-to-left
        // is implemented
        // Assume direction is 'ltr' for now
        // TODO: Handle all the cases for 'rtl' direction.

        let isize = match computed_isize {
            Specified(w) => w,
            _ => fail!("{} {}",
                       "The used value for isize for absolute replaced flow",
                       "should have already been calculated by now.")
        };

        // Distance from the istart edge of the Absolute Containing Block to the
        // istart margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_istart = static_i_offset;

        let (istart, iend, isize, margin_istart, margin_iend) = match (istart, iend) {
            (Auto, Auto) => {
                let istart = static_position_istart;
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                let sum = istart + isize + margin_start + margin_end;
                (istart, available_isize - sum, isize, margin_start, margin_end)
            }
            // If only one is Auto, solve for it
            (Auto, Specified(iend)) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                let sum = iend + isize + margin_start + margin_end;
                (available_isize - sum, iend, isize, margin_start, margin_end)
            }
            (Specified(istart), Auto) => {
                let margin_start = istart_margin.specified_or_zero();
                let margin_end = iend_margin.specified_or_zero();
                let sum = istart + isize + margin_start + margin_end;
                (istart, available_isize - sum, isize, margin_start, margin_end)
            }
            (Specified(istart), Specified(iend)) => {
                match (istart_margin, iend_margin) {
                    (Auto, Auto) => {
                        let total_margin_val = available_isize - istart - iend - isize;
                        if total_margin_val < Au(0) {
                            // margin-istart becomes 0 because direction is 'ltr'.
                            (istart, iend, isize, Au(0), total_margin_val)
                        } else {
                            // Equal margins
                            (istart, iend, isize,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                    }
                    (Specified(margin_start), Auto) => {
                        let sum = istart + iend + isize + margin_start;
                        (istart, iend, isize, margin_start, available_isize - sum)
                    }
                    (Auto, Specified(margin_end)) => {
                        let sum = istart + iend + isize + margin_end;
                        (istart, iend, isize, available_isize - sum, margin_end)
                    }
                    (Specified(margin_start), Specified(margin_end)) => {
                        // Values are over-constrained.
                        // Ignore value for 'iend' cos direction is 'ltr'.
                        let sum = istart + isize + margin_start + margin_end;
                        (istart, available_isize - sum, isize, margin_start, margin_end)
                    }
                }
            }
        };
        ISizeConstraintSolution::for_absolute_flow(istart, iend, isize, margin_istart, margin_iend)
    }

    /// Calculate used value of isize just like we do for inline replaced elements.
    fn initial_computed_isize(&self,
                              block: &mut BlockFlow,
                              _: Au,
                              ctx: &mut LayoutContext)
                              -> MaybeAuto {
        let containing_block_isize = block.containing_block_size(ctx.screen_size).isize;
        let fragment = block.fragment();
        fragment.assign_replaced_isize_if_necessary(containing_block_isize, None);
        // For replaced absolute flow, the rest of the constraint solving will
        // take isize to be specified as the value computed here.
        Specified(fragment.content_isize())
    }

    fn containing_block_isize(&self, block: &mut BlockFlow, _: Au, ctx: &mut LayoutContext) -> Au {
        block.containing_block_size(ctx.screen_size).isize
    }

    fn set_flow_x_coord_if_necessary(&self, block: &mut BlockFlow, solution: ISizeConstraintSolution) {
        // Set the x-coordinate of the absolute flow wrt to its containing block.
        block.base.position.start.i = solution.istart;
    }
}

impl ISizeAndMarginsComputer for BlockNonReplaced {
    /// Compute istart and iend margins and isize.
    fn solve_isize_constraints(&self,
                               block: &mut BlockFlow,
                               input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        self.solve_block_isize_constraints(block, input)
    }
}

impl ISizeAndMarginsComputer for BlockReplaced {
    /// Compute istart and iend margins and isize.
    ///
    /// ISize has already been calculated. We now calculate the margins just
    /// like for non-replaced blocks.
    fn solve_isize_constraints(&self,
                               block: &mut BlockFlow,
                               input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        match input.computed_isize {
            Specified(_) => {},
            Auto => fail!("BlockReplaced: isize should have been computed by now")
        };
        self.solve_block_isize_constraints(block, input)
    }

    /// Calculate used value of isize just like we do for inline replaced elements.
    fn initial_computed_isize(&self,
                              block: &mut BlockFlow,
                              parent_flow_isize: Au,
                              _: &mut LayoutContext)
                              -> MaybeAuto {
        let fragment = block.fragment();
        fragment.assign_replaced_isize_if_necessary(parent_flow_isize, None);
        // For replaced block flow, the rest of the constraint solving will
        // take isize to be specified as the value computed here.
        Specified(fragment.content_isize())
    }

}

impl ISizeAndMarginsComputer for FloatNonReplaced {
    /// CSS Section 10.3.5
    ///
    /// If isize is computed as 'auto', the used value is the 'shrink-to-fit' isize.
    fn solve_isize_constraints(&self,
                               block: &mut BlockFlow,
                               input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        let (computed_isize, istart_margin, iend_margin, available_isize) = (input.computed_isize,
                                                                            input.istart_margin,
                                                                            input.iend_margin,
                                                                            input.available_isize);
        let margin_istart = istart_margin.specified_or_zero();
        let margin_iend = iend_margin.specified_or_zero();
        let available_isize_float = available_isize - margin_istart - margin_iend;
        let shrink_to_fit = block.get_shrink_to_fit_isize(available_isize_float);
        let isize = computed_isize.specified_or_default(shrink_to_fit);
        debug!("assign_isizes_float -- isize: {}", isize);
        ISizeConstraintSolution::new(isize, margin_istart, margin_iend)
    }
}

impl ISizeAndMarginsComputer for FloatReplaced {
    /// CSS Section 10.3.5
    ///
    /// If isize is computed as 'auto', the used value is the 'shrink-to-fit' isize.
    fn solve_isize_constraints(&self, _: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        let (computed_isize, istart_margin, iend_margin) = (input.computed_isize,
                                                           input.istart_margin,
                                                           input.iend_margin);
        let margin_istart = istart_margin.specified_or_zero();
        let margin_iend = iend_margin.specified_or_zero();
        let isize = match computed_isize {
            Specified(w) => w,
            Auto => fail!("FloatReplaced: isize should have been computed by now")
        };
        debug!("assign_isizes_float -- isize: {}", isize);
        ISizeConstraintSolution::new(isize, margin_istart, margin_iend)
    }

    /// Calculate used value of isize just like we do for inline replaced elements.
    fn initial_computed_isize(&self,
                              block: &mut BlockFlow,
                              parent_flow_isize: Au,
                              _: &mut LayoutContext)
                              -> MaybeAuto {
        let fragment = block.fragment();
        fragment.assign_replaced_isize_if_necessary(parent_flow_isize, None);
        // For replaced block flow, the rest of the constraint solving will
        // take isize to be specified as the value computed here.
        Specified(fragment.content_isize())
    }
}

fn propagate_column_isizes_to_child(kid: &mut Flow,
                                    child_index: uint,
                                    content_isize: Au,
                                    column_isizes: &[Au],
                                    istart_margin_edge: &mut Au) {
    // If kid is table_rowgroup or table_row, the column isizes info should be copied from its
    // parent.
    //
    // FIXME(pcwalton): This seems inefficient. Reference count it instead?
    let isize = if kid.is_table() || kid.is_table_rowgroup() || kid.is_table_row() {
        *kid.col_isizes() = column_isizes.iter().map(|&x| x).collect();

        // ISize of kid flow is our content isize.
        content_isize
    } else if kid.is_table_cell() {
        // If kid is table_cell, the x offset and isize for each cell should be
        // calculated from parent's column isizes info.
        *istart_margin_edge = if child_index == 0 {
            Au(0)
        } else {
            *istart_margin_edge + column_isizes[child_index - 1]
        };

        column_isizes[child_index]
    } else {
        // ISize of kid flow is our content isize.
        content_isize
    };

    let kid_base = flow::mut_base(kid);
    kid_base.position.start.i = *istart_margin_edge;
    kid_base.position.size.isize = isize;
}

