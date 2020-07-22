/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Float layout.
//!
//! See CSS 2.1 § 9.5.1: https://www.w3.org/TR/CSS2/visuren.html#float-position

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NodeExt};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::{BoxFragment, CollapsedBlockMargins, CollapsedMargin, FloatFragment};
use crate::fragments::{Fragment, HoistedFloatFragment};
use crate::geom::flow_relative::{Rect, Vec2};
use crate::positioned::PositioningContext;
use crate::style_ext::{ComputedValuesExt, DisplayInside};
use crate::ContainingBlock;
use euclid::num::Zero;
use servo_arc::Arc;
use std::f32;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::ops::Range;
use style::computed_values::clear::T as ClearProperty;
use style::computed_values::float::T as FloatProperty;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::specified::text::TextDecorationLine;

/// A floating box.
#[derive(Debug, Serialize)]
pub(crate) struct FloatBox {
    /// The formatting context that makes up the content of this box.
    pub contents: IndependentFormattingContext,
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
    /// Distances from the logical left side of the block formatting context to the logical sides
    /// of the current containing block.
    pub walls: InlineWalls,
    /// The (logically) lowest margin edge of the last left float.
    pub clear_left_position: Length,
    /// The (logically) lowest margin edge of the last right float.
    pub clear_right_position: Length,
}

/// Distances from the logical left side of the block formatting context to the logical sides of
/// the current containing block.
#[derive(Clone, Copy, Debug)]
pub struct InlineWalls {
    /// The distance from the logical left side of the block formatting context to the logical
    /// left side of the current containing block.
    pub left: Length,
    /// The distance from the logical *left* side of the block formatting context to the logical
    /// right side of this object's containing block.
    pub right: Length,
}

impl FloatContext {
    /// Returns a new float context representing a containing block with the given content
    /// inline-size.
    pub fn new() -> Self {
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
            walls: InlineWalls::new(),
            clear_left_position: Length::zero(),
            clear_right_position: Length::zero(),
        }
    }

    /// Returns the current ceiling value. No new floats may be placed (logically) above this line.
    pub fn ceiling(&self) -> Length {
        self.ceiling
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
        while !first_band.object_fits(&object, &self.walls) {
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
                    Some(band_left) => band_left.max(self.walls.left),
                    None => self.walls.left,
                };
                Vec2 {
                    inline: left_object_edge,
                    block: first_band.top.max(self.ceiling),
                }
            },
            FloatSide::Right => {
                let right_object_edge = match first_band.right {
                    Some(band_right) => band_right.min(self.walls.right),
                    None => self.walls.right,
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
        let new_float_origin = self.place_object(new_float);
        let new_float_extent = match new_float.side {
            FloatSide::Left => new_float_origin.inline + new_float.size.inline,
            FloatSide::Right => new_float_origin.inline,
        };
        let new_float_rect = Rect {
            start_corner: new_float_origin,
            size: new_float.size.clone(),
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

impl InlineWalls {
    fn new() -> InlineWalls {
        InlineWalls {
            left: Length::zero(),
            right: Length::new(f32::INFINITY),
        }
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
    // Determines whether an object fits in a band.
    fn object_fits(&self, object: &PlacementInfo, walls: &InlineWalls) -> bool {
        match object.side {
            FloatSide::Left => {
                // Compute a candidate left position for the object.
                let candidate_left = match self.left {
                    None => walls.left,
                    Some(left) => left.max(walls.left),
                };

                // If this band has an existing left float in it, then make sure that the object
                // doesn't stick out past the right edge (rule 7).
                if self.left.is_some() && candidate_left + object.size.inline > walls.right {
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
                    None => walls.right,
                    Some(right) => right.min(walls.right),
                };

                // If this band has an existing right float in it, then make sure that the new
                // object doesn't stick out past the left edge (rule 7).
                if self.right.is_some() && candidate_right - object.size.inline < walls.left {
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

    // Inserts a new band into the tree. If the band has the same level as a pre-existing one,
    // replaces the existing band with the new one.
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

    // Corrects tree balance:
    //
    //         T          L
    //        / \        / \
    //       L   R  →   A   T      if level(T) = level(L)
    //      / \            / \
    //     A   B          B   R
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

    // Corrects tree balance:
    //
    //         T            R
    //        / \          / \
    //       A   R   →    T   X    if level(T) = level(X)
    //          / \      / \
    //         B   X    A   B
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

// Float boxes

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

    pub fn layout(
        &mut self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        mut sequential_layout_state: Option<&mut SequentialLayoutState>,
    ) {
        let sequential_layout_state = sequential_layout_state
            .as_mut()
            .expect("Tried to lay out a float with no sequential placement state!");

        // Speculate that the float ceiling will be located at the current block position plus the
        // result of solving any margins we're building up. This is usually right, but it can be
        // incorrect if there are more in-flow collapsible margins yet to be seen. An example
        // showing when this can go wrong:
        //
        //      <div style="margin: 5px"></div>
        //      <div style="float: left"></div>
        //      <div style="margin: 10px"></div>
        //
        // Assuming these are all in-flow, the float should be placed 10px down from the start, not
        // 5px, but we can't know that because we haven't seen the block after this float yet.
        //
        // FIXME(pcwalton): Implement the proper behavior when speculation fails. Either detect it
        // afterward and fix it up, or detect this situation ahead of time via lookahead and make
        // sure `current_margin` is accurate before calling this method.
        sequential_layout_state.floats.lower_ceiling(
            sequential_layout_state.bfc_relative_block_position +
                sequential_layout_state.current_margin.solve(),
        );

        let style = match self.contents {
            IndependentFormattingContext::Replaced(ref replaced) => replaced.style.clone(),
            IndependentFormattingContext::NonReplaced(ref non_replaced) => {
                non_replaced.style.clone()
            },
        };
        let float_context = &mut sequential_layout_state.floats;
        let box_fragment = positioning_context.layout_maybe_position_relative_fragment(
            layout_context,
            containing_block,
            &style,
            |mut positioning_context| {
                // Margin is computed this way regardless of whether the element is replaced
                // or non-replaced.
                let pbm = style.padding_border_margin(containing_block);
                let margin = pbm.margin.auto_is(|| Length::zero());
                let pbm_sums = &(&pbm.padding + &pbm.border) + &margin;

                let (content_size, fragments);
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
                            0,
                        );
                        content_size = Vec2 {
                            inline: inline_size,
                            block: box_size
                                .block
                                .auto_is(|| independent_layout.content_block_size),
                        };
                        fragments = independent_layout.fragments;
                    },
                    IndependentFormattingContext::Replaced(ref replaced) => {
                        // https://drafts.csswg.org/css2/#float-replaced-width
                        // https://drafts.csswg.org/css2/#inline-replaced-height
                        content_size = replaced.contents.used_size_as_if_inline_element(
                            &containing_block,
                            &replaced.style,
                            &pbm,
                        );
                        fragments = replaced
                            .contents
                            .make_fragments(&replaced.style, content_size.clone());
                    },
                };

                let margin_box_start_corner = float_context.add_float(&PlacementInfo {
                    size: &content_size + &pbm_sums.sum(),
                    side: FloatSide::from_style(&style).expect("Float box wasn't floated!"),
                    clear: ClearSide::from_style(&style),
                });

                let content_rect = Rect {
                    start_corner: &margin_box_start_corner + &pbm_sums.start_offset(),
                    size: content_size.clone(),
                };

                // Clearance is handled internally by the float placement logic, so there's no need
                // to store it explicitly in the fragment.
                let clearance = Length::zero();

                BoxFragment::new(
                    self.contents.tag(),
                    style.clone(),
                    fragments,
                    content_rect,
                    pbm.padding,
                    pbm.border,
                    margin,
                    clearance,
                    CollapsedBlockMargins::zero(),
                )
            },
        );
        sequential_layout_state.push_float_fragment(ArcRefCell::new(Fragment::Box(box_fragment)));
    }
}

// Float fragment storage

// A persistent linked list that stores float fragments that need to be hoisted to their nearest
// ancestor containing block.
#[derive(Clone)]
struct FloatFragmentList {
    root: FloatFragmentLink,
}

// A single link in the float fragment list.
#[derive(Clone)]
struct FloatFragmentLink(Option<Arc<FloatFragmentNode>>);

// A single node in the float fragment list.
#[derive(Clone)]
struct FloatFragmentNode {
    // The fragment.
    fragment: ArcRefCell<Fragment>,
    // The next fragment (previous in document order).
    next: FloatFragmentLink,
}

impl FloatFragmentList {
    fn new() -> FloatFragmentList {
        FloatFragmentList {
            root: FloatFragmentLink(None),
        }
    }
}

// Sequential layout state

// Layout state that we maintain when doing sequential traversals of the box tree in document
// order.
//
// This data is only needed for float placement and float interaction, and as such is only present
// if the current block formatting context contains floats.
//
// All coordinates here are relative to the start of the nearest ancestor block formatting context.
//
// This structure is expected to be cheap to clone, in order to allow for "snapshots" that enable
// restarting layout at any point in the tree.
#[derive(Clone)]
pub(crate) struct SequentialLayoutState {
    // Holds all floats in this block formatting context.
    pub(crate) floats: FloatContext,
    // A list of all float fragments in this block formatting context. These are gathered up and
    // hoisted to the top of the BFC.
    bfc_float_fragments: FloatFragmentList,
    // The (logically) bottom border edge or top padding edge of the last in-flow block. Floats
    // cannot be placed above this line.
    //
    // This is often, but not always, the same as the float ceiling. The float ceiling can be lower
    // than this value because this value is calculated based on in-flow boxes only, while
    // out-of-flow floats can affect the ceiling as well (see CSS 2.1 § 9.5.1 rule 6).
    bfc_relative_block_position: Length,
    // Any collapsible margins that we've encountered after `bfc_relative_block_position`.
    current_margin: CollapsedMargin,
}

impl SequentialLayoutState {
    // Creates a new empty `SequentialLayoutState`.
    pub(crate) fn new() -> SequentialLayoutState {
        SequentialLayoutState {
            floats: FloatContext::new(),
            current_margin: CollapsedMargin::zero(),
            bfc_relative_block_position: Length::zero(),
            bfc_float_fragments: FloatFragmentList::new(),
        }
    }

    // Moves the current block position (logically) down by `block_distance`.
    //
    // Floats may not be placed higher than the current block position.
    pub(crate) fn advance_block_position(&mut self, block_distance: Length) {
        self.bfc_relative_block_position += block_distance;
        self.floats.lower_ceiling(self.bfc_relative_block_position);
    }

    // Collapses margins, moving the block position down by the collapsed value of `current_margin`
    // and resetting `current_margin` to zero.
    //
    // Call this method before laying out children when it is known that the start margin of the
    // current fragment can't collapse with the margins of any of its children.
    pub(crate) fn collapse_margins(&mut self) {
        self.advance_block_position(self.current_margin.solve());
        self.current_margin = CollapsedMargin::zero();
    }

    // Returns the amount of clearance that a block with the given `clear` value at the current
    // `bfc_relative_block_position` (with top margin included in `current_margin` if applicable)
    // needs to have.
    //
    // https://www.w3.org/TR/2011/REC-CSS2-20110607/visuren.html#flow-control
    pub(crate) fn calculate_clearance(&self, clear_side: ClearSide) -> Length {
        if clear_side == ClearSide::None {
            return Length::zero();
        }

        let hypothetical_block_position =
            self.bfc_relative_block_position + self.current_margin.solve();
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
        clear_position - hypothetical_block_position
    }

    /// Adds a new adjoining margin.
    pub(crate) fn adjoin_assign(&mut self, margin: &CollapsedMargin) {
        self.current_margin.adjoin_assign(margin)
    }

    /// Adds the float fragment to this list.
    pub(crate) fn push_float_fragment(&mut self, new_float_fragment: ArcRefCell<Fragment>) {
        self.bfc_float_fragments.root.0 = Some(Arc::new(FloatFragmentNode {
            fragment: new_float_fragment,
            next: FloatFragmentLink(self.bfc_float_fragments.root.0.take()),
        }));
    }

    /// Adds the float fragments we've been building up to the given vector.
    pub(crate) fn add_float_fragments_to_list(&self, fragment_list: &mut Vec<Fragment>) {
        let start_index = fragment_list.len();
        let mut link = &self.bfc_float_fragments.root;
        while let Some(ref node) = link.0 {
            fragment_list.push(Fragment::HoistedFloat(HoistedFloatFragment {
                fragment: node.fragment.clone(),
            }));
            link = &node.next;
        }
        fragment_list[start_index..].reverse();
    }
}
