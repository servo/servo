/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::flow::inline::{Baseline, VerticalAlignMetrics};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::geom::{PhysicalPoint, PhysicalRect};
#[cfg(debug_assertions)]
use crate::layout_debug;
use app_units::Au;
use gfx::font::FontMetrics as GfxFontMetrics;
use gfx::text::glyph::GlyphStore;
use gfx_traits::print_tree::PrintTree;
#[cfg(not(debug_assertions))]
use serde::ser::{Serialize, Serializer};
use servo_arc::Arc as ServoArc;
use std::sync::Arc;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::dom::OpaqueNode;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::{Length, VerticalAlign};
use style::values::generics::box_::VerticalAlignKeyword;
use style::values::specified::text::TextDecorationLine;
use style::Zero;
use webrender_api::{FontInstanceKey, ImageKey};

// CSS 2.1 does not define the position of the line box's baseline.
// The baseline must be placed where ever it needs to be to fulfill
// vertical-align while minimizing the line box's height. In order
// to avoid back and forth and infinite loop situations where an
// inline element moves the baseline upwards or downwards and a
// following element moves the baseline in the opposite direction,
// we make the line box grow based on the previous overflow corrections,
// so all inline elements will eventually fit within the line box.
static BASELINE_CORRECTION_GROWTH_FACTOR: f32 = 0.1;

#[derive(Serialize)]
pub(crate) enum Fragment {
    Box(BoxFragment),
    Anonymous(AnonymousFragment),
    AbsoluteOrFixedPositioned(AbsoluteOrFixedPositionedFragment),
    Text(TextFragment),
    Image(ImageFragment),
}

#[derive(Serialize)]
pub(crate) struct AbsoluteOrFixedPositionedFragment {
    pub position: ComputedPosition,
    pub hoisted_fragment: ArcRefCell<Option<ArcRefCell<Fragment>>>,
}

#[derive(Serialize)]
pub(crate) struct BoxFragment {
    pub tag: OpaqueNode,
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

    pub block_margins_collapsed_with_children: CollapsedBlockMargins,

    /// The scrollable overflow of this box fragment.
    pub scrollable_overflow_from_children: PhysicalRect<Length>,

    pub vertical_align_metrics: VerticalAlignMetrics,
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
    pub children: Vec<ArcRefCell<Fragment>>,
    pub mode: WritingMode,

    /// The scrollable overflow of this anonymous fragment's children.
    pub scrollable_overflow: PhysicalRect<Length>,

    #[serde(skip_serializing)]
    pub vertical_align: VerticalAlign,
    pub vertical_align_metrics: VerticalAlignMetrics,
}

#[derive(Clone, Copy, Serialize)]
pub(crate) struct FontMetrics {
    pub ascent: Length,
    pub descent: Length,
    pub line_gap: Length,
    pub underline_offset: Length,
    pub underline_size: Length,
    pub strikeout_offset: Length,
    pub strikeout_size: Length,
    pub x_height: Length,
}

impl From<&GfxFontMetrics> for FontMetrics {
    fn from(metrics: &GfxFontMetrics) -> FontMetrics {
        FontMetrics {
            ascent: metrics.ascent.into(),
            descent: metrics.descent.into(),
            line_gap: metrics.line_gap.into(),
            underline_offset: metrics.underline_offset.into(),
            underline_size: metrics.underline_size.into(),
            strikeout_offset: metrics.strikeout_offset.into(),
            strikeout_size: metrics.strikeout_size.into(),
            x_height: metrics.x_height.into(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TextFragment {
    pub debug_id: DebugId,
    pub tag: OpaqueNode,
    #[serde(skip_serializing)]
    pub parent_style: ServoArc<ComputedValues>,
    pub rect: Rect<Length>,
    pub font_metrics: FontMetrics,
    #[serde(skip_serializing)]
    pub font_key: FontInstanceKey,
    pub glyphs: Vec<Arc<GlyphStore>>,
    /// A flag that represents the _used_ value of the text-decoration property.
    pub text_decoration_line: TextDecorationLine,
    pub vertical_align_metrics: VerticalAlignMetrics,
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
    fn position_mut(&mut self) -> Option<&mut Vec2<Length>> {
        match self {
            Fragment::Box(f) => Some(&mut f.content_rect.start_corner),
            Fragment::AbsoluteOrFixedPositioned(_) => None,
            Fragment::Anonymous(f) => Some(&mut f.rect.start_corner),
            Fragment::Text(f) => Some(&mut f.rect.start_corner),
            Fragment::Image(f) => Some(&mut f.rect.start_corner),
        }
    }

    pub fn offset_inline(&mut self, offset: &Length) {
        if let Some(position) = self.position_mut() {
            position.inline += *offset;
        }
    }

    pub fn vertical_align(
        &mut self,
        line_block_size: &mut Length,
        baseline: &mut Baseline,
        processed_baseline: &mut bool,
        accumulated_overflow: &mut Length,
    ) -> bool {
        let (vertical_align, vertical_align_metrics, block_size, pbm_block_start, x_height) =
            match self {
                Fragment::Box(f) => (
                    &f.style.get_box().vertical_align,
                    &f.vertical_align_metrics,
                    f.content_rect.size.block +
                        f.padding.block_sum() +
                        f.border.block_sum() +
                        f.margin.block_sum(),
                    f.padding.block_start + f.border.block_start + f.margin.block_start,
                    // XXX(ferjm) we should add this to Box fragments
                    Length::zero(),
                ),
                Fragment::Text(f) => (
                    &f.parent_style.get_box().vertical_align,
                    &f.vertical_align_metrics,
                    f.rect.size.block,
                    Length::zero(),
                    f.font_metrics.x_height,
                ),
                Fragment::Anonymous(f) => (
                    &f.vertical_align,
                    &f.vertical_align_metrics,
                    f.rect.size.block,
                    Length::zero(),
                    Length::zero(),
                ),
                _ => return false,
            };

        // If this is the first time we process this inline element, we may need
        // to modify the line baseline position based on the element's baseline.
        let mut baseline_or_line_size_changed = false;
        if !*processed_baseline {
            let current_baseline = baseline.clone();
            baseline.max_assign(&vertical_align_metrics.baseline);
            baseline_or_line_size_changed |= current_baseline != *baseline;
            *processed_baseline = true;
        }

        // Align, align, align.
        let mut block_position = match vertical_align {
            VerticalAlign::Keyword(keyword) => match keyword {
                VerticalAlignKeyword::Baseline =>
                // "Align the baseline of the box with the
                // baseline of the parent box.
                // If the box does not have a baseline, align
                // the bottom margin edge with the parent's baseline."
                {
                    baseline.space_above - vertical_align_metrics.baseline.space_above
                },
                VerticalAlignKeyword::Middle =>
                // "Align the vertical midpoint of the box with the
                // baseline of the parent box plus half the x-height
                // of the parent."
                {
                    baseline.space_above - (block_size * 0.5) - (x_height * 0.5)
                },
                VerticalAlignKeyword::Sub => Length::zero(),
                VerticalAlignKeyword::Super => Length::zero(),
                VerticalAlignKeyword::TextTop => Length::zero(),
                VerticalAlignKeyword::TextBottom => Length::zero(),
                VerticalAlignKeyword::Top =>
                // "Align the top of the aligned subtree with the top of the line box.""
                {
                    Length::zero()
                },
                VerticalAlignKeyword::Bottom =>
                // "Align the bottom of the aligned subtree with the bottom of the line box."
                {
                    *line_block_size - block_size
                },
            },
            VerticalAlign::Length(ref length) =>
            // <percentage>
            // Raise (positive value) or lower (negative value) the box
            // by this distance (a percentage of the 'line-height' value).
            // The value '0%' means the same as 'baseline'.
            //
            // <length>
            // Raise (positive value) or lower (negative value) the box by
            // this distance. The value '0cm' means the same as 'baseline'.
            {
                baseline.space_above -
                    vertical_align_metrics.baseline.space_above -
                    length
                        .to_used_value(Au::from_f32_px(line_block_size.px()))
                        .into()
            },
        } + pbm_block_start;

        let overflow = if block_position + block_size > *line_block_size {
            block_position + block_size - *line_block_size
        } else if block_position < Length::zero() {
            block_position
        } else {
            Length::zero()
        };

        if !overflow.is_zero() {
            // The inline element is positioned outside of the line box.
            // Modify the baseline position to fit the new inline element.
            baseline.space_above = baseline.space_above - block_position;
            // And position the element at the top of the baseline.
            // It'll be repositioned based on the new baseline in a following
            // iteration.
            block_position = Length::zero();
            baseline_or_line_size_changed = true;
            // Make the line box grow a little bit.
            // Refer to the BASELINE_CORRECTION_GROWTH_FACTOR declaration to
            // understand why.
            *line_block_size += accumulated_overflow.abs() * BASELINE_CORRECTION_GROWTH_FACTOR;
            *accumulated_overflow += overflow;
        }

        if let Some(position) = self.position_mut() {
            position.block = block_position;
        }
        baseline_or_line_size_changed
    }

    pub fn tag(&self) -> Option<OpaqueNode> {
        match self {
            Fragment::Box(fragment) => Some(fragment.tag),
            Fragment::Text(fragment) => Some(fragment.tag),
            Fragment::AbsoluteOrFixedPositioned(_) |
            Fragment::Anonymous(_) |
            Fragment::Image(_) => None,
        }
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

    pub fn scrollable_overflow(
        &self,
        containing_block: &PhysicalRect<Length>,
    ) -> PhysicalRect<Length> {
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

impl AbsoluteOrFixedPositionedFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!("AbsoluteOrFixedPositionedFragment"));
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
            vertical_align: VerticalAlign::Keyword(VerticalAlignKeyword::Baseline),
            vertical_align_metrics: VerticalAlignMetrics::from_baseline(Baseline::zero()),
        }
    }

    pub fn new(
        rect: Rect<Length>,
        children: Vec<Fragment>,
        mode: WritingMode,
        vertical_align: VerticalAlign,
        baseline: Baseline,
    ) -> Self {
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
            vertical_align,
            vertical_align_metrics: VerticalAlignMetrics::from_baseline(baseline),
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
        tag: OpaqueNode,
        style: ServoArc<ComputedValues>,
        children: Vec<Fragment>,
        content_rect: Rect<Length>,
        padding: Sides<Length>,
        border: Sides<Length>,
        margin: Sides<Length>,
        block_margins_collapsed_with_children: CollapsedBlockMargins,
        vertical_align_metrics: VerticalAlignMetrics,
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
            block_margins_collapsed_with_children,
            scrollable_overflow_from_children,
            vertical_align_metrics,
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
