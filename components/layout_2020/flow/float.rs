/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Float layout.
//!
//! See CSS 2.1 § 9.5.1: https://www.w3.org/TR/CSS2/visuren.html#float-position

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::{BoxFragment, CollapsedBlockMargins, CollapsedMargin, FloatFragment};
use crate::geom::flow_relative::{Rect, Vec2};
use crate::positioned::PositioningContext;
use crate::style_ext::{ComputedValuesExt, DisplayInside};
use crate::ContainingBlock;
use euclid::num::Zero;
use servo_arc::Arc;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::ops::Range;
use std::{f32, mem};
use style::computed_values::clear::T as ClearProperty;
use style::computed_values::float::T as FloatProperty;
use style::properties::ComputedValues;
use style::values::computed::{CSSPixelLength, Length};
use style::values::specified::text::TextDecorationLine;

/// A floating box.
#[derive(Debug, Serialize)]
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
    pub(crate) block_start: Length,
    /// Any uncollapsed block start margins that we have collected between the
    /// block start of the float containing independent block formatting context
    /// and this containing block, including for this containing block.
    pub(crate) block_start_margins_not_collapsed: CollapsedMargin,
    /// The distance from the inline start position of the float containing
    /// independent formatting context and the inline start of this containing
    /// block.
    pub inline_start: Length,
    /// The offset from the inline start position of the float containing
    /// independent formatting context to the inline end of this containing
    /// block.
    pub inline_end: Length,
}

impl ContainingBlockPositionInfo {
    pub fn new_with_inline_offsets(inline_start: Length, inline_end: Length) -> Self {
        Self {
            block_start: Length::zero(),
            block_start_margins_not_collapsed: CollapsedMargin::zero(),
            inline_start,
            inline_end,
        }
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
    /// The current (logically) vertical position. No new floats may be placed (logically) above
    /// this line.
    pub ceiling: Length,
    /// Details about the position of the containing block relative to the
    /// independent block formatting context that contains all of the floats
    /// this `FloatContext` positions.
    pub containing_block_info: ContainingBlockPositionInfo,
    /// The (logically) lowest margin edge of the last left float.
    pub clear_left_position: Length,
    /// The (logically) lowest margin edge of the last right float.
    pub clear_right_position: Length,
}

impl FloatContext {
    /// Returns a new float context representing a containing block with the given content
    /// inline-size.
    pub fn new(max_inline_size: Length) -> Self {
        let mut bands = FloatBandTree::new();
        bands = bands.insert(FloatBand {
            top: Length::zero(),
            left: None,
            right: None,
        });
        bands = bands.insert(FloatBand {
            top: Length::new(f32::INFINITY),
            left: None,
            right: None,
        });
        FloatContext {
            bands,
            ceiling: Length::zero(),
            containing_block_info: ContainingBlockPositionInfo::new_with_inline_offsets(
                Length::zero(),
                max_inline_size,
            ),
            clear_left_position: Length::zero(),
            clear_right_position: Length::zero(),
        }
    }

    /// (Logically) lowers the ceiling to at least `new_ceiling` units.
    ///
    /// If the ceiling is already logically lower (i.e. larger) than this, does nothing.
    pub fn lower_ceiling(&mut self, new_ceiling: Length) {
        self.ceiling = self.ceiling.max(new_ceiling);
    }

    /// Determines where a float with the given placement would go, but leaves the float context
    /// unmodified. Returns the start corner of its margin box.
    ///
    /// This should be used for placing inline elements and block formatting contexts so that they
    /// don't collide with floats.
    pub fn place_object(&self, object: &PlacementInfo) -> Vec2<Length> {
        let ceiling = match object.clear {
            ClearSide::None => self.ceiling,
            ClearSide::Left => self.ceiling.max(self.clear_left_position),
            ClearSide::Right => self.ceiling.max(self.clear_right_position),
            ClearSide::Both => self
                .ceiling
                .max(self.clear_left_position)
                .max(self.clear_right_position),
        };

        // Find the first band this float fits in.
        let mut first_band = self.bands.find(ceiling).unwrap();
        while !first_band.object_fits(&object, &self.containing_block_info) {
            let next_band = self.bands.find_next(first_band.top).unwrap();
            if next_band.top.px().is_infinite() {
                break;
            }
            first_band = next_band;
        }

        // The object fits perfectly here. Place it.
        match object.side {
            FloatSide::Left => {
                let left_object_edge = match first_band.left {
                    Some(band_left) => band_left.max(self.containing_block_info.inline_start),
                    None => self.containing_block_info.inline_start,
                };
                Vec2 {
                    inline: left_object_edge,
                    block: first_band.top.max(self.ceiling),
                }
            },
            FloatSide::Right => {
                let right_object_edge = match first_band.right {
                    Some(band_right) => band_right.min(self.containing_block_info.inline_end),
                    None => self.containing_block_info.inline_end,
                };
                Vec2 {
                    inline: right_object_edge - object.size.inline,
                    block: first_band.top.max(self.ceiling),
                }
            },
        }
    }

    /// Places a new float and adds it to the list. Returns the start corner of its margin box.
    pub fn add_float(&mut self, new_float: &PlacementInfo) -> Vec2<Length> {
        // Place the float.
        let new_float_origin = self.place_object(&new_float);
        let new_float_extent = match new_float.side {
            FloatSide::Left => new_float_origin.inline + new_float.size.inline,
            FloatSide::Right => new_float_origin.inline,
        };

        let new_float_rect = Rect {
            start_corner: new_float_origin,
            // If this float has a negative margin, we should only consider its non-negative
            // block size contribution when determing where to place it. When the margin is
            // so negative that it's placed completely above the current float ceiling, then
            // we should position it as if it had zero block size.
            size: Vec2 {
                inline: new_float.size.inline.max(CSSPixelLength::zero()),
                block: new_float.size.block.max(CSSPixelLength::zero()),
            },
        };

        // Update clear.
        match new_float.side {
            FloatSide::Left => {
                self.clear_left_position = self
                    .clear_left_position
                    .max(new_float_rect.max_block_position())
            },
            FloatSide::Right => {
                self.clear_right_position = self
                    .clear_right_position
                    .max(new_float_rect.max_block_position())
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
        self.ceiling = self.ceiling.max(new_float_rect.start_corner.block);
        new_float_rect.start_corner
    }
}

/// Information needed to place an object so that it doesn't collide with existing floats.
#[derive(Clone, Debug)]
pub struct PlacementInfo {
    /// The *margin* box size of the object.
    pub size: Vec2<Length>,
    /// Whether the object is (logically) aligned to the left or right.
    pub side: FloatSide,
    /// Which side or sides to clear floats on.
    pub clear: ClearSide,
}

/// Whether the float is left or right.
///
/// See CSS 2.1 § 9.5.1: https://www.w3.org/TR/CSS2/visuren.html#float-position
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatSide {
    Left,
    Right,
}

/// Which side or sides to clear floats on.
///
/// See CSS 2.1 § 9.5.2: https://www.w3.org/TR/CSS2/visuren.html#flow-control
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClearSide {
    None = 0,
    Left = 1,
    Right = 2,
    Both = 3,
}

/// Internal data structure that describes a nonoverlapping vertical region in which floats may be
/// placed. Floats must go between "left edge + `left`" and "right edge - `right`".
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatBand {
    /// The logical vertical position of the top of this band.
    pub top: Length,
    /// The distance from the left edge of the block formatting context to the first legal
    /// (logically) horizontal position where floats may be placed. If `None`, there are no floats
    /// to the left; distinguishing between the cases of "a zero-width float is present" and "no
    /// floats at all are present" is necessary to, for example, clear past zero-width floats.
    pub left: Option<Length>,
    /// The distance from the *left* edge of the block formatting context to the first legal
    /// (logically) horizontal position where floats may be placed. If `None`, there are no floats
    /// to the right; distinguishing between the cases of "a zero-width float is present" and "no
    /// floats at all are present" is necessary to, for example, clear past zero-width floats.
    pub right: Option<Length>,
}

impl FloatSide {
    fn from_style(style: &ComputedValues) -> Option<FloatSide> {
        match style.get_box().float {
            FloatProperty::None => None,
            FloatProperty::Left => Some(FloatSide::Left),
            FloatProperty::Right => Some(FloatSide::Right),
        }
    }
}

impl ClearSide {
    pub(crate) fn from_style(style: &ComputedValues) -> ClearSide {
        match style.get_box().clear {
            ClearProperty::None => ClearSide::None,
            ClearProperty::Left => ClearSide::Left,
            ClearProperty::Right => ClearSide::Right,
            ClearProperty::Both => ClearSide::Both,
        }
    }
}

impl FloatBand {
    /// Determines whether an object fits in a band. Returns true if the object fits.
    fn object_fits(&self, object: &PlacementInfo, walls: &ContainingBlockPositionInfo) -> bool {
        match object.side {
            FloatSide::Left => {
                // Compute a candidate left position for the object.
                let candidate_left = match self.left {
                    None => walls.inline_start,
                    Some(left) => left.max(walls.inline_start),
                };

                // If this band has an existing left float in it, then make sure that the object
                // doesn't stick out past the right edge (rule 7).
                if self.left.is_some() && candidate_left + object.size.inline > walls.inline_end {
                    return false;
                }

                // If this band has an existing right float in it, make sure we don't collide with
                // it (rule 3).
                match self.right {
                    None => true,
                    Some(right) => object.size.inline <= right - candidate_left,
                }
            },

            FloatSide::Right => {
                // Compute a candidate right position for the object.
                let candidate_right = match self.right {
                    None => walls.inline_end,
                    Some(right) => right.min(walls.inline_end),
                };

                // If this band has an existing right float in it, then make sure that the new
                // object doesn't stick out past the left edge (rule 7).
                if self.right.is_some() && candidate_right - object.size.inline < walls.inline_start
                {
                    return false;
                }

                // If this band has an existing left float in it, make sure we don't collide with
                // it (rule 3).
                match self.left {
                    None => true,
                    Some(left) => object.size.inline <= candidate_right - left,
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
/// See: https://en.wikipedia.org/wiki/AA_tree
///      https://arxiv.org/pdf/1412.4882.pdf
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
    pub fn find(&self, block_position: Length) -> Option<FloatBand> {
        self.root.find(block_position)
    }

    /// Returns the first band whose top is strictly greater than to the given `block_position`.
    pub fn find_next(&self, block_position: Length) -> Option<FloatBand> {
        self.root.find_next(block_position)
    }

    /// Sets the side values of all bands within the given half-open range to be at least
    /// `new_value`.
    #[must_use]
    pub fn set_range(
        &self,
        range: &Range<Length>,
        side: FloatSide,
        new_value: Length,
    ) -> FloatBandTree {
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
    fn set_range(
        &self,
        range: &Range<Length>,
        side: FloatSide,
        new_value: Length,
    ) -> Arc<FloatBandNode> {
        let mut new_band = self.band.clone();
        if self.band.top >= range.start && self.band.top < range.end {
            match side {
                FloatSide::Left => match new_band.left {
                    None => new_band.left = Some(new_value),
                    Some(ref mut old_value) => *old_value = old_value.max(new_value),
                },
                FloatSide::Right => match new_band.right {
                    None => new_band.right = Some(new_value),
                    Some(ref mut old_value) => *old_value = old_value.min(new_value),
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
    fn find(&self, block_position: Length) -> Option<FloatBand> {
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

        Some(this.band.clone())
    }

    /// Returns the first band whose top is strictly greater than the given `block_position`.
    fn find_next(&self, block_position: Length) -> Option<FloatBand> {
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

        Some(this.band.clone())
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
    ///
    ///         T          L
    ///        / \        / \
    ///       L   R  →   A   T      if level(T) = level(L)
    ///      / \            / \
    ///     A   B          B   R
    fn skew(&self) -> FloatBandLink {
        if let Some(ref this) = self.0 {
            if let Some(ref left) = this.left.0 {
                if this.level == left.level {
                    return FloatBandLink(Some(Arc::new(FloatBandNode {
                        level: this.level,
                        left: left.left.clone(),
                        band: left.band.clone(),
                        right: FloatBandLink(Some(Arc::new(FloatBandNode {
                            level: this.level,
                            left: left.right.clone(),
                            band: this.band.clone(),
                            right: this.right.clone(),
                        }))),
                    })));
                }
            }
        }

        (*self).clone()
    }

    /// Corrects tree balance:
    ///
    ///         T            R
    ///        / \          / \
    ///       A   R   →    T   X    if level(T) = level(X)
    ///          / \      / \
    ///         B   X    A   B
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
                                band: this.band.clone(),
                                right: right.left.clone(),
                            }))),
                            band: right.band.clone(),
                            right: right.right.clone(),
                        })));
                    }
                }
            }
        }

        (*self).clone()
    }
}

impl Debug for FloatFragment {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "FloatFragment")
    }
}

impl FloatBox {
    /// Creates a new float box.
    pub fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        display_inside: DisplayInside,
        contents: Contents,
    ) -> Self {
        Self {
            contents: IndependentFormattingContext::construct(
                context,
                info,
                display_inside,
                contents,
                // Text decorations are not propagated to any out-of-flow descendants
                TextDecorationLine::NONE,
            ),
        }
    }

    /// Lay out this float box and its children. Note that the position will be relative to
    /// the float containing block formatting context. A later step adjusts the position
    /// to be relative to the containing block.
    pub fn layout(
        &mut self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
    ) -> BoxFragment {
        let style = self.contents.style().clone();
        positioning_context.layout_maybe_position_relative_fragment(
            layout_context,
            containing_block,
            &style,
            |mut positioning_context| {
                // Margin is computed this way regardless of whether the element is replaced
                // or non-replaced.
                let pbm = style.padding_border_margin(containing_block);
                let margin = pbm.margin.auto_is(|| Length::zero());
                let pbm_sums = &(&pbm.padding + &pbm.border) + &margin;

                let (content_size, children);
                match self.contents {
                    IndependentFormattingContext::NonReplaced(ref mut non_replaced) => {
                        // Calculate inline size.
                        // https://drafts.csswg.org/css2/#float-width
                        let box_size = non_replaced.style.content_box_size(&containing_block, &pbm);
                        let max_box_size = non_replaced
                            .style
                            .content_max_box_size(&containing_block, &pbm);
                        let min_box_size = non_replaced
                            .style
                            .content_min_box_size(&containing_block, &pbm)
                            .auto_is(Length::zero);

                        let tentative_inline_size = box_size.inline.auto_is(|| {
                            let available_size =
                                containing_block.inline_size - pbm_sums.inline_sum();
                            non_replaced
                                .inline_content_sizes(layout_context)
                                .shrink_to_fit(available_size)
                        });
                        let inline_size = tentative_inline_size
                            .clamp_between_extremums(min_box_size.inline, max_box_size.inline);

                        // Calculate block size.
                        // https://drafts.csswg.org/css2/#block-root-margin
                        // FIXME(pcwalton): Is a tree rank of zero correct here?
                        let containing_block_for_children = ContainingBlock {
                            inline_size,
                            block_size: box_size.block,
                            style: &non_replaced.style,
                        };
                        let independent_layout = non_replaced.layout(
                            layout_context,
                            &mut positioning_context,
                            &containing_block_for_children,
                        );
                        content_size = Vec2 {
                            inline: inline_size,
                            block: box_size
                                .block
                                .auto_is(|| independent_layout.content_block_size),
                        };
                        children = independent_layout.fragments;
                    },
                    IndependentFormattingContext::Replaced(ref replaced) => {
                        // https://drafts.csswg.org/css2/#float-replaced-width
                        // https://drafts.csswg.org/css2/#inline-replaced-height
                        content_size = replaced.contents.used_size_as_if_inline_element(
                            &containing_block,
                            &replaced.style,
                            None,
                            &pbm,
                        );
                        children = replaced
                            .contents
                            .make_fragments(&replaced.style, content_size.clone());
                    },
                };

                let content_rect = Rect {
                    start_corner: Vec2::zero(),
                    size: content_size,
                };

                BoxFragment::new(
                    self.contents.base_fragment_info(),
                    style.clone(),
                    children,
                    content_rect,
                    pbm.padding,
                    pbm.border,
                    margin,
                    // Clearance is handled internally by the float placement logic, so there's no need
                    // to store it explicitly in the fragment.
                    None, // clearance
                    CollapsedBlockMargins::zero(),
                )
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
    pub(crate) bfc_relative_block_position: Length,
    /// Any collapsible margins that we've encountered after `bfc_relative_block_position`.
    pub(crate) current_margin: CollapsedMargin,
}

impl SequentialLayoutState {
    /// Creates a new empty `SequentialLayoutState`.
    pub(crate) fn new(max_inline_size: Length) -> SequentialLayoutState {
        SequentialLayoutState {
            floats: FloatContext::new(max_inline_size),
            current_margin: CollapsedMargin::zero(),
            bfc_relative_block_position: Length::zero(),
        }
    }

    /// Moves the current block position (logically) down by `block_distance`.
    ///
    /// Floats may not be placed higher than the current block position.
    pub(crate) fn advance_block_position(&mut self, block_distance: Length) {
        self.bfc_relative_block_position += block_distance;
        self.floats.lower_ceiling(self.bfc_relative_block_position);
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
    pub(crate) fn current_block_position_including_margins(&self) -> Length {
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

    /// Returns the amount of clearance that a block with the given `clear` value at the current
    /// `bfc_relative_block_position` (with top margin included in `current_margin` if applicable)
    /// needs to have.
    ///
    /// https://www.w3.org/TR/2011/REC-CSS2-20110607/visuren.html#flow-control
    pub(crate) fn calculate_clearance(&self, clear_side: ClearSide) -> Option<Length> {
        if clear_side == ClearSide::None {
            return None;
        }

        let hypothetical_block_position = self.current_block_position_including_margins();
        let clear_position = match clear_side {
            ClearSide::None => unreachable!(),
            ClearSide::Left => self
                .floats
                .clear_left_position
                .max(hypothetical_block_position),
            ClearSide::Right => self
                .floats
                .clear_right_position
                .max(hypothetical_block_position),
            ClearSide::Both => self
                .floats
                .clear_left_position
                .max(self.floats.clear_right_position)
                .max(hypothetical_block_position),
        };
        if hypothetical_block_position >= clear_position {
            return None;
        }
        Some(clear_position - hypothetical_block_position)
    }

    /// Adds a new adjoining margin.
    pub(crate) fn adjoin_assign(&mut self, margin: &CollapsedMargin) {
        self.current_margin.adjoin_assign(margin)
    }

    /// Get the offset of the current containing block and any uncollapsed margins.
    pub(crate) fn current_containing_block_offset(&self) -> CSSPixelLength {
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
        margins_collapsing_with_parent_containing_block: CollapsedMargin,
        block_offset_from_containining_block_top: CSSPixelLength,
    ) {
        let block_start_of_containing_block_in_bfc = self.floats.containing_block_info.block_start +
            self.floats
                .containing_block_info
                .block_start_margins_not_collapsed
                .adjoin(&margins_collapsing_with_parent_containing_block)
                .solve();

        self.floats.lower_ceiling(
            block_start_of_containing_block_in_bfc + block_offset_from_containining_block_top,
        );

        let pbm_sums = &(&box_fragment.padding + &box_fragment.border) + &box_fragment.margin;
        let margin_box_start_corner = self.floats.add_float(&PlacementInfo {
            size: &box_fragment.content_rect.size + &pbm_sums.sum(),
            side: FloatSide::from_style(&box_fragment.style).expect("Float box wasn't floated!"),
            clear: ClearSide::from_style(&box_fragment.style),
        });

        // This is the position of the float in the float-containing block formatting context. We add the
        // existing start corner here because we may have already gotten some relative positioning offset.
        let new_position_in_bfc = &(&margin_box_start_corner + &pbm_sums.start_offset()) +
            &box_fragment.content_rect.start_corner;

        // This is the position of the float relative to the containing block start.
        let new_position_in_containing_block = Vec2 {
            inline: new_position_in_bfc.inline - self.floats.containing_block_info.inline_start,
            block: new_position_in_bfc.block - block_start_of_containing_block_in_bfc,
        };

        box_fragment.content_rect.start_corner = new_position_in_containing_block;
    }
}
