/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Layout for CSS block-level elements.
//!
//! As a terminology note, the term *absolute positioning* here refers to elements with position
//! `absolute` or `fixed`. The term *positioned element* refers to elements with position
//! `relative`, `absolute`, and `fixed`. The term *containing block* (occasionally abbreviated as
//! *CB*) is the containing block for the current flow, which differs from the static containing
//! block if the flow is absolutely-positioned.
//!
//! "CSS 2.1" or "CSS 2.2" refers to the editor's draft of the W3C "Cascading Style Sheets Level 2
//! Revision 2 (CSS 2.2) Specification" available here:
//!
//!   http://dev.w3.org/csswg/css2/
//!
//! "INTRINSIC" refers to L. David Baron's "More Precise Definitions of Inline Layout and Table
//! Layout" available here:
//!
//!   http://dbaron.org/css/intrinsic/
//!
//! "CSS-SIZING" refers to the W3C "CSS Intrinsic & Extrinsic Sizing Module Level 3" document
//! available here:
//!
//!   http://dev.w3.org/csswg/css-sizing/

#![deny(unsafe_blocks)]

use construct::FlowConstructor;
use context::LayoutContext;
use css::node_style::StyledNode;
use display_list_builder::{BlockFlowDisplayListBuilding, FragmentDisplayListBuilding};
use floats::{ClearType, FloatKind, Floats, PlacementInfo};
use flow::{self, AbsolutePositionInfo, BaseFlow, ForceNonfloatedFlag, FlowClass, Flow};
use flow::{ImmutableFlowUtils, MutableFlowUtils, PreorderFlowTraversal};
use flow::{PostorderFlowTraversal, mut_base};
use flow::{HAS_LEFT_FLOATED_DESCENDANTS, HAS_RIGHT_FLOATED_DESCENDANTS};
use flow::{IMPACTED_BY_LEFT_FLOATS, IMPACTED_BY_RIGHT_FLOATS};
use flow::{LAYERS_NEEDED_FOR_DESCENDANTS, NEEDS_LAYER};
use flow::{IS_ABSOLUTELY_POSITIONED};
use flow::{CLEARS_LEFT, CLEARS_RIGHT};
use fragment::{CoordinateSystem, Fragment, FragmentBorderBoxIterator, SpecificFragmentInfo};
use incremental::{REFLOW, REFLOW_OUT_OF_FLOW};
use layout_debug;
use model::{IntrinsicISizes, MarginCollapseInfo};
use model::{MaybeAuto, CollapsibleMargins, specified, specified_or_none};
use table::ColumnComputedInlineSize;
use wrapper::ThreadSafeLayoutNode;

use geom::{Point2D, Rect, Size2D};
use gfx::display_list::{ClippingRegion, DisplayList};
use serialize::{Encoder, Encodable};
use msg::compositor_msg::LayerId;
use servo_util::geometry::{Au, MAX_AU};
use servo_util::logical_geometry::{LogicalPoint, LogicalRect, LogicalSize};
use servo_util::opts;
use std::cmp::{max, min};
use std::fmt;
use style::properties::ComputedValues;
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto, LengthOrPercentageOrNone};
use style::computed_values::{overflow, position, box_sizing, display, float};
use std::sync::Arc;

/// Information specific to floated blocks.
#[derive(Clone, RustcEncodable)]
pub struct FloatedBlockInfo {
    /// The amount of inline size that is available for the float.
    pub containing_inline_size: Au,

    /// The float ceiling, relative to `BaseFlow::position::cur_b` (i.e. the top part of the border
    /// box).
    pub float_ceiling: Au,

    /// Index into the fragment list for inline floats
    pub index: Option<uint>,

    /// Left or right?
    pub float_kind: FloatKind,
}

impl FloatedBlockInfo {
    pub fn new(float_kind: FloatKind) -> FloatedBlockInfo {
        FloatedBlockInfo {
            containing_inline_size: Au(0),
            float_ceiling: Au(0),
            index: None,
            float_kind: float_kind,
        }
    }
}

/// The solutions for the block-size-and-margins constraint equation.
#[derive(Copy)]
struct BSizeConstraintSolution {
    block_start: Au,
    _block_end: Au,
    block_size: Au,
    margin_block_start: Au,
    margin_block_end: Au
}

impl BSizeConstraintSolution {
    fn new(block_start: Au, block_end: Au, block_size: Au, margin_block_start: Au, margin_block_end: Au)
           -> BSizeConstraintSolution {
        BSizeConstraintSolution {
            block_start: block_start,
            _block_end: block_end,
            block_size: block_size,
            margin_block_start: margin_block_start,
            margin_block_end: margin_block_end,
        }
    }

    /// Solve the vertical constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.6.4
    /// Constraint equation:
    /// block-start + block-end + block-size + margin-block-start + margin-block-end
    /// = absolute containing block block-size - (vertical padding and border)
    /// [aka available_block-size]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_nonreplaced(block_size: MaybeAuto,
                                                  block_start_margin: MaybeAuto,
                                                  block_end_margin: MaybeAuto,
                                                  block_start: MaybeAuto,
                                                  block_end: MaybeAuto,
                                                  content_block_size: Au,
                                                  available_block_size: Au,
                                                  static_b_offset: Au)
                                                  -> BSizeConstraintSolution {
        // Distance from the block-start edge of the Absolute Containing Block to the
        // block-start margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_block_start = static_b_offset;

        let (block_start, block_end, block_size, margin_block_start, margin_block_end) = match (block_start, block_end, block_size) {
            (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Auto) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let block_start = static_position_block_start;
                // Now it is the same situation as block-start Specified and block-end
                // and block-size Auto.

                let block_size = content_block_size;
                let sum = block_start + block_size + margin_block_start + margin_block_end;
                (block_start, available_block_size - sum, block_size, margin_block_start, margin_block_end)
            }
            (MaybeAuto::Specified(block_start), MaybeAuto::Specified(block_end), MaybeAuto::Specified(block_size)) => {
                match (block_start_margin, block_end_margin) {
                    (MaybeAuto::Auto, MaybeAuto::Auto) => {
                        let total_margin_val = available_block_size - block_start - block_end - block_size;
                        (block_start, block_end, block_size,
                         total_margin_val.scale_by(0.5),
                         total_margin_val.scale_by(0.5))
                    }
                    (MaybeAuto::Specified(margin_block_start), MaybeAuto::Auto) => {
                        let sum = block_start + block_end + block_size + margin_block_start;
                        (block_start, block_end, block_size, margin_block_start, available_block_size - sum)
                    }
                    (MaybeAuto::Auto, MaybeAuto::Specified(margin_block_end)) => {
                        let sum = block_start + block_end + block_size + margin_block_end;
                        (block_start, block_end, block_size, available_block_size - sum, margin_block_end)
                    }
                    (MaybeAuto::Specified(margin_block_start), MaybeAuto::Specified(margin_block_end)) => {
                        // Values are over-constrained. Ignore value for 'block-end'.
                        let sum = block_start + block_size + margin_block_start + margin_block_end;
                        (block_start, available_block_size - sum, block_size, margin_block_start, margin_block_end)
                    }
                }
            }

            // For the rest of the cases, auto values for margin are set to 0

            // If only one is Auto, solve for it
            (MaybeAuto::Auto, MaybeAuto::Specified(block_end), MaybeAuto::Specified(block_size)) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let sum = block_end + block_size + margin_block_start + margin_block_end;
                (available_block_size - sum, block_end, block_size, margin_block_start, margin_block_end)
            }
            (MaybeAuto::Specified(block_start), MaybeAuto::Auto, MaybeAuto::Specified(block_size)) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let sum = block_start + block_size + margin_block_start + margin_block_end;
                (block_start, available_block_size - sum, block_size, margin_block_start, margin_block_end)
            }
            (MaybeAuto::Specified(block_start), MaybeAuto::Specified(block_end), MaybeAuto::Auto) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let sum = block_start + block_end + margin_block_start + margin_block_end;
                (block_start, block_end, available_block_size - sum, margin_block_start, margin_block_end)
            }

            // If block-size is auto, then block-size is content block-size. Solve for the
            // non-auto value.
            (MaybeAuto::Specified(block_start), MaybeAuto::Auto, MaybeAuto::Auto) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let block_size = content_block_size;
                let sum = block_start + block_size + margin_block_start + margin_block_end;
                (block_start, available_block_size - sum, block_size, margin_block_start, margin_block_end)
            }
            (MaybeAuto::Auto, MaybeAuto::Specified(block_end), MaybeAuto::Auto) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let block_size = content_block_size;
                let sum = block_end + block_size + margin_block_start + margin_block_end;
                (available_block_size - sum, block_end, block_size, margin_block_start, margin_block_end)
            }

            (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Specified(block_size)) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let block_start = static_position_block_start;
                let sum = block_start + block_size + margin_block_start + margin_block_end;
                (block_start, available_block_size - sum, block_size, margin_block_start, margin_block_end)
            }
        };
        BSizeConstraintSolution::new(block_start, block_end, block_size, margin_block_start, margin_block_end)
    }

    /// Solve the vertical constraint equation for absolute replaced elements.
    ///
    /// Assumption: The used value for block-size has already been calculated.
    ///
    /// CSS Section 10.6.5
    /// Constraint equation:
    /// block-start + block-end + block-size + margin-block-start + margin-block-end
    /// = absolute containing block block-size - (vertical padding and border)
    /// [aka available_block-size]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_replaced(block_size: Au,
                                               block_start_margin: MaybeAuto,
                                               block_end_margin: MaybeAuto,
                                               block_start: MaybeAuto,
                                               block_end: MaybeAuto,
                                               _: Au,
                                               available_block_size: Au,
                                               static_b_offset: Au)
                                               -> BSizeConstraintSolution {
        // Distance from the block-start edge of the Absolute Containing Block to the
        // block-start margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_block_start = static_b_offset;

        let (block_start, block_end, block_size, margin_block_start, margin_block_end) = match (block_start, block_end) {
            (MaybeAuto::Auto, MaybeAuto::Auto) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let block_start = static_position_block_start;
                let sum = block_start + block_size + margin_block_start + margin_block_end;
                (block_start, available_block_size - sum, block_size, margin_block_start, margin_block_end)
            }
            (MaybeAuto::Specified(block_start), MaybeAuto::Specified(block_end)) => {
                match (block_start_margin, block_end_margin) {
                    (MaybeAuto::Auto, MaybeAuto::Auto) => {
                        let total_margin_val = available_block_size - block_start - block_end - block_size;
                        (block_start, block_end, block_size,
                         total_margin_val.scale_by(0.5),
                         total_margin_val.scale_by(0.5))
                    }
                    (MaybeAuto::Specified(margin_block_start), MaybeAuto::Auto) => {
                        let sum = block_start + block_end + block_size + margin_block_start;
                        (block_start, block_end, block_size, margin_block_start, available_block_size - sum)
                    }
                    (MaybeAuto::Auto, MaybeAuto::Specified(margin_block_end)) => {
                        let sum = block_start + block_end + block_size + margin_block_end;
                        (block_start, block_end, block_size, available_block_size - sum, margin_block_end)
                    }
                    (MaybeAuto::Specified(margin_block_start), MaybeAuto::Specified(margin_block_end)) => {
                        // Values are over-constrained. Ignore value for 'block-end'.
                        let sum = block_start + block_size + margin_block_start + margin_block_end;
                        (block_start, available_block_size - sum, block_size, margin_block_start, margin_block_end)
                    }
                }
            }

            // If only one is Auto, solve for it
            (MaybeAuto::Auto, MaybeAuto::Specified(block_end)) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let sum = block_end + block_size + margin_block_start + margin_block_end;
                (available_block_size - sum, block_end, block_size, margin_block_start, margin_block_end)
            }
            (MaybeAuto::Specified(block_start), MaybeAuto::Auto) => {
                let margin_block_start = block_start_margin.specified_or_zero();
                let margin_block_end = block_end_margin.specified_or_zero();
                let sum = block_start + block_size + margin_block_start + margin_block_end;
                (block_start, available_block_size - sum, block_size, margin_block_start, margin_block_end)
            }
        };
        BSizeConstraintSolution::new(block_start, block_end, block_size, margin_block_start, margin_block_end)
    }
}

/// Performs block-size calculations potentially multiple times, taking
/// (assuming an horizontal writing mode) `height`, `min-height`, and `max-height`
/// into account. After each call to `next()`, the caller must call `.try()` with the
/// current calculated value of `height`.
///
/// See CSS 2.1 § 10.7.
struct CandidateBSizeIterator {
    block_size: MaybeAuto,
    max_block_size: Option<Au>,
    min_block_size: Au,
    candidate_value: Au,
    status: CandidateBSizeIteratorStatus,
}

impl CandidateBSizeIterator {
    /// Creates a new candidate block-size iterator. `block_container_block-size` is `None` if the block-size
    /// of the block container has not been determined yet. It will always be `Some` in the case of
    /// absolutely-positioned containing blocks.
    pub fn new(fragment: &Fragment, block_container_block_size: Option<Au>)
               -> CandidateBSizeIterator {
        // Per CSS 2.1 § 10.7, (assuming an horizontal writing mode,)
        // percentages in `min-height` and `max-height` refer to the height of
        // the containing block.
        // If that is not determined yet by the time we need to resolve
        // `min-height` and `max-height`, percentage values are ignored.

        let block_size = match (fragment.style.content_block_size(), block_container_block_size) {
            (LengthOrPercentageOrAuto::Percentage(percent), Some(block_container_block_size)) => {
                MaybeAuto::Specified(block_container_block_size.scale_by(percent))
            }
            (LengthOrPercentageOrAuto::Percentage(_), None) | (LengthOrPercentageOrAuto::Auto, _) => MaybeAuto::Auto,
            (LengthOrPercentageOrAuto::Length(length), _) => MaybeAuto::Specified(length),
        };
        let max_block_size = match (fragment.style.max_block_size(), block_container_block_size) {
            (LengthOrPercentageOrNone::Percentage(percent), Some(block_container_block_size)) => {
                Some(block_container_block_size.scale_by(percent))
            }
            (LengthOrPercentageOrNone::Percentage(_), None) |
            (LengthOrPercentageOrNone::None, _) => None,
            (LengthOrPercentageOrNone::Length(length), _) => Some(length),
        };
        let min_block_size = match (fragment.style.min_block_size(), block_container_block_size) {
            (LengthOrPercentage::Percentage(percent), Some(block_container_block_size)) => {
                block_container_block_size.scale_by(percent)
            }
            (LengthOrPercentage::Percentage(_), None) => Au(0),
            (LengthOrPercentage::Length(length), _) => length,
        };

        // If the style includes `box-sizing: border-box`, subtract the border and padding.
        let adjustment_for_box_sizing = match fragment.style.get_box().box_sizing {
            box_sizing::T::border_box => fragment.border_padding.block_start_end(),
            box_sizing::T::content_box => Au(0),
        };

        return CandidateBSizeIterator {
            block_size: block_size.map(|size| adjust(size, adjustment_for_box_sizing)),
            max_block_size: max_block_size.map(|size| adjust(size, adjustment_for_box_sizing)),
            min_block_size: adjust(min_block_size, adjustment_for_box_sizing),
            candidate_value: Au(0),
            status: CandidateBSizeIteratorStatus::Initial,
        };

        fn adjust(size: Au, delta: Au) -> Au {
            max(size - delta, Au(0))
        }
    }
}

impl Iterator for CandidateBSizeIterator {
    type Item = MaybeAuto;
    fn next(&mut self) -> Option<MaybeAuto> {
        self.status = match self.status {
            CandidateBSizeIteratorStatus::Initial => CandidateBSizeIteratorStatus::Trying,
            CandidateBSizeIteratorStatus::Trying => {
                match self.max_block_size {
                    Some(max_block_size) if self.candidate_value > max_block_size => {
                        CandidateBSizeIteratorStatus::TryingMax
                    }
                    _ if self.candidate_value < self.min_block_size => CandidateBSizeIteratorStatus::TryingMin,
                    _ => CandidateBSizeIteratorStatus::Found,
                }
            }
            CandidateBSizeIteratorStatus::TryingMax => {
                if self.candidate_value < self.min_block_size {
                    CandidateBSizeIteratorStatus::TryingMin
                } else {
                    CandidateBSizeIteratorStatus::Found
                }
            }
            CandidateBSizeIteratorStatus::TryingMin | CandidateBSizeIteratorStatus::Found => {
                CandidateBSizeIteratorStatus::Found
            }
        };

        match self.status {
            CandidateBSizeIteratorStatus::Trying => Some(self.block_size),
            CandidateBSizeIteratorStatus::TryingMax => {
                Some(MaybeAuto::Specified(self.max_block_size.unwrap()))
            }
            CandidateBSizeIteratorStatus::TryingMin => {
                Some(MaybeAuto::Specified(self.min_block_size))
            }
            CandidateBSizeIteratorStatus::Found => None,
            CandidateBSizeIteratorStatus::Initial => panic!(),
        }
    }
}

enum CandidateBSizeIteratorStatus {
    Initial,
    Trying,
    TryingMax,
    TryingMin,
    Found,
}

// A helper function used in block-size calculation.
fn translate_including_floats(cur_b: &mut Au, delta: Au, floats: &mut Floats) {
    *cur_b = *cur_b + delta;
    let writing_mode = floats.writing_mode;
    floats.translate(LogicalSize::new(writing_mode, Au(0), -delta));
}

/// The real assign-block-sizes traversal for flows with position 'absolute'.
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
struct AbsoluteAssignBSizesTraversal<'a>(&'a LayoutContext<'a>);

impl<'a> PreorderFlowTraversal for AbsoluteAssignBSizesTraversal<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        let block_flow = flow.as_block();

        // The root of the absolute flow tree is definitely not absolutely
        // positioned. Nothing to process here.
        if block_flow.is_root_of_absolute_flow_tree() {
            return;
        }

        assert!(block_flow.base.flags.contains(IS_ABSOLUTELY_POSITIONED));
        if !block_flow.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
            return
        }

        let AbsoluteAssignBSizesTraversal(ref ctx) = *self;
        block_flow.calculate_absolute_block_size_and_margins(*ctx);
    }
}

/// The store-overflow traversal particular to absolute flows.
///
/// Propagate overflow up the Absolute flow tree and update overflow up to and
/// not including the root of the Absolute flow tree.
/// After that, it is up to the normal store-overflow traversal to propagate
/// it further up.
struct AbsoluteStoreOverflowTraversal<'a>{
    layout_context: &'a LayoutContext<'a>,
}

impl<'a> PostorderFlowTraversal for AbsoluteStoreOverflowTraversal<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        // This will be taken care of by the normal store-overflow traversal.
        if flow.is_root_of_absolute_flow_tree() {
            return;
        }

        flow.store_overflow(self.layout_context);
    }
}

pub enum BlockType {
    Replaced,
    NonReplaced,
    AbsoluteReplaced,
    AbsoluteNonReplaced,
    FloatReplaced,
    FloatNonReplaced,
}

#[derive(Clone, PartialEq)]
pub enum MarginsMayCollapseFlag {
    MarginsMayCollapse,
    MarginsMayNotCollapse,
}

#[derive(PartialEq)]
enum FormattingContextType {
    None,
    Block,
    Other,
}

// Propagates the `layers_needed_for_descendants` flag appropriately from a child. This is called
// as part of block-size assignment.
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
        if kid_base.flags.contains(NEEDS_LAYER) {
            *layers_needed_for_descendants = true
        }
    } else {
        let kid_base = flow::mut_base(kid);
        if kid_base.flags.contains(LAYERS_NEEDED_FOR_DESCENDANTS) {
            *layers_needed_for_descendants = true
        }
    }
}

// A block formatting context.
#[derive(RustcEncodable)]
pub struct BlockFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// The associated fragment.
    pub fragment: Fragment,

    /// Static y offset of an absolute flow from its CB.
    pub static_b_offset: Au,

    /// The sum of the inline-sizes of all logically left floats that precede this block. This is
    /// used to speculatively lay out block formatting contexts.
    inline_size_of_preceding_left_floats: Au,

    /// The sum of the inline-sizes of all logically right floats that precede this block. This is
    /// used to speculatively lay out block formatting contexts.
    inline_size_of_preceding_right_floats: Au,

    /// The hypothetical position, used for absolutely-positioned flows.
    hypothetical_position: LogicalPoint<Au>,

    /// Additional floating flow members.
    pub float: Option<Box<FloatedBlockInfo>>,

    /// Various flags.
    pub flags: BlockFlowFlags,
}

bitflags! {
    flags BlockFlowFlags: u8 {
        #[doc="If this is set, then this block flow is the root flow."]
        const IS_ROOT = 0x01,
    }
}

impl Encodable for BlockFlowFlags {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        self.bits().encode(e)
    }
}

impl BlockFlow {
    pub fn from_node(constructor: &mut FlowConstructor, node: &ThreadSafeLayoutNode) -> BlockFlow {
        let writing_mode = node.style().writing_mode;
        BlockFlow {
            base: BaseFlow::new(Some((*node).clone()), writing_mode, ForceNonfloatedFlag::ForceNonfloated),
            fragment: Fragment::new(constructor, node),
            static_b_offset: Au::new(0),
            inline_size_of_preceding_left_floats: Au(0),
            inline_size_of_preceding_right_floats: Au(0),
            hypothetical_position: LogicalPoint::new(writing_mode, Au(0), Au(0)),
            float: None,
            flags: BlockFlowFlags::empty(),
        }
    }

    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode, fragment: Fragment) -> BlockFlow {
        let writing_mode = node.style().writing_mode;
        BlockFlow {
            base: BaseFlow::new(Some((*node).clone()), writing_mode, ForceNonfloatedFlag::ForceNonfloated),
            fragment: fragment,
            static_b_offset: Au::new(0),
            inline_size_of_preceding_left_floats: Au(0),
            inline_size_of_preceding_right_floats: Au(0),
            hypothetical_position: LogicalPoint::new(writing_mode, Au(0), Au(0)),
            float: None,
            flags: BlockFlowFlags::empty(),
        }
    }

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> BlockFlow {
        let writing_mode = node.style().writing_mode;
        BlockFlow {
            base: BaseFlow::new(Some((*node).clone()), writing_mode, ForceNonfloatedFlag::FloatIfNecessary),
            fragment: Fragment::new(constructor, node),
            static_b_offset: Au::new(0),
            inline_size_of_preceding_left_floats: Au(0),
            inline_size_of_preceding_right_floats: Au(0),
            hypothetical_position: LogicalPoint::new(writing_mode, Au(0), Au(0)),
            float: Some(box FloatedBlockInfo::new(float_kind)),
            flags: BlockFlowFlags::empty(),
        }
    }

    pub fn float_from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                        fragment: Fragment,
                                        float_kind: FloatKind)
                                        -> BlockFlow {
        let writing_mode = node.style().writing_mode;
        BlockFlow {
            base: BaseFlow::new(Some((*node).clone()), writing_mode, ForceNonfloatedFlag::FloatIfNecessary),
            fragment: fragment,
            static_b_offset: Au::new(0),
            inline_size_of_preceding_left_floats: Au(0),
            inline_size_of_preceding_right_floats: Au(0),
            hypothetical_position: LogicalPoint::new(writing_mode, Au(0), Au(0)),
            float: Some(box FloatedBlockInfo::new(float_kind)),
            flags: BlockFlowFlags::empty(),
        }
    }

    /// Return the type of this block.
    ///
    /// This determines the algorithm used to calculate inline-size, block-size, and the
    /// relevant margins for this Block.
    pub fn block_type(&self) -> BlockType {
        if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            if self.is_replaced_content() {
                BlockType::AbsoluteReplaced
            } else {
                BlockType::AbsoluteNonReplaced
            }
        } else if self.base.flags.is_float() {
            if self.is_replaced_content() {
                BlockType::FloatReplaced
            } else {
                BlockType::FloatNonReplaced
            }
        } else {
            if self.is_replaced_content() {
                BlockType::Replaced
            } else {
                BlockType::NonReplaced
            }
        }
    }

    /// Compute the actual inline size and position for this block.
    pub fn compute_used_inline_size(&mut self,
                                    ctx: &LayoutContext,
                                    containing_block_inline_size: Au) {
        let block_type = self.block_type();
        match block_type {
            BlockType::AbsoluteReplaced => {
                let inline_size_computer = AbsoluteReplaced;
                inline_size_computer.compute_used_inline_size(self, ctx, containing_block_inline_size);
            }
            BlockType::AbsoluteNonReplaced => {
                let inline_size_computer = AbsoluteNonReplaced;
                inline_size_computer.compute_used_inline_size(self, ctx, containing_block_inline_size);
            }
            BlockType::FloatReplaced => {
                let inline_size_computer = FloatReplaced;
                inline_size_computer.compute_used_inline_size(self, ctx, containing_block_inline_size);
            }
            BlockType::FloatNonReplaced => {
                let inline_size_computer = FloatNonReplaced;
                inline_size_computer.compute_used_inline_size(self, ctx, containing_block_inline_size);
            }
            BlockType::Replaced => {
                let inline_size_computer = BlockReplaced;
                inline_size_computer.compute_used_inline_size(self, ctx, containing_block_inline_size);
            }
            BlockType::NonReplaced => {
                let inline_size_computer = BlockNonReplaced;
                inline_size_computer.compute_used_inline_size(self, ctx, containing_block_inline_size);
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
        debug_assert!(self.base.flags.contains(IS_ABSOLUTELY_POSITIONED));
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
    /// which was bubbled up during normal assign-block-size).
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_preorder_absolute_flows<T:PreorderFlowTraversal>(&mut self,
                                                                 traversal: &mut T) {
        let flow = self as &mut Flow;

        traversal.process(flow);

        let cb_block_start_edge_offset = flow.generated_containing_block_rect().start.b;
        let descendant_offset_iter = mut_base(flow).abs_descendants.iter_with_offset();
        // Pass in the respective static y offset for each descendant.
        for (ref mut descendant_link, ref y_offset) in descendant_offset_iter {
            let block = descendant_link.as_block();
            // The stored y_offset is wrt to the flow box.
            // Translate it to the CB (which is the padding box).
            block.static_b_offset = **y_offset - cb_block_start_edge_offset;
            block.traverse_preorder_absolute_flows(traversal);
        }
    }

    /// Traverse the Absolute flow tree in postorder.
    ///
    /// Return true if the traversal is to continue or false to stop.
    fn traverse_postorder_absolute_flows<T:PostorderFlowTraversal>(&mut self,
                                                                   traversal: &mut T) {
        let flow = self as &mut Flow;

        for descendant_link in mut_base(flow).abs_descendants.iter() {
            let block = descendant_link.as_block();
            block.traverse_postorder_absolute_flows(traversal);
        }

        traversal.process(flow)
    }

    /// Return true if this has a replaced fragment.
    ///
    /// The only two types of replaced fragments currently are text fragments
    /// and image fragments.
    fn is_replaced_content(&self) -> bool {
        match self.fragment.specific {
            SpecificFragmentInfo::ScannedText(_) | SpecificFragmentInfo::Image(_) | SpecificFragmentInfo::InlineBlock(_) => true,
            _ => false,
        }
    }

    /// Return shrink-to-fit inline-size.
    ///
    /// This is where we use the preferred inline-sizes and minimum inline-sizes
    /// calculated in the bubble-inline-sizes traversal.
    pub fn get_shrink_to_fit_inline_size(&self, available_inline_size: Au) -> Au {
        let content_intrinsic_inline_sizes = self.content_intrinsic_inline_sizes();
        min(content_intrinsic_inline_sizes.preferred_inline_size,
            max(content_intrinsic_inline_sizes.minimum_inline_size, available_inline_size))
    }

    /// If this is the root flow, shifts all kids down and adjusts our size to account for
    /// root flow margins, which should never be collapsed according to CSS § 8.3.1.
    ///
    /// TODO(#2017, pcwalton): This is somewhat inefficient (traverses kids twice); can we do
    /// better?
    fn adjust_fragments_for_collapsed_margins_if_root(&mut self) {
        if !self.is_root() {
            return
        }

        let (block_start_margin_value, block_end_margin_value) = match self.base.collapsible_margins {
            CollapsibleMargins::CollapseThrough(_) => panic!("Margins unexpectedly collapsed through root flow."),
            CollapsibleMargins::Collapse(block_start_margin, block_end_margin) => {
                (block_start_margin.collapse(), block_end_margin.collapse())
            }
            CollapsibleMargins::None(block_start, block_end) => (block_start, block_end),
        };

        // Shift all kids down (or up, if margins are negative) if necessary.
        if block_start_margin_value != Au(0) {
            for kid in self.base.child_iter() {
                let kid_base = flow::mut_base(kid);
                kid_base.position.start.b = kid_base.position.start.b + block_start_margin_value
            }
        }

        self.base.position.size.block = self.base.position.size.block + block_start_margin_value +
            block_end_margin_value;
        self.fragment.border_box.size.block = self.fragment.border_box.size.block + block_start_margin_value +
            block_end_margin_value;
    }

    /// Assign block-size for current flow.
    ///
    /// * Collapse margins for flow's children and set in-flow child flows' block offsets now that
    ///   we know their block-sizes.
    /// * Calculate and set the block-size of the current flow.
    /// * Calculate block-size, vertical margins, and block offset for the flow's box using CSS §
    ///   10.6.7.
    ///
    /// For absolute flows, we store the calculated content block-size for the flow. We defer the
    /// calculation of the other values until a later traversal.
    ///
    /// `inline(always)` because this is only ever called by in-order or non-in-order top-level
    /// methods.
    #[inline(always)]
    pub fn assign_block_size_block_base<'a>(&mut self,
                                            layout_context: &'a LayoutContext<'a>,
                                            margins_may_collapse: MarginsMayCollapseFlag) {
        let _scope = layout_debug_scope!("assign_block_size_block_base {:x}", self.base.debug_id());

        if self.base.restyle_damage.contains(REFLOW) {
            // Our current border-box position.
            let mut cur_b = Au(0);

            // Absolute positioning establishes a block formatting context. Don't propagate floats
            // in or out. (But do propagate them between kids.)
            if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) ||
                    margins_may_collapse != MarginsMayCollapseFlag::MarginsMayCollapse {
                self.base.floats = Floats::new(self.fragment.style.writing_mode);
            }

            let mut margin_collapse_info = MarginCollapseInfo::new();
            self.base.floats.translate(LogicalSize::new(
                self.fragment.style.writing_mode, -self.fragment.inline_start_offset(), Au(0)));

            // The sum of our block-start border and block-start padding.
            let block_start_offset = self.fragment.border_padding.block_start;
            translate_including_floats(&mut cur_b, block_start_offset, &mut self.base.floats);

            let can_collapse_block_start_margin_with_kids =
                margins_may_collapse == MarginsMayCollapseFlag::MarginsMayCollapse &&
                !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) &&
                self.fragment.border_padding.block_start == Au(0);
            margin_collapse_info.initialize_block_start_margin(
                &self.fragment,
                can_collapse_block_start_margin_with_kids);

            // At this point, `cur_b` is at the content edge of our box. Now iterate over children.
            let mut floats = self.base.floats.clone();
            let mut layers_needed_for_descendants = false;
            for kid in self.base.child_iter() {
                if flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED) {
                    // Assume that the *hypothetical box* for an absolute flow starts immediately
                    // after the block-end border edge of the previous flow.
                    kid.as_block().hypothetical_position.b = cur_b;
                    kid.place_float_if_applicable(layout_context);
                    if !flow::base(kid).flags.is_float() {
                        kid.assign_block_size_for_inorder_child_if_necessary(layout_context);
                    }
                    propagate_layer_flag_from_child(&mut layers_needed_for_descendants, kid);

                    // Skip the collapsing and float processing for absolute flow kids and continue
                    // with the next flow.
                    continue
                }

                // Assign block-size now for the child if it was impacted by floats and we couldn't
                // before.
                flow::mut_base(kid).floats = floats.clone();
                if flow::base(kid).flags.is_float() {
                    flow::mut_base(kid).position.start.b = cur_b;
                    {
                        let kid_block = kid.as_block();
                        kid_block.float.as_mut().unwrap().float_ceiling =
                            margin_collapse_info.current_float_ceiling();
                    }
                    propagate_layer_flag_from_child(&mut layers_needed_for_descendants, kid);
                    kid.place_float_if_applicable(layout_context);

                    let kid_base = flow::mut_base(kid);
                    floats = kid_base.floats.clone();
                    continue
                }

                // If we have clearance, assume there are no floats in.
                //
                // FIXME(#2008, pcwalton): This could be wrong if we have `clear: left` or `clear:
                // right` and there are still floats to impact, of course. But this gets
                // complicated with margin collapse. Possibly the right thing to do is to lay out
                // the block again in this rare case. (Note that WebKit can lay blocks out twice;
                // this may be related, although I haven't looked into it closely.)
                if flow::base(kid).flags.clears_floats() {
                    flow::mut_base(kid).floats = Floats::new(self.fragment.style.writing_mode)
                }

                // Lay the child out if this was an in-order traversal.
                let need_to_process_child_floats =
                    kid.assign_block_size_for_inorder_child_if_necessary(layout_context);

                // Mark flows for layerization if necessary to handle painting order correctly.
                propagate_layer_flag_from_child(&mut layers_needed_for_descendants, kid);

                // Handle any (possibly collapsed) top margin.
                let delta = margin_collapse_info.advance_block_start_margin(
                    &flow::base(kid).collapsible_margins);
                translate_including_floats(&mut cur_b, delta, &mut floats);

                // Clear past the floats that came in, if necessary.
                let clearance = match (flow::base(kid).flags.contains(CLEARS_LEFT),
                                       flow::base(kid).flags.contains(CLEARS_RIGHT)) {
                    (false, false) => Au(0),
                    (true, false) => floats.clearance(ClearType::Left),
                    (false, true) => floats.clearance(ClearType::Right),
                    (true, true) => floats.clearance(ClearType::Both),
                };
                translate_including_floats(&mut cur_b, clearance, &mut floats);

                // At this point, `cur_b` is at the border edge of the child.
                flow::mut_base(kid).position.start.b = cur_b;

                // Now pull out the child's outgoing floats. We didn't do this immediately after
                // the `assign_block_size_for_inorder_child_if_necessary` call because clearance on
                // a block operates on the floats that come *in*, not the floats that go *out*.
                if need_to_process_child_floats {
                    floats = flow::mut_base(kid).floats.clone()
                }

                // Move past the child's border box. Do not use the `translate_including_floats`
                // function here because the child has already translated floats past its border
                // box.
                let kid_base = flow::mut_base(kid);
                cur_b = cur_b + kid_base.position.size.block;

                // Handle any (possibly collapsed) block-end margin.
                let delta =
                    margin_collapse_info.advance_block_end_margin(&kid_base.collapsible_margins);
                translate_including_floats(&mut cur_b, delta, &mut floats);
            }

            // Mark ourselves for layerization if that will be necessary to paint in the proper
            // order (CSS 2.1, Appendix E).
            self.base.flags.set(LAYERS_NEEDED_FOR_DESCENDANTS, layers_needed_for_descendants);

            // Collect various offsets needed by absolutely positioned descendants.
            (&mut *self as &mut Flow).collect_static_block_offsets_from_children();

            // Add in our block-end margin and compute our collapsible margins.
            let can_collapse_block_end_margin_with_kids =
                margins_may_collapse == MarginsMayCollapseFlag::MarginsMayCollapse &&
                !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) &&
                self.fragment.border_padding.block_end == Au(0);
            let (collapsible_margins, delta) =
                margin_collapse_info.finish_and_compute_collapsible_margins(
                &self.fragment,
                can_collapse_block_end_margin_with_kids);
            self.base.collapsible_margins = collapsible_margins;
            translate_including_floats(&mut cur_b, delta, &mut floats);

            // FIXME(#2003, pcwalton): The max is taken here so that you can scroll the page, but
            // this is not correct behavior according to CSS 2.1 § 10.5. Instead I think we should
            // treat the root element as having `overflow: scroll` and use the layers-based
            // scrolling infrastructure to make it scrollable.
            let mut block_size = cur_b - block_start_offset;
            let is_root = self.is_root();
            if is_root {
                let screen_size = LogicalSize::from_physical(self.fragment.style.writing_mode,
                                                             layout_context.shared.screen_size);
                block_size = Au::max(screen_size.block, block_size)
            }

            if is_root || self.formatting_context_type() != FormattingContextType::None ||
                    self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                // The content block-size includes all the floats per CSS 2.1 § 10.6.7. The easiest
                // way to handle this is to just treat it as clearance.
                block_size = block_size + floats.clearance(ClearType::Both);
            }

            if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                // Fixed position layers get layers.
                if self.is_fixed() {
                    self.base.flags.insert(NEEDS_LAYER);
                }

                // Store the content block-size for use in calculating the absolute flow's
                // dimensions later.
                //
                // FIXME(pcwalton): This looks not idempotent. Is it?
                self.fragment.border_box.size.block = block_size;
                return
            }

            // Compute any explicitly-specified block size.
            // Can't use `for` because we assign to `candidate_block_size_iterator.candidate_value`.
            let mut candidate_block_size_iterator = CandidateBSizeIterator::new(
                &self.fragment,
                self.base.block_container_explicit_block_size);
            loop {
                match candidate_block_size_iterator.next() {
                    Some(candidate_block_size) => {
                        candidate_block_size_iterator.candidate_value =
                            match candidate_block_size {
                                MaybeAuto::Auto => block_size,
                                MaybeAuto::Specified(value) => value
                            }
                    }
                    None => break,
                }
            }

            // Adjust `cur_b` as necessary to account for the explicitly-specified block-size.
            block_size = candidate_block_size_iterator.candidate_value;
            let delta = block_size - (cur_b - block_start_offset);
            translate_including_floats(&mut cur_b, delta, &mut floats);

            // Take border and padding into account.
            let block_end_offset = self.fragment.border_padding.block_end;
            translate_including_floats(&mut cur_b, block_end_offset, &mut floats);

            // Now that `cur_b` is at the block-end of the border box, compute the final border box
            // position.
            self.fragment.border_box.size.block = cur_b;
            self.fragment.border_box.start.b = Au(0);

            if !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                self.base.position.size.block = cur_b;
            }

            // Store the current set of floats in the flow so that flows that come later in the
            // document can access them.
            self.base.floats = floats.clone();
            self.adjust_fragments_for_collapsed_margins_if_root();
        } else {
            // We don't need to reflow, but we still need to perform in-order traversals if
            // necessary.
            for kid in self.base.child_iter() {
                kid.assign_block_size_for_inorder_child_if_necessary(layout_context);
            }
        }

        if self.is_root_of_absolute_flow_tree() {
            // Assign block-sizes for all flows in this absolute flow tree.
            // This is preorder because the block-size of an absolute flow may depend on
            // the block-size of its containing block, which may also be an absolute flow.
            self.traverse_preorder_absolute_flows(&mut AbsoluteAssignBSizesTraversal(
                    layout_context));
            // Store overflow for all absolute descendants.
            self.traverse_postorder_absolute_flows(&mut AbsoluteStoreOverflowTraversal {
                layout_context: layout_context,
            });
        }

        // Don't remove the dirty bits yet if we're absolutely-positioned, since our final size
        // has not been calculated yet. (See `calculate_absolute_block_size_and_margins` for that.)
        // Also don't remove the dirty bits if we're a block formatting context since our inline
        // size has not yet been computed. (See `assign_inline_position_for_formatting_context()`.)
        if (self.base.flags.is_float() ||
                self.formatting_context_type() == FormattingContextType::None) &&
                !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
        }
    }

    /// Add placement information about current float flow for use by the parent.
    ///
    /// Also, use information given by parent about other floats to find out our relative position.
    ///
    /// This does not give any information about any float descendants because they do not affect
    /// elements outside of the subtree rooted at this float.
    ///
    /// This function is called on a kid flow by a parent. Therefore, `assign_block_size_float` was
    /// already called on this kid flow by the traversal function. So, the values used are
    /// well-defined.
    pub fn place_float(&mut self) {
        let block_size = self.fragment.border_box.size.block;
        let clearance = match self.fragment.clear() {
            None => Au(0),
            Some(clear) => self.base.floats.clearance(clear),
        };

        let float_info: FloatedBlockInfo = (**self.float.as_ref().unwrap()).clone();
        let info = PlacementInfo {
            size: LogicalSize::new(
                self.fragment.style.writing_mode,
                self.base.position.size.inline,
                block_size + self.fragment.margin.block_start_end()),
            ceiling: clearance + float_info.float_ceiling,
            max_inline_size: float_info.containing_inline_size,
            kind: float_info.float_kind,
        };

        // Place the float and return the `Floats` back to the parent flow.
        // After, grab the position and use that to set our position.
        self.base.floats.add_float(&info);

        // Move in from the margin edge, as per CSS 2.1 § 9.5, floats may not overlap anything on
        // their margin edges.
        let float_offset = self.base.floats.last_float_pos().unwrap();
        let writing_mode = self.base.floats.writing_mode;
        let margin_offset = LogicalPoint::new(writing_mode,
                                              Au(0),
                                              self.fragment.margin.block_start);

        if !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            let mut origin = LogicalPoint::new(self.base.writing_mode,
                                               self.hypothetical_position.i,
                                               self.base.position.start.b);
            origin = origin.add_point(&float_offset).add_point(&margin_offset);
            self.base.position = LogicalRect::from_point_size(self.base.writing_mode,
                                                              origin,
                                                              self.base.position.size);
        }
    }


    /// Calculate and set the block-size, offsets, etc. for absolutely positioned flow.
    ///
    /// The layout for its in-flow children has been done during normal layout.
    /// This is just the calculation of:
    /// + block-size for the flow
    /// + position in the block direction of the flow with respect to its Containing Block.
    /// + block-size, vertical margins, and y-coordinate for the flow's box.
    fn calculate_absolute_block_size_and_margins(&mut self, ctx: &LayoutContext) {
        let containing_block_block_size = self.containing_block_size(ctx.shared.screen_size).block;
        let static_b_offset = self.static_b_offset;

        // This is the stored content block-size value from assign-block-size
        let content_block_size = self.fragment.content_box().size.block;

        let mut solution = None;
        {
            // Non-auto margin-block-start and margin-block-end values have already been
            // calculated during assign-inline-size.
            let margin = self.fragment.style().logical_margin();
            let margin_block_start = match margin.block_start {
                LengthOrPercentageOrAuto::Auto => MaybeAuto::Auto,
                _ => MaybeAuto::Specified(self.fragment.margin.block_start)
            };
            let margin_block_end = match margin.block_end {
                LengthOrPercentageOrAuto::Auto => MaybeAuto::Auto,
                _ => MaybeAuto::Specified(self.fragment.margin.block_end)
            };

            let block_start;
            let block_end;
            {
                let position = self.fragment.style().logical_position();
                block_start = MaybeAuto::from_style(position.block_start,
                                                    containing_block_block_size);
                block_end = MaybeAuto::from_style(position.block_end, containing_block_block_size);
            }

            let available_block_size = containing_block_block_size -
                self.fragment.border_padding.block_start_end();
            if self.is_replaced_content() {
                // Calculate used value of block-size just like we do for inline replaced elements.
                // TODO: Pass in the containing block block-size when Fragment's
                // assign-block-size can handle it correctly.
                self.fragment.assign_replaced_block_size_if_necessary(containing_block_block_size);
                // TODO: Right now, this content block-size value includes the
                // margin because of erroneous block-size calculation in fragment.
                // Check this when that has been fixed.
                let block_size_used_val = self.fragment.border_box.size.block;
                solution = Some(BSizeConstraintSolution::solve_vertical_constraints_abs_replaced(
                        block_size_used_val,
                        margin_block_start,
                        margin_block_end,
                        block_start,
                        block_end,
                        content_block_size,
                        available_block_size,
                        static_b_offset));
            } else {
                let mut candidate_block_size_iterator =
                    CandidateBSizeIterator::new(&self.fragment, Some(containing_block_block_size));

                // Can't use `for` because we assign to
                // `candidate_block_size_iterator.candidate_value`.
                loop {
                    match candidate_block_size_iterator.next() {
                        Some(block_size_used_val) => {
                            solution =
                                Some(BSizeConstraintSolution::
                                     solve_vertical_constraints_abs_nonreplaced(
                                    block_size_used_val,
                                    margin_block_start,
                                    margin_block_end,
                                    block_start,
                                    block_end,
                                    content_block_size,
                                    available_block_size,
                                    static_b_offset));

                            candidate_block_size_iterator.candidate_value
                                = solution.unwrap().block_size;
                        }
                        None => break,
                    }
                }
            }
        }

        let solution = solution.unwrap();
        self.fragment.margin.block_start = solution.margin_block_start;
        self.fragment.margin.block_end = solution.margin_block_end;
        self.fragment.border_box.start.b = Au(0);

        self.base.position.start.b = solution.block_start + self.fragment.margin.block_start;

        let block_size = solution.block_size + self.fragment.border_padding.block_start_end();
        self.fragment.border_box.size.block = block_size;
        self.base.position.size.block = block_size;

        self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
    }

    // Our inline-size was set to the inline-size of the containing block by the flow's parent.
    // This computes the real value, and is run in the `AssignISizes` traversal.
    pub fn propagate_and_compute_used_inline_size(&mut self, layout_context: &LayoutContext) {
        let containing_block_inline_size = self.base.block_container_inline_size;
        self.compute_used_inline_size(layout_context, containing_block_inline_size);
        if self.base.flags.is_float() {
            self.float.as_mut().unwrap().containing_inline_size = containing_block_inline_size;
        }
    }

    /// Return the block-start outer edge of the hypothetical box for an absolute flow.
    ///
    /// This is wrt its parent flow box.
    ///
    /// During normal layout assign-block-size, the absolute flow's position is
    /// roughly set to its static position (the position it would have had in
    /// the normal flow).
    pub fn get_hypothetical_block_start_edge(&self) -> Au {
        self.hypothetical_position.b
    }

    /// Assigns the computed inline-start content edge and inline-size to all the children of this
    /// block flow. Also computes whether each child will be impacted by floats.
    ///
    /// `#[inline(always)]` because this is called only from block or table inline-size assignment
    /// and the code for block layout is significantly simpler.
    #[inline(always)]
    pub fn propagate_assigned_inline_size_to_children(
            &mut self,
            layout_context: &LayoutContext,
            inline_start_content_edge: Au,
            content_inline_size: Au,
            optional_column_computed_inline_sizes: Option<&[ColumnComputedInlineSize]>) {
        // Keep track of whether floats could impact each child.
        let mut inline_start_floats_impact_child =
            self.base.flags.contains(IMPACTED_BY_LEFT_FLOATS);
        let mut inline_end_floats_impact_child =
            self.base.flags.contains(IMPACTED_BY_RIGHT_FLOATS);

        let absolute_static_i_offset = if self.is_positioned() {
            // This flow is the containing block. The static inline offset will be the inline-start
            // padding edge.
            self.fragment.border_padding.inline_start
                - self.fragment.style().logical_border_width().inline_start
        } else {
            // For kids, the inline-start margin edge will be at our inline-start content edge. The
            // current static offset is at our inline-start margin edge. So move in to the
            // inline-start content edge.
            self.base.absolute_static_i_offset + inline_start_content_edge
        };

        let fixed_static_i_offset = self.base.fixed_static_i_offset + inline_start_content_edge;
        let flags = self.base.flags.clone();

        // This value is used only for table cells.
        let mut inline_start_margin_edge = inline_start_content_edge;

        // Remember the inline-sizes of the last left and right floats, if there were any. These
        // are used for estimating the inline-sizes of block formatting contexts. (We estimate that
        // the inline-size of any block formatting context that we see will be based on the
        // inline-size of the containing block as well as the last float seen before it in each
        // direction.)
        let mut inline_size_of_preceding_left_floats = Au(0);
        let mut inline_size_of_preceding_right_floats = Au(0);
        if self.formatting_context_type() == FormattingContextType::None {
            inline_size_of_preceding_left_floats = self.inline_size_of_preceding_left_floats;
            inline_size_of_preceding_right_floats = self.inline_size_of_preceding_right_floats;
        }

        // Calculate non-auto block size to pass to children.
        let content_block_size = self.fragment.style().content_block_size();
        let explicit_content_size = match (content_block_size,
                                           self.base.block_container_explicit_block_size) {
            (LengthOrPercentageOrAuto::Percentage(percent), Some(container_size)) => {
                Some(container_size.scale_by(percent))
            }
            (LengthOrPercentageOrAuto::Percentage(_), None) | (LengthOrPercentageOrAuto::Auto, _) => None,
            (LengthOrPercentageOrAuto::Length(length), _) => Some(length),
        };

        // Calculate containing block inline size.
        let containing_block_size = if flags.contains(IS_ABSOLUTELY_POSITIONED) {
            self.containing_block_size(layout_context.shared.screen_size).inline
        } else {
            content_inline_size
        };

        for (i, kid) in self.base.child_iter().enumerate() {
            {
                let kid_base = flow::mut_base(kid);
                kid_base.block_container_explicit_block_size = explicit_content_size;
                kid_base.absolute_static_i_offset = absolute_static_i_offset;
                kid_base.fixed_static_i_offset = fixed_static_i_offset;
            }

            match flow::base(kid).flags.float_kind() {
                float::T::none => {}
                float::T::left => {
                    inline_size_of_preceding_left_floats = inline_size_of_preceding_left_floats +
                        flow::base(kid).intrinsic_inline_sizes.preferred_inline_size;
                }
                float::T::right => {
                    inline_size_of_preceding_right_floats = inline_size_of_preceding_right_floats +
                        flow::base(kid).intrinsic_inline_sizes.preferred_inline_size;
                }
            }

            // The inline-start margin edge of the child flow is at our inline-start content edge,
            // and its inline-size is our content inline-size.
            {
                let kid_base = flow::mut_base(kid);
                if !kid_base.flags.contains(IS_ABSOLUTELY_POSITIONED) &&
                        !kid_base.flags.is_float() {
                    kid_base.position.start.i = inline_start_content_edge
                }
                kid_base.block_container_inline_size = content_inline_size;
            }
            if kid.is_block_like() {
                kid.as_block().hypothetical_position.i = inline_start_content_edge
            }

            // Determine float impaction.
            if flow::base(kid).flags.contains(CLEARS_LEFT) {
                inline_start_floats_impact_child = false;
                inline_size_of_preceding_left_floats = Au(0);
            }
            if flow::base(kid).flags.contains(CLEARS_RIGHT) {
                inline_end_floats_impact_child = false;
                inline_size_of_preceding_right_floats = Au(0);
            }

            {
                let kid_base = flow::mut_base(kid);
                inline_start_floats_impact_child = inline_start_floats_impact_child ||
                    kid_base.flags.contains(HAS_LEFT_FLOATED_DESCENDANTS);
                inline_end_floats_impact_child = inline_end_floats_impact_child ||
                    kid_base.flags.contains(HAS_RIGHT_FLOATED_DESCENDANTS);
                kid_base.flags.set(IMPACTED_BY_LEFT_FLOATS, inline_start_floats_impact_child);
                kid_base.flags.set(IMPACTED_BY_RIGHT_FLOATS, inline_end_floats_impact_child);
            }

            if kid.is_block_flow() {
                let kid_block = kid.as_block();
                kid_block.inline_size_of_preceding_left_floats =
                    inline_size_of_preceding_left_floats;
                kid_block.inline_size_of_preceding_right_floats =
                    inline_size_of_preceding_right_floats;
            }

            // Handle tables.
            match optional_column_computed_inline_sizes {
                Some(ref column_computed_inline_sizes) => {
                    propagate_column_inline_sizes_to_child(kid,
                                                           i,
                                                           content_inline_size,
                                                           *column_computed_inline_sizes,
                                                           &mut inline_start_margin_edge)
                }
                None => {}
            }

            // Per CSS 2.1 § 16.3.1, text alignment propagates to all children in flow.
            //
            // TODO(#2018, pcwalton): Do this in the cascade instead.
            flow::mut_base(kid).flags.propagate_text_alignment_from_parent(flags.clone());

            // Handle `text-indent` on behalf of any inline children that we have. This is
            // necessary because any percentages are relative to the containing block, which only
            // we know.
            if kid.is_inline_flow() {
                kid.as_inline().first_line_indentation =
                    specified(self.fragment.style().get_inheritedtext().text_indent,
                              containing_block_size);
            }
        }
    }

    /// Determines the type of formatting context this is. See the definition of
    /// `FormattingContextType`.
    fn formatting_context_type(&self) -> FormattingContextType {
        let style = self.fragment.style();
        if style.get_box().float != float::T::none {
            return FormattingContextType::Other
        }
        match style.get_box().display {
            display::T::table_cell |
            display::T::table_caption |
            display::T::inline_block => {
                FormattingContextType::Other
            }
            _ if style.get_box().overflow != overflow::T::visible => FormattingContextType::Block,
            _ => FormattingContextType::None,
        }
    }

    /// Per CSS 2.1 § 9.5, block formatting contexts' inline widths and positions are affected by
    /// the presence of floats. This is the part of the assign-heights traversal that computes
    /// the final inline position and width for such flows.
    ///
    /// Note that this is part of the assign-block-sizes traversal, not the assign-inline-sizes
    /// traversal as one might expect. That is because, in general, float placement cannot occur
    /// until heights are assigned. To work around this unfortunate circular dependency, by the
    /// time we get here we have already estimated the width of the block formatting context based
    /// on the floats we could see at the time of inline-size assignment. The job of this function,
    /// therefore, is not only to assign the final size but also to perform the layout again for
    /// this block formatting context if our speculation was wrong.
    ///
    /// FIXME(pcwalton): This code is not incremental-reflow-safe (i.e. not idempotent).
    fn assign_inline_position_for_formatting_context(&mut self) {
        debug_assert!(self.formatting_context_type() != FormattingContextType::None);

        if !self.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
            return
        }

        let info = PlacementInfo {
            size: LogicalSize::new(self.fragment.style.writing_mode,
                                   self.base.position.size.inline +
                                        self.fragment.margin.inline_start_end() +
                                        self.fragment.border_padding.inline_start_end(),
                                   self.fragment.border_box.size.block),
            ceiling: self.base.position.start.b,
            max_inline_size: MAX_AU,
            kind: FloatKind::Left,
        };

        // Offset our position by whatever displacement is needed to not impact the floats.
        let rect = self.base.floats.place_between_floats(&info);
        self.base.position.start.i = self.hypothetical_position.i + rect.start.i;

        // TODO(pcwalton): If the inline-size of this flow is different from the size we estimated
        // earlier, lay it out again.

        self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
    }

    fn is_inline_block(&self) -> bool {
        self.fragment.style().get_box().display == display::T::inline_block
    }

    /// Computes the content portion (only) of the intrinsic inline sizes of this flow. This is
    /// used for calculating shrink-to-fit width. Assumes that intrinsic sizes have already been
    /// computed for this flow.
    fn content_intrinsic_inline_sizes(&self) -> IntrinsicISizes {
        let surrounding_inline_size = self.fragment.surrounding_intrinsic_inline_size();
        IntrinsicISizes {
            minimum_inline_size: self.base.intrinsic_inline_sizes.minimum_inline_size -
                                    surrounding_inline_size,
            preferred_inline_size: self.base.intrinsic_inline_sizes.preferred_inline_size -
                                    surrounding_inline_size,
        }
    }
}

impl Flow for BlockFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Block
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        self
    }

    fn as_immutable_block<'a>(&'a self) -> &'a BlockFlow {
        self
    }

    /// Pass 1 of reflow: computes minimum and preferred inline-sizes.
    ///
    /// Recursively (bottom-up) determine the flow's minimum and preferred inline-sizes. When
    /// called on this flow, all child flows have had their minimum and preferred inline-sizes set.
    /// This function must decide minimum/preferred inline-sizes based on its children's
    /// inline-sizes and the dimensions of any fragments it is responsible for flowing.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("block::bubble_inline_sizes {:x}", self.base.debug_id());

        let mut flags = self.base.flags;
        flags.remove(HAS_LEFT_FLOATED_DESCENDANTS);
        flags.remove(HAS_RIGHT_FLOATED_DESCENDANTS);

        // If this block has a fixed width, just use that for the minimum
        // and preferred width, rather than bubbling up children inline
        // width.
        let fixed_width = match self.fragment.style().get_box().width {
            LengthOrPercentageOrAuto::Length(_) => true,
            _ => false,
        };

        // Find the maximum inline-size from children.
        let mut computation = self.fragment.compute_intrinsic_inline_sizes();
        let mut left_float_width = Au(0);
        let mut right_float_width = Au(0);
        for kid in self.base.child_iter() {
            let is_absolutely_positioned = flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED);
            let child_base = flow::mut_base(kid);
            let float_kind = child_base.flags.float_kind();
            if !is_absolutely_positioned && !fixed_width {
                computation.content_intrinsic_sizes.minimum_inline_size =
                    max(computation.content_intrinsic_sizes.minimum_inline_size,
                        child_base.intrinsic_inline_sizes.minimum_inline_size);

                match float_kind {
                    float::T::none => {
                        computation.content_intrinsic_sizes.preferred_inline_size =
                            max(computation.content_intrinsic_sizes.preferred_inline_size,
                                child_base.intrinsic_inline_sizes.preferred_inline_size);
                    }
                    float::T::left => {
                        left_float_width = left_float_width +
                            child_base.intrinsic_inline_sizes.preferred_inline_size;
                    }
                    float::T::right => {
                        right_float_width = right_float_width +
                            child_base.intrinsic_inline_sizes.preferred_inline_size;
                    }
                }
            }

            flags.union_floated_descendants_flags(child_base.flags);
        }

        // FIXME(pcwalton): This should consider all float descendants, not just children.
        // FIXME(pcwalton): This is not well-spec'd; INTRINSIC specifies to do this, but CSS-SIZING
        // says not to. In practice, Gecko and WebKit both do this.
        computation.content_intrinsic_sizes.preferred_inline_size =
            max(computation.content_intrinsic_sizes.preferred_inline_size,
                left_float_width + right_float_width);

        self.base.intrinsic_inline_sizes = computation.finish();

        match self.fragment.style().get_box().float {
            float::T::none => {}
            float::T::left => flags.insert(HAS_LEFT_FLOATED_DESCENDANTS),
            float::T::right => flags.insert(HAS_RIGHT_FLOATED_DESCENDANTS),
        }
        self.base.flags = flags
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    ///
    /// Dual fragments consume some inline-size first, and the remainder is assigned to all child
    /// (block) contexts.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("block::assign_inline_sizes {:x}", self.base.debug_id());

        if !self.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
            return
        }

        debug!("assign_inline_sizes({}): assigning inline_size for flow",
               if self.base.flags.is_float() {
                   "float"
               } else {
                   "block"
               });

        self.base.floats = Floats::new(self.base.writing_mode);

        if self.is_root() {
            debug!("Setting root position");
            self.base.position.start = LogicalPoint::zero(self.base.writing_mode);
            self.base.block_container_inline_size = LogicalSize::from_physical(
                self.base.writing_mode, layout_context.shared.screen_size).inline;

            // The root element is never impacted by floats.
            self.base.flags.remove(IMPACTED_BY_LEFT_FLOATS);
            self.base.flags.remove(IMPACTED_BY_RIGHT_FLOATS);
        }

        // Our inline-size was set to the inline-size of the containing block by the flow's parent.
        // Now compute the real value.
        self.propagate_and_compute_used_inline_size(layout_context);

        // Formatting contexts are never impacted by floats.
        match self.formatting_context_type() {
            FormattingContextType::None => {}
            FormattingContextType::Block => {
                self.base.flags.remove(IMPACTED_BY_LEFT_FLOATS);
                self.base.flags.remove(IMPACTED_BY_RIGHT_FLOATS);

                // We can't actually compute the inline-size of this block now, because floats
                // might affect it. Speculate that its inline-size is equal to the inline-size
                // computed above minus the inline-size of the previous left and/or right floats.
                self.fragment.border_box.size.inline =
                    self.fragment.border_box.size.inline -
                    self.inline_size_of_preceding_left_floats -
                    self.inline_size_of_preceding_right_floats;
            }
            FormattingContextType::Other => {
                self.base.flags.remove(IMPACTED_BY_LEFT_FLOATS);
                self.base.flags.remove(IMPACTED_BY_RIGHT_FLOATS);
            }
        }

        // Move in from the inline-start border edge.
        let inline_start_content_edge = self.fragment.border_box.start.i +
            self.fragment.border_padding.inline_start;
        let padding_and_borders = self.fragment.border_padding.inline_start_end();
        let content_inline_size = self.fragment.border_box.size.inline - padding_and_borders;

        self.propagate_assigned_inline_size_to_children(layout_context,
                                                        inline_start_content_edge,
                                                        content_inline_size,
                                                        None);
    }

    fn place_float_if_applicable<'a>(&mut self, _: &'a LayoutContext<'a>) {
        if self.base.flags.is_float() {
            self.place_float();
        }
    }

    fn assign_block_size_for_inorder_child_if_necessary<'a>(&mut self,
                                                            layout_context: &'a LayoutContext<'a>)
                                                            -> bool {
        if self.base.flags.is_float() {
            return false
        }

        let is_formatting_context = self.formatting_context_type() != FormattingContextType::None;
        if !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) && is_formatting_context {
            self.assign_inline_position_for_formatting_context();
        }

        if self.base.flags.impacted_by_floats() {
            if self.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
                self.assign_block_size(layout_context);
                // Don't remove the restyle damage; `assign_block_size` decides whether that is
                // appropriate (which in the case of e.g. absolutely-positioned flows, it is not).
            }
            return true
        }

        if is_formatting_context {
            // If this is a formatting context and was *not* impacted by floats, then we must
            // translate the floats past us.
            let writing_mode = self.base.floats.writing_mode;
            let delta = self.base.position.size.block;
            self.base.floats.translate(LogicalSize::new(writing_mode, Au(0), -delta));
            return true
        }

        false
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        if self.is_replaced_content() {
            let _scope = layout_debug_scope!("assign_replaced_block_size_if_necessary {:x}",
                                                self.base.debug_id());

            // Assign block-size for fragment if it is an image fragment.
            let containing_block_block_size =
                self.base.block_container_explicit_block_size.unwrap_or(Au(0));
            self.fragment.assign_replaced_block_size_if_necessary(containing_block_block_size);
            if !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                self.base.position.size.block = self.fragment.border_box.size.block;
            }
        } else if self.is_root() || self.base.flags.is_float() || self.is_inline_block() {
            // Root element margins should never be collapsed according to CSS § 8.3.1.
            debug!("assign_block_size: assigning block_size for root flow");
            self.assign_block_size_block_base(ctx, MarginsMayCollapseFlag::MarginsMayNotCollapse);
        } else {
            debug!("assign_block_size: assigning block_size for block");
            self.assign_block_size_block_base(ctx, MarginsMayCollapseFlag::MarginsMayCollapse);
        }
    }

    fn compute_absolute_position(&mut self) {
        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();

        if self.is_root() {
            self.base.clip = ClippingRegion::max()
        }

        if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            let position_start = self.base.position.start.to_physical(self.base.writing_mode,
                                                                      container_size);

            // Compute our position relative to the nearest ancestor stacking context. This will be
            // passed down later as part of containing block details for absolute descendants.
            self.base.stacking_relative_position = if self.is_fixed() {
                // The viewport is initially at (0, 0).
                position_start
            } else {
                // Absolute position of the containing block + position of absolute
                // flow w.r.t. the containing block.
                self.base
                    .absolute_position_info
                    .stacking_relative_position_of_absolute_containing_block + position_start
            }
        }

        // For relatively-positioned descendants, the containing block formed by a block is just
        // the content box. The containing block for absolutely-positioned descendants, on the
        // other hand, is only established if we are positioned.
        let relative_offset =
            self.fragment.relative_position(&self.base
                                                 .absolute_position_info
                                                 .relative_containing_block_size);
        if self.is_positioned() {
            self.base
                .absolute_position_info
                .stacking_relative_position_of_absolute_containing_block =
                    self.base.stacking_relative_position +
                    (self.generated_containing_block_rect().start +
                     relative_offset).to_physical(self.base.writing_mode, container_size)
        }

        // Compute absolute position info for children.
        let stacking_relative_position_of_absolute_containing_block_for_children =
            if self.fragment.establishes_stacking_context() {
                let logical_border_width = self.fragment.style().logical_border_width();
                let position = LogicalPoint::new(self.base.writing_mode,
                                                 logical_border_width.inline_start,
                                                 logical_border_width.block_start);
                let position = position.to_physical(self.base.writing_mode, container_size);
                if self.is_positioned() {
                    position
                } else {
                    // We establish a stacking context but are not positioned. (This will happen
                    // if, for example, the element has `position: static` but has `opacity` or
                    // `transform` set.) In this case, absolutely-positioned children will not be
                    // positioned relative to us but will instead be positioned relative to our
                    // containing block.
                    position - self.base.stacking_relative_position
                }
            } else {
                self.base
                    .absolute_position_info
                    .stacking_relative_position_of_absolute_containing_block
            };
        let absolute_position_info_for_children = AbsolutePositionInfo {
            stacking_relative_position_of_absolute_containing_block:
                stacking_relative_position_of_absolute_containing_block_for_children,
            relative_containing_block_size: self.fragment.content_box().size,
            layers_needed_for_positioned_flows: self.base
                                                    .flags
                                                    .contains(LAYERS_NEEDED_FOR_DESCENDANTS),
        };

        // Compute the origin and clipping rectangle for children.
        let relative_offset = relative_offset.to_physical(self.base.writing_mode);
        let origin_for_children;
        let clip_in_child_coordinate_system;
        if self.fragment.establishes_stacking_context() {
            // We establish a stacking context, so the position of our children is vertically
            // correct, but has to be adjusted to accommodate horizontal margins. (Note the
            // calculation involving `position` below and recall that inline-direction flow
            // positions are relative to the edges of the margin box.)
            //
            // FIXME(pcwalton): Is this vertical-writing-direction-safe?
            let margin = self.fragment.margin.to_physical(self.base.writing_mode);
            origin_for_children = Point2D(-margin.left, Au(0)) + relative_offset;
            clip_in_child_coordinate_system =
                self.base.clip.translate(&-self.base.stacking_relative_position)
        } else {
            origin_for_children = self.base.stacking_relative_position + relative_offset;
            clip_in_child_coordinate_system = self.base.clip.clone()
        }
        let stacking_relative_border_box =
            self.fragment
                .stacking_relative_border_box(&self.base.stacking_relative_position,
                                              &self.base
                                                   .absolute_position_info
                                                   .relative_containing_block_size,
                                              CoordinateSystem::Self);
        let clip = self.fragment.clipping_region_for_children(&clip_in_child_coordinate_system,
                                                              &stacking_relative_border_box);

        // Process children.
        for kid in self.base.child_iter() {
            if !flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED) {
                let kid_base = flow::mut_base(kid);
                kid_base.stacking_relative_position = origin_for_children +
                    kid_base.position.start.to_physical(kid_base.writing_mode, container_size);
            }

            flow::mut_base(kid).absolute_position_info = absolute_position_info_for_children;
            flow::mut_base(kid).clip = clip.clone()
        }
    }

    fn mark_as_root(&mut self) {
        self.flags.insert(IS_ROOT)
    }

    fn is_root(&self) -> bool {
        self.flags.contains(IS_ROOT)
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
        LayerId(self.fragment.node.id() as uint, fragment_index)
    }

    fn is_absolute_containing_block(&self) -> bool {
        self.is_positioned()
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) &&
                self.fragment.style().logical_position().inline_start ==
                    LengthOrPercentageOrAuto::Auto &&
                self.fragment.style().logical_position().inline_end ==
                LengthOrPercentageOrAuto::Auto {
            self.base.position.start.i = inline_position
        }
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) &&
                self.fragment.style().logical_position().block_start ==
                    LengthOrPercentageOrAuto::Auto &&
                self.fragment.style().logical_position().block_end ==
                LengthOrPercentageOrAuto::Auto {
            self.base.position.start.b = block_position
        }
    }

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        self.build_display_list_for_block(box DisplayList::new(), layout_context);
        if opts::get().validate_display_list_geometry {
            self.base.validate_display_list_geometry();
        }
    }

    fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.fragment.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Rect<Au> {
        self.fragment.compute_overflow()
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             stacking_context_position: &Point2D<Au>) {
        if !iterator.should_process(&self.fragment) {
            return
        }

        iterator.process(&self.fragment,
                         &self.fragment
                              .stacking_relative_border_box(&self.base.stacking_relative_position,
                                                            &self.base
                                                                 .absolute_position_info
                                                                 .relative_containing_block_size,
                                                            CoordinateSystem::Parent)
                              .translate(stacking_context_position));
    }
}

impl fmt::Debug for BlockFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:?} - {:x}: frag={:?} ({:?})",
               self.class(),
               self.base.debug_id(),
               self.fragment,
               self.base)
    }
}

/// The inputs for the inline-sizes-and-margins constraint equation.
#[derive(Debug, Copy)]
pub struct ISizeConstraintInput {
    pub computed_inline_size: MaybeAuto,
    pub inline_start_margin: MaybeAuto,
    pub inline_end_margin: MaybeAuto,
    pub inline_start: MaybeAuto,
    pub inline_end: MaybeAuto,
    pub available_inline_size: Au,
    pub static_i_offset: Au,
}

impl ISizeConstraintInput {
    pub fn new(computed_inline_size: MaybeAuto,
               inline_start_margin: MaybeAuto,
               inline_end_margin: MaybeAuto,
               inline_start: MaybeAuto,
               inline_end: MaybeAuto,
               available_inline_size: Au,
               static_i_offset: Au)
           -> ISizeConstraintInput {
        ISizeConstraintInput {
            computed_inline_size: computed_inline_size,
            inline_start_margin: inline_start_margin,
            inline_end_margin: inline_end_margin,
            inline_start: inline_start,
            inline_end: inline_end,
            available_inline_size: available_inline_size,
            static_i_offset: static_i_offset,
        }
    }
}

/// The solutions for the inline-size-and-margins constraint equation.
#[derive(Copy, Debug)]
pub struct ISizeConstraintSolution {
    pub inline_start: Au,
    pub inline_end: Au,
    pub inline_size: Au,
    pub margin_inline_start: Au,
    pub margin_inline_end: Au
}

impl ISizeConstraintSolution {
    pub fn new(inline_size: Au, margin_inline_start: Au, margin_inline_end: Au) -> ISizeConstraintSolution {
        ISizeConstraintSolution {
            inline_start: Au(0),
            inline_end: Au(0),
            inline_size: inline_size,
            margin_inline_start: margin_inline_start,
            margin_inline_end: margin_inline_end,
        }
    }

    fn for_absolute_flow(inline_start: Au,
                         inline_end: Au,
                         inline_size: Au,
                         margin_inline_start: Au,
                         margin_inline_end: Au)
                         -> ISizeConstraintSolution {
        ISizeConstraintSolution {
            inline_start: inline_start,
            inline_end: inline_end,
            inline_size: inline_size,
            margin_inline_start: margin_inline_start,
            margin_inline_end: margin_inline_end,
        }
    }
}

// Trait to encapsulate the ISize and Margin calculation.
//
// CSS Section 10.3
pub trait ISizeAndMarginsComputer {
    /// Compute the inputs for the ISize constraint equation.
    ///
    /// This is called only once to compute the initial inputs. For calculations involving
    /// minimum and maximum inline-size, we don't need to recompute these.
    fn compute_inline_size_constraint_inputs(&self,
                                             block: &mut BlockFlow,
                                             parent_flow_inline_size: Au,
                                             layout_context: &LayoutContext)
                                             -> ISizeConstraintInput {
        let containing_block_inline_size =
            self.containing_block_inline_size(block, parent_flow_inline_size, layout_context);

        block.fragment.compute_block_direction_margins(containing_block_inline_size);
        block.fragment.compute_inline_direction_margins(containing_block_inline_size);
        block.fragment.compute_border_and_padding(containing_block_inline_size);

        let mut computed_inline_size = self.initial_computed_inline_size(block,
                                                                         parent_flow_inline_size,
                                                                         layout_context);

        let style = block.fragment.style();
        match (computed_inline_size, style.get_box().box_sizing) {
            (MaybeAuto::Specified(size), box_sizing::T::border_box) => {
                computed_inline_size =
                    MaybeAuto::Specified(size - block.fragment.border_padding.inline_start_end())
            }
            (MaybeAuto::Auto, box_sizing::T::border_box) |
            (_, box_sizing::T::content_box) => {}
        }

        // The text alignment of a block flow is the text alignment of its box's style.
        block.base.flags.set_text_align(style.get_inheritedtext().text_align);

        let margin = style.logical_margin();
        let position = style.logical_position();

        let available_inline_size = containing_block_inline_size -
            block.fragment.border_padding.inline_start_end();
        return ISizeConstraintInput::new(
            computed_inline_size,
            MaybeAuto::from_style(margin.inline_start, containing_block_inline_size),
            MaybeAuto::from_style(margin.inline_end, containing_block_inline_size),
            MaybeAuto::from_style(position.inline_start, containing_block_inline_size),
            MaybeAuto::from_style(position.inline_end, containing_block_inline_size),
            available_inline_size,
            block.static_i_offset());
    }

    /// Set the used values for inline-size and margins from the relevant constraint equation.
    /// This is called only once.
    ///
    /// Set:
    /// * Used values for content inline-size, inline-start margin, and inline-end margin for this
    ///   flow's box;
    /// * Inline-start coordinate of this flow's box;
    /// * Inline-start coordinate of the flow with respect to its containing block (if this is an
    ///   absolute flow).
    fn set_inline_size_constraint_solutions(&self,
                                            block: &mut BlockFlow,
                                            solution: ISizeConstraintSolution) {
        let inline_size;
        let extra_inline_size_from_margin;
        {
            let fragment = block.fragment();
            fragment.margin.inline_start = solution.margin_inline_start;
            fragment.margin.inline_end = solution.margin_inline_end;

            // Left border edge.
            fragment.border_box.start.i = fragment.margin.inline_start;

            // The associated fragment has the border box of this flow.
            inline_size = solution.inline_size + fragment.border_padding.inline_start_end();
            fragment.border_box.size.inline = inline_size;

            // To calculate the total size of this block, we also need to account for any additional
            // size contribution from positive margins. Negative margins means the block isn't made
            // larger at all by the margin.
            extra_inline_size_from_margin = max(Au(0), fragment.margin.inline_start) +
                                            max(Au(0), fragment.margin.inline_end);
        }

        // We also resize the block itself, to ensure that overflow is not calculated
        // as the inline-size of our parent. We might be smaller and we might be larger if we
        // overflow.
        flow::mut_base(block).position.size.inline = inline_size + extra_inline_size_from_margin;
    }

    /// Set the x coordinate of the given flow if it is absolutely positioned.
    fn set_flow_x_coord_if_necessary(&self, _: &mut BlockFlow, _: ISizeConstraintSolution) {}

    /// Solve the inline-size and margins constraints for this block flow.
    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution;

    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    ctx: &LayoutContext)
                                    -> MaybeAuto {
        MaybeAuto::from_style(block.fragment().style().content_inline_size(),
                              self.containing_block_inline_size(block,
                                                                parent_flow_inline_size,
                                                                ctx))
    }

    fn containing_block_inline_size(&self,
                              _: &mut BlockFlow,
                              parent_flow_inline_size: Au,
                              _: &LayoutContext)
                              -> Au {
        parent_flow_inline_size
    }

    /// Compute the used value of inline-size, taking care of min-inline-size and max-inline-size.
    ///
    /// CSS Section 10.4: Minimum and Maximum inline-sizes
    fn compute_used_inline_size(&self,
                                block: &mut BlockFlow,
                                layout_context: &LayoutContext,
                                parent_flow_inline_size: Au) {
        let mut input = self.compute_inline_size_constraint_inputs(block,
                                                                   parent_flow_inline_size,
                                                                   layout_context);

        let containing_block_inline_size =
            self.containing_block_inline_size(block, parent_flow_inline_size, layout_context);

        let mut solution = self.solve_inline_size_constraints(block, &input);

        // If the tentative used inline-size is greater than 'max-inline-size', inline-size should
        // be recalculated, but this time using the computed value of 'max-inline-size' as the
        // computed value for 'inline-size'.
        match specified_or_none(block.fragment().style().max_inline_size(),
                                containing_block_inline_size) {
            Some(max_inline_size) if max_inline_size < solution.inline_size => {
                input.computed_inline_size = MaybeAuto::Specified(max_inline_size);
                solution = self.solve_inline_size_constraints(block, &input);
            }
            _ => {}
        }

        // If the resulting inline-size is smaller than 'min-inline-size', inline-size should be
        // recalculated, but this time using the value of 'min-inline-size' as the computed value
        // for 'inline-size'.
        let computed_min_inline_size = specified(block.fragment().style().min_inline_size(),
                                                 containing_block_inline_size);
        if computed_min_inline_size > solution.inline_size {
            input.computed_inline_size = MaybeAuto::Specified(computed_min_inline_size);
            solution = self.solve_inline_size_constraints(block, &input);
        }

        self.set_inline_size_constraint_solutions(block, solution);
        self.set_flow_x_coord_if_necessary(block, solution);
    }

    /// Computes inline-start and inline-end margins and inline-size.
    ///
    /// This is used by both replaced and non-replaced Blocks.
    ///
    /// CSS 2.1 Section 10.3.3.
    /// Constraint Equation: margin-inline-start + margin-inline-end + inline-size =
    /// available_inline-size
    /// where available_inline-size = CB inline-size - (horizontal border + padding)
    fn solve_block_inline_size_constraints(&self,
                                           block: &mut BlockFlow,
                                           input: &ISizeConstraintInput)
                                           -> ISizeConstraintSolution {
        let (computed_inline_size, inline_start_margin, inline_end_margin, available_inline_size) =
            (input.computed_inline_size,
             input.inline_start_margin,
             input.inline_end_margin,
             input.available_inline_size);

        // If inline-size is not 'auto', and inline-size + margins > available_inline-size, all
        // 'auto' margins are treated as 0.
        let (inline_start_margin, inline_end_margin) = match computed_inline_size {
            MaybeAuto::Auto => (inline_start_margin, inline_end_margin),
            MaybeAuto::Specified(inline_size) => {
                let inline_start = inline_start_margin.specified_or_zero();
                let inline_end = inline_end_margin.specified_or_zero();

                if (inline_start + inline_end + inline_size) > available_inline_size {
                    (MaybeAuto::Specified(inline_start), MaybeAuto::Specified(inline_end))
                } else {
                    (inline_start_margin, inline_end_margin)
                }
            }
        };

        // Invariant: inline-start_margin + inline-size + inline-end_margin ==
        // available_inline-size
        let (inline_start_margin, mut inline_size, inline_end_margin) =
            match (inline_start_margin, computed_inline_size, inline_end_margin) {
                // If all have a computed value other than 'auto', the system is
                // over-constrained so we discard the end margin.
                (MaybeAuto::Specified(margin_start), MaybeAuto::Specified(inline_size), MaybeAuto::Specified(_margin_end)) =>
                    (margin_start, inline_size, available_inline_size -
                     (margin_start + inline_size)),

                // If exactly one value is 'auto', solve for it
                (MaybeAuto::Auto, MaybeAuto::Specified(inline_size), MaybeAuto::Specified(margin_end)) =>
                    (available_inline_size - (inline_size + margin_end), inline_size, margin_end),
                (MaybeAuto::Specified(margin_start), MaybeAuto::Auto, MaybeAuto::Specified(margin_end)) =>
                    (margin_start, available_inline_size - (margin_start + margin_end),
                     margin_end),
                (MaybeAuto::Specified(margin_start), MaybeAuto::Specified(inline_size), MaybeAuto::Auto) =>
                    (margin_start, inline_size, available_inline_size -
                     (margin_start + inline_size)),

                // If inline-size is set to 'auto', any other 'auto' value becomes '0',
                // and inline-size is solved for
                (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Specified(margin_end)) =>
                    (Au::new(0), available_inline_size - margin_end, margin_end),
                (MaybeAuto::Specified(margin_start), MaybeAuto::Auto, MaybeAuto::Auto) =>
                    (margin_start, available_inline_size - margin_start, Au::new(0)),
                (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Auto) =>
                    (Au::new(0), available_inline_size, Au::new(0)),

                // If inline-start and inline-end margins are auto, they become equal
                (MaybeAuto::Auto, MaybeAuto::Specified(inline_size), MaybeAuto::Auto) => {
                    let margin = (available_inline_size - inline_size).scale_by(0.5);
                    (margin, inline_size, margin)
                }
            };

        // If inline-size is set to 'auto', and this is an inline block, use the
        // shrink to fit algorithm (see CSS 2.1 § 10.3.9)
        if computed_inline_size == MaybeAuto::Auto && block.is_inline_block() {
            inline_size = block.get_shrink_to_fit_inline_size(inline_size);
        }

        ISizeConstraintSolution::new(inline_size, inline_start_margin, inline_end_margin)
    }
}

/// The different types of Blocks.
///
/// They mainly differ in the way inline-size and block-sizes and margins are calculated
/// for them.
pub struct AbsoluteNonReplaced;
pub struct AbsoluteReplaced;
pub struct BlockNonReplaced;
pub struct BlockReplaced;
pub struct FloatNonReplaced;
pub struct FloatReplaced;

impl ISizeAndMarginsComputer for AbsoluteNonReplaced {
    /// Solve the horizontal constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.3.7
    /// Constraint equation:
    /// inline-start + inline-end + inline-size + margin-inline-start + margin-inline-end
    /// = absolute containing block inline-size - (horizontal padding and border)
    /// [aka available_inline-size]
    ///
    /// Return the solution for the equation.
    fn solve_inline_size_constraints(&self,
                               block: &mut BlockFlow,
                               input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        let &ISizeConstraintInput {
            computed_inline_size,
            inline_start_margin,
            inline_end_margin,
            inline_start,
            inline_end,
            available_inline_size,
            static_i_offset,
            ..
        } = input;

        // TODO: Check for direction of parent flow (NOT Containing Block)
        // when right-to-left is implemented.
        // Assume direction is 'ltr' for now

        // Distance from the inline-start edge of the Absolute Containing Block to the
        // inline-start margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_inline_start = static_i_offset;

        let (inline_start, inline_end, inline_size, margin_inline_start, margin_inline_end) = match (inline_start, inline_end, computed_inline_size) {
            (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Auto) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                let inline_start = static_position_inline_start;
                // Now it is the same situation as inline-start Specified and inline-end
                // and inline-size Auto.

                // Set inline-end to zero to calculate inline-size
                let inline_size = block.get_shrink_to_fit_inline_size(
                    available_inline_size - (inline_start + margin_start + margin_end));
                let sum = inline_start + inline_size + margin_start + margin_end;
                (inline_start, available_inline_size - sum, inline_size, margin_start, margin_end)
            }
            (MaybeAuto::Specified(inline_start), MaybeAuto::Specified(inline_end), MaybeAuto::Specified(inline_size)) => {
                match (inline_start_margin, inline_end_margin) {
                    (MaybeAuto::Auto, MaybeAuto::Auto) => {
                        let total_margin_val = available_inline_size - inline_start - inline_end - inline_size;
                        if total_margin_val < Au(0) {
                            // margin-inline-start becomes 0 because direction is 'ltr'.
                            // TODO: Handle 'rtl' when it is implemented.
                            (inline_start, inline_end, inline_size, Au(0), total_margin_val)
                        } else {
                            // Equal margins
                            (inline_start, inline_end, inline_size,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                    }
                    (MaybeAuto::Specified(margin_start), MaybeAuto::Auto) => {
                        let sum = inline_start + inline_end + inline_size + margin_start;
                        (inline_start, inline_end, inline_size, margin_start, available_inline_size - sum)
                    }
                    (MaybeAuto::Auto, MaybeAuto::Specified(margin_end)) => {
                        let sum = inline_start + inline_end + inline_size + margin_end;
                        (inline_start, inline_end, inline_size, available_inline_size - sum, margin_end)
                    }
                    (MaybeAuto::Specified(margin_start), MaybeAuto::Specified(margin_end)) => {
                        // Values are over-constrained.
                        // Ignore value for 'inline-end' cos direction is 'ltr'.
                        // TODO: Handle 'rtl' when it is implemented.
                        let sum = inline_start + inline_size + margin_start + margin_end;
                        (inline_start, available_inline_size - sum, inline_size, margin_start, margin_end)
                    }
                }
            }
            // For the rest of the cases, auto values for margin are set to 0

            // If only one is Auto, solve for it
            (MaybeAuto::Auto, MaybeAuto::Specified(inline_end), MaybeAuto::Specified(inline_size)) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                let sum = inline_end + inline_size + margin_start + margin_end;
                (available_inline_size - sum, inline_end, inline_size, margin_start, margin_end)
            }
            (MaybeAuto::Specified(inline_start), MaybeAuto::Auto, MaybeAuto::Specified(inline_size)) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                let sum = inline_start + inline_size + margin_start + margin_end;
                (inline_start, available_inline_size - sum, inline_size, margin_start, margin_end)
            }
            (MaybeAuto::Specified(inline_start), MaybeAuto::Specified(inline_end), MaybeAuto::Auto) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                let sum = inline_start + inline_end + margin_start + margin_end;
                (inline_start, inline_end, available_inline_size - sum, margin_start, margin_end)
            }

            // If inline-size is auto, then inline-size is shrink-to-fit. Solve for the
            // non-auto value.
            (MaybeAuto::Specified(inline_start), MaybeAuto::Auto, MaybeAuto::Auto) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                // Set inline-end to zero to calculate inline-size
                let inline_size = block.get_shrink_to_fit_inline_size(
                    available_inline_size - (inline_start + margin_start + margin_end));
                let sum = inline_start + inline_size + margin_start + margin_end;
                (inline_start, available_inline_size - sum, inline_size, margin_start, margin_end)
            }
            (MaybeAuto::Auto, MaybeAuto::Specified(inline_end), MaybeAuto::Auto) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                // Set inline-start to zero to calculate inline-size
                let inline_size = block.get_shrink_to_fit_inline_size(
                    available_inline_size - (inline_end + margin_start + margin_end));
                let sum = inline_end + inline_size + margin_start + margin_end;
                (available_inline_size - sum, inline_end, inline_size, margin_start, margin_end)
            }

            (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Specified(inline_size)) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                // Setting 'inline-start' to static position because direction is 'ltr'.
                // TODO: Handle 'rtl' when it is implemented.
                let inline_start = static_position_inline_start;
                let sum = inline_start + inline_size + margin_start + margin_end;
                (inline_start, available_inline_size - sum, inline_size, margin_start, margin_end)
            }
        };
        ISizeConstraintSolution::for_absolute_flow(inline_start, inline_end, inline_size, margin_inline_start, margin_inline_end)
    }

    fn containing_block_inline_size(&self, block: &mut BlockFlow, _: Au, ctx: &LayoutContext) -> Au {
        block.containing_block_size(ctx.shared.screen_size).inline
    }

    fn set_flow_x_coord_if_necessary(&self,
                                     block: &mut BlockFlow,
                                     solution: ISizeConstraintSolution) {
        // Set the x-coordinate of the absolute flow wrt to its containing block.
        block.base.position.start.i = solution.inline_start;
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
    /// inline-start + inline-end + inline-size + margin-inline-start + margin-inline-end
    /// = absolute containing block inline-size - (horizontal padding and border)
    /// [aka available_inline-size]
    ///
    /// Return the solution for the equation.
    fn solve_inline_size_constraints(&self, _: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        let &ISizeConstraintInput {
            computed_inline_size,
            inline_start_margin,
            inline_end_margin,
            inline_start,
            inline_end,
            available_inline_size,
            static_i_offset,
            ..
        } = input;
        // TODO: Check for direction of static-position Containing Block (aka
        // parent flow, _not_ the actual Containing Block) when right-to-left
        // is implemented
        // Assume direction is 'ltr' for now
        // TODO: Handle all the cases for 'rtl' direction.

        let inline_size = match computed_inline_size {
            MaybeAuto::Specified(w) => w,
            _ => panic!("{} {}",
                       "The used value for inline_size for absolute replaced flow",
                       "should have already been calculated by now.")
        };

        // Distance from the inline-start edge of the Absolute Containing Block to the
        // inline-start margin edge of a hypothetical box that would have been the
        // first box of the element.
        let static_position_inline_start = static_i_offset;

        let (inline_start, inline_end, inline_size, margin_inline_start, margin_inline_end) = match (inline_start, inline_end) {
            (MaybeAuto::Auto, MaybeAuto::Auto) => {
                let inline_start = static_position_inline_start;
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                let sum = inline_start + inline_size + margin_start + margin_end;
                (inline_start, available_inline_size - sum, inline_size, margin_start, margin_end)
            }
            // If only one is Auto, solve for it
            (MaybeAuto::Auto, MaybeAuto::Specified(inline_end)) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                let sum = inline_end + inline_size + margin_start + margin_end;
                (available_inline_size - sum, inline_end, inline_size, margin_start, margin_end)
            }
            (MaybeAuto::Specified(inline_start), MaybeAuto::Auto) => {
                let margin_start = inline_start_margin.specified_or_zero();
                let margin_end = inline_end_margin.specified_or_zero();
                let sum = inline_start + inline_size + margin_start + margin_end;
                (inline_start, available_inline_size - sum, inline_size, margin_start, margin_end)
            }
            (MaybeAuto::Specified(inline_start), MaybeAuto::Specified(inline_end)) => {
                match (inline_start_margin, inline_end_margin) {
                    (MaybeAuto::Auto, MaybeAuto::Auto) => {
                        let total_margin_val = available_inline_size - inline_start - inline_end - inline_size;
                        if total_margin_val < Au(0) {
                            // margin-inline-start becomes 0 because direction is 'ltr'.
                            (inline_start, inline_end, inline_size, Au(0), total_margin_val)
                        } else {
                            // Equal margins
                            (inline_start, inline_end, inline_size,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                    }
                    (MaybeAuto::Specified(margin_start), MaybeAuto::Auto) => {
                        let sum = inline_start + inline_end + inline_size + margin_start;
                        (inline_start, inline_end, inline_size, margin_start, available_inline_size - sum)
                    }
                    (MaybeAuto::Auto, MaybeAuto::Specified(margin_end)) => {
                        let sum = inline_start + inline_end + inline_size + margin_end;
                        (inline_start, inline_end, inline_size, available_inline_size - sum, margin_end)
                    }
                    (MaybeAuto::Specified(margin_start), MaybeAuto::Specified(margin_end)) => {
                        // Values are over-constrained.
                        // Ignore value for 'inline-end' cos direction is 'ltr'.
                        let sum = inline_start + inline_size + margin_start + margin_end;
                        (inline_start, available_inline_size - sum, inline_size, margin_start, margin_end)
                    }
                }
            }
        };
        ISizeConstraintSolution::for_absolute_flow(inline_start, inline_end, inline_size, margin_inline_start, margin_inline_end)
    }

    /// Calculate used value of inline-size just like we do for inline replaced elements.
    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    _: Au,
                                    layout_context: &LayoutContext)
                                    -> MaybeAuto {
        let containing_block_inline_size =
            block.containing_block_size(layout_context.shared.screen_size).inline;
        let fragment = block.fragment();
        fragment.assign_replaced_inline_size_if_necessary(containing_block_inline_size);
        // For replaced absolute flow, the rest of the constraint solving will
        // take inline-size to be specified as the value computed here.
        MaybeAuto::Specified(fragment.content_inline_size())
    }

    fn containing_block_inline_size(&self, block: &mut BlockFlow, _: Au, ctx: &LayoutContext)
                                    -> Au {
        block.containing_block_size(ctx.shared.screen_size).inline
    }

    fn set_flow_x_coord_if_necessary(&self,
                                     block: &mut BlockFlow,
                                     solution: ISizeConstraintSolution) {
        // Set the x-coordinate of the absolute flow wrt to its containing block.
        block.base.position.start.i = solution.inline_start;
    }
}

impl ISizeAndMarginsComputer for BlockNonReplaced {
    /// Compute inline-start and inline-end margins and inline-size.
    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        self.solve_block_inline_size_constraints(block, input)
    }
}

impl ISizeAndMarginsComputer for BlockReplaced {
    /// Compute inline-start and inline-end margins and inline-size.
    ///
    /// ISize has already been calculated. We now calculate the margins just
    /// like for non-replaced blocks.
    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        match input.computed_inline_size {
            MaybeAuto::Specified(_) => {},
            MaybeAuto::Auto => panic!("BlockReplaced: inline_size should have been computed by now")
        };
        self.solve_block_inline_size_constraints(block, input)
    }

    /// Calculate used value of inline-size just like we do for inline replaced elements.
    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    _: &LayoutContext)
                                    -> MaybeAuto {
        let fragment = block.fragment();
        fragment.assign_replaced_inline_size_if_necessary(parent_flow_inline_size);
        // For replaced block flow, the rest of the constraint solving will
        // take inline-size to be specified as the value computed here.
        MaybeAuto::Specified(fragment.content_inline_size())
    }

}

impl ISizeAndMarginsComputer for FloatNonReplaced {
    /// CSS Section 10.3.5
    ///
    /// If inline-size is computed as 'auto', the used value is the 'shrink-to-fit' inline-size.
    fn solve_inline_size_constraints(&self,
                               block: &mut BlockFlow,
                               input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        let (computed_inline_size, inline_start_margin, inline_end_margin, available_inline_size) = (input.computed_inline_size,
                                                                            input.inline_start_margin,
                                                                            input.inline_end_margin,
                                                                            input.available_inline_size);
        let margin_inline_start = inline_start_margin.specified_or_zero();
        let margin_inline_end = inline_end_margin.specified_or_zero();
        let available_inline_size_float = available_inline_size - margin_inline_start - margin_inline_end;
        let shrink_to_fit = block.get_shrink_to_fit_inline_size(available_inline_size_float);
        let inline_size = computed_inline_size.specified_or_default(shrink_to_fit);
        debug!("assign_inline_sizes_float -- inline_size: {:?}", inline_size);
        ISizeConstraintSolution::new(inline_size, margin_inline_start, margin_inline_end)
    }
}

impl ISizeAndMarginsComputer for FloatReplaced {
    /// CSS Section 10.3.5
    ///
    /// If inline-size is computed as 'auto', the used value is the 'shrink-to-fit' inline-size.
    fn solve_inline_size_constraints(&self, _: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        let (computed_inline_size, inline_start_margin, inline_end_margin) = (input.computed_inline_size,
                                                           input.inline_start_margin,
                                                           input.inline_end_margin);
        let margin_inline_start = inline_start_margin.specified_or_zero();
        let margin_inline_end = inline_end_margin.specified_or_zero();
        let inline_size = match computed_inline_size {
            MaybeAuto::Specified(w) => w,
            MaybeAuto::Auto => panic!("FloatReplaced: inline_size should have been computed by now")
        };
        debug!("assign_inline_sizes_float -- inline_size: {:?}", inline_size);
        ISizeConstraintSolution::new(inline_size, margin_inline_start, margin_inline_end)
    }

    /// Calculate used value of inline-size just like we do for inline replaced elements.
    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    _: &LayoutContext)
                                    -> MaybeAuto {
        let fragment = block.fragment();
        fragment.assign_replaced_inline_size_if_necessary(parent_flow_inline_size);
        // For replaced block flow, the rest of the constraint solving will
        // take inline-size to be specified as the value computed here.
        MaybeAuto::Specified(fragment.content_inline_size())
    }
}

fn propagate_column_inline_sizes_to_child(
        kid: &mut Flow,
        child_index: uint,
        content_inline_size: Au,
        column_computed_inline_sizes: &[ColumnComputedInlineSize],
        inline_start_margin_edge: &mut Au) {
    // If kid is table_rowgroup or table_row, the column inline-sizes info should be copied from
    // its parent.
    //
    // FIXME(pcwalton): This seems inefficient. Reference count it instead?
    let inline_size = if kid.is_table() || kid.is_table_rowgroup() || kid.is_table_row() {
        *kid.column_computed_inline_sizes() =
            column_computed_inline_sizes.iter().map(|&x| x).collect();

        // ISize of kid flow is our content inline-size.
        content_inline_size
    } else if kid.is_table_cell() {
        column_computed_inline_sizes[child_index].size
    } else {
        // ISize of kid flow is our content inline-size.
        content_inline_size
    };

    {
        let kid_base = flow::mut_base(kid);
        kid_base.position.start.i = *inline_start_margin_edge;
        kid_base.block_container_inline_size = inline_size;
    }

    if kid.is_table_cell() {
        *inline_start_margin_edge = *inline_start_margin_edge + inline_size
    }
}
