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

#![deny(unsafe_code)]

use app_units::{Au, MAX_AU};
use context::LayoutContext;
use display_list_builder::{BorderPaintingMode, DisplayListBuildState, FragmentDisplayListBuilding};
use display_list_builder::BlockFlowDisplayListBuilding;
use euclid::{Matrix4D, Point2D, Rect, Size2D};
use floats::{ClearType, FloatKind, Floats, PlacementInfo};
use flow::{self, BaseFlow, EarlyAbsolutePositionInfo, Flow, FlowClass, ForceNonfloatedFlag};
use flow::{BLOCK_POSITION_IS_STATIC, CLEARS_LEFT, CLEARS_RIGHT};
use flow::{CONTAINS_TEXT_OR_REPLACED_FRAGMENTS, INLINE_POSITION_IS_STATIC};
use flow::{FragmentationContext, MARGINS_CANNOT_COLLAPSE, PreorderFlowTraversal};
use flow::{ImmutableFlowUtils, LateAbsolutePositionInfo, MutableFlowUtils, OpaqueFlow};
use flow::IS_ABSOLUTELY_POSITIONED;
use flow_list::FlowList;
use fragment::{CoordinateSystem, Fragment, FragmentBorderBoxIterator, Overflow};
use fragment::{IS_INLINE_FLEX_ITEM, IS_BLOCK_FLEX_ITEM};
use gfx::display_list::ClippingRegion;
use gfx_traits::print_tree::PrintTree;
use layout_debug;
use model::{AdjoiningMargins, CollapsibleMargins, IntrinsicISizes, MarginCollapseInfo, MaybeAuto};
use model::{specified, specified_or_none};
use sequential;
use serde::{Serialize, Serializer};
use std::cmp::{max, min};
use std::fmt;
use std::sync::Arc;
use style::computed_values::{border_collapse, box_sizing, display, float, overflow_x, overflow_y};
use style::computed_values::{position, text_align};
use style::context::SharedStyleContext;
use style::logical_geometry::{LogicalPoint, LogicalRect, LogicalSize, WritingMode};
use style::properties::ServoComputedValues;
use style::servo::restyle_damage::{BUBBLE_ISIZES, REFLOW, REFLOW_OUT_OF_FLOW, REPOSITION};
use style::values::computed::{LengthOrPercentageOrNone, LengthOrPercentage};
use style::values::computed::LengthOrPercentageOrAuto;

/// Information specific to floated blocks.
#[derive(Clone, Serialize)]
pub struct FloatedBlockInfo {
    /// The amount of inline size that is available for the float.
    pub containing_inline_size: Au,

    /// The float ceiling, relative to `BaseFlow::position::cur_b` (i.e. the top part of the border
    /// box).
    pub float_ceiling: Au,

    /// Left or right?
    pub float_kind: FloatKind,
}

impl FloatedBlockInfo {
    pub fn new(float_kind: FloatKind) -> FloatedBlockInfo {
        FloatedBlockInfo {
            containing_inline_size: Au(0),
            float_ceiling: Au(0),
            float_kind: float_kind,
        }
    }
}

/// The solutions for the block-size-and-margins constraint equation.
#[derive(Copy, Clone)]
struct BSizeConstraintSolution {
    block_start: Au,
    block_size: Au,
    margin_block_start: Au,
    margin_block_end: Au
}

impl BSizeConstraintSolution {
    fn new(block_start: Au,
           block_size: Au,
           margin_block_start: Au,
           margin_block_end: Au)
           -> BSizeConstraintSolution {
        BSizeConstraintSolution {
            block_start: block_start,
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
                                                  available_block_size: Au)
                                                  -> BSizeConstraintSolution {
        let (block_start, block_size, margin_block_start, margin_block_end) =
            match (block_start, block_end, block_size) {
                (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Auto) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    // Now it is the same situation as block-start Specified and block-end
                    // and block-size Auto.
                    let block_size = content_block_size;
                    // Use a dummy value for `block_start`, since it has the static position.
                    (Au(0), block_size, margin_block_start, margin_block_end)
                }
                (MaybeAuto::Specified(block_start),
                 MaybeAuto::Specified(block_end),
                 MaybeAuto::Specified(block_size)) => {
                    match (block_start_margin, block_end_margin) {
                        (MaybeAuto::Auto, MaybeAuto::Auto) => {
                            let total_margin_val =
                                available_block_size - block_start - block_end - block_size;
                            (block_start,
                             block_size,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                        (MaybeAuto::Specified(margin_block_start), MaybeAuto::Auto) => {
                            let sum = block_start + block_end + block_size + margin_block_start;
                            (block_start,
                             block_size,
                             margin_block_start,
                             available_block_size - sum)
                        }
                        (MaybeAuto::Auto, MaybeAuto::Specified(margin_block_end)) => {
                            let sum = block_start + block_end + block_size + margin_block_end;
                            (block_start, block_size, available_block_size - sum, margin_block_end)
                        }
                        (MaybeAuto::Specified(margin_block_start),
                         MaybeAuto::Specified(margin_block_end)) => {
                            // Values are over-constrained. Ignore value for 'block-end'.
                            (block_start, block_size, margin_block_start, margin_block_end)
                        }
                    }
                }

                // For the rest of the cases, auto values for margin are set to 0

                // If only one is Auto, solve for it
                (MaybeAuto::Auto,
                 MaybeAuto::Specified(block_end),
                 MaybeAuto::Specified(block_size)) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    let sum = block_end + block_size + margin_block_start + margin_block_end;
                    (available_block_size - sum, block_size, margin_block_start, margin_block_end)
                }
                (MaybeAuto::Specified(block_start),
                 MaybeAuto::Auto,
                 MaybeAuto::Specified(block_size)) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    (block_start, block_size, margin_block_start, margin_block_end)
                }
                (MaybeAuto::Specified(block_start),
                 MaybeAuto::Specified(block_end),
                 MaybeAuto::Auto) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    let sum = block_start + block_end + margin_block_start + margin_block_end;
                    (block_start, available_block_size - sum, margin_block_start, margin_block_end)
                }

                // If block-size is auto, then block-size is content block-size. Solve for the
                // non-auto value.
                (MaybeAuto::Specified(block_start), MaybeAuto::Auto, MaybeAuto::Auto) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    let block_size = content_block_size;
                    (block_start, block_size, margin_block_start, margin_block_end)
                }
                (MaybeAuto::Auto, MaybeAuto::Specified(block_end), MaybeAuto::Auto) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    let block_size = content_block_size;
                    let sum = block_end + block_size + margin_block_start + margin_block_end;
                    (available_block_size - sum, block_size, margin_block_start, margin_block_end)
                }

                (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Specified(block_size)) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    // Use a dummy value for `block_start`, since it has the static position.
                    (Au(0), block_size, margin_block_start, margin_block_end)
                }
            };

        BSizeConstraintSolution::new(block_start, block_size, margin_block_start, margin_block_end)
    }

    /// Solve the vertical constraint equation for absolute replaced elements.
    ///
    /// Assumption: The used value for block-size has already been calculated.
    ///
    /// CSS Section 10.6.5
    /// Constraint equation:
    /// block-start + block-end + block-size + margin-block-start + margin-block-end
    /// = absolute containing block block-size - (vertical padding and border)
    /// [aka available block-size]
    ///
    /// Return the solution for the equation.
    fn solve_vertical_constraints_abs_replaced(block_size: Au,
                                               block_start_margin: MaybeAuto,
                                               block_end_margin: MaybeAuto,
                                               block_start: MaybeAuto,
                                               block_end: MaybeAuto,
                                               _: Au,
                                               available_block_size: Au)
                                               -> BSizeConstraintSolution {
        let (block_start, block_size, margin_block_start, margin_block_end) =
            match (block_start, block_end) {
                (MaybeAuto::Auto, MaybeAuto::Auto) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    // Use a dummy value for `block_start`, since it has the static position.
                    (Au(0), block_size, margin_block_start, margin_block_end)
                }
                (MaybeAuto::Specified(block_start), MaybeAuto::Specified(block_end)) => {
                    match (block_start_margin, block_end_margin) {
                        (MaybeAuto::Auto, MaybeAuto::Auto) => {
                            let total_margin_val = available_block_size - block_start - block_end -
                                block_size;
                            (block_start,
                             block_size,
                             total_margin_val.scale_by(0.5),
                             total_margin_val.scale_by(0.5))
                        }
                        (MaybeAuto::Specified(margin_block_start), MaybeAuto::Auto) => {
                            let sum = block_start + block_end + block_size + margin_block_start;
                            (block_start,
                             block_size,
                             margin_block_start,
                             available_block_size - sum)
                        }
                        (MaybeAuto::Auto, MaybeAuto::Specified(margin_block_end)) => {
                            let sum = block_start + block_end + block_size + margin_block_end;
                            (block_start, block_size, available_block_size - sum, margin_block_end)
                        }
                        (MaybeAuto::Specified(margin_block_start),
                         MaybeAuto::Specified(margin_block_end)) => {
                            // Values are over-constrained. Ignore value for 'block-end'.
                            (block_start, block_size, margin_block_start, margin_block_end)
                        }
                    }
                }

                // If only one is Auto, solve for it
                (MaybeAuto::Auto, MaybeAuto::Specified(block_end)) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    let sum = block_end + block_size + margin_block_start + margin_block_end;
                    (available_block_size - sum, block_size, margin_block_start, margin_block_end)
                }
                (MaybeAuto::Specified(block_start), MaybeAuto::Auto) => {
                    let margin_block_start = block_start_margin.specified_or_zero();
                    let margin_block_end = block_end_margin.specified_or_zero();
                    (block_start, block_size, margin_block_start, margin_block_end)
                }
            };
        BSizeConstraintSolution::new(block_start, block_size, margin_block_start, margin_block_end)
    }
}

/// Performs block-size calculations potentially multiple times, taking
/// (assuming an horizontal writing mode) `height`, `min-height`, and `max-height`
/// into account. After each call to `next()`, the caller must call `.try()` with the
/// current calculated value of `height`.
///
/// See CSS 2.1 § 10.7.
pub struct CandidateBSizeIterator {
    block_size: MaybeAuto,
    max_block_size: Option<Au>,
    min_block_size: Au,
    pub candidate_value: Au,
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
            (LengthOrPercentageOrAuto::Calc(calc), Some(block_container_block_size)) => {
                MaybeAuto::Specified(calc.length() + block_container_block_size.scale_by(calc.percentage()))
            }
            (LengthOrPercentageOrAuto::Percentage(_), None) |
            (LengthOrPercentageOrAuto::Auto, _) |
            (LengthOrPercentageOrAuto::Calc(_), _) => MaybeAuto::Auto,
            (LengthOrPercentageOrAuto::Length(length), _) => MaybeAuto::Specified(length),
        };
        let max_block_size = match (fragment.style.max_block_size(), block_container_block_size) {
            (LengthOrPercentageOrNone::Percentage(percent), Some(block_container_block_size)) => {
                Some(block_container_block_size.scale_by(percent))
            }
            (LengthOrPercentageOrNone::Calc(calc), Some(block_container_block_size)) => {
                Some(block_container_block_size.scale_by(calc.percentage()) + calc.length())
            }
            (LengthOrPercentageOrNone::Calc(_), _) |
            (LengthOrPercentageOrNone::Percentage(_), None) |
            (LengthOrPercentageOrNone::None, _) => None,
            (LengthOrPercentageOrNone::Length(length), _) => Some(length),
        };
        let min_block_size = match (fragment.style.min_block_size(), block_container_block_size) {
            (LengthOrPercentage::Percentage(percent), Some(block_container_block_size)) => {
                block_container_block_size.scale_by(percent)
            }
            (LengthOrPercentage::Calc(calc), Some(block_container_block_size)) => {
                calc.length() + block_container_block_size.scale_by(calc.percentage())
            }
            (LengthOrPercentage::Calc(calc), None) => calc.length(),
            (LengthOrPercentage::Percentage(_), None) => Au(0),
            (LengthOrPercentage::Length(length), _) => length,
        };

        // If the style includes `box-sizing: border-box`, subtract the border and padding.
        let adjustment_for_box_sizing = match fragment.style.get_position().box_sizing {
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
                    _ if self.candidate_value < self.min_block_size => {
                        CandidateBSizeIteratorStatus::TryingMin
                    }
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
pub struct AbsoluteAssignBSizesTraversal<'a>(pub &'a SharedStyleContext);

impl<'a> PreorderFlowTraversal for AbsoluteAssignBSizesTraversal<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        {
            // The root of the absolute flow tree is definitely not absolutely
            // positioned. Nothing to process here.
            let flow: &Flow = flow;
            if flow.contains_roots_of_absolute_flow_tree() {
                return;
            }
            if !flow.is_block_like() {
                return
            }
        }

        let block = flow.as_mut_block();
        debug_assert!(block.base.flags.contains(IS_ABSOLUTELY_POSITIONED));
        if !block.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
            return
        }

        block.calculate_absolute_block_size_and_margins(self.0);
    }
}

pub enum BlockType {
    Replaced,
    NonReplaced,
    AbsoluteReplaced,
    AbsoluteNonReplaced,
    FloatReplaced,
    FloatNonReplaced,
    InlineBlockReplaced,
    InlineBlockNonReplaced,
    InlineFlexItem,
}

#[derive(Clone, PartialEq)]
pub enum MarginsMayCollapseFlag {
    MarginsMayCollapse,
    MarginsMayNotCollapse,
}

#[derive(PartialEq)]
pub enum FormattingContextType {
    None,
    Block,
    Other,
}

// A block formatting context.
#[derive(Serialize)]
pub struct BlockFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// The associated fragment.
    pub fragment: Fragment,

    /// Additional floating flow members.
    pub float: Option<Box<FloatedBlockInfo>>,

    /// Various flags.
    flags: BlockFlowFlags,
}

bitflags! {
    flags BlockFlowFlags: u8 {
        #[doc = "If this is set, then this block flow is the root flow."]
        const IS_ROOT = 0b0000_0001,
        #[doc = "If this is set, then this block flow has overflow and it will scroll."]
        const HAS_SCROLLING_OVERFLOW = 0b0000_0010,
    }
}

impl Serialize for BlockFlowFlags {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bits().serialize(serializer)
    }
}

impl BlockFlow {
    pub fn from_fragment(fragment: Fragment) -> BlockFlow {
        BlockFlow::from_fragment_and_float_kind(fragment, None)
    }

    pub fn from_fragment_and_float_kind(fragment: Fragment, float_kind: Option<FloatKind>)
                                        -> BlockFlow {
        let writing_mode = fragment.style().writing_mode;
        BlockFlow {
            base: BaseFlow::new(Some(fragment.style()), writing_mode, match float_kind {
                Some(_) => ForceNonfloatedFlag::FloatIfNecessary,
                None => ForceNonfloatedFlag::ForceNonfloated,
            }),
            fragment: fragment,
            float: float_kind.map(|kind| box FloatedBlockInfo::new(kind)),
            flags: BlockFlowFlags::empty(),
        }
    }

    /// Return the type of this block.
    ///
    /// This determines the algorithm used to calculate inline-size, block-size, and the
    /// relevant margins for this Block.
    pub fn block_type(&self) -> BlockType {
        if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            if self.fragment.is_replaced() {
                BlockType::AbsoluteReplaced
            } else {
                BlockType::AbsoluteNonReplaced
            }
        } else if self.is_inline_flex_item() {
            BlockType::InlineFlexItem
        } else if self.base.flags.is_float() {
            if self.fragment.is_replaced() {
                BlockType::FloatReplaced
            } else {
                BlockType::FloatNonReplaced
            }
        } else if self.is_inline_block_or_inline_flex() {
            if self.fragment.is_replaced() {
                BlockType::InlineBlockReplaced
            } else {
                BlockType::InlineBlockNonReplaced
            }
        } else {
            if self.fragment.is_replaced() {
                BlockType::Replaced
            } else {
                BlockType::NonReplaced
            }
        }
    }

    /// Compute the actual inline size and position for this block.
    pub fn compute_used_inline_size(&mut self,
                                    shared_context: &SharedStyleContext,
                                    containing_block_inline_size: Au) {
        let block_type = self.block_type();
        match block_type {
            BlockType::AbsoluteReplaced => {
                let inline_size_computer = AbsoluteReplaced;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
            BlockType::AbsoluteNonReplaced => {
                let inline_size_computer = AbsoluteNonReplaced;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
            BlockType::FloatReplaced => {
                let inline_size_computer = FloatReplaced;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
            BlockType::FloatNonReplaced => {
                let inline_size_computer = FloatNonReplaced;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
            BlockType::InlineBlockReplaced => {
                let inline_size_computer = InlineBlockReplaced;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
            BlockType::InlineBlockNonReplaced => {
                let inline_size_computer = InlineBlockNonReplaced;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
            BlockType::Replaced => {
                let inline_size_computer = BlockReplaced;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
            BlockType::NonReplaced => {
                let inline_size_computer = BlockNonReplaced;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
            BlockType::InlineFlexItem => {
                let inline_size_computer = InlineFlexItem;
                inline_size_computer.compute_used_inline_size(self,
                                                              shared_context,
                                                              containing_block_inline_size);
            }
        }
    }

    /// Return this flow's fragment.
    pub fn fragment(&mut self) -> &mut Fragment {
        &mut self.fragment
    }

    /// Return the size of the containing block for the given immediate absolute descendant of this
    /// flow.
    ///
    /// Right now, this only gets the containing block size for absolutely positioned elements.
    /// Note: We assume this is called in a top-down traversal, so it is ok to reference the CB.
    #[inline]
    pub fn containing_block_size(&self, viewport_size: &Size2D<Au>, descendant: OpaqueFlow)
                                 -> LogicalSize<Au> {
        debug_assert!(self.base.flags.contains(IS_ABSOLUTELY_POSITIONED));
        if self.is_fixed() {
            // Initial containing block is the CB for the root
            LogicalSize::from_physical(self.base.writing_mode, *viewport_size)
        } else {
            self.base.absolute_cb.generated_containing_block_size(descendant)
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
    fn adjust_fragments_for_collapsed_margins_if_root(&mut self,
                                                      shared_context: &SharedStyleContext) {
        if !self.is_root() {
            return
        }

        let (block_start_margin_value, block_end_margin_value) =
            match self.base.collapsible_margins {
                CollapsibleMargins::CollapseThrough(_) => {
                    panic!("Margins unexpectedly collapsed through root flow.")
                }
                CollapsibleMargins::Collapse(block_start_margin, block_end_margin) => {
                    (block_start_margin.collapse(), block_end_margin.collapse())
                }
                CollapsibleMargins::None(block_start, block_end) => (block_start, block_end),
            };

        // Shift all kids down (or up, if margins are negative) if necessary.
        if block_start_margin_value != Au(0) {
            for kid in self.base.child_iter_mut() {
                let kid_base = flow::mut_base(kid);
                kid_base.position.start.b = kid_base.position.start.b + block_start_margin_value
            }
        }

        // FIXME(#2003, pcwalton): The max is taken here so that you can scroll the page, but this
        // is not correct behavior according to CSS 2.1 § 10.5. Instead I think we should treat the
        // root element as having `overflow: scroll` and use the layers-based scrolling
        // infrastructure to make it scrollable.
        let viewport_size =
            LogicalSize::from_physical(self.fragment.style.writing_mode,
                                       shared_context.viewport_size());
        let block_size = max(viewport_size.block,
                             self.fragment.border_box.size.block + block_start_margin_value +
                             block_end_margin_value);

        self.base.position.size.block = block_size;
        self.fragment.border_box.size.block = block_size;
    }

    // FIXME: Record enough info to deal with fragmented decorations.
    // See https://drafts.csswg.org/css-break/#break-decoration
    // For borders, this might be `enum FragmentPosition { First, Middle, Last }`
    fn clone_with_children(&self, new_children: FlowList) -> BlockFlow {
        BlockFlow {
            base: self.base.clone_with_children(new_children),
            fragment: self.fragment.clone(),
            float: self.float.clone(),
            ..*self
        }
    }

    /// Writes in the size of the relative containing block for children. (This information
    /// is also needed to handle RTL.)
    fn propagate_early_absolute_position_info_to_children(&mut self) {
        for kid in self.base.child_iter_mut() {
            flow::mut_base(kid).early_absolute_position_info = EarlyAbsolutePositionInfo {
                relative_containing_block_size: self.fragment.content_box().size,
                relative_containing_block_mode: self.fragment.style().writing_mode,
            }
        }
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
    /// When `fragmentation_context` is given (not `None`), this should fit as much of the content
    /// as possible within the available block size.
    /// If there is more content (that doesn’t fit), this flow is *fragmented*
    /// with the extra content moved to another fragment (a flow like this one) which is returned.
    /// See `Flow::fragment`.
    ///
    /// The return value is always `None` when `fragmentation_context` is `None`.
    ///
    /// `inline(always)` because this is only ever called by in-order or non-in-order top-level
    /// methods.
    #[inline(always)]
    pub fn assign_block_size_block_base(&mut self,
                                        layout_context: &LayoutContext,
                                        mut fragmentation_context: Option<FragmentationContext>,
                                        margins_may_collapse: MarginsMayCollapseFlag)
                                        -> Option<Arc<Flow>> {
        let _scope = layout_debug_scope!("assign_block_size_block_base {:x}",
                                         self.base.debug_id());

        let mut break_at = None;
        let content_box = self.fragment.content_box();
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
            let writing_mode = self.base.floats.writing_mode;
            self.base.floats.translate(LogicalSize::new(
                writing_mode, -self.fragment.inline_start_offset(), Au(0)));

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
            let thread_id = self.base.thread_id;
            let (mut had_floated_children, mut had_children_with_clearance) = (false, false);
            for (child_index, kid) in self.base.child_iter_mut().enumerate() {
                if flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED) {
                    // Assume that the *hypothetical box* for an absolute flow starts immediately
                    // after the margin-end border edge of the previous flow.
                    if flow::base(kid).flags.contains(BLOCK_POSITION_IS_STATIC) {
                        let previous_bottom_margin = margin_collapse_info.current_float_ceiling();

                        flow::mut_base(kid).position.start.b = cur_b +
                            flow::base(kid).collapsible_margins
                                           .block_start_margin_for_noncollapsible_context() +
                            previous_bottom_margin
                    }
                    kid.place_float_if_applicable();
                    if !flow::base(kid).flags.is_float() {
                        kid.assign_block_size_for_inorder_child_if_necessary(layout_context,
                                                                             thread_id,
                                                                             content_box);
                    }

                    // Skip the collapsing and float processing for absolute flow kids and continue
                    // with the next flow.
                    continue
                }

                let previous_b = cur_b;
                if let Some(ctx) = fragmentation_context {
                    let child_ctx = FragmentationContext {
                        available_block_size: ctx.available_block_size - cur_b,
                        this_fragment_is_empty: ctx.this_fragment_is_empty,
                    };
                    if let Some(remaining) = kid.fragment(layout_context, Some(child_ctx)) {
                        break_at = Some((child_index + 1, Some(remaining)));
                    }
                }

                // Assign block-size now for the child if it might have floats in and we couldn't
                // before.
                flow::mut_base(kid).floats = floats.clone();
                if flow::base(kid).flags.is_float() {
                    had_floated_children = true;
                    flow::mut_base(kid).position.start.b = cur_b;
                    {
                        let kid_block = kid.as_mut_block();
                        let float_ceiling = margin_collapse_info.current_float_ceiling();
                        kid_block.float.as_mut().unwrap().float_ceiling = float_ceiling
                    }
                    kid.place_float_if_applicable();

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
                    kid.assign_block_size_for_inorder_child_if_necessary(layout_context,
                                                                         thread_id,
                                                                         content_box);

                if !had_children_with_clearance &&
                        floats.is_present() &&
                        (flow::base(kid).flags.contains(CLEARS_LEFT) ||
                         flow::base(kid).flags.contains(CLEARS_RIGHT)) {
                    had_children_with_clearance = true
                }

                // Handle any (possibly collapsed) top margin.
                let delta = margin_collapse_info.advance_block_start_margin(
                    &flow::base(kid).collapsible_margins,
                    !had_children_with_clearance);
                translate_including_floats(&mut cur_b, delta, &mut floats);

                // Collapse-through margins should be placed at the top edge,
                // so we'll handle the delta after the bottom margin is processed
                if let CollapsibleMargins::CollapseThrough(_) = flow::base(kid).collapsible_margins {
                    cur_b = cur_b - delta;
                }

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

                // Collapse-through margin should be placed at the top edge of the flow.
                let collapse_delta = match kid_base.collapsible_margins {
                    CollapsibleMargins::CollapseThrough(_) => {
                        let delta = margin_collapse_info.current_float_ceiling();
                        cur_b = cur_b + delta;
                        kid_base.position.start.b = kid_base.position.start.b + delta;
                        delta
                    }
                    _ => Au(0)
                };

                if break_at.is_some() {
                    break
                }

                if let Some(ref mut ctx) = fragmentation_context {
                    if cur_b > ctx.available_block_size && !ctx.this_fragment_is_empty {
                        break_at = Some((child_index, None));
                        cur_b = previous_b;
                        break
                    }
                    ctx.this_fragment_is_empty = false
                }

                // For consecutive collapse-through flows, their top margin should be calculated
                // from the same baseline.
                cur_b = cur_b - collapse_delta;
            }

            // Add in our block-end margin and compute our collapsible margins.
            let can_collapse_block_end_margin_with_kids =
                margins_may_collapse == MarginsMayCollapseFlag::MarginsMayCollapse &&
                !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) &&
                self.fragment.border_padding.block_end == Au(0);
            let (collapsible_margins, delta) =
                margin_collapse_info.finish_and_compute_collapsible_margins(
                &self.fragment,
                self.base.block_container_explicit_block_size,
                can_collapse_block_end_margin_with_kids,
                !had_floated_children);
            self.base.collapsible_margins = collapsible_margins;
            translate_including_floats(&mut cur_b, delta, &mut floats);

            let mut block_size = cur_b - block_start_offset;
            let is_root = self.is_root();

            if is_root || self.formatting_context_type() != FormattingContextType::None ||
                    self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                // The content block-size includes all the floats per CSS 2.1 § 10.6.7. The easiest
                // way to handle this is to just treat it as clearance.
                block_size = block_size + floats.clearance(ClearType::Both);
            }

            if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                // FIXME(#2003, pcwalton): The max is taken here so that you can scroll the page,
                // but this is not correct behavior according to CSS 2.1 § 10.5. Instead I think we
                // should treat the root element as having `overflow: scroll` and use the layers-
                // based scrolling infrastructure to make it scrollable.
                if is_root {
                    let viewport_size =
                        LogicalSize::from_physical(self.fragment.style.writing_mode,
                                                   layout_context.shared_context().viewport_size());
                    block_size = max(viewport_size.block, block_size)
                }

                // Store the content block-size for use in calculating the absolute flow's
                // dimensions later.
                //
                // FIXME(pcwalton): This looks not idempotent. Is it?
                self.fragment.border_box.size.block = block_size;
            }


            if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                self.propagate_early_absolute_position_info_to_children();
                return None
            }

            // Compute any explicitly-specified block size.
            // Can't use `for` because we assign to `candidate_block_size_iterator.candidate_value`.
            let mut candidate_block_size_iterator = CandidateBSizeIterator::new(
                &self.fragment,
                self.base.block_container_explicit_block_size);
            while let Some(candidate_block_size) = candidate_block_size_iterator.next() {
                candidate_block_size_iterator.candidate_value =
                    match candidate_block_size {
                        MaybeAuto::Auto => block_size,
                        MaybeAuto::Specified(value) => value
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
            self.base.position.size.block = cur_b;

            self.propagate_early_absolute_position_info_to_children();

            // Translate the current set of floats back into the parent coordinate system in the
            // inline direction, and store them in the flow so that flows that come later in the
            // document can access them.
            floats.translate(LogicalSize::new(writing_mode,
                                              self.fragment.inline_start_offset(),
                                              Au(0)));
            self.base.floats = floats.clone();
            self.adjust_fragments_for_collapsed_margins_if_root(layout_context.shared_context());
        } else {
            // We don't need to reflow, but we still need to perform in-order traversals if
            // necessary.
            let thread_id = self.base.thread_id;
            for kid in self.base.child_iter_mut() {
                kid.assign_block_size_for_inorder_child_if_necessary(layout_context,
                                                                     thread_id,
                                                                     content_box);
            }
        }

        if (&*self as &Flow).contains_roots_of_absolute_flow_tree() {
            // Assign block-sizes for all flows in this absolute flow tree.
            // This is preorder because the block-size of an absolute flow may depend on
            // the block-size of its containing block, which may also be an absolute flow.
            (&mut *self as &mut Flow).traverse_preorder_absolute_flows(
                &mut AbsoluteAssignBSizesTraversal(layout_context.shared_context()));
        }

        // Don't remove the dirty bits yet if we're absolutely-positioned, since our final size
        // has not been calculated yet. (See `calculate_absolute_block_size_and_margins` for that.)
        // Also don't remove the dirty bits if we're a block formatting context since our inline
        // size has not yet been computed. (See `assign_inline_position_for_formatting_context()`.)
        if (self.base.flags.is_float() ||
                self.formatting_context_type() == FormattingContextType::None) &&
                !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
            self.fragment.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
        }

        break_at.and_then(|(i, child_remaining)| {
            if i == self.base.children.len() && child_remaining.is_none() {
                None
            } else {
                let mut children = self.base.children.split_off(i);
                if let Some(child) = child_remaining {
                    children.push_front_arc(child);
                }
                Some(Arc::new(self.clone_with_children(children)) as Arc<Flow>)
            }
        })
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

        // Our `position` field accounts for positive margins, but not negative margins. (See
        // calculation of `extra_inline_size_from_margin` below.) Negative margins must be taken
        // into account for float placement, however. So we add them in here.
        let inline_size_for_float_placement = self.base.position.size.inline +
            min(Au(0), self.fragment.margin.inline_start_end());

        let info = PlacementInfo {
            size: LogicalSize::new(
                self.fragment.style.writing_mode,
                inline_size_for_float_placement,
                block_size + self.fragment.margin.block_start_end())
                      .convert(self.fragment.style.writing_mode, self.base.floats.writing_mode),
            ceiling: clearance + float_info.float_ceiling,
            max_inline_size: float_info.containing_inline_size,
            kind: float_info.float_kind,
        };

        // Place the float and return the `Floats` back to the parent flow.
        // After, grab the position and use that to set our position.
        self.base.floats.add_float(&info);

        // FIXME (mbrubeck) Get the correct container size for self.base.floats;
        let container_size = Size2D::new(self.base.block_container_inline_size, Au(0));

        // Move in from the margin edge, as per CSS 2.1 § 9.5, floats may not overlap anything on
        // their margin edges.
        let float_offset = self.base.floats.last_float_pos().unwrap()
                                           .convert(self.base.floats.writing_mode,
                                                    self.base.writing_mode,
                                                    container_size)
                                           .start;
        let margin_offset = LogicalPoint::new(self.base.writing_mode,
                                              Au(0),
                                              self.fragment.margin.block_start);

        let mut origin = LogicalPoint::new(self.base.writing_mode,
                                           self.base.position.start.i,
                                           self.base.position.start.b);
        origin = origin.add_point(&float_offset).add_point(&margin_offset);
        self.base.position = LogicalRect::from_point_size(self.base.writing_mode,
                                                          origin,
                                                          self.base.position.size);
    }

    pub fn explicit_block_containing_size(&self, shared_context: &SharedStyleContext) -> Option<Au> {
        if self.is_root() || self.is_fixed() {
            let viewport_size = LogicalSize::from_physical(self.fragment.style.writing_mode,
                                                           shared_context.viewport_size());
            Some(viewport_size.block)
        } else if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) &&
                  self.base.block_container_explicit_block_size.is_none() {
            self.base.absolute_cb.explicit_block_containing_size(shared_context)
        } else {
            self.base.block_container_explicit_block_size
        }
    }

    pub fn explicit_block_size(&self, containing_block_size: Option<Au>) -> Option<Au> {
        let content_block_size = self.fragment.style().content_block_size();

        match (content_block_size, containing_block_size) {
            (LengthOrPercentageOrAuto::Calc(calc), Some(container_size)) => {
                Some(container_size.scale_by(calc.percentage()) + calc.length())
            }
            (LengthOrPercentageOrAuto::Length(length), _) => Some(length),
            (LengthOrPercentageOrAuto::Percentage(percent), Some(container_size)) => {
                Some(container_size.scale_by(percent))
            }
            (LengthOrPercentageOrAuto::Percentage(_), None) |
            (LengthOrPercentageOrAuto::Calc(_), None) |
            (LengthOrPercentageOrAuto::Auto, None) => {
                None
            }
            (LengthOrPercentageOrAuto::Auto, Some(container_size)) => {
                let (block_start, block_end) = {
                    let position = self.fragment.style().logical_position();
                    (MaybeAuto::from_style(position.block_start, container_size),
                     MaybeAuto::from_style(position.block_end, container_size))
                };

                match (block_start, block_end) {
                    (MaybeAuto::Specified(block_start), MaybeAuto::Specified(block_end)) => {
                        let available_block_size = container_size - self.fragment.border_padding.block_start_end();

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

                        let margin_block_start = margin_block_start.specified_or_zero();
                        let margin_block_end = margin_block_end.specified_or_zero();
                        let sum = block_start + block_end + margin_block_start + margin_block_end;
                        Some(available_block_size - sum)
                    }

                    (_, _) => {
                        None
                    }
                }
            }
        }
    }

    fn calculate_absolute_block_size_and_margins(&mut self, shared_context: &SharedStyleContext) {
        let opaque_self = OpaqueFlow::from_flow(self);
        let containing_block_block_size =
            self.containing_block_size(&shared_context.viewport_size(), opaque_self).block;

        // This is the stored content block-size value from assign-block-size
        let content_block_size = self.fragment.border_box.size.block;

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
            if self.fragment.is_replaced() {
                // Calculate used value of block-size just like we do for inline replaced elements.
                // TODO: Pass in the containing block block-size when Fragment's
                // assign-block-size can handle it correctly.
                self.fragment.assign_replaced_block_size_if_necessary();
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
                        available_block_size))
            } else {
                let mut candidate_block_size_iterator =
                    CandidateBSizeIterator::new(&self.fragment, Some(containing_block_block_size));

                // Can't use `for` because we assign to
                // `candidate_block_size_iterator.candidate_value`.
                while let Some(block_size_used_val) =  candidate_block_size_iterator.next() {
                    solution = Some(
                        BSizeConstraintSolution::solve_vertical_constraints_abs_nonreplaced(
                            block_size_used_val,
                            margin_block_start,
                            margin_block_end,
                            block_start,
                            block_end,
                            content_block_size,
                            available_block_size));

                    candidate_block_size_iterator.candidate_value =
                        solution.unwrap().block_size;
                }
            }
        }

        let solution = solution.unwrap();
        self.fragment.margin.block_start = solution.margin_block_start;
        self.fragment.margin.block_end = solution.margin_block_end;
        self.fragment.border_box.start.b = Au(0);

        if !self.base.flags.contains(BLOCK_POSITION_IS_STATIC) {
            self.base.position.start.b = solution.block_start + self.fragment.margin.block_start
        }

        let block_size = solution.block_size + self.fragment.border_padding.block_start_end();
        self.fragment.border_box.size.block = block_size;
        self.base.position.size.block = block_size;

        self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
        self.fragment.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
    }

    /// Compute inline size based using the `block_container_inline_size` set by the parent flow.
    ///
    /// This is run in the `AssignISizes` traversal.
    fn propagate_and_compute_used_inline_size(&mut self, shared_context: &SharedStyleContext) {
        let containing_block_inline_size = self.base.block_container_inline_size;
        self.compute_used_inline_size(shared_context, containing_block_inline_size);
        if self.base.flags.is_float() {
            self.float.as_mut().unwrap().containing_inline_size = containing_block_inline_size
        }
    }

    /// Assigns the computed inline-start content edge and inline-size to all the children of this
    /// block flow. The given `callback`, if supplied, will be called once per child; it is
    /// currently used to push down column sizes for tables.
    ///
    /// `#[inline(always)]` because this is called only from block or table inline-size assignment
    /// and the code for block layout is significantly simpler.
    #[inline(always)]
    pub fn propagate_assigned_inline_size_to_children<F>(&mut self,
                                                         shared_context: &SharedStyleContext,
                                                         inline_start_content_edge: Au,
                                                         inline_end_content_edge: Au,
                                                         content_inline_size: Au,
                                                         mut callback: F)
                                                         where F: FnMut(&mut Flow,
                                                                        usize,
                                                                        Au,
                                                                        WritingMode,
                                                                        &mut Au,
                                                                        &mut Au) {
        let flags = self.base.flags.clone();

        let opaque_self = OpaqueFlow::from_flow(self);

        // Calculate non-auto block size to pass to children.
        let box_border = match self.fragment.style().get_position().box_sizing {
            box_sizing::T::border_box => self.fragment.border_padding.block_start_end(),
            box_sizing::T::content_box => Au(0),
        };
        let parent_container_size = self.explicit_block_containing_size(shared_context);
        // https://drafts.csswg.org/css-ui-3/#box-sizing
        let mut explicit_content_size = self
                                    .explicit_block_size(parent_container_size)
                                    .map(|x| if x < box_border { Au(0) } else { x - box_border });
        if self.is_root() { explicit_content_size = max(parent_container_size, explicit_content_size); }
        // Calculate containing block inline size.
        let containing_block_size = if flags.contains(IS_ABSOLUTELY_POSITIONED) {
            self.containing_block_size(&shared_context.viewport_size(), opaque_self).inline
        } else {
            content_inline_size
        };
        // FIXME (mbrubeck): Get correct mode for absolute containing block
        let containing_block_mode = self.base.writing_mode;

        let mut inline_start_margin_edge = inline_start_content_edge;
        let mut inline_end_margin_edge = inline_end_content_edge;

        let mut iterator = self.base.child_iter_mut().enumerate().peekable();
        while let Some((i, kid)) = iterator.next() {
            flow::mut_base(kid).block_container_explicit_block_size = explicit_content_size;

            // The inline-start margin edge of the child flow is at our inline-start content edge,
            // and its inline-size is our content inline-size.
            let kid_mode = flow::base(kid).writing_mode;
            {
                // Don't assign positions to children unless they're going to be reflowed.
                // Otherwise, the position we assign might be incorrect and never fixed up. (Issue
                // #13704.)
                //
                // For instance, floats have their true inline position calculated in
                // `assign_block_size()`, which won't do anything unless `REFLOW` is set. So, if a
                // float child does not have `REFLOW` set, we must be careful to avoid touching its
                // inline position, as no logic will run afterward to set its true value.
                let kid_base = flow::mut_base(kid);
                let reflow_damage = if kid_base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                    REFLOW_OUT_OF_FLOW
                } else {
                    REFLOW
                };
                if kid_base.flags.contains(INLINE_POSITION_IS_STATIC) &&
                        kid_base.restyle_damage.contains(reflow_damage) {
                    kid_base.position.start.i =
                        if kid_mode.is_bidi_ltr() == containing_block_mode.is_bidi_ltr() {
                            inline_start_content_edge
                        } else {
                            // The kid's inline 'start' is at the parent's 'end'
                            inline_end_content_edge
                        };
                }
                kid_base.block_container_inline_size = content_inline_size;
                kid_base.block_container_writing_mode = containing_block_mode;
            }

            // Call the callback to propagate extra inline size information down to the child. This
            // is currently used for tables.
            callback(kid,
                     i,
                     content_inline_size,
                     containing_block_mode,
                     &mut inline_start_margin_edge,
                     &mut inline_end_margin_edge);

            // Per CSS 2.1 § 16.3.1, text alignment propagates to all children in flow.
            //
            // TODO(#2265, pcwalton): Do this in the cascade instead.
            let containing_block_text_align = self.fragment.style().get_inheritedtext().text_align;
            flow::mut_base(kid).flags.set_text_align(containing_block_text_align);

            // Handle `text-indent` on behalf of any inline children that we have. This is
            // necessary because any percentages are relative to the containing block, which only
            // we know.
            if kid.is_inline_flow() {
                kid.as_mut_inline().first_line_indentation =
                    specified(self.fragment.style().get_inheritedtext().text_indent,
                              containing_block_size);
            }
        }
    }

    /// Determines the type of formatting context this is. See the definition of
    /// `FormattingContextType`.
    pub fn formatting_context_type(&self) -> FormattingContextType {
        let style = self.fragment.style();
        if style.get_box().float != float::T::none {
            return FormattingContextType::Other
        }
        match style.get_box().display {
            display::T::table_cell |
            display::T::table_caption |
            display::T::table_row_group |
            display::T::table |
            display::T::inline_block |
            display::T::flex => {
                FormattingContextType::Other
            }
            _ if style.get_box().overflow_x != overflow_x::T::visible ||
                    style.get_box().overflow_y != overflow_y::T(overflow_x::T::visible) ||
                    style.is_multicol() => {
                FormattingContextType::Block
            }
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
    fn assign_inline_position_for_formatting_context(&mut self,
                                                     layout_context: &LayoutContext,
                                                     content_box: LogicalRect<Au>) {
        debug_assert!(self.formatting_context_type() != FormattingContextType::None);

        if !self.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
            return
        }

        // We do this first to avoid recomputing our inline size when we propagate it.
        self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
        self.fragment.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);

        // The code below would completely wreck the layout if run on a flex item, however:
        //   * Flex items are always the children of flex containers.
        //   * Flex containers only contain flex items.
        //   * Floats cannot intrude into flex containers.
        //   * Floats cannot escape flex items.
        //   * Flex items cannot also be floats.
        // Therefore, a flex item cannot be impacted by a float.
        // See also: https://www.w3.org/TR/css-flexbox-1/#flex-containers
        if !self.base.might_have_floats_in() {
            return
        }

        // If you remove the might_have_floats_in conditional, this will go off.
        debug_assert!(!self.is_inline_flex_item());

        // Compute the available space for us, based on the actual floats.
        let rect = self.base.floats.available_rect(Au(0),
                                                   self.fragment.border_box.size.block,
                                                   content_box.size.inline);
        let available_inline_size = if let Some(rect) = rect {
            // Offset our position by whatever displacement is needed to not impact the floats.
            // Also, account for margins sliding behind floats.
            let inline_offset = if self.fragment.margin.inline_start < rect.start.i {
                // Do not do anything for negative margins; those are handled separately.
                rect.start.i - max(Au(0), self.fragment.margin.inline_start)
            } else {
                Au(0)
            };
            self.base.position.start.i = content_box.start.i + inline_offset;
            // Handle the end margin sliding behind the float.
            let end = content_box.size.inline - rect.start.i - rect.size.inline;
            let inline_end_offset = if self.fragment.margin.inline_end < end {
                end - max(Au(0), self.fragment.margin.inline_end)
            } else {
                Au(0)
            };
            content_box.size.inline - inline_offset - inline_end_offset
        } else {
            content_box.size.inline
        } - self.fragment.margin.inline_start_end();
        let max_inline_size = specified_or_none(
            self.fragment.style().max_inline_size(),
            self.base.block_container_inline_size
        ).unwrap_or(MAX_AU);
        let min_inline_size = specified(
            self.fragment.style().min_inline_size(),
            self.base.block_container_inline_size
        );
        let specified_inline_size = self.fragment.style().content_inline_size();
        let container_size = self.base.block_container_inline_size;
        let inline_size =
            if let MaybeAuto::Specified(size) = MaybeAuto::from_style(specified_inline_size,
                                                                      container_size) {
                match self.fragment.style().get_position().box_sizing {
                    box_sizing::T::border_box => size,
                    box_sizing::T::content_box =>
                        size + self.fragment.border_padding.inline_start_end(),
                }
            } else {
                max(min_inline_size, min(available_inline_size, max_inline_size))
            };
        self.base.position.size.inline = inline_size + self.fragment.margin.inline_start_end();

        // If float speculation failed, fixup our layout, and re-layout all the children.
        if self.fragment.margin_box_inline_size() != self.base.position.size.inline {
            debug!("assign_inline_position_for_formatting_context: float speculation failed");
            // Fix-up our own layout.
            // We can't just traverse_flow_tree_preorder ourself, because that would re-run
            // float speculation, instead of acting on the actual results.
            self.fragment.border_box.size.inline = inline_size;
            // Assign final-final inline sizes on all our children.
            self.assign_inline_sizes(layout_context);
            // Re-run layout on our children.
            for child in flow::mut_base(self).children.iter_mut() {
                sequential::traverse_flow_tree_preorder(child, layout_context);
            }
            // Assign our final-final block size.
            self.assign_block_size(layout_context);
        }

        debug_assert_eq!(self.fragment.margin_box_inline_size(), self.base.position.size.inline);
    }

    fn is_inline_block_or_inline_flex(&self) -> bool {
        self.fragment.style().get_box().display == display::T::inline_block ||
        self.fragment.style().get_box().display == display::T::inline_flex
    }

    /// Computes the content portion (only) of the intrinsic inline sizes of this flow. This is
    /// used for calculating shrink-to-fit width. Assumes that intrinsic sizes have already been
    /// computed for this flow.
    fn content_intrinsic_inline_sizes(&self) -> IntrinsicISizes {
        let (border_padding, margin) = self.fragment.surrounding_intrinsic_inline_size();
        IntrinsicISizes {
            minimum_inline_size: self.base.intrinsic_inline_sizes.minimum_inline_size -
                                    border_padding - margin,
            preferred_inline_size: self.base.intrinsic_inline_sizes.preferred_inline_size -
                                    border_padding - margin,
        }
    }

    /// Computes intrinsic inline sizes for a block.
    pub fn bubble_inline_sizes_for_block(&mut self, consult_children: bool) {
        let _scope = layout_debug_scope!("block::bubble_inline_sizes {:x}", self.base.debug_id());

        let mut flags = self.base.flags;
        if self.definitely_has_zero_block_size() {
            // This is kind of a hack for Acid2. But it's a harmless one, because (a) this behavior
            // is unspecified; (b) it matches the behavior one would intuitively expect, since
            // floats don't flow around blocks that take up no space in the block direction.
            flags.remove(CONTAINS_TEXT_OR_REPLACED_FRAGMENTS);
        } else if self.fragment.is_text_or_replaced() {
            flags.insert(CONTAINS_TEXT_OR_REPLACED_FRAGMENTS);
        } else {
            flags.remove(CONTAINS_TEXT_OR_REPLACED_FRAGMENTS);
            for kid in self.base.children.iter() {
                if flow::base(kid).flags.contains(CONTAINS_TEXT_OR_REPLACED_FRAGMENTS) {
                    flags.insert(CONTAINS_TEXT_OR_REPLACED_FRAGMENTS);
                    break
                }
            }
        }

        // Find the maximum inline-size from children.
        //
        // See: https://lists.w3.org/Archives/Public/www-style/2014Nov/0085.html
        //
        // FIXME(pcwalton): This doesn't exactly follow that algorithm at the moment.
        // FIXME(pcwalton): This should consider all float descendants, not just children.
        let mut computation = self.fragment.compute_intrinsic_inline_sizes();
        let (mut left_float_width, mut right_float_width) = (Au(0), Au(0));
        let (mut left_float_width_accumulator, mut right_float_width_accumulator) = (Au(0), Au(0));
        let mut preferred_inline_size_of_children_without_text_or_replaced_fragments = Au(0);
        for kid in self.base.child_iter_mut() {
            if flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED) || !consult_children {
                continue
            }

            let child_base = flow::mut_base(kid);
            let float_kind = child_base.flags.float_kind();
            computation.content_intrinsic_sizes.minimum_inline_size =
                max(computation.content_intrinsic_sizes.minimum_inline_size,
                    child_base.intrinsic_inline_sizes.minimum_inline_size);

            if child_base.flags.contains(CLEARS_LEFT) {
                left_float_width = max(left_float_width, left_float_width_accumulator);
                left_float_width_accumulator = Au(0)
            }
            if child_base.flags.contains(CLEARS_RIGHT) {
                right_float_width = max(right_float_width, right_float_width_accumulator);
                right_float_width_accumulator = Au(0)
            }

            match (float_kind, child_base.flags.contains(CONTAINS_TEXT_OR_REPLACED_FRAGMENTS)) {
                (float::T::none, true) => {
                    computation.content_intrinsic_sizes.preferred_inline_size =
                        max(computation.content_intrinsic_sizes.preferred_inline_size,
                            child_base.intrinsic_inline_sizes.preferred_inline_size);
                }
                (float::T::none, false) => {
                    preferred_inline_size_of_children_without_text_or_replaced_fragments = max(
                        preferred_inline_size_of_children_without_text_or_replaced_fragments,
                        child_base.intrinsic_inline_sizes.preferred_inline_size)
                }
                (float::T::left, _) => {
                    left_float_width_accumulator = left_float_width_accumulator +
                        child_base.intrinsic_inline_sizes.preferred_inline_size;
                }
                (float::T::right, _) => {
                    right_float_width_accumulator = right_float_width_accumulator +
                        child_base.intrinsic_inline_sizes.preferred_inline_size;
                }
            }
        }

        left_float_width = max(left_float_width, left_float_width_accumulator);
        right_float_width = max(right_float_width, right_float_width_accumulator);

        computation.content_intrinsic_sizes.preferred_inline_size =
            computation.content_intrinsic_sizes.preferred_inline_size + left_float_width +
            right_float_width;
        computation.content_intrinsic_sizes.preferred_inline_size =
            max(computation.content_intrinsic_sizes.preferred_inline_size,
                preferred_inline_size_of_children_without_text_or_replaced_fragments);

        self.base.intrinsic_inline_sizes = computation.finish();
        self.base.flags = flags
    }

    pub fn block_stacking_context_type(&self) -> BlockStackingContextType {
        if self.fragment.establishes_stacking_context() {
            return BlockStackingContextType::StackingContext
        }

        if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) ||
                self.fragment.style.get_box().position != position::T::static_ ||
                self.base.flags.is_float() {
            BlockStackingContextType::PseudoStackingContext
        } else {
            BlockStackingContextType::NonstackingContext
        }
    }

    pub fn style_permits_scrolling_overflow(&self) -> bool {
        match (self.fragment.style().get_box().overflow_x,
               self.fragment.style().get_box().overflow_y.0) {
            (overflow_x::T::auto, _) | (overflow_x::T::scroll, _) |
            (_, overflow_x::T::auto) | (_, overflow_x::T::scroll) => true,
            (_, _) => false,
        }
    }

    pub fn compute_inline_sizes(&mut self, shared_context: &SharedStyleContext) {
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

        self.initialize_container_size_for_root(shared_context);

        // Our inline-size was set to the inline-size of the containing block by the flow's parent.
        // Now compute the real value.
        self.propagate_and_compute_used_inline_size(shared_context);

        self.guess_inline_size_for_block_formatting_context_if_necessary()
    }

    /// If this is the root flow, initialize values that would normally be set by the parent.
    ///
    /// Should be called during `assign_inline_sizes` for flows that may be the root.
    pub fn initialize_container_size_for_root(&mut self, shared_context: &SharedStyleContext) {
        if self.is_root() {
            debug!("Setting root position");
            self.base.position.start = LogicalPoint::zero(self.base.writing_mode);
            self.base.block_container_inline_size = LogicalSize::from_physical(
                self.base.writing_mode, shared_context.viewport_size()).inline;
            self.base.block_container_writing_mode = self.base.writing_mode;
        }
    }

    fn guess_inline_size_for_block_formatting_context_if_necessary(&mut self) {
        // We don't need to guess anything unless this is a block formatting context.
        if self.formatting_context_type() != FormattingContextType::Block {
            return
        }

        // If `max-width` is set, then don't perform this speculation. We guess that the
        // page set `max-width` in order to avoid hitting floats. The search box on Google
        // SERPs falls into this category.
        if self.fragment.style.max_inline_size() != LengthOrPercentageOrNone::None {
            return
        }

        // At this point, we know we can't precisely compute the inline-size of this block now,
        // because floats might affect it. Speculate that its inline-size is equal to the
        // inline-size computed above minus the inline-size of the previous left and/or right
        // floats.
        let speculated_left_float_size = if self.fragment.margin.inline_start >= Au(0) &&
                self.base.speculated_float_placement_in.left > self.fragment.margin.inline_start {
            self.base.speculated_float_placement_in.left - self.fragment.margin.inline_start
        } else {
            Au(0)
        };
        let speculated_right_float_size = if self.fragment.margin.inline_end >= Au(0) &&
                self.base.speculated_float_placement_in.right > self.fragment.margin.inline_end {
            self.base.speculated_float_placement_in.right - self.fragment.margin.inline_end
        } else {
            Au(0)
        };
        self.fragment.border_box.size.inline = self.fragment.border_box.size.inline -
            speculated_left_float_size - speculated_right_float_size
    }

    fn definitely_has_zero_block_size(&self) -> bool {
        if !self.fragment.style.content_block_size().is_definitely_zero() {
            return false
        }
        let border_width = self.fragment.border_width();
        if border_width.block_start != Au(0) || border_width.block_end != Au(0) {
            return false
        }
        let padding = self.fragment.style.logical_padding();
        padding.block_start.is_definitely_zero() && padding.block_end.is_definitely_zero()
    }

    pub fn is_inline_flex_item(&self) -> bool {
        self.fragment.flags.contains(IS_INLINE_FLEX_ITEM)
    }

    pub fn is_block_flex_item(&self) -> bool {
        self.fragment.flags.contains(IS_BLOCK_FLEX_ITEM)
    }

    /// Changes this block's clipping region from its parent's coordinate system to its own
    /// coordinate system if necessary (i.e. if this block is a stacking context).
    ///
    /// The clipping region is initially in each block's parent's coordinate system because the
    /// parent of each block does not have enough information to determine what the child's
    /// coordinate system is on its own. Specifically, if the child is absolutely positioned, the
    /// parent does not know where the child's absolute position is at the time it assigns clipping
    /// regions, because flows compute their own absolute positions.
    fn switch_coordinate_system_if_necessary(&mut self) {
        // Avoid overflows!
        if self.base.clip.is_max() {
            return
        }

        if !self.fragment.establishes_stacking_context() {
            return
        }

        let stacking_relative_border_box =
            self.fragment.stacking_relative_border_box(&self.base.stacking_relative_position,
                                                       &self.base
                                                            .early_absolute_position_info
                                                            .relative_containing_block_size,
                                                       self.base
                                                           .early_absolute_position_info
                                                           .relative_containing_block_mode,
                                                       CoordinateSystem::Parent);
        self.base.clip = self.base.clip.translate(&-stacking_relative_border_box.origin);

        // Account for `transform`, if applicable.
        if self.fragment.style.get_box().transform.0.is_none() {
            return
        }
        let transform = match self.fragment
                                  .transform_matrix(&stacking_relative_border_box)
                                  .unwrap_or(Matrix4D::identity())
                                  .inverse() {
            Some(transform) => transform,
            None => {
                // Singular matrix. Ignore it.
                return
            }
        };

        // FIXME(pcwalton): This is inaccurate: not all transforms are 2D, and not all clips are
        // axis-aligned.
        let bounding_rect = self.base.clip.bounding_rect();
        let bounding_rect = Rect::new(Point2D::new(bounding_rect.origin.x.to_f32_px(),
                                                   bounding_rect.origin.y.to_f32_px()),
                                      Size2D::new(bounding_rect.size.width.to_f32_px(),
                                                  bounding_rect.size.height.to_f32_px()));
        let clip_rect = transform.to_2d().transform_rect(&bounding_rect);
        let clip_rect = Rect::new(Point2D::new(Au::from_f32_px(clip_rect.origin.x),
                                               Au::from_f32_px(clip_rect.origin.y)),
                                  Size2D::new(Au::from_f32_px(clip_rect.size.width),
                                              Au::from_f32_px(clip_rect.size.height)));
        self.base.clip = ClippingRegion::from_rect(&clip_rect)
    }

    pub fn mark_scrolling_overflow(&mut self, has_scrolling_overflow: bool) {
        if has_scrolling_overflow {
            self.flags.insert(HAS_SCROLLING_OVERFLOW);
        } else {
            self.flags.remove(HAS_SCROLLING_OVERFLOW);
        }
    }

    pub fn has_scrolling_overflow(&mut self) -> bool {
        self.flags.contains(HAS_SCROLLING_OVERFLOW)
    }

}

impl Flow for BlockFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Block
    }

    fn as_mut_block(&mut self) -> &mut BlockFlow {
        self
    }

    fn as_block(&self) -> &BlockFlow {
        self
    }

    /// Pass 1 of reflow: computes minimum and preferred inline-sizes.
    ///
    /// Recursively (bottom-up) determine the flow's minimum and preferred inline-sizes. When
    /// called on this flow, all child flows have had their minimum and preferred inline-sizes set.
    /// This function must decide minimum/preferred inline-sizes based on its children's
    /// inline-sizes and the dimensions of any fragments it is responsible for flowing.
    fn bubble_inline_sizes(&mut self) {
        // If this block has a fixed width, just use that for the minimum and preferred width,
        // rather than bubbling up children inline width.
        let consult_children = match self.fragment.style().get_position().width {
            LengthOrPercentageOrAuto::Length(_) => false,
            _ => true,
        };
        self.bubble_inline_sizes_for_block(consult_children);
        self.fragment.restyle_damage.remove(BUBBLE_ISIZES);
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    ///
    /// Dual fragments consume some inline-size first, and the remainder is assigned to all child
    /// (block) contexts.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("block::assign_inline_sizes {:x}", self.base.debug_id());

        let shared_context = layout_context.shared_context();
        self.compute_inline_sizes(shared_context);

        // Move in from the inline-start border edge.
        let inline_start_content_edge = self.fragment.border_box.start.i +
            self.fragment.border_padding.inline_start;

        let padding_and_borders = self.fragment.border_padding.inline_start_end();

        // Distance from the inline-end margin edge to the inline-end content edge.
        let inline_end_content_edge =
            self.fragment.margin.inline_end +
            self.fragment.border_padding.inline_end;

        let content_inline_size = self.fragment.border_box.size.inline - padding_and_borders;

        self.propagate_assigned_inline_size_to_children(shared_context,
                                                        inline_start_content_edge,
                                                        inline_end_content_edge,
                                                        content_inline_size,
                                                        |_, _, _, _, _, _| {});
    }

    fn place_float_if_applicable<'a>(&mut self) {
        if self.base.flags.is_float() {
            self.place_float();
        }
    }

    fn assign_block_size_for_inorder_child_if_necessary(&mut self,
                                                        layout_context: &LayoutContext,
                                                        parent_thread_id: u8,
                                                        content_box: LogicalRect<Au>)
                                                        -> bool {
        if self.base.flags.is_float() {
            return false
        }

        let is_formatting_context = self.formatting_context_type() != FormattingContextType::None;
        if !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) && is_formatting_context {
            self.assign_inline_position_for_formatting_context(layout_context, content_box);
        }

        if (self as &Flow).floats_might_flow_through() {
            self.base.thread_id = parent_thread_id;
            if self.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
                self.assign_block_size(layout_context);
                // Don't remove the restyle damage; `assign_block_size` decides whether that is
                // appropriate (which in the case of e.g. absolutely-positioned flows, it is not).
            }
            return true
        }

        if is_formatting_context {
            // If this is a formatting context and definitely did not have floats in, then we must
            // translate the floats past us.
            let writing_mode = self.base.floats.writing_mode;
            let delta = self.base.position.size.block;
            self.base.floats.translate(LogicalSize::new(writing_mode, Au(0), -delta));
            return true
        }

        false
    }

    fn assign_block_size(&mut self, ctx: &LayoutContext) {
        let remaining = Flow::fragment(self, ctx, None);
        debug_assert!(remaining.is_none());
    }

    fn fragment(&mut self, layout_context: &LayoutContext,
                fragmentation_context: Option<FragmentationContext>)
                -> Option<Arc<Flow>> {
        if self.fragment.is_replaced() {
            let _scope = layout_debug_scope!("assign_replaced_block_size_if_necessary {:x}",
                                             self.base.debug_id());

            // Assign block-size for fragment if it is an image fragment.
            self.fragment.assign_replaced_block_size_if_necessary();
            if !self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
                self.base.position.size.block = self.fragment.border_box.size.block;
                let mut block_start = AdjoiningMargins::from_margin(self.fragment.margin.block_start);
                let block_end = AdjoiningMargins::from_margin(self.fragment.margin.block_end);
                if self.fragment.border_box.size.block == Au(0) {
                    block_start.union(block_end);
                    self.base.collapsible_margins = CollapsibleMargins::CollapseThrough(block_start);
                } else {
                    self.base.collapsible_margins = CollapsibleMargins::Collapse(block_start, block_end);
                }
                self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
                self.fragment.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
            }
            None
        } else if self.is_root() ||
                self.formatting_context_type() != FormattingContextType::None ||
                self.base.flags.contains(MARGINS_CANNOT_COLLAPSE) {
            // Root element margins should never be collapsed according to CSS § 8.3.1.
            debug!("assign_block_size: assigning block_size for root flow {:?}",
                   flow::base(self).debug_id());
            self.assign_block_size_block_base(
                layout_context,
                fragmentation_context,
                MarginsMayCollapseFlag::MarginsMayNotCollapse)
        } else {
            debug!("assign_block_size: assigning block_size for block {:?}",
                   flow::base(self).debug_id());
            self.assign_block_size_block_base(
                layout_context,
                fragmentation_context,
                MarginsMayCollapseFlag::MarginsMayCollapse)
        }
    }

    fn compute_absolute_position(&mut self, _layout_context: &LayoutContext) {
        // FIXME (mbrubeck): Get the real container size, taking the container writing mode into
        // account.  Must handle vertical writing modes.
        let container_size = Size2D::new(self.base.block_container_inline_size, Au(0));

        if self.is_root() {
            self.base.clip = ClippingRegion::max();
        }

        if self.base.flags.contains(IS_ABSOLUTELY_POSITIONED) {
            let position_start = self.base.position.start.to_physical(self.base.writing_mode,
                                                                      container_size);

            // Compute our position relative to the nearest ancestor stacking context. This will be
            // passed down later as part of containing block details for absolute descendants.
            let absolute_stacking_relative_position = if self.is_fixed() {
                // The viewport is initially at (0, 0).
                position_start
            } else {
                // Absolute position of the containing block + position of absolute
                // flow w.r.t. the containing block.
                self.base
                    .late_absolute_position_info
                    .stacking_relative_position_of_absolute_containing_block + position_start
            };

            if !self.base.writing_mode.is_vertical() {
                if !self.base.flags.contains(INLINE_POSITION_IS_STATIC) {
                    self.base.stacking_relative_position.x = absolute_stacking_relative_position.x
                }
                if !self.base.flags.contains(BLOCK_POSITION_IS_STATIC) {
                    self.base.stacking_relative_position.y = absolute_stacking_relative_position.y
                }
            } else {
                if !self.base.flags.contains(INLINE_POSITION_IS_STATIC) {
                    self.base.stacking_relative_position.y = absolute_stacking_relative_position.y
                }
                if !self.base.flags.contains(BLOCK_POSITION_IS_STATIC) {
                    self.base.stacking_relative_position.x = absolute_stacking_relative_position.x
                }
            }
        }

        // For relatively-positioned descendants, the containing block formed by a block is just
        // the content box. The containing block for absolutely-positioned descendants, on the
        // other hand, is only established if we are positioned.
        let relative_offset =
            self.fragment.relative_position(&self.base
                                                 .early_absolute_position_info
                                                 .relative_containing_block_size);
        if self.contains_positioned_fragments() {
            let border_box_origin = (self.fragment.border_box -
                self.fragment.style.logical_border_width()).start;
            self.base
                .late_absolute_position_info
                .stacking_relative_position_of_absolute_containing_block =
                    self.base.stacking_relative_position +
                     (border_box_origin + relative_offset).to_physical(self.base.writing_mode,
                                                                       container_size)
        }

        // Compute absolute position info for children.
        let stacking_relative_position_of_absolute_containing_block_for_children =
            if self.fragment.establishes_stacking_context() {
                let logical_border_width = self.fragment.style().logical_border_width();
                let position = LogicalPoint::new(self.base.writing_mode,
                                                 logical_border_width.inline_start,
                                                 logical_border_width.block_start);
                let position = position.to_physical(self.base.writing_mode, container_size);
                if self.contains_positioned_fragments() {
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
                    .late_absolute_position_info
                    .stacking_relative_position_of_absolute_containing_block
            };
        let late_absolute_position_info_for_children = LateAbsolutePositionInfo {
            stacking_relative_position_of_absolute_containing_block:
                stacking_relative_position_of_absolute_containing_block_for_children,
        };
        let container_size_for_children =
            self.base.position.size.to_physical(self.base.writing_mode);

        // Compute the origin and clipping rectangle for children.
        let relative_offset = relative_offset.to_physical(self.base.writing_mode);
        let is_stacking_context = self.fragment.establishes_stacking_context();
        let origin_for_children = if is_stacking_context {
            // We establish a stacking context, so the position of our children is vertically
            // correct, but has to be adjusted to accommodate horizontal margins. (Note the
            // calculation involving `position` below and recall that inline-direction flow
            // positions are relative to the edges of the margin box.)
            //
            // FIXME(pcwalton): Is this vertical-writing-direction-safe?
            let margin = self.fragment.margin.to_physical(self.base.writing_mode);
            Point2D::new(-margin.left, Au(0))
        } else {
            self.base.stacking_relative_position + relative_offset
        };

        let stacking_relative_border_box =
            self.fragment
                .stacking_relative_border_box(&self.base.stacking_relative_position,
                                              &self.base
                                                   .early_absolute_position_info
                                                   .relative_containing_block_size,
                                              self.base
                                                  .early_absolute_position_info
                                                  .relative_containing_block_mode,
                                              CoordinateSystem::Own);

        // Our parent set our `clip` field to the clipping region in its coordinate system. Change
        // it to our coordinate system.
        self.switch_coordinate_system_if_necessary();
        self.fragment.adjust_clip_for_style(&mut self.base.clip, &stacking_relative_border_box);

        // Compute the clipping region for children, taking our `overflow` properties and so forth
        // into account.
        let mut clip_for_children = self.base.clip.clone();
        self.fragment.adjust_clipping_region_for_children(&mut clip_for_children,
                                                          &stacking_relative_border_box);

        // Process children.
        for kid in self.base.child_iter_mut() {
            if flow::base(kid).flags.contains(INLINE_POSITION_IS_STATIC) ||
                    flow::base(kid).flags.contains(BLOCK_POSITION_IS_STATIC) {
                let kid_base = flow::mut_base(kid);
                let physical_position = kid_base.position.to_physical(kid_base.writing_mode,
                                                                      container_size_for_children);

                // Set the inline and block positions as necessary.
                if !kid_base.writing_mode.is_vertical() {
                    if kid_base.flags.contains(INLINE_POSITION_IS_STATIC) {
                        kid_base.stacking_relative_position.x = origin_for_children.x +
                            physical_position.origin.x
                    }
                    if kid_base.flags.contains(BLOCK_POSITION_IS_STATIC) {
                        kid_base.stacking_relative_position.y = origin_for_children.y +
                            physical_position.origin.y
                    }
                } else {
                    if kid_base.flags.contains(INLINE_POSITION_IS_STATIC) {
                        kid_base.stacking_relative_position.y = origin_for_children.y +
                            physical_position.origin.y
                    }
                    if kid_base.flags.contains(BLOCK_POSITION_IS_STATIC) {
                        kid_base.stacking_relative_position.x = origin_for_children.x +
                            physical_position.origin.x
                    }
                }
            }

            flow::mut_base(kid).late_absolute_position_info =
                late_absolute_position_info_for_children;

            // This clipping region is in our coordinate system. The child will fix it up to be in
            // its own coordinate system by itself if necessary.
            //
            // Rationale: If the child is absolutely positioned, it hasn't been positioned at this
            // point (as absolutely-positioned flows position themselves in
            // `compute_absolute_position()`). Therefore, we don't always know what the child's
            // coordinate system is here. So we store the clipping region in our coordinate system
            // for now; the child will move it later if needed.
            flow::mut_base(kid).clip = clip_for_children.clone()
        }

        self.base.restyle_damage.remove(REPOSITION)
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

    /// Return the dimensions of the containing block generated by this flow for absolutely-
    /// positioned descendants. For block flows, this is the padding box.
    fn generated_containing_block_size(&self, _: OpaqueFlow) -> LogicalSize<Au> {
        (self.fragment.border_box - self.fragment.style().logical_border_width()).size
    }

    fn is_absolute_containing_block(&self) -> bool {
        self.contains_positioned_fragments()
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

    fn collect_stacking_contexts(&mut self, state: &mut DisplayListBuildState) {
        self.collect_stacking_contexts_for_block(state);
    }

    fn build_display_list(&mut self, state: &mut DisplayListBuildState) {
        self.build_display_list_for_block(state, BorderPaintingMode::Separate);
    }

    fn repair_style(&mut self, new_style: &Arc<ServoComputedValues>) {
        self.fragment.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Overflow {
        let flow_size = self.base.position.size.to_physical(self.base.writing_mode);
        self.fragment.compute_overflow(&flow_size,
                                       &self.base
                                            .early_absolute_position_info
                                            .relative_containing_block_size)
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             level: i32,
                                             stacking_context_position: &Point2D<Au>) {
        if !iterator.should_process(&self.fragment) {
            return
        }

        iterator.process(&self.fragment,
                         level,
                         &self.fragment
                              .stacking_relative_border_box(&self.base.stacking_relative_position,
                                                            &self.base
                                                                 .early_absolute_position_info
                                                                 .relative_containing_block_size,
                                                            self.base
                                                                .early_absolute_position_info
                                                                .relative_containing_block_mode,
                                                            CoordinateSystem::Own)
                              .translate(stacking_context_position));
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        (*mutator)(&mut self.fragment)
    }

    fn print_extra_flow_children(&self, print_tree: &mut PrintTree) {
        print_tree.add_item(format!("↑↑ Fragment for block: {:?}", self.fragment));
    }
}

impl fmt::Debug for BlockFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:?}({:x}) {:?}",
               self.class(),
               self.base.debug_id(),
               self.base)
    }
}

/// The inputs for the inline-sizes-and-margins constraint equation.
#[derive(Debug, Copy, Clone)]
pub struct ISizeConstraintInput {
    pub computed_inline_size: MaybeAuto,
    pub inline_start_margin: MaybeAuto,
    pub inline_end_margin: MaybeAuto,
    pub inline_start: MaybeAuto,
    pub inline_end: MaybeAuto,
    pub text_align: text_align::T,
    pub available_inline_size: Au,
}

impl ISizeConstraintInput {
    pub fn new(computed_inline_size: MaybeAuto,
               inline_start_margin: MaybeAuto,
               inline_end_margin: MaybeAuto,
               inline_start: MaybeAuto,
               inline_end: MaybeAuto,
               text_align: text_align::T,
               available_inline_size: Au)
           -> ISizeConstraintInput {
        ISizeConstraintInput {
            computed_inline_size: computed_inline_size,
            inline_start_margin: inline_start_margin,
            inline_end_margin: inline_end_margin,
            inline_start: inline_start,
            inline_end: inline_end,
            text_align: text_align,
            available_inline_size: available_inline_size,
        }
    }
}

/// The solutions for the inline-size-and-margins constraint equation.
#[derive(Copy, Clone, Debug)]
pub struct ISizeConstraintSolution {
    pub inline_start: Au,
    pub inline_size: Au,
    pub margin_inline_start: Au,
    pub margin_inline_end: Au
}

impl ISizeConstraintSolution {
    pub fn new(inline_size: Au, margin_inline_start: Au, margin_inline_end: Au)
               -> ISizeConstraintSolution {
        ISizeConstraintSolution {
            inline_start: Au(0),
            inline_size: inline_size,
            margin_inline_start: margin_inline_start,
            margin_inline_end: margin_inline_end,
        }
    }

    fn for_absolute_flow(inline_start: Au,
                         inline_size: Au,
                         margin_inline_start: Au,
                         margin_inline_end: Au)
                         -> ISizeConstraintSolution {
        ISizeConstraintSolution {
            inline_start: inline_start,
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
    /// Instructs the fragment to compute its border and padding.
    fn compute_border_and_padding(&self, block: &mut BlockFlow, containing_block_inline_size: Au) {
        block.fragment.compute_border_and_padding(containing_block_inline_size,
                                                  border_collapse::T::separate);
    }

    /// Compute the inputs for the ISize constraint equation.
    ///
    /// This is called only once to compute the initial inputs. For calculations involving
    /// minimum and maximum inline-size, we don't need to recompute these.
    fn compute_inline_size_constraint_inputs(&self,
                                             block: &mut BlockFlow,
                                             parent_flow_inline_size: Au,
                                             shared_context: &SharedStyleContext)
                                             -> ISizeConstraintInput {
        let containing_block_inline_size =
            self.containing_block_inline_size(block, parent_flow_inline_size, shared_context);

        block.fragment.compute_block_direction_margins(containing_block_inline_size);
        block.fragment.compute_inline_direction_margins(containing_block_inline_size);
        self.compute_border_and_padding(block, containing_block_inline_size);

        let mut computed_inline_size = self.initial_computed_inline_size(block,
                                                                         parent_flow_inline_size,
                                                                         shared_context);
        let style = block.fragment.style();
        match (computed_inline_size, style.get_position().box_sizing) {
            (MaybeAuto::Specified(size), box_sizing::T::border_box) => {
                computed_inline_size =
                    MaybeAuto::Specified(size - block.fragment.border_padding.inline_start_end())
            }
            (MaybeAuto::Auto, box_sizing::T::border_box) |
            (_, box_sizing::T::content_box) => {}
        }

        let margin = style.logical_margin();
        let position = style.logical_position();

        let available_inline_size = containing_block_inline_size -
            block.fragment.border_padding.inline_start_end();
        ISizeConstraintInput::new(computed_inline_size,
                                  MaybeAuto::from_style(margin.inline_start,
                                                        containing_block_inline_size),
                                  MaybeAuto::from_style(margin.inline_end,
                                                        containing_block_inline_size),
                                  MaybeAuto::from_style(position.inline_start,
                                                        containing_block_inline_size),
                                  MaybeAuto::from_style(position.inline_end,
                                                        containing_block_inline_size),
                                  style.get_inheritedtext().text_align,
                                  available_inline_size)
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
            let block_mode = block.base.writing_mode;

            // FIXME (mbrubeck): Get correct containing block for positioned blocks?
            let container_mode = block.base.block_container_writing_mode;
            let container_size = block.base.block_container_inline_size;

            let fragment = block.fragment();
            fragment.margin.inline_start = solution.margin_inline_start;
            fragment.margin.inline_end = solution.margin_inline_end;

            // The associated fragment has the border box of this flow.
            inline_size = solution.inline_size + fragment.border_padding.inline_start_end();
            fragment.border_box.size.inline = inline_size;

            // Start border edge.
            // FIXME (mbrubeck): Handle vertical writing modes.
            fragment.border_box.start.i =
                if container_mode.is_bidi_ltr() == block_mode.is_bidi_ltr() {
                    fragment.margin.inline_start
                } else {
                    // The parent's "start" direction is the child's "end" direction.
                    container_size - inline_size - fragment.margin.inline_end
                };

            // To calculate the total size of this block, we also need to account for any
            // additional size contribution from positive margins. Negative margins means the block
            // isn't made larger at all by the margin.
            extra_inline_size_from_margin = max(Au(0), fragment.margin.inline_start) +
                                            max(Au(0), fragment.margin.inline_end);
        }

        // We also resize the block itself, to ensure that overflow is not calculated
        // as the inline-size of our parent. We might be smaller and we might be larger if we
        // overflow.
        flow::mut_base(block).position.size.inline = inline_size + extra_inline_size_from_margin;
    }

    /// Set the inline coordinate of the given flow if it is absolutely positioned.
    fn set_inline_position_of_flow_if_necessary(&self,
                                                _: &mut BlockFlow,
                                                _: ISizeConstraintSolution) {}

    /// Solve the inline-size and margins constraints for this block flow.
    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution;

    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    shared_context: &SharedStyleContext)
                                    -> MaybeAuto {
        MaybeAuto::from_style(block.fragment().style().content_inline_size(),
                              self.containing_block_inline_size(block,
                                                                parent_flow_inline_size,
                                                                shared_context))
    }

    fn containing_block_inline_size(&self,
                                    _: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    _: &SharedStyleContext)
                              -> Au {
        parent_flow_inline_size
    }

    /// Compute the used value of inline-size, taking care of min-inline-size and max-inline-size.
    ///
    /// CSS Section 10.4: Minimum and Maximum inline-sizes
    fn compute_used_inline_size(&self,
                                block: &mut BlockFlow,
                                shared_context: &SharedStyleContext,
                                parent_flow_inline_size: Au) {
        let mut input = self.compute_inline_size_constraint_inputs(block,
                                                                   parent_flow_inline_size,
                                                                   shared_context);

        let containing_block_inline_size =
            self.containing_block_inline_size(block, parent_flow_inline_size, shared_context);

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
        self.set_inline_position_of_flow_if_necessary(block, solution);
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

        // Check for direction of parent flow (NOT Containing Block)
        let block_mode = block.base.writing_mode;
        let container_mode = block.base.block_container_writing_mode;
        let block_align = block.base.flags.text_align();

        // FIXME (mbrubeck): Handle vertical writing modes.
        let parent_has_same_direction = container_mode.is_bidi_ltr() == block_mode.is_bidi_ltr();

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
        let (inline_start_margin, inline_size, inline_end_margin) =
            match (inline_start_margin, computed_inline_size, inline_end_margin) {
                // If all have a computed value other than 'auto', the system is over-constrained.
                (MaybeAuto::Specified(margin_start),
                 MaybeAuto::Specified(inline_size),
                 MaybeAuto::Specified(margin_end)) => {
                    // servo_left, servo_right, and servo_center are used to implement
                    // the "align descendants" rule in HTML5 § 14.2.
                    if block_align == text_align::T::servo_center {
                        // Ignore any existing margins, and make the inline-start and
                        // inline-end margins equal.
                        let margin = (available_inline_size - inline_size).scale_by(0.5);
                        (margin, inline_size, margin)
                    } else {
                        let ignore_end_margin = match block_align {
                            text_align::T::servo_left => block_mode.is_bidi_ltr(),
                            text_align::T::servo_right => !block_mode.is_bidi_ltr(),
                            _ => parent_has_same_direction,
                        };
                        if ignore_end_margin {
                            (margin_start, inline_size, available_inline_size -
                             (margin_start + inline_size))
                        } else {
                            (available_inline_size - (margin_end + inline_size),
                             inline_size,
                             margin_end)
                        }
                    }
                }
                // If exactly one value is 'auto', solve for it
                (MaybeAuto::Auto,
                 MaybeAuto::Specified(inline_size),
                 MaybeAuto::Specified(margin_end)) =>
                    (available_inline_size - (inline_size + margin_end), inline_size, margin_end),
                (MaybeAuto::Specified(margin_start),
                 MaybeAuto::Auto,
                 MaybeAuto::Specified(margin_end)) => {
                    (margin_start,
                     available_inline_size - (margin_start + margin_end),
                     margin_end)
                }
                (MaybeAuto::Specified(margin_start),
                 MaybeAuto::Specified(inline_size),
                 MaybeAuto::Auto) => {
                    (margin_start,
                     inline_size,
                     available_inline_size - (margin_start + inline_size))
                }

                // If inline-size is set to 'auto', any other 'auto' value becomes '0',
                // and inline-size is solved for
                (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Specified(margin_end)) => {
                    (Au(0), available_inline_size - margin_end, margin_end)
                }
                (MaybeAuto::Specified(margin_start), MaybeAuto::Auto, MaybeAuto::Auto) => {
                    (margin_start, available_inline_size - margin_start, Au(0))
                }
                (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Auto) => {
                    (Au(0), available_inline_size, Au(0))
                }

                // If inline-start and inline-end margins are auto, they become equal
                (MaybeAuto::Auto, MaybeAuto::Specified(inline_size), MaybeAuto::Auto) => {
                    let margin = (available_inline_size - inline_size).scale_by(0.5);
                    (margin, inline_size, margin)
                }
            };

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
pub struct InlineBlockNonReplaced;
pub struct InlineBlockReplaced;
pub struct InlineFlexItem;

impl ISizeAndMarginsComputer for AbsoluteNonReplaced {
    /// Solve the horizontal constraint equation for absolute non-replaced elements.
    ///
    /// CSS Section 10.3.7
    /// Constraint equation:
    /// inline-start + inline-end + inline-size + margin-inline-start + margin-inline-end
    /// = absolute containing block inline-size - (horizontal padding and border)
    /// [aka available inline-size]
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
            ..
        } = input;

        // Check for direction of parent flow (NOT Containing Block)
        let block_mode = block.base.writing_mode;
        let container_mode = block.base.block_container_writing_mode;

        // FIXME (mbrubeck): Handle vertical writing modes.
        let parent_has_same_direction = container_mode.is_bidi_ltr() == block_mode.is_bidi_ltr();

        let (inline_start, inline_size, margin_inline_start, margin_inline_end) =
            match (inline_start, inline_end, computed_inline_size) {
                (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Auto) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    // Now it is the same situation as inline-start Specified and inline-end
                    // and inline-size Auto.

                    // Set inline-end to zero to calculate inline-size.
                    let inline_size =
                        block.get_shrink_to_fit_inline_size(available_inline_size -
                                                            (margin_start + margin_end));
                    (Au(0), inline_size, margin_start, margin_end)
                }
                (MaybeAuto::Specified(inline_start),
                 MaybeAuto::Specified(inline_end),
                 MaybeAuto::Specified(inline_size)) => {
                    match (inline_start_margin, inline_end_margin) {
                        (MaybeAuto::Auto, MaybeAuto::Auto) => {
                            let total_margin_val =
                                available_inline_size - inline_start - inline_end - inline_size;
                            if total_margin_val < Au(0) {
                                if parent_has_same_direction {
                                    // margin-inline-start becomes 0
                                    (inline_start, inline_size, Au(0), total_margin_val)
                                } else {
                                    // margin-inline-end becomes 0, because it's toward the parent's
                                    // inline-start edge.
                                    (inline_start, inline_size, total_margin_val, Au(0))
                                }
                            } else {
                                // Equal margins
                                (inline_start,
                                 inline_size,
                                 total_margin_val.scale_by(0.5),
                                 total_margin_val.scale_by(0.5))
                            }
                        }
                        (MaybeAuto::Specified(margin_start), MaybeAuto::Auto) => {
                            let sum = inline_start + inline_end + inline_size + margin_start;
                            (inline_start, inline_size, margin_start, available_inline_size - sum)
                        }
                        (MaybeAuto::Auto, MaybeAuto::Specified(margin_end)) => {
                            let sum = inline_start + inline_end + inline_size + margin_end;
                            (inline_start, inline_size, available_inline_size - sum, margin_end)
                        }
                        (MaybeAuto::Specified(margin_start), MaybeAuto::Specified(margin_end)) => {
                            // Values are over-constrained.
                            let sum = inline_start + inline_size + margin_start + margin_end;
                            if parent_has_same_direction {
                                // Ignore value for 'inline-end'
                                (inline_start, inline_size, margin_start, margin_end)
                            } else {
                                // Ignore value for 'inline-start'
                                (available_inline_size - sum,
                                 inline_size,
                                 margin_start,
                                 margin_end)
                            }
                        }
                    }
                }
                // For the rest of the cases, auto values for margin are set to 0

                // If only one is Auto, solve for it
                (MaybeAuto::Auto,
                 MaybeAuto::Specified(inline_end),
                 MaybeAuto::Specified(inline_size)) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    let sum = inline_end + inline_size + margin_start + margin_end;
                    (available_inline_size - sum, inline_size, margin_start, margin_end)
                }
                (MaybeAuto::Specified(inline_start),
                 MaybeAuto::Auto,
                 MaybeAuto::Specified(inline_size)) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    (inline_start, inline_size, margin_start, margin_end)
                }
                (MaybeAuto::Specified(inline_start),
                 MaybeAuto::Specified(inline_end),
                 MaybeAuto::Auto) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    let sum = inline_start + inline_end + margin_start + margin_end;
                    (inline_start, available_inline_size - sum, margin_start, margin_end)
                }

                // If inline-size is auto, then inline-size is shrink-to-fit. Solve for the
                // non-auto value.
                (MaybeAuto::Specified(inline_start), MaybeAuto::Auto, MaybeAuto::Auto) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    // Set inline-end to zero to calculate inline-size
                    let inline_size =
                        block.get_shrink_to_fit_inline_size(available_inline_size -
                                                            (margin_start + margin_end));
                    (inline_start, inline_size, margin_start, margin_end)
                }
                (MaybeAuto::Auto, MaybeAuto::Specified(inline_end), MaybeAuto::Auto) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    // Set inline-start to zero to calculate inline-size
                    let inline_size =
                        block.get_shrink_to_fit_inline_size(available_inline_size -
                                                            (margin_start + margin_end));
                    let sum = inline_end + inline_size + margin_start + margin_end;
                    (available_inline_size - sum, inline_size, margin_start, margin_end)
                }

                (MaybeAuto::Auto, MaybeAuto::Auto, MaybeAuto::Specified(inline_size)) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    // Setting 'inline-start' to static position because direction is 'ltr'.
                    // TODO: Handle 'rtl' when it is implemented.
                    (Au(0), inline_size, margin_start, margin_end)
                }
            };
        ISizeConstraintSolution::for_absolute_flow(inline_start,
                                                   inline_size,
                                                   margin_inline_start,
                                                   margin_inline_end)
    }

    fn containing_block_inline_size(&self,
                                    block: &mut BlockFlow,
                                    _: Au,
                                    shared_context: &SharedStyleContext)
                                    -> Au {
        let opaque_block = OpaqueFlow::from_flow(block);
        block.containing_block_size(&shared_context.viewport_size(), opaque_block).inline
    }

    fn set_inline_position_of_flow_if_necessary(&self,
                                                block: &mut BlockFlow,
                                                solution: ISizeConstraintSolution) {
        // Set the inline position of the absolute flow wrt to its containing block.
        if !block.base.flags.contains(INLINE_POSITION_IS_STATIC) {
            block.base.position.start.i = solution.inline_start;
        }
    }
}

impl ISizeAndMarginsComputer for AbsoluteReplaced {
    /// Solve the horizontal constraint equation for absolute replaced elements.
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

        let (inline_start, inline_size, margin_inline_start, margin_inline_end) =
            match (inline_start, inline_end) {
                (MaybeAuto::Auto, MaybeAuto::Auto) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    (Au(0), inline_size, margin_start, margin_end)
                }
                // If only one is Auto, solve for it
                (MaybeAuto::Auto, MaybeAuto::Specified(inline_end)) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    let sum = inline_end + inline_size + margin_start + margin_end;
                    (available_inline_size - sum, inline_size, margin_start, margin_end)
                }
                (MaybeAuto::Specified(inline_start), MaybeAuto::Auto) => {
                    let margin_start = inline_start_margin.specified_or_zero();
                    let margin_end = inline_end_margin.specified_or_zero();
                    (inline_start, inline_size, margin_start, margin_end)
                }
                (MaybeAuto::Specified(inline_start), MaybeAuto::Specified(inline_end)) => {
                    match (inline_start_margin, inline_end_margin) {
                        (MaybeAuto::Auto, MaybeAuto::Auto) => {
                            let total_margin_val = available_inline_size - inline_start -
                                inline_end - inline_size;
                            if total_margin_val < Au(0) {
                                // margin-inline-start becomes 0 because direction is 'ltr'.
                                (inline_start, inline_size, Au(0), total_margin_val)
                            } else {
                                // Equal margins
                                (inline_start,
                                 inline_size,
                                 total_margin_val.scale_by(0.5),
                                 total_margin_val.scale_by(0.5))
                            }
                        }
                        (MaybeAuto::Specified(margin_start), MaybeAuto::Auto) => {
                            let sum = inline_start + inline_end + inline_size + margin_start;
                            (inline_start, inline_size, margin_start, available_inline_size - sum)
                        }
                        (MaybeAuto::Auto, MaybeAuto::Specified(margin_end)) => {
                            let sum = inline_start + inline_end + inline_size + margin_end;
                            (inline_start, inline_size, available_inline_size - sum, margin_end)
                        }
                        (MaybeAuto::Specified(margin_start), MaybeAuto::Specified(margin_end)) => {
                            // Values are over-constrained.
                            // Ignore value for 'inline-end' cos direction is 'ltr'.
                            (inline_start, inline_size, margin_start, margin_end)
                        }
                    }
                }
            };
        ISizeConstraintSolution::for_absolute_flow(inline_start,
                                                   inline_size,
                                                   margin_inline_start,
                                                   margin_inline_end)
    }

    /// Calculate used value of inline-size just like we do for inline replaced elements.
    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    _: Au,
                                    shared_context: &SharedStyleContext)
                                    -> MaybeAuto {
        let opaque_block = OpaqueFlow::from_flow(block);
        let containing_block_inline_size =
            block.containing_block_size(&shared_context.viewport_size(), opaque_block).inline;
        let container_block_size = block.explicit_block_containing_size(shared_context);
        let fragment = block.fragment();
        fragment.assign_replaced_inline_size_if_necessary(containing_block_inline_size, container_block_size);
        // For replaced absolute flow, the rest of the constraint solving will
        // take inline-size to be specified as the value computed here.
        MaybeAuto::Specified(fragment.content_box().size.inline)
    }

    fn containing_block_inline_size(&self,
                                    block: &mut BlockFlow,
                                    _: Au,
                                    shared_context: &SharedStyleContext)
                                    -> Au {
        let opaque_block = OpaqueFlow::from_flow(block);
        block.containing_block_size(&shared_context.viewport_size(), opaque_block).inline
    }

    fn set_inline_position_of_flow_if_necessary(&self,
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
            MaybeAuto::Auto => {
                panic!("BlockReplaced: inline_size should have been computed by now")
            }
        };
        self.solve_block_inline_size_constraints(block, input)
    }

    /// Calculate used value of inline-size just like we do for inline replaced elements.
    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    shared_context: &SharedStyleContext)
                                    -> MaybeAuto {
        let container_block_size = block.explicit_block_containing_size(shared_context);
        let fragment = block.fragment();
        fragment.assign_replaced_inline_size_if_necessary(parent_flow_inline_size, container_block_size);
        // For replaced block flow, the rest of the constraint solving will
        // take inline-size to be specified as the value computed here.
        MaybeAuto::Specified(fragment.content_box().size.inline)
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
        let (computed_inline_size, inline_start_margin, inline_end_margin, available_inline_size) =
            (input.computed_inline_size,
             input.inline_start_margin,
             input.inline_end_margin,
             input.available_inline_size);
        let margin_inline_start = inline_start_margin.specified_or_zero();
        let margin_inline_end = inline_end_margin.specified_or_zero();
        let available_inline_size_float = available_inline_size - margin_inline_start -
            margin_inline_end;
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
        let (computed_inline_size, inline_start_margin, inline_end_margin) =
            (input.computed_inline_size, input.inline_start_margin, input.inline_end_margin);
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
                                    shared_context: &SharedStyleContext)
                                    -> MaybeAuto {
        let container_block_size = block.explicit_block_containing_size(shared_context);
        let fragment = block.fragment();
        fragment.assign_replaced_inline_size_if_necessary(parent_flow_inline_size, container_block_size);
        // For replaced block flow, the rest of the constraint solving will
        // take inline-size to be specified as the value computed here.
        MaybeAuto::Specified(fragment.content_box().size.inline)
    }
}

impl ISizeAndMarginsComputer for InlineBlockNonReplaced {
    /// Compute inline-start and inline-end margins and inline-size.
    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        let (computed_inline_size,
             inline_start_margin,
             inline_end_margin,
             available_inline_size) =
            (input.computed_inline_size,
             input.inline_start_margin,
             input.inline_end_margin,
             input.available_inline_size);

        // For inline-blocks, `auto` margins compute to 0.
        let inline_start_margin = inline_start_margin.specified_or_zero();
        let inline_end_margin = inline_end_margin.specified_or_zero();

        // If inline-size is set to 'auto', and this is an inline block, use the
        // shrink to fit algorithm (see CSS 2.1 § 10.3.9)
        let inline_size = match computed_inline_size {
            MaybeAuto::Auto => {
                block.get_shrink_to_fit_inline_size(available_inline_size - (inline_start_margin +
                                                                             inline_end_margin))
            }
            MaybeAuto::Specified(inline_size) => inline_size,
        };

        ISizeConstraintSolution::new(inline_size, inline_start_margin, inline_end_margin)
    }
}

impl ISizeAndMarginsComputer for InlineBlockReplaced {
    /// Compute inline-start and inline-end margins and inline-size.
    ///
    /// ISize has already been calculated. We now calculate the margins just
    /// like for non-replaced blocks.
    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        debug_assert!(match input.computed_inline_size {
            MaybeAuto::Specified(_) => true,
            MaybeAuto::Auto => false,
        });

        let (computed_inline_size,
             inline_start_margin,
             inline_end_margin,
             available_inline_size) =
            (input.computed_inline_size,
             input.inline_start_margin,
             input.inline_end_margin,
             input.available_inline_size);

        // For inline-blocks, `auto` margins compute to 0.
        let inline_start_margin = inline_start_margin.specified_or_zero();
        let inline_end_margin = inline_end_margin.specified_or_zero();

        // If inline-size is set to 'auto', and this is an inline block, use the
        // shrink to fit algorithm (see CSS 2.1 § 10.3.9)
        let inline_size = match computed_inline_size {
            MaybeAuto::Auto => {
                block.get_shrink_to_fit_inline_size(available_inline_size - (inline_start_margin +
                                                                             inline_end_margin))
            }
            MaybeAuto::Specified(inline_size) => inline_size,
        };

        ISizeConstraintSolution::new(inline_size, inline_start_margin, inline_end_margin)
    }

    /// Calculate used value of inline-size just like we do for inline replaced elements.
    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    shared_context: &SharedStyleContext)
                                    -> MaybeAuto {
        let container_block_size = block.explicit_block_containing_size(shared_context);
        let fragment = block.fragment();
        fragment.assign_replaced_inline_size_if_necessary(parent_flow_inline_size, container_block_size);
        // For replaced block flow, the rest of the constraint solving will
        // take inline-size to be specified as the value computed here.
        MaybeAuto::Specified(fragment.content_box().size.inline)
    }
}

impl ISizeAndMarginsComputer for InlineFlexItem {
    // Replace the default method directly to prevent recalculating and setting margins again
    // which has already been set by its parent.
    fn compute_used_inline_size(&self,
                                block: &mut BlockFlow,
                                shared_context: &SharedStyleContext,
                                parent_flow_inline_size: Au) {
        let container_block_size = block.explicit_block_containing_size(shared_context);
        block.fragment.assign_replaced_inline_size_if_necessary(parent_flow_inline_size,
                                                                container_block_size);
    }

    // The used inline size and margins are set by parent flex flow, do nothing here.
    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     _: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        let fragment = block.fragment();
        ISizeConstraintSolution::new(fragment.border_box.size.inline,
                                     fragment.margin.inline_start,
                                     fragment.margin.inline_end)
    }
}

/// A stacking context, a pseudo-stacking context, or a non-stacking context.
#[derive(Copy, Clone, PartialEq)]
pub enum BlockStackingContextType {
    NonstackingContext,
    PseudoStackingContext,
    StackingContext,
}
