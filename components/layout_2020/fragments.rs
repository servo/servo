/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::geom::flow_relative::{Rect, Sides};
use crate::geom::{PhysicalPoint, PhysicalRect};
#[cfg(debug_assertions)]
use crate::layout_debug;
use crate::positioned::HoistedFragmentId;
use gfx::text::glyph::GlyphStore;
use gfx_traits::print_tree::PrintTree;
#[cfg(not(debug_assertions))]
use serde::ser::{Serialize, Serializer};
use servo_arc::Arc as ServoArc;
use std::sync::Arc;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::dom::OpaqueNode;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::Zero;
use webrender_api::{FontInstanceKey, ImageKey};

#[derive(Serialize)]
pub(crate) enum Fragment {
    Box(BoxFragment),
    Anonymous(AnonymousFragment),
    AbsoluteOrFixedPositioned(AbsoluteOrFixedPositionedFragment),
    Text(TextFragment),
    Image(ImageFragment),
}

#[derive(Serialize)]
pub(crate) struct AbsoluteOrFixedPositionedFragment(pub HoistedFragmentId);

#[derive(Serialize)]
pub(crate) struct BoxFragment {
    pub tag: OpaqueNode,
    pub debug_id: DebugId,
    #[serde(skip_serializing)]
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

    /// The scrollable overflow of this box fragment.
    pub scrollable_overflow_from_children: PhysicalRect<Length>,

    /// XXX Add thsi
    pub hoisted_fragment_id: Option<HoistedFragmentId>,
}

#[derive(Serialize)]
pub(crate) struct CollapsedBlockMargins {
    pub collapsed_through: bool,
    pub start: CollapsedMargin,
    pub end: CollapsedMargin,
}

#[derive(Clone, Copy, Serialize)]
pub(crate) struct CollapsedMargin {
    max_positive: Length,
    min_negative: Length,
}

/// Can contain child fragments with relative coordinates, but does not contribute to painting itself.
#[derive(Serialize)]
pub(crate) struct AnonymousFragment {
    pub debug_id: DebugId,
    pub rect: Rect<Length>,
    pub children: Vec<Fragment>,
    pub mode: WritingMode,

    /// The scrollable overflow of this anonymous fragment's children.
    pub scrollable_overflow: PhysicalRect<Length>,
}

#[derive(Serialize)]
pub(crate) struct TextFragment {
    pub debug_id: DebugId,
    pub tag: OpaqueNode,
    #[serde(skip_serializing)]
    pub parent_style: ServoArc<ComputedValues>,
    pub rect: Rect<Length>,
    pub ascent: Length,
    #[serde(skip_serializing)]
    pub font_key: FontInstanceKey,
    pub glyphs: Vec<Arc<GlyphStore>>,
}

#[derive(Serialize)]
pub(crate) struct ImageFragment {
    pub debug_id: DebugId,
    #[serde(skip_serializing)]
    pub style: ServoArc<ComputedValues>,
    pub rect: Rect<Length>,
    #[serde(skip_serializing)]
    pub image_key: ImageKey,
}

impl Fragment {
    pub fn offset_inline(&mut self, offset: &Length) {
        let position = match self {
            Fragment::Box(f) => &mut f.content_rect.start_corner,
            Fragment::AbsoluteOrFixedPositioned(_) => return,
            Fragment::Anonymous(f) => &mut f.rect.start_corner,
            Fragment::Text(f) => &mut f.rect.start_corner,
            Fragment::Image(f) => &mut f.rect.start_corner,
        };

        position.inline += *offset;
    }

    pub fn print(&self, tree: &mut PrintTree) {
        match self {
            Fragment::Box(fragment) => fragment.print(tree),
            Fragment::AbsoluteOrFixedPositioned(fragment) => fragment.print(tree),
            Fragment::Anonymous(fragment) => fragment.print(tree),
            Fragment::Text(fragment) => fragment.print(tree),
            Fragment::Image(fragment) => fragment.print(tree),
        }
    }

    pub fn scrollable_overflow(&self) -> PhysicalRect<Length> {
        // FIXME(mrobinson, bug 25564): We should be using the containing block
        // here to properly convert scrollable overflow to physical geometry.
        let containing_block = PhysicalRect::zero();
        match self {
            Fragment::Box(fragment) => fragment.scrollable_overflow_for_parent(&containing_block),
            Fragment::AbsoluteOrFixedPositioned(_) => PhysicalRect::zero(),
            Fragment::Anonymous(fragment) => fragment.scrollable_overflow.clone(),
            Fragment::Text(fragment) => fragment
                .rect
                .to_physical(fragment.parent_style.writing_mode, &containing_block),
            Fragment::Image(fragment) => fragment
                .rect
                .to_physical(fragment.style.writing_mode, &containing_block),
        }
    }

    pub fn is_hoisted(&self) -> bool {
        match self {
            Fragment::Box(fragment) if fragment.hoisted_fragment_id.is_some() => true,
            _ => false,
        }
    }

    pub fn hoisted_fragment_id(&self) -> Option<&HoistedFragmentId> {
        match self {
            Fragment::Box(fragment) => fragment.hoisted_fragment_id.as_ref(),
            _ => None,
        }
    }
}

impl AbsoluteOrFixedPositionedFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!("AbsoluteOrFixedPositionedFragment({:?})", self.0));
    }
}

impl AnonymousFragment {
    pub fn no_op(mode: WritingMode) -> Self {
        Self {
            debug_id: DebugId::new(),
            children: vec![],
            rect: Rect::zero(),
            mode,
            scrollable_overflow: PhysicalRect::zero(),
        }
    }

    pub fn new(rect: Rect<Length>, children: Vec<Fragment>, mode: WritingMode) -> Self {
        let content_origin = rect.start_corner.to_physical(mode);
        let scrollable_overflow = children.iter().fold(PhysicalRect::zero(), |acc, child| {
            acc.union(
                &child
                    .scrollable_overflow()
                    .translate(content_origin.to_vector()),
            )
        });
        AnonymousFragment {
            debug_id: DebugId::new(),
            rect,
            children,
            mode,
            scrollable_overflow,
        }
    }

    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!(
            "Anonymous\
                \nrect={:?}\
                \nscrollable_overflow={:?}",
            self.rect, self.scrollable_overflow
        ));

        for child in &self.children {
            child.print(tree);
        }
        tree.end_level();
    }
}

impl BoxFragment {
    pub fn new(
        tag: OpaqueNode,
        style: ServoArc<ComputedValues>,
        children: Vec<Fragment>,
        content_rect: Rect<Length>,
        padding: Sides<Length>,
        border: Sides<Length>,
        margin: Sides<Length>,
        block_margins_collapsed_with_children: CollapsedBlockMargins,
        hoisted_fragment_id: Option<HoistedFragmentId>,
    ) -> BoxFragment {
        let scrollable_overflow_from_children =
            children.iter().fold(PhysicalRect::zero(), |acc, child| {
                acc.union(&child.scrollable_overflow())
            });
        BoxFragment {
            tag,
            debug_id: DebugId::new(),
            style,
            children,
            content_rect,
            padding,
            border,
            margin,
            block_margins_collapsed_with_children,
            scrollable_overflow_from_children,
            hoisted_fragment_id,
        }
    }

    pub fn scrollable_overflow(&self) -> PhysicalRect<Length> {
        // FIXME(mrobinson, bug 25564): We should be using the containing block
        // here to properly convert scrollable overflow to physical geometry.
        let physical_padding_rect = self
            .padding_rect()
            .to_physical(self.style.writing_mode, &PhysicalRect::zero());

        let content_origin = self
            .content_rect
            .start_corner
            .to_physical(self.style.writing_mode);
        physical_padding_rect.union(
            &self
                .scrollable_overflow_from_children
                .translate(content_origin.to_vector()),
        )
    }

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
                \nborder rect={:?}\
                \nscrollable_overflow={:?}\
                \noverflow={:?} / {:?}\
                \nstyle={:p}\
                \nhoisted_id={:?}",
            self.content_rect,
            self.padding_rect(),
            self.border_rect(),
            self.scrollable_overflow(),
            self.style.get_box().overflow_x,
            self.style.get_box().overflow_y,
            self.style,
            self.hoisted_fragment_id,
        ));

        for child in &self.children {
            child.print(tree);
        }
        tree.end_level();
    }

    pub fn scrollable_overflow_for_parent(
        &self,
        containing_block: &PhysicalRect<Length>,
    ) -> PhysicalRect<Length> {
        let mut overflow = self
            .border_rect()
            .to_physical(self.style.writing_mode, containing_block);

        if self.style.get_box().overflow_y != ComputedOverflow::Visible &&
            self.style.get_box().overflow_x != ComputedOverflow::Visible
        {
            return overflow;
        }

        // https://www.w3.org/TR/css-overflow-3/#scrollable
        // Only include the scrollable overflow of a child box if it has overflow: visible.
        let scrollable_overflow = self.scrollable_overflow();
        let bottom_right = PhysicalPoint::new(
            overflow.max_x().max(scrollable_overflow.max_x()),
            overflow.max_y().max(scrollable_overflow.max_y()),
        );

        if self.style.get_box().overflow_y == ComputedOverflow::Visible {
            overflow.origin.y = overflow.origin.y.min(scrollable_overflow.origin.y);
            overflow.size.height = bottom_right.y - overflow.origin.y;
        }

        if self.style.get_box().overflow_x == ComputedOverflow::Visible {
            overflow.origin.x = overflow.origin.x.min(scrollable_overflow.origin.x);
            overflow.size.width = bottom_right.x - overflow.origin.x;
        }

        overflow
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

#[cfg(not(debug_assertions))]
#[derive(Clone)]
pub struct DebugId;

#[cfg(debug_assertions)]
#[derive(Clone, Serialize)]
#[serde(transparent)]
pub struct DebugId(u16);

#[cfg(not(debug_assertions))]
impl DebugId {
    pub fn new() -> DebugId {
        DebugId
    }
}

#[cfg(debug_assertions)]
impl DebugId {
    pub fn new() -> DebugId {
        DebugId(layout_debug::generate_unique_debug_id())
    }
}

#[cfg(not(debug_assertions))]
impl Serialize for DebugId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:p}", &self))
    }
}
