/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::dom_traversal::{NodeAndStyleInfo, NodeExt, WhichPseudoElement};
use crate::geom::flow_relative::{Rect, Sides};
use crate::geom::{PhysicalPoint, PhysicalRect};
#[cfg(debug_assertions)]
use crate::layout_debug;
use crate::positioned::HoistedSharedFragment;
use gfx::font::FontMetrics as GfxFontMetrics;
use gfx::text::glyph::GlyphStore;
use gfx_traits::print_tree::PrintTree;
use gfx_traits::{combine_id_with_fragment_type, FragmentType};
#[cfg(not(debug_assertions))]
use serde::ser::{Serialize, Serializer};
use servo_arc::Arc as ServoArc;
use std::sync::Arc;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::dom::OpaqueNode;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::specified::text::TextDecorationLine;
use style::Zero;
use webrender_api::{FontInstanceKey, ImageKey};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum Tag {
    Node(OpaqueNode),
    BeforePseudo(OpaqueNode),
    AfterPseudo(OpaqueNode),
}

impl Tag {
    pub(crate) fn node(&self) -> OpaqueNode {
        match self {
            Self::Node(node) | Self::AfterPseudo(node) | Self::BeforePseudo(node) => *node,
        }
    }

    pub(crate) fn to_display_list_fragment_id(&self) -> u64 {
        let (node, content_type) = match self {
            Self::Node(node) => (node, FragmentType::FragmentBody),
            Self::AfterPseudo(node) => (node, FragmentType::BeforePseudoContent),
            Self::BeforePseudo(node) => (node, FragmentType::AfterPseudoContent),
        };
        combine_id_with_fragment_type(node.id() as usize, content_type) as u64
    }

    pub(crate) fn from_node_and_style_info<'dom, Node>(info: &NodeAndStyleInfo<Node>) -> Self
    where
        Node: NodeExt<'dom>,
    {
        let opaque_node = info.node.as_opaque();
        match info.pseudo_element_type {
            None => Self::Node(opaque_node),
            Some(WhichPseudoElement::Before) => Self::BeforePseudo(opaque_node),
            Some(WhichPseudoElement::After) => Self::AfterPseudo(opaque_node),
        }
    }
}

#[derive(Serialize)]
pub(crate) enum Fragment {
    Box(BoxFragment),
    // The original document position of a float in the document tree.
    Float,
    // A float hoisted up from its original position (where a placeholder `Fragment::Float` is) to
    // its containing block.
    HoistedFloat(HoistedFloatFragment),
    Anonymous(AnonymousFragment),
    AbsoluteOrFixedPositioned(AbsoluteOrFixedPositionedFragment),
    Text(TextFragment),
    Image(ImageFragment),
}

// A float hoisted up from its original position (where a placeholder `Fragment::Float` is) to its
// containing block.
#[derive(Serialize)]
pub(crate) struct HoistedFloatFragment {
    pub fragment: ArcRefCell<Fragment>,
}

#[derive(Serialize)]
pub(crate) struct AbsoluteOrFixedPositionedFragment {
    pub position: ComputedPosition,
    pub hoisted_fragment: ArcRefCell<HoistedSharedFragment>,
}

#[derive(Serialize)]
pub(crate) struct BoxFragment {
    pub tag: Tag,
    pub debug_id: DebugId,
    #[serde(skip_serializing)]
    pub style: ServoArc<ComputedValues>,
    pub children: Vec<ArcRefCell<Fragment>>,

    /// From the containing block’s start corner…?
    /// This might be broken when the containing block is in a different writing mode:
    /// https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
    pub content_rect: Rect<Length>,

    pub padding: Sides<Length>,
    pub border: Sides<Length>,
    pub margin: Sides<Length>,

    pub clearance: Length,

    pub block_margins_collapsed_with_children: CollapsedBlockMargins,

    /// The scrollable overflow of this box fragment.
    pub scrollable_overflow_from_children: PhysicalRect<Length>,
}

#[derive(Serialize)]
pub(crate) struct FloatFragment {
    pub box_fragment: BoxFragment,
}

#[derive(Serialize)]
pub(crate) struct CollapsedBlockMargins {
    pub collapsed_through: bool,
    pub start: CollapsedMargin,
    pub end: CollapsedMargin,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct CollapsedMargin {
    max_positive: Length,
    min_negative: Length,
}

/// Can contain child fragments with relative coordinates, but does not contribute to painting itself.
#[derive(Serialize)]
pub(crate) struct AnonymousFragment {
    pub debug_id: DebugId,
    pub rect: Rect<Length>,
    pub children: Vec<ArcRefCell<Fragment>>,
    pub mode: WritingMode,

    /// The scrollable overflow of this anonymous fragment's children.
    pub scrollable_overflow: PhysicalRect<Length>,
}

#[derive(Clone, Copy, Serialize)]
pub(crate) struct FontMetrics {
    pub ascent: Length,
    pub line_gap: Length,
    pub underline_offset: Length,
    pub underline_size: Length,
    pub strikeout_offset: Length,
    pub strikeout_size: Length,
}

impl From<&GfxFontMetrics> for FontMetrics {
    fn from(metrics: &GfxFontMetrics) -> FontMetrics {
        FontMetrics {
            ascent: metrics.ascent.into(),
            line_gap: metrics.line_gap.into(),
            underline_offset: metrics.underline_offset.into(),
            underline_size: metrics.underline_size.into(),
            strikeout_offset: metrics.strikeout_offset.into(),
            strikeout_size: metrics.strikeout_size.into(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TextFragment {
    pub debug_id: DebugId,
    pub tag: Tag,
    #[serde(skip_serializing)]
    pub parent_style: ServoArc<ComputedValues>,
    pub rect: Rect<Length>,
    pub font_metrics: FontMetrics,
    #[serde(skip_serializing)]
    pub font_key: FontInstanceKey,
    pub glyphs: Vec<Arc<GlyphStore>>,
    /// A flag that represents the _used_ value of the text-decoration property.
    pub text_decoration_line: TextDecorationLine,
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
            Fragment::HoistedFloat(_) |
            Fragment::Float |
            Fragment::AbsoluteOrFixedPositioned(_) => return,
            Fragment::Anonymous(f) => &mut f.rect.start_corner,
            Fragment::Text(f) => &mut f.rect.start_corner,
            Fragment::Image(f) => &mut f.rect.start_corner,
        };

        position.inline += *offset;
    }

    pub fn tag(&self) -> Option<Tag> {
        match self {
            Fragment::Box(fragment) => Some(fragment.tag),
            Fragment::Text(fragment) => Some(fragment.tag),
            Fragment::Float |
            Fragment::AbsoluteOrFixedPositioned(_) |
            Fragment::Anonymous(_) |
            Fragment::Image(_) |
            Fragment::HoistedFloat(_) => None,
        }
    }

    pub fn print(&self, tree: &mut PrintTree) {
        match self {
            Fragment::Box(fragment) => fragment.print(tree),
            Fragment::HoistedFloat(fragment) => fragment.print(tree),
            Fragment::Float => tree.add_item(format!("Float")),
            Fragment::AbsoluteOrFixedPositioned(fragment) => fragment.print(tree),
            Fragment::Anonymous(fragment) => fragment.print(tree),
            Fragment::Text(fragment) => fragment.print(tree),
            Fragment::Image(fragment) => fragment.print(tree),
        }
    }

    pub fn scrollable_overflow(
        &self,
        containing_block: &PhysicalRect<Length>,
    ) -> PhysicalRect<Length> {
        match self {
            Fragment::Box(fragment) => fragment.scrollable_overflow_for_parent(&containing_block),
            Fragment::HoistedFloat(fragment) => {
                (*fragment.fragment.borrow()).scrollable_overflow(&containing_block)
            },
            Fragment::Float | Fragment::AbsoluteOrFixedPositioned(_) => PhysicalRect::zero(),
            Fragment::Anonymous(fragment) => fragment.scrollable_overflow.clone(),
            Fragment::Text(fragment) => fragment
                .rect
                .to_physical(fragment.parent_style.writing_mode, &containing_block),
            Fragment::Image(fragment) => fragment
                .rect
                .to_physical(fragment.style.writing_mode, &containing_block),
        }
    }

    pub(crate) fn find<T>(
        &self,
        containing_block: &PhysicalRect<Length>,
        process_func: &mut impl FnMut(&Fragment, &PhysicalRect<Length>) -> Option<T>,
    ) -> Option<T> {
        if let Some(result) = process_func(self, containing_block) {
            return Some(result);
        }

        match self {
            Fragment::Box(fragment) => {
                let new_containing_block = fragment
                    .content_rect
                    .to_physical(fragment.style.writing_mode, containing_block)
                    .translate(containing_block.origin.to_vector());
                fragment
                    .children
                    .iter()
                    .find_map(|child| child.borrow().find(&new_containing_block, process_func))
            },
            Fragment::Anonymous(fragment) => {
                let new_containing_block = fragment
                    .rect
                    .to_physical(fragment.mode, containing_block)
                    .translate(containing_block.origin.to_vector());
                fragment
                    .children
                    .iter()
                    .find_map(|child| child.borrow().find(&new_containing_block, process_func))
            },
            _ => None,
        }
    }
}

impl HoistedFloatFragment {
    #[allow(dead_code)]
    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!("HoistedFloatFragment"));
        self.fragment.borrow().print(tree);
        tree.end_level();
    }
}

impl AbsoluteOrFixedPositionedFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!("AbsoluteOrFixedPositionedFragment"));
    }
}

impl AnonymousFragment {
    pub fn new(rect: Rect<Length>, children: Vec<Fragment>, mode: WritingMode) -> Self {
        // FIXME(mrobinson, bug 25564): We should be using the containing block
        // here to properly convert scrollable overflow to physical geometry.
        let containing_block = PhysicalRect::zero();
        let content_origin = rect.start_corner.to_physical(mode);
        let scrollable_overflow = children.iter().fold(PhysicalRect::zero(), |acc, child| {
            acc.union(
                &child
                    .scrollable_overflow(&containing_block)
                    .translate(content_origin.to_vector()),
            )
        });
        AnonymousFragment {
            debug_id: DebugId::new(),
            rect,
            children: children
                .into_iter()
                .map(|fragment| ArcRefCell::new(fragment))
                .collect(),
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
            child.borrow().print(tree);
        }
        tree.end_level();
    }
}

impl BoxFragment {
    pub fn new(
        tag: Tag,
        style: ServoArc<ComputedValues>,
        children: Vec<Fragment>,
        content_rect: Rect<Length>,
        padding: Sides<Length>,
        border: Sides<Length>,
        margin: Sides<Length>,
        clearance: Length,
        block_margins_collapsed_with_children: CollapsedBlockMargins,
    ) -> BoxFragment {
        // FIXME(mrobinson, bug 25564): We should be using the containing block
        // here to properly convert scrollable overflow to physical geometry.
        let containing_block = PhysicalRect::zero();
        let scrollable_overflow_from_children =
            children.iter().fold(PhysicalRect::zero(), |acc, child| {
                acc.union(&child.scrollable_overflow(&containing_block))
            });
        BoxFragment {
            tag,
            debug_id: DebugId::new(),
            style,
            children: children
                .into_iter()
                .map(|fragment| ArcRefCell::new(fragment))
                .collect(),
            content_rect,
            padding,
            border,
            margin,
            clearance,
            block_margins_collapsed_with_children,
            scrollable_overflow_from_children,
        }
    }

    pub fn scrollable_overflow(
        &self,
        containing_block: &PhysicalRect<Length>,
    ) -> PhysicalRect<Length> {
        let physical_padding_rect = self
            .padding_rect()
            .to_physical(self.style.writing_mode, containing_block);

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
                \nstyle={:p}",
            self.content_rect,
            self.padding_rect(),
            self.border_rect(),
            self.scrollable_overflow(&PhysicalRect::zero()),
            self.style.get_box().overflow_x,
            self.style.get_box().overflow_y,
            self.style,
        ));

        for child in &self.children {
            child.borrow().print(tree);
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
        let scrollable_overflow = self.scrollable_overflow(&containing_block);
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
