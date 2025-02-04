/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Float layout.
//!
//! See CSS 2.1 § 9.5.1: <https://www.w3.org/TR/CSS2/visuren.html#float-position>

use std::collections::VecDeque;
use std::fmt::Debug;
use std::mem;
use std::ops::Range;

use app_units::{Au, MAX_AU, MIN_AU};
use euclid::num::Zero;
use servo_arc::Arc;
use style::computed_values::float::T as FloatProperty;
use style::computed_values::position::T as Position;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::Clear as StyleClear;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::{BoxFragment, CollapsedMargin};
use crate::geom::{LogicalRect, LogicalVec2, ToLogical};
use crate::positioned::{relative_adjustement, PositioningContext};
use crate::style_ext::{DisplayInside, PaddingBorderMargin};
use crate::{ContainingBlock, PropagatedBoxTreeData};

/// A floating box.
#[derive(Debug)]
pub(crate) struct FloatBox {
    /// The formatting context that makes up the content of this box.
    pub contents: IndependentFormattingContext,
}

/// `FloatContext` positions floats relative to the independent block formatting
/// context which contains the floating elements. The Fragment tree positions
/// elements relative to their containing blocks. This data structure is used to
/// help map between these two coordinate systems.
#[derive(Clone, Copy, Debug)]
pub struct ContainingBlockPositionInfo {
    /// The distance from the block start of the independent block formatting
    /// context that contains the floats and the block start of the current
    /// containing block, excluding uncollapsed block start margins. Note that
    /// this does not include uncollapsed block start margins because we don't
    /// know the value of collapsed margins until we lay out children.
    pub(crate) block_start: Au,
    /// Any uncollapsed block start margins that we have collected between the
    /// block start of the float containing independent block formatting context
    /// and this containing block, including for this containing block.
    pub(crate) block_start_margins_not_collapsed: CollapsedMargin,
    /// The distance from the inline start position of the float containing
    /// independent formatting context and the inline start of this containing
    /// block.
    pub inline_start: Au,
    /// The offset from the inline start position of the float containing
    /// independent formatting context to the inline end of this containing
    /// block.
    pub inline_end: Au,
}

impl ContainingBlockPositionInfo {
    pub fn new_with_inline_offsets(inline_start: Au, inline_end: Au) -> Self {
        Self {
            block_start: Au::zero(),
            block_start_margins_not_collapsed: CollapsedMargin::zero(),
            inline_start,
            inline_end,
        }
    }
}

/// This data strucure is used to try to place non-floating content among float content.
/// This is used primarily to place replaced content and independent formatting contexts
/// next to floats, as the specifcation dictates.
pub(crate) struct PlacementAmongFloats<'a> {
    /// The [FloatContext] to use for this placement.
    float_context: &'a FloatContext,
    /// The current bands we are considering for this placement.
    current_bands: VecDeque<FloatBand>,
    /// The next band, needed to know the height of the last band in current_bands.
    next_band: FloatBand,
    /// The size of the object to place.
    object_size: LogicalVec2<Au>,
    /// The minimum position in the block direction for the placement. Objects should not
    /// be placed before this point.
    ceiling: Au,
    /// The inline position where the object would be if there were no floats. The object
    /// can be placed after it due to floats, but not before it.
    min_inline_start: Au,
    /// The maximum inline position that the object can attain when avoiding floats.
    max_inline_end: Au,
}

impl<'a> PlacementAmongFloats<'a> {
    pub(crate) fn new(
        float_context: &'a FloatContext,
        ceiling: Au,
        object_size: LogicalVec2<Au>,
        pbm: &PaddingBorderMargin,
    ) -> Self {
        let mut ceiling_band = float_context.bands.find(ceiling).unwrap();
        let (current_bands, next_band) = if ceiling == MAX_AU {
            (VecDeque::new(), ceiling_band)
        } else {
            ceiling_band.top = ceiling;
            let current_bands = VecDeque::from([ceiling_band]);
            let next_band = float_context.bands.find_next(ceiling).unwrap();
            (current_bands, next_band)
        };
        let min_inline_start = float_context.containing_block_info.inline_start +
            pbm.margin.inline_start.auto_is(Au::zero);
        let max_inline_end = (float_context.containing_block_info.inline_end -
            pbm.margin.inline_end.auto_is(Au::zero))
        .max(min_inline_start + object_size.inline);
        PlacementAmongFloats {
            float_context,
            current_bands,
            next_band,
            object_size,
            ceiling,
            min_inline_start,
            max_inline_end,
        }
    }

    /// The top of the bands under consideration. This is initially the ceiling provided
    /// during creation of this [`PlacementAmongFloats`], but may be larger if the top
    /// band is discarded.
    fn top_of_bands(&self) -> Option<Au> {
        self.current_bands.front().map(|band| band.top)
    }

    /// The height of the bands under consideration.
    fn current_bands_height(&self) -> Au {
        if self.next_band.top == MAX_AU {
            // Treat MAX_AU as infinity.
            MAX_AU
        } else {
            let top = self
                .top_of_bands()
                .expect("Should have bands before reaching the end");
            self.next_band.top - top
        }
    }

    /// Add a single band to the bands under consideration and calculate the new
    /// [`PlacementAmongFloats::next_band`].
    fn add_one_band(&mut self) {
        assert!(self.next_band.top != MAX_AU);
        self.current_bands.push_back(self.next_band);
        self.next_band = self
            .float_context
            .bands
            .find_next(self.next_band.top)
            .unwrap();
    }

    /// Adds bands to the set of bands under consideration until their block size is at
    /// least large enough to contain the block size of the object being placed.
    fn accumulate_enough_bands_for_block_size(&mut self) {
        while self.current_bands_height() < self.object_size.block {
            self.add_one_band();
        }
    }

    /// Find the start and end of the inline space provided by the current set of bands
    /// under consideration.
    fn calculate_inline_start_and_end(&self) -> (Au, Au) {
        let mut max_inline_start = self.min_inline_start;
        let mut min_inline_end = self.max_inline_end;
        for band in self.current_bands.iter() {
            if let Some(inline_start) = band.inline_start {
                max_inline_start.max_assign(inline_start);
            }
            if let Some(inline_end) = band.inline_end {
                min_inline_end.min_assign(inline_end);
            }
        }
        (max_inline_start, min_inline_end)
    }

    /// Find the total inline size provided by the current set of bands under consideration.
    fn calculate_viable_inline_size(&self) -> Au {
        let (inline_start, inline_end) = self.calculate_inline_start_and_end();
        inline_end - inline_start
    }

    fn try_place_once(&mut self) -> Option<LogicalRect<Au>> {
        assert!(!self.current_bands.is_empty());
        self.accumulate_enough_bands_for_block_size();
        let (inline_start, inline_end) = self.calculate_inline_start_and_end();
        let available_inline_size = inline_end - inline_start;
        if available_inline_size < self.object_size.inline {
            return None;
        }
        Some(LogicalRect {
            start_corner: LogicalVec2 {
                inline: inline_start,
                block: self.top_of_bands().unwrap(),
            },
            size: LogicalVec2 {
                inline: available_inline_size,
                block: self.current_bands_height(),
            },
        })
    }

    /// Checks if we either have bands or we have gone past all of them.
    /// This is an invariant that should hold, otherwise we are in a broken state.
    fn has_bands_or_at_end(&self) -> bool {
        !self.current_bands.is_empty() || self.next_band.top == MAX_AU
    }

    fn pop_front_band_ensuring_has_bands_or_at_end(&mut self) {
        self.current_bands.pop_front();
        if !self.has_bands_or_at_end() {
            self.add_one_band();
        }
    }

    /// Run the placement algorithm for this [PlacementAmongFloats].
    pub(crate) fn place(&mut self) -> LogicalRect<Au> {
        debug_assert!(self.has_bands_or_at_end());
        while !self.current_bands.is_empty() {
            if let Some(result) = self.try_place_once() {
                return result;
            }
            self.pop_front_band_ensuring_has_bands_or_at_end();
        }
        debug_assert!(self.has_bands_or_at_end());

        // We could not fit the object in among the floats, so we place it as if it
        // cleared all floats.
        LogicalRect {
            start_corner: LogicalVec2 {
                inline: self.min_inline_start,
                block: self
                    .ceiling
                    .max(self.float_context.clear_inline_start_position)
                    .max(self.float_context.clear_inline_end_position),
            },
            size: LogicalVec2 {
                inline: self.max_inline_end - self.min_inline_start,
                block: MAX_AU,
            },
        }
    }

    /// After placing an object with `height: auto` (and using the minimum inline and
    /// block size as the object size) and then laying it out, try to fit the object into
    /// the current set of bands, given block size after layout and the available inline
    /// space from the original placement. This will return true if the object fits at the
    /// original placement location or false if the placement and layout must be run again
    /// (with this [PlacementAmongFloats]).
    pub(crate) fn try_to_expand_for_auto_block_size(
        &mut self,
        block_size_after_layout: Au,
        size_from_placement: &LogicalVec2<Au>,
    ) -> bool {
        debug_assert!(self.has_bands_or_at_end());
        debug_assert_eq!(size_from_placement.block, self.current_bands_height());
        debug_assert_eq!(
            size_from_placement.inline,
            self.calculate_viable_inline_size()
        );

        // If the object after layout fits into the originally calculated placement, then
        // it fits without any more work.
        if block_size_after_layout <= size_from_placement.block {
            return true;
        }

        // Keep searching until we have found an area with enough height
        // to contain the block after layout.
        let old_num_bands = self.current_bands.len();
        assert!(old_num_bands > 0);
        while self.current_bands_height() < block_size_after_layout {
            self.add_one_band();

            // If the new inline size is narrower, we must stop and run layout again.
            // Normally, a narrower block size means a bigger height, but in some
            // circumstances, such as when aspect ratio is used a narrower inline size
            // can counter-interuitively lead to a smaller block size after layout!
            let available_inline_size = self.calculate_viable_inline_size();
            if available_inline_size < size_from_placement.inline {
                // If the inline size becomes smaller than the minimum inline size, then
                // the current set of bands will never work and we must try removing the
                // first and searching starting from the second.
                if available_inline_size < self.object_size.inline {
                    self.next_band = self.current_bands[old_num_bands];
                    self.current_bands.truncate(old_num_bands);
                    self.pop_front_band_ensuring_has_bands_or_at_end();
                }
                return false;
            }
        }
        true
    }
}

/// Data kept during layout about the floats in a given block formatting context.
///
/// This is a persistent data structure. Each float has its own private copy of the float context,
/// although such copies may share portions of the `bands` tree.
#[derive(Clone, Debug)]
pub struct FloatContext {
    /// A persistent AA tree of float bands.
    ///
    /// This tree is immutable; modification operations return the new tree, which may share nodes
    /// with previous versions of the tree.
    pub bands: FloatBandTree,
    /// The block-direction "ceiling" defined by the placement of other floated content of
    /// this FloatContext. No new floats can be placed at a lower block start than this value.
    pub ceiling_from_floats: Au,
    /// The block-direction "ceiling" defined by the placement of non-floated content that
    /// precedes floated content in the document. Note that this may actually decrease as
    /// content is laid out in the case that content overflows its container.
    pub ceiling_from_non_floats: Au,
    /// Details about the position of the containing block relative to the
    /// independent block formatting context that contains all of the floats
    /// this `FloatContext` positions.
    pub containing_block_info: ContainingBlockPositionInfo,
    /// The (logically) lowest margin edge of the last inline-start float.
    pub clear_inline_start_position: Au,
    /// The (logically) lowest margin edge of the last inline-end float.
    pub clear_inline_end_position: Au,
}

impl FloatContext {
    /// Returns a new float context representing a containing block with the given content
    /// inline-size.
    pub fn new(max_inline_size: Au) -> Self {
        let mut bands = FloatBandTree::new();
        bands = bands.insert(FloatBand {
            top: MIN_AU,
            inline_start: None,
            inline_end: None,
        });
        bands = bands.insert(FloatBand {
            top: MAX_AU,
            inline_start: None,
            inline_end: None,
        });
        FloatContext {
            bands,
            ceiling_from_floats: Au::zero(),
            ceiling_from_non_floats: Au::zero(),
            containing_block_info: ContainingBlockPositionInfo::new_with_inline_offsets(
                Au::zero(),
                max_inline_size,
            ),
            clear_inline_start_position: Au::zero(),
            clear_inline_end_position: Au::zero(),
        }
    }

    /// (Logically) lowers the ceiling to at least `new_ceiling` units.
    ///
    /// If the ceiling is already logically lower (i.e. larger) than this, does nothing.
    pub fn set_ceiling_from_non_floats(&mut self, new_ceiling: Au) {
        self.ceiling_from_non_floats = new_ceiling;
    }

    /// The "ceiling" used for float placement. This is the minimum block position value
    /// that should be used for placing any new float.
    fn ceiling(&mut self) -> Au {
        self.ceiling_from_floats.max(self.ceiling_from_non_floats)
    }

    /// Determines where a float with the given placement would go, but leaves the float context
    /// unmodified. Returns the start corner of its margin box.
    ///
    /// This should be used for placing inline elements and block formatting contexts so that they
    /// don't collide with floats.
    pub(crate) fn place_object(&self, object: &PlacementInfo, ceiling: Au) -> LogicalVec2<Au> {
        let ceiling = match object.clear {
            Clear::None => ceiling,
            Clear::InlineStart => ceiling.max(self.clear_inline_start_position),
            Clear::InlineEnd => ceiling.max(self.clear_inline_end_position),
            Clear::Both => ceiling
                .max(self.clear_inline_start_position)
                .max(self.clear_inline_end_position),
        };

        // Find the first band this float fits in.
        let mut first_band = self.bands.find(ceiling).unwrap();
        while !first_band.object_fits(object, &self.containing_block_info) {
            let next_band = self.bands.find_next(first_band.top).unwrap();
            if next_band.top == MAX_AU {
                break;
            }
            first_band = next_band;
        }

        // The object fits perfectly here. Place it.
        match object.side {
            FloatSide::InlineStart => {
                let inline_start_object_edge = match first_band.inline_start {
                    Some(inline_start) => inline_start.max(self.containing_block_info.inline_start),
                    None => self.containing_block_info.inline_start,
                };
                LogicalVec2 {
                    inline: inline_start_object_edge,
                    block: first_band.top.max(ceiling),
                }
            },
            FloatSide::InlineEnd => {
                let inline_end_object_edge = match first_band.inline_end {
                    Some(inline_end) => inline_end.min(self.containing_block_info.inline_end),
                    None => self.containing_block_info.inline_end,
                };
                LogicalVec2 {
                    inline: inline_end_object_edge - object.size.inline,
                    block: first_band.top.max(ceiling),
                }
            },
        }
    }

    /// Places a new float and adds it to the list. Returns the start corner of its margin box.
    pub fn add_float(&mut self, new_float: &PlacementInfo) -> LogicalVec2<Au> {
        // Place the float.
        let ceiling = self.ceiling();
        let new_float_origin = self.place_object(new_float, ceiling);
        let new_float_extent = match new_float.side {
            FloatSide::InlineStart => new_float_origin.inline + new_float.size.inline,
            FloatSide::InlineEnd => new_float_origin.inline,
        };

        let new_float_rect = LogicalRect {
            start_corner: new_float_origin,
            // If this float has a negative margin, we should only consider its non-negative
            // block size contribution when determing where to place it. When the margin is
            // so negative that it's placed completely above the current float ceiling, then
            // we should position it as if it had zero block size.
            size: LogicalVec2 {
                inline: new_float.size.inline.max(Au::zero()),
                block: new_float.size.block.max(Au::zero()),
            },
        };

        // Update clear.
        match new_float.side {
            FloatSide::InlineStart => {
                self.clear_inline_start_position
                    .max_assign(new_float_rect.max_block_position());
            },
            FloatSide::InlineEnd => {
                self.clear_inline_end_position
                    .max_assign(new_float_rect.max_block_position());
            },
        }

        // Split the first band if necessary.
        let mut first_band = self.bands.find(new_float_rect.start_corner.block).unwrap();
        first_band.top = new_float_rect.start_corner.block;
        self.bands = self.bands.insert(first_band);

        // Split the last band if necessary.
        let mut last_band = self
            .bands
            .find(new_float_rect.max_block_position())
            .unwrap();
        last_band.top = new_float_rect.max_block_position();
        self.bands = self.bands.insert(last_band);

        // Update all bands that contain this float to reflect the new available size.
        let block_range = new_float_rect.start_corner.block..new_float_rect.max_block_position();
        self.bands = self
            .bands
            .set_range(&block_range, new_float.side, new_float_extent);

        // CSS 2.1 § 9.5.1 rule 6: The outer top of a floating box may not be higher than the outer
        // top of any block or floated box generated by an element earlier in the source document.
        self.ceiling_from_floats
            .max_assign(new_float_rect.start_corner.block);

        new_float_rect.start_corner
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Clear {
    None,
    InlineStart,
    InlineEnd,
    Both,
}

impl Clear {
    pub(crate) fn from_style_and_container_writing_mode(
        style: &ComputedValues,
        container_writing_mode: WritingMode,
    ) -> Self {
        match style.get_box().clear {
            StyleClear::None => Self::None,
            StyleClear::Both => Self::Both,
            StyleClear::InlineStart => Self::InlineStart,
            StyleClear::InlineEnd => Self::InlineEnd,
            StyleClear::Left if container_writing_mode.is_bidi_ltr() => Self::InlineStart,
            StyleClear::Left => Self::InlineEnd,
            StyleClear::Right if container_writing_mode.is_bidi_ltr() => Self::InlineEnd,
            StyleClear::Right => Self::InlineStart,
        }
    }
}

/// Information needed to place a float so that it doesn't collide with existing floats.
#[derive(Clone, Debug)]
pub struct PlacementInfo {
    /// The *margin* box size of the float.
    pub size: LogicalVec2<Au>,
    /// Which side of the containing block the float is aligned to.
    pub side: FloatSide,
    /// Which side or sides to clear existing floats on.
    pub clear: Clear,
}

/// Whether the float is aligned to the inline-start or inline-end side of its containing block.
///
/// See CSS 2.1 § 9.5.1: <https://www.w3.org/TR/CSS2/visuren.html#float-position>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatSide {
    InlineStart,
    InlineEnd,
}

/// Internal data structure that describes a nonoverlapping vertical region in which floats may be placed.
/// Floats must go between "inline-start edge + `inline_start`" and "inline-end edge - `inline_end`".
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatBand {
    /// The logical vertical position of the top of this band.
    pub top: Au,
    /// The distance from the inline-start edge of the block formatting context to the first legal
    /// (logically) horizontal position where floats may be placed. If `None`, there are no floats
    /// to the inline-start; distinguishing between the cases of "a zero-width float is present" and
    /// "no floats at all are present" is necessary to, for example, clear past zero-width floats.
    pub inline_start: Option<Au>,
    /// The distance from the *inline-start* edge of the block formatting context to the last legal
    /// (logically) horizontal position where floats may be placed. If `None`, there are no floats
    /// to the inline-end; distinguishing between the cases of "a zero-width float is present" and
    /// "no floats at all are present" is necessary to, for example, clear past zero-width floats.
    pub inline_end: Option<Au>,
}

impl FloatSide {
    pub(crate) fn from_style_and_container_writing_mode(
        style: &ComputedValues,
        container_writing_mode: WritingMode,
    ) -> Option<FloatSide> {
        Some(match style.get_box().float {
            FloatProperty::None => return None,
            FloatProperty::InlineStart => Self::InlineStart,
            FloatProperty::InlineEnd => Self::InlineEnd,
            FloatProperty::Left if container_writing_mode.is_bidi_ltr() => Self::InlineStart,
            FloatProperty::Left => Self::InlineEnd,
            FloatProperty::Right if container_writing_mode.is_bidi_ltr() => Self::InlineEnd,
            FloatProperty::Right => Self::InlineStart,
        })
    }
}

impl FloatBand {
    /// Determines whether an object fits in a band. Returns true if the object fits.
    fn object_fits(&self, object: &PlacementInfo, walls: &ContainingBlockPositionInfo) -> bool {
        match object.side {
            FloatSide::InlineStart => {
                // Compute a candidate inline-start position for the object.
                let candidate_inline_start = match self.inline_start {
                    None => walls.inline_start,
                    Some(inline_start) => inline_start.max(walls.inline_start),
                };

                // If this band has an existing inline-start float in it, then make sure that the object
                // doesn't stick out past the inline-end edge (rule 7).
                if self.inline_start.is_some() &&
                    candidate_inline_start + object.size.inline > walls.inline_end
                {
                    return false;
                }

                // If this band has an existing inline-end float in it, make sure we don't collide with
                // it (rule 3).
                match self.inline_end {
                    None => true,
                    Some(inline_end) => object.size.inline <= inline_end - candidate_inline_start,
                }
            },

            FloatSide::InlineEnd => {
                // Compute a candidate inline-end position for the object.
                let candidate_inline_end = match self.inline_end {
                    None => walls.inline_end,
                    Some(inline_end) => inline_end.min(walls.inline_end),
                };

                // If this band has an existing inline-end float in it, then make sure that the new
                // object doesn't stick out past the inline-start edge (rule 7).
                if self.inline_end.is_some() &&
                    candidate_inline_end - object.size.inline < walls.inline_start
                {
                    return false;
                }

                // If this band has an existing inline-start float in it, make sure we don't collide with
                // it (rule 3).
                match self.inline_start {
                    None => true,
                    Some(inline_start) => object.size.inline <= candidate_inline_end - inline_start,
                }
            },
        }
    }
}

// Float band storage

/// A persistent AA tree for float band storage.
///
/// Bands here are nonoverlapping, and there is guaranteed to be a band at block-position 0 and
/// another band at block-position infinity.
///
/// AA trees were chosen for simplicity.
///
/// See: <https://en.wikipedia.org/wiki/AA_tree>
///      <https://arxiv.org/pdf/1412.4882.pdf>
#[derive(Clone, Debug)]
pub struct FloatBandTree {
    pub root: FloatBandLink,
}

/// A single edge (or lack thereof) in the float band tree.
#[derive(Clone, Debug)]
pub struct FloatBandLink(pub Option<Arc<FloatBandNode>>);

/// A single node in the float band tree.
#[derive(Clone, Debug)]
pub struct FloatBandNode {
    /// The actual band.
    pub band: FloatBand,
    /// The left child.
    pub left: FloatBandLink,
    /// The right child.
    pub right: FloatBandLink,
    /// The level, which increases as you go up the tree.
    ///
    /// This value is needed for tree balancing.
    pub level: i32,
}

impl FloatBandTree {
    /// Creates a new float band tree.
    pub fn new() -> FloatBandTree {
        FloatBandTree {
            root: FloatBandLink(None),
        }
    }

    /// Returns the first band whose top is less than or equal to the given `block_position`.
    pub fn find(&self, block_position: Au) -> Option<FloatBand> {
        self.root.find(block_position)
    }

    /// Returns the first band whose top is strictly greater than to the given `block_position`.
    pub fn find_next(&self, block_position: Au) -> Option<FloatBand> {
        self.root.find_next(block_position)
    }

    /// Sets the side values of all bands within the given half-open range to be at least
    /// `new_value`.
    #[must_use]
    pub fn set_range(&self, range: &Range<Au>, side: FloatSide, new_value: Au) -> FloatBandTree {
        FloatBandTree {
            root: FloatBandLink(
                self.root
                    .0
                    .as_ref()
                    .map(|root| root.set_range(range, side, new_value)),
            ),
        }
    }

    /// Inserts a new band into the tree. If the band has the same level as a pre-existing one,
    /// replaces the existing band with the new one.
    #[must_use]
    pub fn insert(&self, band: FloatBand) -> FloatBandTree {
        FloatBandTree {
            root: self.root.insert(band),
        }
    }
}

impl Default for FloatBandTree {
    fn default() -> Self {
        Self::new()
    }
}

impl FloatBandNode {
    fn new(band: FloatBand) -> FloatBandNode {
        FloatBandNode {
            band,
            left: FloatBandLink(None),
            right: FloatBandLink(None),
            level: 1,
        }
    }

    /// Sets the side values of all bands within the given half-open range to be at least
    /// `new_value`.
    fn set_range(&self, range: &Range<Au>, side: FloatSide, new_value: Au) -> Arc<FloatBandNode> {
        let mut new_band = self.band;
        if self.band.top >= range.start && self.band.top < range.end {
            match side {
                FloatSide::InlineStart => {
                    new_band.inline_start = match new_band.inline_start {
                        Some(old_value) => Some(std::cmp::max(old_value, new_value)),
                        None => Some(new_value),
                    };
                },
                FloatSide::InlineEnd => {
                    new_band.inline_end = match new_band.inline_end {
                        Some(old_value) => Some(std::cmp::min(old_value, new_value)),
                        None => Some(new_value),
                    };
                },
            }
        }

        let new_left = match self.left.0 {
            None => FloatBandLink(None),
            Some(ref old_left) if range.start < new_band.top => {
                FloatBandLink(Some(old_left.set_range(range, side, new_value)))
            },
            Some(ref old_left) => FloatBandLink(Some((*old_left).clone())),
        };

        let new_right = match self.right.0 {
            None => FloatBandLink(None),
            Some(ref old_right) if range.end > new_band.top => {
                FloatBandLink(Some(old_right.set_range(range, side, new_value)))
            },
            Some(ref old_right) => FloatBandLink(Some((*old_right).clone())),
        };

        Arc::new(FloatBandNode {
            band: new_band,
            left: new_left,
            right: new_right,
            level: self.level,
        })
    }
}

impl FloatBandLink {
    /// Returns the first band whose top is less than or equal to the given `block_position`.
    fn find(&self, block_position: Au) -> Option<FloatBand> {
        let this = match self.0 {
            None => return None,
            Some(ref node) => node,
        };

        if block_position < this.band.top {
            return this.left.find(block_position);
        }

        // It's somewhere in this subtree, but we aren't sure whether it's here or in the right
        // subtree.
        if let Some(band) = this.right.find(block_position) {
            return Some(band);
        }

        Some(this.band)
    }

    /// Returns the first band whose top is strictly greater than the given `block_position`.
    fn find_next(&self, block_position: Au) -> Option<FloatBand> {
        let this = match self.0 {
            None => return None,
            Some(ref node) => node,
        };

        if block_position >= this.band.top {
            return this.right.find_next(block_position);
        }

        // It's somewhere in this subtree, but we aren't sure whether it's here or in the left
        // subtree.
        if let Some(band) = this.left.find_next(block_position) {
            return Some(band);
        }

        Some(this.band)
    }

    /// Inserts a new band into the tree. If the band has the same level as a pre-existing one,
    /// replaces the existing band with the new one.
    fn insert(&self, band: FloatBand) -> FloatBandLink {
        let mut this = match self.0 {
            None => return FloatBandLink(Some(Arc::new(FloatBandNode::new(band)))),
            Some(ref this) => (**this).clone(),
        };

        if band.top < this.band.top {
            this.left = this.left.insert(band);
            return FloatBandLink(Some(Arc::new(this))).skew().split();
        }
        if band.top > this.band.top {
            this.right = this.right.insert(band);
            return FloatBandLink(Some(Arc::new(this))).skew().split();
        }

        this.band = band;
        FloatBandLink(Some(Arc::new(this)))
    }

    /// Corrects tree balance:
    ///```text
    ///         T          L
    ///        / \        / \
    ///       L   R  →   A   T      if level(T) = level(L)
    ///      / \            / \
    ///     A   B          B   R
    /// ```
    fn skew(&self) -> FloatBandLink {
        if let Some(ref this) = self.0 {
            if let Some(ref left) = this.left.0 {
                if this.level == left.level {
                    return FloatBandLink(Some(Arc::new(FloatBandNode {
                        level: this.level,
                        left: left.left.clone(),
                        band: left.band,
                        right: FloatBandLink(Some(Arc::new(FloatBandNode {
                            level: this.level,
                            left: left.right.clone(),
                            band: this.band,
                            right: this.right.clone(),
                        }))),
                    })));
                }
            }
        }

        (*self).clone()
    }

    /// Corrects tree balance:
    ///```text
    ///         T            R
    ///        / \          / \
    ///       A   R   →    T   X    if level(T) = level(X)
    ///          / \      / \
    ///         B   X    A   B
    /// ```
    fn split(&self) -> FloatBandLink {
        if let Some(ref this) = self.0 {
            if let Some(ref right) = this.right.0 {
                if let Some(ref right_right) = right.right.0 {
                    if this.level == right_right.level {
                        return FloatBandLink(Some(Arc::new(FloatBandNode {
                            level: this.level + 1,
                            left: FloatBandLink(Some(Arc::new(FloatBandNode {
                                level: this.level,
                                left: this.left.clone(),
                                band: this.band,
                                right: right.left.clone(),
                            }))),
                            band: right.band,
                            right: right.right.clone(),
                        })));
                    }
                }
            }
        }

        (*self).clone()
    }
}

impl FloatBox {
    /// Creates a new float box.
    pub fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        display_inside: DisplayInside,
        contents: Contents,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        Self {
            contents: IndependentFormattingContext::construct(
                context,
                info,
                display_inside,
                contents,
                // Text decorations are not propagated to any out-of-flow descendants
                propagated_data.without_text_decorations(),
            ),
        }
    }

    /// Lay out this float box and its children. Note that the position will be relative to
    /// the float containing block formatting context. A later step adjusts the position
    /// to be relative to the containing block.
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
    ) -> BoxFragment {
        let style = self.contents.style().clone();
        positioning_context.layout_maybe_position_relative_fragment(
            layout_context,
            containing_block,
            &style,
            |positioning_context| {
                self.contents
                    .layout_float_or_atomic_inline(
                        layout_context,
                        positioning_context,
                        containing_block,
                    )
                    .fragment
            },
        )
    }
}

/// Layout state that we maintain when doing sequential traversals of the box tree in document
/// order.
///
/// This data is only needed for float placement and float interaction, and as such is only present
/// if the current block formatting context contains floats.
///
/// All coordinates here are relative to the start of the nearest ancestor block formatting context.
///
/// This structure is expected to be cheap to clone, in order to allow for "snapshots" that enable
/// restarting layout at any point in the tree.
#[derive(Clone)]
pub(crate) struct SequentialLayoutState {
    /// Holds all floats in this block formatting context.
    pub(crate) floats: FloatContext,
    /// The (logically) bottom border edge or top padding edge of the last in-flow block. Floats
    /// cannot be placed above this line.
    ///
    /// This is often, but not always, the same as the float ceiling. The float ceiling can be lower
    /// than this value because this value is calculated based on in-flow boxes only, while
    /// out-of-flow floats can affect the ceiling as well (see CSS 2.1 § 9.5.1 rule 6).
    pub(crate) bfc_relative_block_position: Au,
    /// Any collapsible margins that we've encountered after `bfc_relative_block_position`.
    pub(crate) current_margin: CollapsedMargin,
}

impl SequentialLayoutState {
    /// Creates a new empty `SequentialLayoutState`.
    pub(crate) fn new(max_inline_size: Au) -> SequentialLayoutState {
        SequentialLayoutState {
            floats: FloatContext::new(max_inline_size),
            current_margin: CollapsedMargin::zero(),
            bfc_relative_block_position: Au::zero(),
        }
    }

    /// Moves the current block position (logically) down by `block_distance`. This may be
    /// a negative advancement in the case that that block content overflows its
    /// container, when the container is adjusting the block position of the
    /// [`SequentialLayoutState`] after processing its overflowing content.
    ///
    /// Floats may not be placed higher than the current block position.
    pub(crate) fn advance_block_position(&mut self, block_distance: Au) {
        self.bfc_relative_block_position += block_distance;
        self.floats
            .set_ceiling_from_non_floats(self.bfc_relative_block_position);
    }

    /// Replace the entire [ContainingBlockPositionInfo] data structure stored
    /// by this [SequentialLayoutState]. Return the old data structure.
    pub(crate) fn replace_containing_block_position_info(
        &mut self,
        mut position_info: ContainingBlockPositionInfo,
    ) -> ContainingBlockPositionInfo {
        mem::swap(&mut position_info, &mut self.floats.containing_block_info);
        position_info
    }

    /// Return the current block position in the float containing block formatting
    /// context and any uncollapsed block margins.
    pub(crate) fn current_block_position_including_margins(&self) -> Au {
        self.bfc_relative_block_position + self.current_margin.solve()
    }

    /// Collapses margins, moving the block position down by the collapsed value of `current_margin`
    /// and resetting `current_margin` to zero.
    ///
    /// Call this method before laying out children when it is known that the start margin of the
    /// current fragment can't collapse with the margins of any of its children.
    pub(crate) fn collapse_margins(&mut self) {
        self.advance_block_position(self.current_margin.solve());
        self.current_margin = CollapsedMargin::zero();
    }

    /// Computes the position of the block-start border edge of an element
    /// with the provided `block_start_margin`, assuming no clearance.
    pub(crate) fn position_without_clearance(&self, block_start_margin: &CollapsedMargin) -> Au {
        // Adjoin `current_margin` and `block_start_margin` since there is no clearance.
        self.bfc_relative_block_position + self.current_margin.adjoin(block_start_margin).solve()
    }

    /// Computes the position of the block-start border edge of an element
    /// with the provided `block_start_margin`, assuming a clearance of 0px.
    pub(crate) fn position_with_zero_clearance(&self, block_start_margin: &CollapsedMargin) -> Au {
        // Clearance prevents `current_margin` and `block_start_margin` from being
        // adjoining, so we need to solve them separately and then sum.
        self.bfc_relative_block_position + self.current_margin.solve() + block_start_margin.solve()
    }

    /// Returns the block-end outer edge of the lowest float that is to be cleared (if any)
    /// by an element with the provided `clear` and `block_start_margin`.
    pub(crate) fn calculate_clear_position(
        &self,
        clear: Clear,
        block_start_margin: &CollapsedMargin,
    ) -> Option<Au> {
        if clear == Clear::None {
            return None;
        }

        // Calculate the hypothetical position where the element's top border edge
        // would have been if the element's `clear` property had been `none`.
        let hypothetical_block_position = self.position_without_clearance(block_start_margin);

        // Check if the hypothetical position is past the relevant floats,
        // in that case we don't need to add clearance.
        let clear_position = match clear {
            Clear::None => unreachable!(),
            Clear::InlineStart => self.floats.clear_inline_start_position,
            Clear::InlineEnd => self.floats.clear_inline_end_position,
            Clear::Both => self
                .floats
                .clear_inline_start_position
                .max(self.floats.clear_inline_end_position),
        };
        if hypothetical_block_position >= clear_position {
            None
        } else {
            Some(clear_position)
        }
    }

    /// Returns the amount of clearance (if any) that a block with the given `clear` value
    /// needs to have at `current_block_position_including_margins()`.
    /// `block_start_margin` is the top margin of the block, after collapsing (if possible)
    /// with the margin of its contents. This must not be included in `current_margin`,
    /// since adding clearance will prevent `current_margin` and `block_start_margin`
    /// from collapsing together.
    ///
    /// <https://www.w3.org/TR/2011/REC-CSS2-20110607/visuren.html#flow-control>
    pub(crate) fn calculate_clearance(
        &self,
        clear: Clear,
        block_start_margin: &CollapsedMargin,
    ) -> Option<Au> {
        self.calculate_clear_position(clear, block_start_margin)
            .map(|offset| offset - self.position_with_zero_clearance(block_start_margin))
    }

    /// A block that is replaced or establishes an independent formatting context can't overlap floats,
    /// it has to be placed next to them, and may get some clearance if there isn't enough space.
    /// Given such a block with the provided 'clear', 'block_start_margin', 'pbm' and 'object_size',
    /// this method finds an area that is big enough and doesn't overlap floats.
    /// It returns a tuple with:
    /// - The clearance amount (if any), which includes both the effect of 'clear'
    ///   and the extra space to avoid floats.
    /// - The LogicalRect in which the block can be placed without overlapping floats.
    pub(crate) fn calculate_clearance_and_inline_adjustment(
        &self,
        clear: Clear,
        block_start_margin: &CollapsedMargin,
        pbm: &PaddingBorderMargin,
        object_size: LogicalVec2<Au>,
    ) -> (Option<Au>, LogicalRect<Au>) {
        // First compute the clear position required by the 'clear' property.
        // The code below may then add extra clearance when the element can't fit
        // next to floats not covered by 'clear'.
        let clear_position = self.calculate_clear_position(clear, block_start_margin);
        let ceiling =
            clear_position.unwrap_or_else(|| self.position_without_clearance(block_start_margin));
        let mut placement = PlacementAmongFloats::new(&self.floats, ceiling, object_size, pbm);
        let placement_rect = placement.place();
        let position = &placement_rect.start_corner;
        let has_clearance = clear_position.is_some() || position.block > ceiling;
        let clearance = has_clearance
            .then(|| position.block - self.position_with_zero_clearance(block_start_margin));
        (clearance, placement_rect)
    }

    /// Adds a new adjoining margin.
    pub(crate) fn adjoin_assign(&mut self, margin: &CollapsedMargin) {
        self.current_margin.adjoin_assign(margin)
    }

    /// Get the offset of the current containing block and any uncollapsed margins.
    pub(crate) fn current_containing_block_offset(&self) -> Au {
        self.floats.containing_block_info.block_start +
            self.floats
                .containing_block_info
                .block_start_margins_not_collapsed
                .solve()
    }

    /// This function places a Fragment that has been created for a FloatBox.
    pub(crate) fn place_float_fragment(
        &mut self,
        box_fragment: &mut BoxFragment,
        containing_block: &ContainingBlock,
        margins_collapsing_with_parent_containing_block: CollapsedMargin,
        block_offset_from_containing_block_top: Au,
    ) {
        let block_start_of_containing_block_in_bfc = self.floats.containing_block_info.block_start +
            self.floats
                .containing_block_info
                .block_start_margins_not_collapsed
                .adjoin(&margins_collapsing_with_parent_containing_block)
                .solve();

        self.floats.set_ceiling_from_non_floats(
            block_start_of_containing_block_in_bfc + block_offset_from_containing_block_top,
        );

        let container_writing_mode = containing_block.style.writing_mode;
        let logical_float_size = box_fragment
            .content_rect
            .size
            .to_logical(container_writing_mode);
        let pbm_sums = box_fragment
            .padding_border_margin()
            .to_logical(container_writing_mode);
        let margin_box_start_corner = self.floats.add_float(&PlacementInfo {
            size: logical_float_size + pbm_sums.sum(),
            side: FloatSide::from_style_and_container_writing_mode(
                &box_fragment.style,
                container_writing_mode,
            )
            .expect("Float box wasn't floated!"),
            clear: Clear::from_style_and_container_writing_mode(
                &box_fragment.style,
                container_writing_mode,
            ),
        });

        // Re-calculate relative adjustment so that it is not lost when the BoxFragment's
        // `content_rect` is overwritten below.
        let relative_offset = match box_fragment.style.clone_position() {
            Position::Relative => relative_adjustement(&box_fragment.style, containing_block),
            _ => LogicalVec2::zero(),
        };

        // This is the position of the float in the float-containing block formatting context. We add the
        // existing start corner here because we may have already gotten some relative positioning offset.
        let new_position_in_bfc =
            margin_box_start_corner + pbm_sums.start_offset() + relative_offset;

        // This is the position of the float relative to the containing block start.
        let new_position_in_containing_block = LogicalVec2 {
            inline: new_position_in_bfc.inline - self.floats.containing_block_info.inline_start,
            block: new_position_in_bfc.block - block_start_of_containing_block_in_bfc,
        };

        box_fragment.content_rect = LogicalRect {
            start_corner: new_position_in_containing_block,
            size: box_fragment
                .content_rect
                .size
                .to_logical(container_writing_mode),
        }
        .as_physical(Some(containing_block));
    }
}
