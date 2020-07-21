/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Float layout.
//!
//! See CSS 2.1 § 9.5.1: https://www.w3.org/TR/CSS2/visuren.html#float-position

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NodeExt};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::geom::flow_relative::{Rect, Vec2};
use crate::style_ext::DisplayInside;
use euclid::num::Zero;
use servo_arc::Arc;
use std::f32;
use std::ops::Range;
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
    /// The logical width of this context. No floats may extend outside the width of this context
    /// unless they are as far (logically) left or right as possible.
    pub inline_size: Length,
    /// The current (logically) vertical position. No new floats may be placed (logically) above
    /// this line.
    pub ceiling: Length,
}

impl FloatContext {
    /// Returns a new float context representing a containing block with the given content
    /// inline-size.
    pub fn new(inline_size: Length) -> Self {
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
            inline_size,
            ceiling: Length::zero(),
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

    /// Returns the highest block position that is both logically below the current ceiling and
    /// clear of floats on the given side or sides.
    pub fn clearance(&self, side: ClearSide) -> Length {
        let mut band = self.bands.find(self.ceiling).unwrap();
        while !band.is_clear(side) {
            let next_band = self.bands.find_next(band.top).unwrap();
            if next_band.top.px().is_infinite() {
                break;
            }
            band = next_band;
        }
        band.top.max(self.ceiling)
    }

    /// Places a new float and adds it to the list. Returns the start corner of its margin box.
    pub fn add_float(&mut self, new_float: FloatInfo) -> Vec2<Length> {
        // Find the first band this float fits in.
        let mut first_band = self.bands.find(self.ceiling).unwrap();
        while !first_band.float_fits(&new_float, self.inline_size) {
            let next_band = self.bands.find_next(first_band.top).unwrap();
            if next_band.top.px().is_infinite() {
                break;
            }
            first_band = next_band;
        }

        // The float fits perfectly here. Place it.
        let (new_float_origin, new_float_extent);
        match new_float.side {
            FloatSide::Left => {
                new_float_origin = Vec2 {
                    inline: first_band.left.unwrap_or(Length::zero()),
                    block: first_band.top.max(self.ceiling),
                };
                new_float_extent = new_float_origin.inline + new_float.size.inline;
            },
            FloatSide::Right => {
                new_float_origin = Vec2 {
                    inline: first_band.right.unwrap_or(self.inline_size) - new_float.size.inline,
                    block: first_band.top.max(self.ceiling),
                };
                new_float_extent = new_float_origin.inline;
            },
        };
        let new_float_rect = Rect {
            start_corner: new_float_origin,
            size: new_float.size,
        };

        // Split the first band if necessary.
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

/// Information needed to place a float.
#[derive(Clone, Debug)]
pub struct FloatInfo {
    /// The *margin* box size of the float.
    pub size: Vec2<Length>,
    /// Whether the float is left or right.
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
    /// The distance from the left edge to the first legal (logically) horizontal position where
    /// floats may be placed. If `None`, there are no floats to the left; distinguishing between
    /// the cases of "a zero-width float is present" and "no floats at all are present" is
    /// necessary to, for example, clear past zero-width floats.
    pub left: Option<Length>,
    /// The distance from the right edge to the first legal (logically) horizontal position where
    /// floats may be placed. If `None`, there are no floats to the right; distinguishing between
    /// the cases of "a zero-width float is present" and "no floats at all are present" is
    /// necessary to, for example, clear past zero-width floats.
    pub right: Option<Length>,
}

impl FloatBand {
    fn is_clear(&self, side: ClearSide) -> bool {
        match (side, self.left, self.right) {
            (ClearSide::Left, Some(_), _) |
            (ClearSide::Right, _, Some(_)) |
            (ClearSide::Both, Some(_), _) |
            (ClearSide::Both, _, Some(_)) => false,
            (ClearSide::None, _, _) |
            (ClearSide::Left, None, _) |
            (ClearSide::Right, _, None) |
            (ClearSide::Both, None, None) => true,
        }
    }

    fn float_fits(&self, new_float: &FloatInfo, container_inline_size: Length) -> bool {
        let available_space =
            self.right.unwrap_or(container_inline_size) - self.left.unwrap_or(Length::zero());
        self.is_clear(new_float.clear) &&
            (new_float.size.inline <= available_space ||
                (self.left.is_none() && self.right.is_none()))
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
}
