/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::geom::flow_relative::{Rect, Sides, Vec2};
use gfx::text::glyph::GlyphStore;
use gfx_traits::print_tree::PrintTree;
use servo_arc::Arc as ServoArc;
use std::sync::Arc;
use style::dom::OpaqueNode;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::Zero;
use webrender_api::{FontInstanceKey, ImageKey};

pub(crate) enum Fragment {
    Box(BoxFragment),
    Anonymous(AnonymousFragment),
    Text(TextFragment),
    Image(ImageFragment),
}

pub(crate) struct BoxFragment {
    pub tag: OpaqueNode,
    pub style: ServoArc<ComputedValues>,
    pub children: Vec<Fragment>,

    /// From the containing block’s start corner…?
    /// This might be broken when the containing block is in a different writing mode:
    /// https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
    pub content_rect: Rect<Length>,

    pub padding: Sides<Length>,
    pub border: Sides<Length>,
    pub margin: Sides<Length>,

    pub block_margins_collapsed_with_children: CollapsedBlockMargins,
}

pub(crate) struct CollapsedBlockMargins {
    pub collapsed_through: bool,
    pub start: CollapsedMargin,
    pub end: CollapsedMargin,
}

#[derive(Clone, Copy)]
pub(crate) struct CollapsedMargin {
    max_positive: Length,
    min_negative: Length,
}

/// Can contain child fragments with relative coordinates, but does not contribute to painting itself.
pub(crate) struct AnonymousFragment {
    pub rect: Rect<Length>,
    pub children: Vec<Fragment>,
    pub mode: WritingMode,
}

pub(crate) struct TextFragment {
    pub tag: OpaqueNode,
    pub parent_style: ServoArc<ComputedValues>,
    pub rect: Rect<Length>,
    pub ascent: Length,
    pub font_key: FontInstanceKey,
    pub glyphs: Vec<Arc<GlyphStore>>,
}

pub(crate) struct ImageFragment {
    pub style: ServoArc<ComputedValues>,
    pub rect: Rect<Length>,
    pub image_key: ImageKey,
}

impl Fragment {
    pub fn position_mut(&mut self) -> &mut Vec2<Length> {
        match self {
            Fragment::Box(f) => &mut f.content_rect.start_corner,
            Fragment::Anonymous(f) => &mut f.rect.start_corner,
            Fragment::Text(f) => &mut f.rect.start_corner,
            Fragment::Image(f) => &mut f.rect.start_corner,
        }
    }

    pub fn print(&self, tree: &mut PrintTree) {
        match self {
            Fragment::Box(fragment) => fragment.print(tree),
            Fragment::Anonymous(fragment) => fragment.print(tree),
            Fragment::Text(fragment) => fragment.print(tree),
            Fragment::Image(fragment) => fragment.print(tree),
        }
    }
}

impl AnonymousFragment {
    pub fn no_op(mode: WritingMode) -> Self {
        Self {
            children: vec![],
            rect: Rect::zero(),
            mode,
        }
    }

    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!(
            "Anonymous\
                \nrect={:?}",
            self.rect
        ));

        for child in &self.children {
            child.print(tree);
        }
        tree.end_level();
    }
}

impl BoxFragment {
    pub fn padding_rect(&self) -> Rect<Length> {
        self.content_rect.inflate(&self.padding)
    }

    pub fn border_rect(&self) -> Rect<Length> {
        self.padding_rect().inflate(&self.border)
    }

    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!(
            "Box\
                \ncontent={:?}\
                \npadding rect={:?}\
                \nborder rect={:?}",
            self.content_rect,
            self.padding_rect(),
            self.border_rect()
        ));

        for child in &self.children {
            child.print(tree);
        }
        tree.end_level();
    }
}

impl TextFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!(
            "Text num_glyphs={}",
            self.glyphs
                .iter()
                .map(|glyph_store| glyph_store.len().0)
                .sum::<isize>()
        ));
    }
}

impl ImageFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!(
            "Image\
                \nrect={:?}",
            self.rect
        ));
    }
}

impl CollapsedBlockMargins {
    pub fn from_margin(margin: &Sides<Length>) -> Self {
        Self {
            collapsed_through: false,
            start: CollapsedMargin::new(margin.block_start),
            end: CollapsedMargin::new(margin.block_end),
        }
    }

    pub fn zero() -> Self {
        Self {
            collapsed_through: false,
            start: CollapsedMargin::zero(),
            end: CollapsedMargin::zero(),
        }
    }
}

impl CollapsedMargin {
    pub fn zero() -> Self {
        Self {
            max_positive: Length::zero(),
            min_negative: Length::zero(),
        }
    }

    pub fn new(margin: Length) -> Self {
        Self {
            max_positive: margin.max(Length::zero()),
            min_negative: margin.min(Length::zero()),
        }
    }

    pub fn adjoin(&self, other: &Self) -> Self {
        Self {
            max_positive: self.max_positive.max(other.max_positive),
            min_negative: self.min_negative.min(other.min_negative),
        }
    }

    pub fn adjoin_assign(&mut self, other: &Self) {
        *self = self.adjoin(other);
    }

    pub fn solve(&self) -> Length {
        self.max_positive + self.min_negative
    }
}
