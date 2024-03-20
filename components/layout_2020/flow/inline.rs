/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! # Inline Formatting Context Layout
//!
//! Inline layout is divided into three phases:
//!
//! 1. Box Tree Construction
//! 2. Box to Line Layout
//! 3. Line to Fragment Layout
//!
//! The first phase happens during normal box tree constrution, while the second two phases happen
//! during fragment tree construction (sometimes called just "layout").
//!
//! ## Box Tree Construction
//!
//! During box tree construction, DOM elements are transformed into a box tree. This phase collects
//! all of the inline boxes, text, atomic inline elements (boxes with `display: inline-block` or
//! `display: inline-table` as well as things like images and canvas), absolutely positioned blocks,
//! and floated blocks.
//!
//! During the last part of this phase, whitespace is collapsed and text is segmented into
//! [`TextRun`]s based on script, chosen font, and line breaking opportunities. In addition, default
//! fonts are selected for every inline box. Each segment of text is shaped using HarfBuzz and
//! turned into a series of glyphs, which all have a size and a position relative to the origin of
//! the [`TextRun`] (calculated in later phases).
//!
//! The code for this phase is mainly in `construct.rs`, but text handling can also be found in
//! `text_runs.rs.`
//!
//! ## Box to Line Layout
//!
//! During the first phase of fragment tree construction, box tree items are laid out into
//! [`LineItem`]s and fragmented based on line boundaries. This is where line breaking happens. This
//! part of layout fragments boxes and their contents across multiple lines while positioning floats
//! and making sure non-floated contents flow around them. In addition, all atomic elements are laid
//! out, which may descend into their respective trees and create fragments. Finally, absolutely
//! positioned content is collected in order to later hoist it to the containing block for
//! absolutes.
//!
//! Note that during this phase, layout does not know the final block position of content. Only
//! during line to fragment layout, are the final block positions calculated based on the line's
//! final content and its vertical alignment. Instead, positions and line heights are calculated
//! relative to the line's final baseline which will be determined in the final phase.
//!
//! [`LineItem`]s represent a particular set of content on a line. Currently this is represented by
//! a linear series of items that describe the line's hierarchy of inline boxes and content. The
//! item types are:
//!
//!  - [`LineItem::TextRun`]
//!  - [`LineItem::StartInlineBox`]
//!  - [`LineItem::EndInlineBox`]
//!  - [`LineItem::Atomic`]
//!  - [`LineItem::AbsolutelyPositioned`]
//!  - [`LineItem::Float`]
//!
//! The code for this can be found by looking for methods of the form `layout_into_line_item()`.
//!
//! ## Line to Fragment Layout
//!
//! During the second phase of fragment tree construction, the final block position of [`LineItem`]s
//! is calculated and they are converted into [`Fragment`]s. After layout, the [`LineItem`]s are
//! discarded and the new fragments are incorporated into the fragment tree. The final static
//! position of absolutely positioned content is calculated and it is hoisted to its containing
//! block via [`PositioningContext`].
//!
//! The code for this phase, can mainly be found in `line.rs`.
//!

use std::cell::OnceCell;
use std::mem;

use app_units::Au;
use bitflags::bitflags;
use gfx::font::FontMetrics;
use gfx::text::glyph::GlyphStore;
use serde::Serialize;
use servo_arc::Arc;
use style::computed_values::vertical_align::T as VerticalAlign;
use style::computed_values::white_space::T as WhiteSpace;
use style::context::QuirksMode;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::generics::box_::VerticalAlignKeyword;
use style::values::generics::text::LineHeight;
use style::values::specified::text::{TextAlignKeyword, TextDecorationLine};
use style::values::specified::{TextAlignLast, TextJustify};
use style::Zero;
use webrender_api::FontInstanceKey;

use super::float::PlacementAmongFloats;
use super::line::{
    layout_line_items, AbsolutelyPositionedLineItem, AtomicLineItem, FloatLineItem,
    InlineBoxLineItem, LineItem, LineItemLayoutState, LineMetrics, TextRunLineItem,
};
use super::text_run::{add_or_get_font, get_font_for_first_font_for_style, TextRun};
use super::CollapsibleWithParentStartMargin;
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::flow::float::{FloatBox, SequentialLayoutState};
use crate::flow::FlowLayout;
use crate::formatting_contexts::{Baselines, IndependentFormattingContext};
use crate::fragment_tree::{
    BaseFragmentInfo, BoxFragment, CollapsedBlockMargins, CollapsedMargin, Fragment, FragmentFlags,
    PositioningFragment,
};
use crate::geom::{LogicalRect, LogicalVec2};
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext};
use crate::sizing::ContentSizes;
use crate::style_ext::{ComputedValuesExt, PaddingBorderMargin};
use crate::ContainingBlock;

// From gfxFontConstants.h in Firefox.
static FONT_SUBSCRIPT_OFFSET_RATIO: f32 = 0.20;
static FONT_SUPERSCRIPT_OFFSET_RATIO: f32 = 0.34;

#[derive(Debug, Serialize)]
pub(crate) struct InlineFormattingContext {
    pub(super) inline_level_boxes: Vec<ArcRefCell<InlineLevelBox>>,

    /// A store of font information for all the shaped segments in this formatting
    /// context in order to avoid duplicating this information.
    pub font_metrics: Vec<FontKeyAndMetrics>,

    pub(super) text_decoration_line: TextDecorationLine,

    /// Whether this IFC contains the 1st formatted line of an element:
    /// <https://www.w3.org/TR/css-pseudo-4/#first-formatted-line>.
    pub(super) has_first_formatted_line: bool,

    /// Whether or not this [`InlineFormattingContext`] contains floats.
    pub(super) contains_floats: bool,
}

/// A collection of data used to cache [`FontMetrics`] in the [`InlineFormattingContext`]
#[derive(Debug, Serialize)]
pub(crate) struct FontKeyAndMetrics {
    pub key: FontInstanceKey,
    pub pt_size: Au,
    pub metrics: FontMetrics,
}

#[derive(Debug, Serialize)]
pub(crate) enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(TextRun),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
    OutOfFlowFloatBox(FloatBox),
    Atomic(IndependentFormattingContext),
}

#[derive(Debug, Serialize)]
pub(crate) struct InlineBox {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    pub is_first_fragment: bool,
    pub is_last_fragment: bool,
    pub children: Vec<ArcRefCell<InlineLevelBox>>,
    /// The index of the default font in the [`InlineFormattingContext`]'s font metrics store.
    /// This is initialized during IFC shaping.
    pub default_font_index: Option<usize>,
}

/// Information about the current line under construction for a particular
/// [`InlineFormattingContextState`]. This tracks position and size information while
/// [`LineItem`]s are collected and is used as input when those [`LineItem`]s are
/// converted into [`Fragment`]s during the final phase of line layout. Note that this
/// does not store the [`LineItem`]s themselves, as they are stored as part of the
/// nesting state in the [`InlineFormattingContextState`].
struct LineUnderConstruction {
    /// The position where this line will start once it is laid out. This includes any
    /// offset from `text-indent`.
    start_position: LogicalVec2<Length>,

    /// The current inline position in the line being laid out into [`LineItem`]s in this
    /// [`InlineFormattingContext`] independent of the depth in the nesting level.
    inline_position: Length,

    /// The maximum block size of all boxes that ended and are in progress in this line.
    /// This uses [`LineBlockSizes`] instead of a simple value, because the final block size
    /// depends on vertical alignment.
    max_block_size: LineBlockSizes,

    /// Whether any active linebox has added a glyph or atomic element to this line, which
    /// indicates that the next run that exceeds the line length can cause a line break.
    has_content: bool,

    /// Whether or not there are floats that did not fit on the current line. Before
    /// the [`LineItem`]s of this line are laid out, these floats will need to be
    /// placed directly below this line, but still as children of this line's Fragments.
    has_floats_waiting_to_be_placed: bool,

    /// A rectangular area (relative to the containing block / inline formatting
    /// context boundaries) where we can fit the line box without overlapping floats.
    /// Note that when this is not empty, its start corner takes precedence over
    /// [`LineUnderConstruction::start_position`].
    placement_among_floats: OnceCell<LogicalRect<Length>>,

    /// The LineItems for the current line under construction that have already
    /// been committed to this line.
    line_items: Vec<LineItem>,
}

impl LineUnderConstruction {
    fn new(start_position: LogicalVec2<Length>) -> Self {
        Self {
            inline_position: start_position.inline,
            start_position,
            max_block_size: LineBlockSizes::zero(),
            has_content: false,
            has_floats_waiting_to_be_placed: false,
            placement_among_floats: OnceCell::new(),
            line_items: Vec::new(),
        }
    }

    fn line_block_start_considering_placement_among_floats(&self) -> Au {
        match self.placement_among_floats.get() {
            Some(placement_among_floats) => placement_among_floats.start_corner.block.into(),
            None => self.start_position.block.into(),
        }
    }

    fn replace_placement_among_floats(&mut self, new_placement: LogicalRect<Length>) {
        self.placement_among_floats.take();
        let _ = self.placement_among_floats.set(new_placement);
    }

    /// Trim the trailing whitespace in this line and return the width of the whitespace trimmed.
    fn trim_trailing_whitespace(&mut self) -> Length {
        // From <https://www.w3.org/TR/css-text-3/#white-space-phase-2>:
        // > 3. A sequence of collapsible spaces at the end of a line is removed,
        // >    as well as any trailing U+1680   OGHAM SPACE MARK whose white-space
        // >    property is normal, nowrap, or pre-line.
        let mut whitespace_trimmed = Length::zero();
        for item in self.line_items.iter_mut().rev() {
            if !item.trim_whitespace_at_end(&mut whitespace_trimmed) {
                break;
            }
        }

        whitespace_trimmed
    }

    /// Count the number of justification opportunities in this line.
    fn count_justification_opportunities(&self) -> usize {
        self.line_items
            .iter()
            .filter_map(|item| match item {
                LineItem::TextRun(text_run) => Some(
                    text_run
                        .text
                        .iter()
                        .map(|glyph_store| glyph_store.total_word_separators())
                        .sum::<usize>(),
                ),
                _ => None,
            })
            .sum()
    }
}

/// A block size relative to a line's final baseline. This is to track the size
/// contribution of a particular element of a line above and below the baseline.
/// These sizes can be combined with other baseline relative sizes before the
/// final baseline position is known. The values here are relative to the
/// overall line's baseline and *not* the nested baseline of an inline box.
#[derive(Clone, Debug)]
struct BaselineRelativeSize {
    /// The ascent above the baseline, where a positive value means a larger
    /// ascent. Thus, the top of this size contribution is `baseline_offset -
    /// ascent`.
    ascent: Au,

    /// The descent below the baseline, where a positive value means a larger
    /// descent. Thus, the bottom of this size contribution is `baseline_offset +
    /// descent`.
    descent: Au,
}

impl BaselineRelativeSize {
    fn zero() -> Self {
        Self {
            ascent: Au::zero(),
            descent: Au::zero(),
        }
    }

    fn max(&self, other: &Self) -> Self {
        BaselineRelativeSize {
            ascent: self.ascent.max(other.ascent),
            descent: self.descent.max(other.descent),
        }
    }

    /// Given an offset from the line's root baseline, adjust this [`BaselineRelativeSize`]
    /// by that offset. This is used to adjust a [`BaselineRelativeSize`] for different kinds
    /// of baseline-relative `vertical-align`. This will "move" measured size of a particular
    /// inline box's block size. For example, in the following HTML:
    ///
    /// ```html
    ///     <div>
    ///         <span style="vertical-align: 5px">child content</span>
    ///     </div>
    /// ````
    ///
    /// If this [`BaselineRelativeSize`] is for the `<span>` then the adjustment
    /// passed here would be equivalent to -5px.
    fn adjust_for_nested_baseline_offset(&mut self, baseline_offset: Au) {
        self.ascent -= baseline_offset;
        self.descent += baseline_offset;
    }
}

#[derive(Clone, Debug)]
struct LineBlockSizes {
    line_height: Length,
    baseline_relative_size_for_line_height: Option<BaselineRelativeSize>,
    size_for_baseline_positioning: BaselineRelativeSize,
}

impl LineBlockSizes {
    fn zero() -> Self {
        LineBlockSizes {
            line_height: Length::zero(),
            baseline_relative_size_for_line_height: None,
            size_for_baseline_positioning: BaselineRelativeSize::zero(),
        }
    }

    fn resolve(&self) -> Length {
        let height_from_ascent_and_descent = self
            .baseline_relative_size_for_line_height
            .as_ref()
            .map(|size| (size.ascent + size.descent).abs())
            .unwrap_or_else(Au::zero);
        self.line_height.max(height_from_ascent_and_descent.into())
    }

    fn max(&self, other: &LineBlockSizes) -> LineBlockSizes {
        let baseline_relative_size = match (
            self.baseline_relative_size_for_line_height.as_ref(),
            other.baseline_relative_size_for_line_height.as_ref(),
        ) {
            (Some(our_size), Some(other_size)) => Some(our_size.max(other_size)),
            (our_size, other_size) => our_size.or(other_size).cloned(),
        };
        Self {
            line_height: self.line_height.max(other.line_height),
            baseline_relative_size_for_line_height: baseline_relative_size,
            size_for_baseline_positioning: self
                .size_for_baseline_positioning
                .max(&other.size_for_baseline_positioning),
        }
    }

    fn max_assign(&mut self, other: &LineBlockSizes) {
        *self = self.max(other);
    }

    fn adjust_for_baseline_offset(&mut self, baseline_offset: Au) {
        if let Some(size) = self.baseline_relative_size_for_line_height.as_mut() {
            size.adjust_for_nested_baseline_offset(baseline_offset)
        }
        self.size_for_baseline_positioning
            .adjust_for_nested_baseline_offset(baseline_offset);
    }

    /// From <https://drafts.csswg.org/css2/visudet.html#line-height>:
    ///  > The inline-level boxes are aligned vertically according to their 'vertical-align'
    ///  > property. In case they are aligned 'top' or 'bottom', they must be aligned so as
    ///  > to minimize the line box height. If such boxes are tall enough, there are multiple
    ///  > solutions and CSS 2 does not define the position of the line box's baseline (i.e.,
    ///  > the position of the strut, see below).
    fn find_baseline_offset(&self) -> Au {
        match self.baseline_relative_size_for_line_height.as_ref() {
            Some(size) => size.ascent,
            None => {
                // This is the case mentinoned above where there are multiple solutions.
                // This code is putting the baseline roughly in the middle of the line.
                let leading = Au::from(self.resolve()) -
                    (self.size_for_baseline_positioning.ascent +
                        self.size_for_baseline_positioning.descent);
                leading.scale_by(0.5) + self.size_for_baseline_positioning.ascent
            },
        }
    }
}

/// The current unbreakable segment under construction for an inline formatting context.
/// Items accumulate here until we reach a soft line break opportunity during processing
/// of inline content or we reach the end of the formatting context.
struct UnbreakableSegmentUnderConstruction {
    /// The size of this unbreakable segment in both dimension.
    inline_size: Length,

    /// The maximum block size that this segment has. This uses [`LineBlockSizes`] instead of a
    /// simple value, because the final block size depends on vertical alignment.
    max_block_size: LineBlockSizes,

    /// The LineItems for the segment under construction
    line_items: Vec<LineItem>,

    /// The depth in the inline box hierarchy at the start of this segment. This is used
    /// to prefix this segment when it is pushed to a new line.
    inline_box_hierarchy_depth: Option<usize>,

    /// Whether any active linebox has added a glyph or atomic element to this line
    /// segment, which indicates that the next run that exceeds the line length can cause
    /// a line break.
    has_content: bool,

    /// The inline size of any trailing whitespace in this segment.
    trailing_whitespace_size: Length,
}

impl UnbreakableSegmentUnderConstruction {
    fn new() -> Self {
        Self {
            inline_size: Length::zero(),
            max_block_size: LineBlockSizes {
                line_height: Length::zero(),
                baseline_relative_size_for_line_height: None,
                size_for_baseline_positioning: BaselineRelativeSize::zero(),
            },
            line_items: Vec::new(),
            inline_box_hierarchy_depth: None,
            has_content: false,
            trailing_whitespace_size: Length::zero(),
        }
    }

    /// Reset this segment after its contents have been committed to a line.
    fn reset(&mut self) {
        assert!(self.line_items.is_empty()); // Preserve allocated memory.
        self.inline_size = Length::zero();
        self.max_block_size = LineBlockSizes::zero();
        self.inline_box_hierarchy_depth = None;
        self.has_content = false;
        self.trailing_whitespace_size = Length::zero();
    }

    /// Push a single line item to this segment. In addition, record the inline box
    /// hierarchy depth if this is the first segment. The hierarchy depth is used to
    /// duplicate the necessary `StartInlineBox` tokens if this segment is ultimately
    /// placed on a new empty line.
    fn push_line_item(&mut self, line_item: LineItem, inline_box_hierarchy_depth: usize) {
        if self.line_items.is_empty() {
            self.inline_box_hierarchy_depth = Some(inline_box_hierarchy_depth);
        }
        self.line_items.push(line_item);
    }

    /// Trim whitespace from the beginning of this UnbreakbleSegmentUnderConstruction.
    ///
    /// From <https://www.w3.org/TR/css-text-3/#white-space-phase-2>:
    ///
    /// > Then, the entire block is rendered. Inlines are laid out, taking bidi
    /// > reordering into account, and wrapping as specified by the text-wrap
    /// > property. As each line is laid out,
    /// >  1. A sequence of collapsible spaces at the beginning of a line is removed.
    ///
    /// This prevents whitespace from being added to the beginning of a line.
    fn trim_leading_whitespace(&mut self) {
        let mut whitespace_trimmed = Length::zero();
        for item in self.line_items.iter_mut() {
            if !item.trim_whitespace_at_start(&mut whitespace_trimmed) {
                break;
            }
        }
        self.inline_size -= whitespace_trimmed;
    }

    /// Prepare this segment for placement on a new and empty line. This happens when the
    /// segment is too large to fit on the current line and needs to be placed on a new
    /// one.
    fn prepare_for_placement_on_empty_line(
        &mut self,
        line: &LineUnderConstruction,
        current_hierarchy_depth: usize,
    ) {
        self.trim_leading_whitespace();

        // The segment may start in the middle of an already processed inline box. In that
        // case we need to duplicate the `StartInlineBox` tokens as a prefix of the new
        // lines. For instance if the following segment is going to be placed on a new line:
        //
        // line = [StartInlineBox "every"]
        // segment = ["good" EndInlineBox "boy"]
        //
        // Then the segment must be prefixed with `StartInlineBox` before it is committed
        // to the empty line.
        let mut hierarchy_depth = self
            .inline_box_hierarchy_depth
            .unwrap_or(current_hierarchy_depth);
        if hierarchy_depth == 0 {
            return;
        }
        let mut hierarchy = Vec::new();
        let mut skip_depth = 0;
        for item in line.line_items.iter().rev() {
            match item {
                // We need to skip over any inline boxes that are not in our hierarchy. If
                // any inline box ends, we skip until it starts.
                LineItem::StartInlineBox(_) if skip_depth > 0 => skip_depth -= 1,
                LineItem::EndInlineBox => skip_depth += 1,

                // Otherwise copy the inline box to the hierarchy we are collecting.
                LineItem::StartInlineBox(inline_box) => {
                    let mut cloned_inline_box = inline_box.clone();
                    cloned_inline_box.is_first_fragment = false;
                    hierarchy.push(LineItem::StartInlineBox(cloned_inline_box));
                    hierarchy_depth -= 1;
                    if hierarchy_depth == 0 {
                        break;
                    }
                },
                _ => {},
            }
        }

        let segment_items = mem::take(&mut self.line_items);
        self.line_items = hierarchy.into_iter().rev().chain(segment_items).collect();
    }
}

struct InlineContainerState {
    /// The style of this inline container.
    style: Arc<ComputedValues>,

    /// Whether or not we have processed any content (an atomic element or text) for
    /// this inline box on the current line OR any previous line.
    has_content: bool,

    /// Indicates whether this nesting level have text decorations in effect.
    /// From <https://drafts.csswg.org/css-text-decor/#line-decoration>
    // "When specified on or propagated to a block container that establishes
    //  an IFC..."
    text_decoration_line: TextDecorationLine,

    /// The block size contribution of this container's default font ie the size of the
    /// "strut." Whether this is integrated into the [`Self::nested_strut_block_sizes`]
    /// depends on the line-height quirk described in
    /// <https://quirks.spec.whatwg.org/#the-line-height-calculation-quirk>.
    strut_block_sizes: LineBlockSizes,

    /// The strut block size of this inline container maxed with the strut block
    /// sizes of all inline container ancestors. In quirks mode, this will be
    /// zero, until we know that an element has inline content.
    nested_strut_block_sizes: LineBlockSizes,

    /// The baseline offset of this container from the baseline of the line. The is the
    /// cumulative offset of this container and all of its parents. In contrast to the
    /// `vertical-align` property a positive value indicates an offset "below" the
    /// baseline while a negative value indicates one "above" it (when the block direction
    /// is vertical).
    baseline_offset: Au,

    /// The font metrics of the non-fallback font for this container.
    font_metrics: FontMetrics,
}

struct InlineBoxContainerState {
    /// The container state common to both [`InlineBox`] and the root of the
    /// [`InlineFormattingContext`].
    base: InlineContainerState,

    /// The [`BaseFragmentInfo`] of the [`InlineBox`] that this state tracks.
    base_fragment_info: BaseFragmentInfo,

    /// The [`PaddingBorderMargin`] of the [`InlineBox`] that this state tracks.
    pbm: PaddingBorderMargin,

    /// Whether this is the last fragment of this InlineBox. This may not be the case if
    /// the InlineBox is split due to an block-in-inline-split and this is not the last of
    /// that split.
    is_last_fragment: bool,
}

pub(super) struct InlineFormattingContextState<'a, 'b> {
    positioning_context: &'a mut PositioningContext,
    containing_block: &'b ContainingBlock<'b>,
    sequential_layout_state: Option<&'a mut SequentialLayoutState>,
    layout_context: &'b LayoutContext<'b>,

    /// The list of [`FontMetrics`] used by the [`InlineFormattingContext`] that
    /// we are laying out.
    fonts: &'a Vec<FontKeyAndMetrics>,

    /// The [`InlineContainerState`] for the container formed by the root of the
    /// [`InlineFormattingContext`]. This is effectively the "root inline box" described
    /// by <https://drafts.csswg.org/css-inline/#model>:
    ///
    /// > The block container also generates a root inline box, which is an anonymous
    /// > inline box that holds all of its inline-level contents. (Thus, all text in an
    /// > inline formatting context is directly contained by an inline box, whether the root
    /// > inline box or one of its descendants.) The root inline box inherits from its
    /// > parent block container, but is otherwise unstyleable.
    root_nesting_level: InlineContainerState,

    /// A stack of [`InlineBoxContainerState`] that is used to produce [`LineItem`]s either when we
    /// reach the end of an inline box or when we reach the end of a line. Only at the end
    /// of the inline box is the state popped from the stack.
    inline_box_state_stack: Vec<InlineBoxContainerState>,

    /// A vector of fragment that are laid out. This includes one [`Fragment::Positioning`]
    /// per line that is currently laid out plus fragments for all floats, which
    /// are currently laid out at the top-level of each [`InlineFormattingContext`].
    fragments: Vec<Fragment>,

    /// Information about the line currently being laid out into [`LineItem`]s.
    current_line: LineUnderConstruction,

    /// Information about the unbreakable line segment currently being laid out into [`LineItem`]s.
    current_line_segment: UnbreakableSegmentUnderConstruction,

    /// After a forced line break (for instance from a `<br>` element) we wait to actually
    /// break the line until seeing more content. This allows ongoing inline boxes to finish,
    /// since in the case where they have no more content they should not be on the next
    /// line.
    ///
    /// For instance:
    ///
    /// ``` html
    ///    <span style="border-right: 30px solid blue;">
    ///         first line<br>
    ///    </span>
    ///    second line
    /// ```
    ///
    /// In this case, the `<span>` should not extend to the second line. If we linebreak
    /// as soon as we encounter the `<br>` the `<span>`'s ending inline borders would be
    /// placed on the second line, because we add those borders in
    /// [`InlineFormattingContextState::finish_inline_box()`].
    linebreak_before_new_content: bool,

    /// Whether or not a soft wrap opportunity is queued. Soft wrap opportunities are
    /// queued after replaced content and they are processed when the next text content
    /// is encountered.
    pub have_deferred_soft_wrap_opportunity: bool,

    /// Whether or not a soft wrap opportunity should be prevented before the next atomic
    /// element encountered in the inline formatting context. See
    /// `char_prevents_soft_wrap_opportunity_when_before_or_after_atomic` for more
    /// details.
    pub prevent_soft_wrap_opportunity_before_next_atomic: bool,

    /// Whether or not this InlineFormattingContext has processed any in flow content at all.
    had_inflow_content: bool,

    /// The currently white-space setting of this line. This is stored on the
    /// [`InlineFormattingContextState`] because when a soft wrap opportunity is defined
    /// by the boundary between two characters, the white-space property of their nearest
    /// common ancestor is used.
    white_space: WhiteSpace,

    /// The offset of the first and last baselines in the inline formatting context that we
    /// are laying out. This is used to propagate baselines to the ancestors of
    /// `display: inline-block` elements and table content.
    baselines: Baselines,
}

impl<'a, 'b> InlineFormattingContextState<'a, 'b> {
    fn current_inline_container_state(&self) -> &InlineContainerState {
        match self.inline_box_state_stack.last() {
            Some(inline_box_state) => &inline_box_state.base,
            None => &self.root_nesting_level,
        }
    }

    fn current_inline_container_state_mut(&mut self) -> &mut InlineContainerState {
        match self.inline_box_state_stack.last_mut() {
            Some(inline_box_state) => &mut inline_box_state.base,
            None => &mut self.root_nesting_level,
        }
    }

    fn current_line_max_block_size_including_nested_containers(&self) -> LineBlockSizes {
        self.current_inline_container_state()
            .nested_strut_block_sizes
            .max(&self.current_line.max_block_size)
    }

    fn propagate_current_nesting_level_white_space_style(&mut self) {
        let style = match self.inline_box_state_stack.last() {
            Some(inline_box_state) => &inline_box_state.base.style,
            None => self.containing_block.style,
        };
        self.white_space = style.get_inherited_text().white_space;
    }

    fn processing_br_element(&self) -> bool {
        self.inline_box_state_stack
            .last()
            .map(|state| {
                state
                    .base_fragment_info
                    .flags
                    .contains(FragmentFlags::IS_BR_ELEMENT)
            })
            .unwrap_or(false)
    }

    /// Start laying out a particular [`InlineBox`] into line items. This will push
    /// a new [`InlineBoxContainerState`] onto [`Self::inline_box_state_stack`].
    fn start_inline_box(&mut self, inline_box: &InlineBox) {
        let mut inline_box_state = InlineBoxContainerState::new(
            inline_box,
            self.containing_block,
            self.layout_context,
            self.current_inline_container_state(),
            inline_box.is_last_fragment,
            inline_box
                .default_font_index
                .map(|index| &self.fonts[index].metrics),
        );

        if inline_box.is_first_fragment {
            self.current_line.inline_position += Length::from(
                inline_box_state.pbm.padding.inline_start +
                    inline_box_state.pbm.border.inline_start,
            ) + inline_box_state
                .pbm
                .margin
                .inline_start
                .auto_is(Au::zero)
                .into()
        }

        let line_item = inline_box_state
            .layout_into_line_item(inline_box.is_first_fragment, inline_box.is_last_fragment);
        self.push_line_item_to_unbreakable_segment(LineItem::StartInlineBox(line_item));
        self.inline_box_state_stack.push(inline_box_state);
    }

    /// Finish laying out a particular [`InlineBox`] into line items. This will add the
    /// final [`InlineBoxLineItem`] to the state and pop its state off of
    /// [`Self::inline_box_state_stack`].
    fn finish_inline_box(&mut self) {
        let inline_box_state = match self.inline_box_state_stack.pop() {
            Some(inline_box_state) => inline_box_state,
            None => return, // We are at the root.
        };

        self.push_line_item_to_unbreakable_segment(LineItem::EndInlineBox);
        self.current_line_segment
            .max_block_size
            .max_assign(&inline_box_state.base.nested_strut_block_sizes);

        // If the inline box that we just finished had any content at all, we want to propagate
        // the `white-space` property of its parent to future inline children. This is because
        // when a soft wrap opportunity is defined by the boundary between two elements, the
        // `white-space` used is that of their nearest common ancestor.
        if inline_box_state.base.has_content {
            self.propagate_current_nesting_level_white_space_style();
        }

        if inline_box_state.is_last_fragment {
            let pbm_end = Length::from(
                inline_box_state.pbm.padding.inline_end + inline_box_state.pbm.border.inline_end,
            ) + inline_box_state
                .pbm
                .margin
                .inline_end
                .auto_is(Au::zero)
                .into();
            self.current_line_segment.inline_size += pbm_end;
        }
    }

    fn finish_last_line(&mut self) {
        // We are at the end of the IFC, and we need to do a few things to make sure that
        // the current segment is committed and that the final line is finished.
        //
        // A soft wrap opportunity makes it so the current segment is placed on a new line
        // if it doesn't fit on the current line under construction.
        self.process_soft_wrap_opportunity();

        // `process_soft_line_wrap_opportunity` does not commit the segment to a line if
        // there is no line wrapping, so this forces the segment into the current line.
        self.commit_current_segment_to_line();

        // Finally we finish the line itself and convert all of the LineItems into
        // fragments.
        self.finish_current_line_and_reset(true /* last_line_or_forced_line_break */);
    }

    /// Finish layout of all inline boxes for the current line. This will gather all
    /// [`LineItem`]s and turn them into [`Fragment`]s, then reset the
    /// [`InlineFormattingContextState`] preparing it for laying out a new line.
    fn finish_current_line_and_reset(&mut self, last_line_or_forced_line_break: bool) {
        let whitespace_trimmed = self.current_line.trim_trailing_whitespace();
        let (inline_start_position, justification_adjustment) = self
            .calculate_current_line_inline_start_and_justification_adjustment(
                whitespace_trimmed,
                last_line_or_forced_line_break,
            );

        let block_start_position = self
            .current_line
            .line_block_start_considering_placement_among_floats();
        let had_inline_advance =
            self.current_line.inline_position != self.current_line.start_position.inline;

        let effective_block_advance = if self.current_line.has_content ||
            had_inline_advance ||
            self.linebreak_before_new_content
        {
            self.current_line_max_block_size_including_nested_containers()
        } else {
            LineBlockSizes::zero()
        };

        let block_end_position = block_start_position + effective_block_advance.resolve().into();
        if let Some(sequential_layout_state) = self.sequential_layout_state.as_mut() {
            // This amount includes both the block size of the line and any extra space
            // added to move the line down in order to avoid overlapping floats.
            let increment = block_end_position - self.current_line.start_position.block.into();
            sequential_layout_state.advance_block_position(increment);
        }

        let mut line_items = std::mem::take(&mut self.current_line.line_items);
        if self.current_line.has_floats_waiting_to_be_placed {
            place_pending_floats(self, &mut line_items);
        }

        // Set up the new line now that we no longer need the old one.
        self.current_line = LineUnderConstruction::new(LogicalVec2 {
            inline: Length::zero(),
            block: block_end_position.into(),
        });

        let baseline_offset = effective_block_advance.find_baseline_offset();

        let mut state = LineItemLayoutState {
            inline_position: inline_start_position,
            parent_offset: LogicalVec2::zero(),
            baseline_offset,
            ifc_containing_block: self.containing_block,
            positioning_context: self.positioning_context,
            justification_adjustment,
            line_metrics: &LineMetrics {
                block_offset: block_start_position.into(),
                block_size: effective_block_advance.resolve(),
                baseline_block_offset: baseline_offset,
            },
        };

        let positioning_context_length = state.positioning_context.len();
        let mut saw_end = false;
        let fragments = layout_line_items(
            &mut line_items.into_iter(),
            self.layout_context,
            &mut state,
            &mut saw_end,
        );

        let line_had_content =
            !fragments.is_empty() || state.positioning_context.len() != positioning_context_length;

        // If the line doesn't have any fragments, we don't need to add a containing fragment for it.
        if !line_had_content {
            return;
        }

        let baseline = baseline_offset + block_start_position;
        self.baselines.first.get_or_insert(baseline);
        self.baselines.last = Some(baseline);

        let line_rect = LogicalRect {
            // The inline part of this start offset was taken into account when determining
            // the inline start of the line in `calculate_inline_start_for_current_line` so
            // we do not need to include it in the `start_corner` of the line's main Fragment.
            start_corner: LogicalVec2 {
                inline: Length::zero(),
                block: block_start_position.into(),
            },
            size: LogicalVec2 {
                inline: self.containing_block.inline_size.into(),
                block: effective_block_advance.resolve(),
            },
        };

        state
            .positioning_context
            .adjust_static_position_of_hoisted_fragments_with_offset(
                &line_rect.start_corner,
                positioning_context_length,
            );

        self.fragments
            .push(Fragment::Positioning(PositioningFragment::new_anonymous(
                line_rect,
                fragments,
                self.containing_block.style.writing_mode,
            )));
    }

    /// Given the amount of whitespace trimmed from the line and taking into consideration
    /// the `text-align` property, calculate where the line under construction starts in
    /// the inline axis as well as the adjustment needed for every justification opportunity
    /// to account for `text-align: justify`.
    fn calculate_current_line_inline_start_and_justification_adjustment(
        &self,
        whitespace_trimmed: Length,
        last_line_or_forced_line_break: bool,
    ) -> (Length, Length) {
        enum TextAlign {
            Start,
            Center,
            End,
        }
        let style = self.containing_block.style;
        let mut text_align_keyword = style.clone_text_align();

        if last_line_or_forced_line_break {
            text_align_keyword = match style.clone_text_align_last() {
                TextAlignLast::Auto if text_align_keyword == TextAlignKeyword::Justify => {
                    TextAlignKeyword::Start
                },
                TextAlignLast::Auto => text_align_keyword,
                TextAlignLast::Start => TextAlignKeyword::Start,
                TextAlignLast::End => TextAlignKeyword::End,
                TextAlignLast::Left => TextAlignKeyword::Left,
                TextAlignLast::Right => TextAlignKeyword::Right,
                TextAlignLast::Center => TextAlignKeyword::Center,
                TextAlignLast::Justify => TextAlignKeyword::Justify,
            };
        }

        let text_align = match text_align_keyword {
            TextAlignKeyword::Start => TextAlign::Start,
            TextAlignKeyword::Center | TextAlignKeyword::ServoCenter => TextAlign::Center,
            TextAlignKeyword::End => TextAlign::End,
            TextAlignKeyword::Left | TextAlignKeyword::ServoLeft => {
                if style.writing_mode.line_left_is_inline_start() {
                    TextAlign::Start
                } else {
                    TextAlign::End
                }
            },
            TextAlignKeyword::Right | TextAlignKeyword::ServoRight => {
                if style.writing_mode.line_left_is_inline_start() {
                    TextAlign::End
                } else {
                    TextAlign::Start
                }
            },
            TextAlignKeyword::Justify => TextAlign::Start,
        };

        let (line_start, available_space) = match self.current_line.placement_among_floats.get() {
            Some(placement_among_floats) => (
                placement_among_floats.start_corner.inline,
                placement_among_floats.size.inline,
            ),
            None => (Length::zero(), self.containing_block.inline_size.into()),
        };

        // Properly handling text-indent requires that we do not align the text
        // into the text-indent.
        // See <https://drafts.csswg.org/css-text/#text-indent-property>
        // "This property specifies the indentation applied to lines of inline content in
        // a block. The indent is treated as a margin applied to the start edge of the
        // line box."
        let text_indent = self.current_line.start_position.inline;
        let line_length = self.current_line.inline_position - whitespace_trimmed - text_indent;
        let adjusted_line_start = line_start +
            match text_align {
                TextAlign::Start => text_indent,
                TextAlign::End => (available_space - line_length).max(text_indent),
                TextAlign::Center => {
                    ((available_space - line_length + text_indent) / 2.).max(text_indent)
                },
            };

        // Calculate the justification adjustment. This is simply the remaining space on the line,
        // dividided by the number of justficiation opportunities that we recorded when building
        // the line.
        let text_justify = self.containing_block.style.clone_text_justify();
        let justification_adjustment = match (text_align_keyword, text_justify) {
            // `text-justify: none` should disable text justification.
            // TODO: Handle more `text-justify` values.
            (TextAlignKeyword::Justify, TextJustify::None) => Length::zero(),
            (TextAlignKeyword::Justify, _) => {
                match self.current_line.count_justification_opportunities() {
                    0 => Length::zero(),
                    num_justification_opportunities => {
                        (available_space - line_length) / (num_justification_opportunities as f32)
                    },
                }
            },
            _ => Length::zero(),
        };

        (adjusted_line_start, justification_adjustment)
    }

    fn place_float_fragment(&mut self, fragment: &mut BoxFragment) {
        let state = self
            .sequential_layout_state
            .as_mut()
            .expect("Tried to lay out a float with no sequential placement state!");

        let block_offset_from_containining_block_top = state
            .current_block_position_including_margins() -
            state.current_containing_block_offset();
        state.place_float_fragment(
            fragment,
            CollapsedMargin::zero(),
            block_offset_from_containining_block_top,
        );
    }

    /// Place a FloatLineItem. This is done when an unbreakable segment is committed to
    /// the current line. Placement of FloatLineItems might need to be deferred until the
    /// line is complete in the case that floats stop fitting on the current line.
    ///
    /// When placing floats we do not want to take into account any trailing whitespace on
    /// the line, because that whitespace will be trimmed in the case that the line is
    /// broken. Thus this function takes as an argument the new size (without whitespace) of
    /// the line that these floats are joining.
    fn place_float_line_item_for_commit_to_line(
        &mut self,
        float_item: &mut FloatLineItem,
        line_inline_size_without_trailing_whitespace: Length,
    ) {
        let margin_box = float_item
            .fragment
            .border_rect()
            .inflate(&float_item.fragment.margin);
        let inline_size = margin_box.size.inline.max(Length::zero());

        let available_inline_size = match self.current_line.placement_among_floats.get() {
            Some(placement_among_floats) => placement_among_floats.size.inline,
            None => self.containing_block.inline_size.into(),
        } - line_inline_size_without_trailing_whitespace;

        // If this float doesn't fit on the current line or a previous float didn't fit on
        // the current line, we need to place it starting at the next line BUT still as
        // children of this line's hierarchy of inline boxes (for the purposes of properly
        // parenting in their stacking contexts). Once all the line content is gathered we
        // will place them later.
        let has_content = self.current_line.has_content || self.current_line_segment.has_content;
        let fits_on_line = !has_content || inline_size <= available_inline_size;
        let needs_placement_later =
            self.current_line.has_floats_waiting_to_be_placed || !fits_on_line;

        if needs_placement_later {
            self.current_line.has_floats_waiting_to_be_placed = true;
        } else {
            self.place_float_fragment(&mut float_item.fragment);
            float_item.needs_placement = false;
        }

        // We've added a new float to the IFC, but this may have actually changed the
        // position of the current line. In order to determine that we regenerate the
        // placement among floats for the current line, which may adjust its inline
        // start position.
        let new_placement = self.place_line_among_floats(&LogicalVec2 {
            inline: line_inline_size_without_trailing_whitespace,
            block: self.current_line.max_block_size.resolve(),
        });
        self.current_line
            .replace_placement_among_floats(new_placement);
    }

    /// Given a new potential line size for the current line, create a "placement" for that line.
    /// This tells us whether or not the new potential line will fit in the current block position
    /// or need to be moved. In addition, the placement rect determines the inline start and end
    /// of the line if it's used as the final placement among floats.
    fn place_line_among_floats(
        &self,
        potential_line_size: &LogicalVec2<Length>,
    ) -> LogicalRect<Length> {
        let sequential_layout_state = self
            .sequential_layout_state
            .as_ref()
            .expect("Should not have called this function without having floats.");

        let ifc_offset_in_float_container = LogicalVec2 {
            inline: sequential_layout_state
                .floats
                .containing_block_info
                .inline_start,
            block: sequential_layout_state.current_containing_block_offset(),
        };

        let ceiling = self
            .current_line
            .line_block_start_considering_placement_among_floats();
        let mut placement = PlacementAmongFloats::new(
            &sequential_layout_state.floats,
            ceiling + ifc_offset_in_float_container.block,
            LogicalVec2 {
                inline: potential_line_size.inline.into(),
                block: potential_line_size.block.into(),
            },
            &PaddingBorderMargin::zero(),
        );

        let mut placement_rect = placement.place();
        placement_rect.start_corner = &placement_rect.start_corner - &ifc_offset_in_float_container;
        placement_rect.into()
    }

    /// Returns true if a new potential line size for the current line would require a line
    /// break. This takes into account floats and will also update the "placement among
    /// floats" for this line if the potential line size would not cause a line break.
    /// Thus, calling this method has side effects and should only be done while in the
    /// process of laying out line content that is always going to be committed to this
    /// line or the next.
    fn new_potential_line_size_causes_line_break(
        &mut self,
        potential_line_size: &LogicalVec2<Length>,
    ) -> bool {
        let available_line_space = if self.sequential_layout_state.is_some() {
            self.current_line
                .placement_among_floats
                .get_or_init(|| self.place_line_among_floats(potential_line_size))
                .size
        } else {
            LogicalVec2 {
                inline: self.containing_block.inline_size.into(),
                block: Length::new(f32::INFINITY),
            }
        };

        let inline_would_overflow = potential_line_size.inline > available_line_space.inline;
        let block_would_overflow = potential_line_size.block > available_line_space.block;

        // The first content that is added to a line cannot trigger a line break and
        // the `white-space` propertly can also prevent all line breaking.
        let can_break = self.current_line.has_content;

        // If this is the first content on the line and we already have a float placement,
        // that means that the placement was initialized by a leading float in the IFC.
        // This placement needs to be updated, because the first line content might push
        // the block start of the line downward. If there is no float placement, we want
        // to make one to properly set the block position of the line.
        if !can_break {
            // Even if we cannot break, adding content to this line might change its position.
            // In that case we need to redo our placement among floats.
            if self.sequential_layout_state.is_some() &&
                (inline_would_overflow || block_would_overflow)
            {
                let new_placement = self.place_line_among_floats(potential_line_size);
                self.current_line
                    .replace_placement_among_floats(new_placement);
            }

            return false;
        }

        // If the potential line is larger than the containing block we do not even need to consider
        // floats. We definitely have to do a linebreak.
        if potential_line_size.inline > self.containing_block.inline_size.into() {
            return true;
        }

        // Not fitting in the block space means that our block size has changed and we had a
        // placement among floats that is no longer valid. This same placement might just
        // need to be expanded or perhaps we need to line break.
        if block_would_overflow {
            // If we have a limited block size then we are wedging this line between floats.
            assert!(self.sequential_layout_state.is_some());
            let new_placement = self.place_line_among_floats(potential_line_size);
            if new_placement.start_corner.block !=
                self.current_line
                    .line_block_start_considering_placement_among_floats()
                    .into()
            {
                return true;
            } else {
                self.current_line
                    .replace_placement_among_floats(new_placement);
                return false;
            }
        }

        // Otherwise the new potential line size will require a newline if it fits in the
        // inline space available for this line. This space may be smaller than the
        // containing block if floats shrink the available inline space.
        inline_would_overflow
    }

    pub(super) fn defer_forced_line_break(&mut self) {
        // If this hard line break happens in the middle of an unbreakable segment, there are two
        // scenarios:
        //    1. The current portion of the unbreakable segment fits on the current line in which
        //       case we commit it.
        //    2. The current portion of the unbreakable segment does not fit in which case we
        //       need to put it on a new line *before* actually triggering the hard line break.
        //
        // `process_soft_wrap_opportunity` handles both of these cases.
        self.process_soft_wrap_opportunity();

        // Defer the actual line break until we've cleared all ending inline boxes.
        self.linebreak_before_new_content = true;

        // In quirks mode, the line-height isn't automatically added to the line. If we consider a
        // forced line break a kind of preserved white space, quirks mode requires that we add the
        // line-height of the current element to the line box height.
        //
        // The exception here is `<br>` elements. They are implemented with `pre-line` in Servo, but
        // this is an implementation detail. The "magic" behavior of `<br>` elements is that they
        // add line-height to the line conditionally: only when they are on an otherwise empty line.
        let line_is_empty =
            !self.current_line_segment.has_content && !self.current_line.has_content;
        if !self.processing_br_element() || line_is_empty {
            let strut_size = self
                .current_inline_container_state()
                .strut_block_sizes
                .clone();
            self.update_unbreakable_segment_for_new_content(
                &strut_size,
                Length::zero(),
                SegmentContentFlags::empty(),
            );
        }

        self.had_inflow_content = true;
    }

    pub(super) fn possibly_flush_deferred_forced_line_break(&mut self) {
        if !self.linebreak_before_new_content {
            return;
        }

        self.commit_current_segment_to_line();
        self.process_line_break(true /* forced_line_break */);
        self.linebreak_before_new_content = false;
    }

    fn push_line_item_to_unbreakable_segment(&mut self, line_item: LineItem) {
        self.current_line_segment
            .push_line_item(line_item, self.inline_box_state_stack.len());
    }

    pub(super) fn push_glyph_store_to_unbreakable_segment(
        &mut self,
        glyph_store: std::sync::Arc<GlyphStore>,
        text_run: &TextRun,
        font_index: usize,
    ) {
        let inline_advance = Length::from(glyph_store.total_advance());
        let flags = if glyph_store.is_whitespace() {
            SegmentContentFlags::from(text_run.parent_style.get_inherited_text().white_space)
        } else {
            SegmentContentFlags::empty()
        };

        // If the metrics of this font don't match the default font, we are likely using a fallback
        // font and need to adjust the line size to account for a potentially different font.
        // If somehow the metrics match, the line size won't change.
        let ifc_font_info = &self.fonts[font_index];
        let font_metrics = ifc_font_info.metrics.clone();
        let using_fallback_font =
            self.current_inline_container_state().font_metrics != font_metrics;

        let quirks_mode = self.layout_context.style_context.quirks_mode() != QuirksMode::NoQuirks;
        let strut_size = if using_fallback_font {
            // TODO(mrobinson): This value should probably be cached somewhere.
            let container_state = self.current_inline_container_state();
            let vertical_align = effective_vertical_align(
                &container_state.style,
                self.inline_box_state_stack.last().map(|c| &c.base),
            );
            let mut block_size =
                container_state.get_block_size_contribution(vertical_align, &font_metrics);
            block_size.adjust_for_baseline_offset(container_state.baseline_offset);
            block_size
        } else if quirks_mode && !flags.is_collapsible_whitespace() {
            // Normally, the strut is incorporated into the nested block size. In quirks mode though
            // if we find any text that isn't collapsed whitespace, we need to incorporate the strut.
            // TODO(mrobinson): This isn't quite right for situations where collapsible white space
            // ultimately does not collapse because it is between two other pieces of content.
            self.current_inline_container_state()
                .strut_block_sizes
                .clone()
        } else {
            LineBlockSizes::zero()
        };
        self.update_unbreakable_segment_for_new_content(&strut_size, inline_advance, flags);

        match self.current_line_segment.line_items.last_mut() {
            Some(LineItem::TextRun(line_item)) if ifc_font_info.key == line_item.font_key => {
                line_item.text.push(glyph_store);
                return;
            },
            _ => {},
        }

        self.push_line_item_to_unbreakable_segment(LineItem::TextRun(TextRunLineItem {
            text: vec![glyph_store],
            base_fragment_info: text_run.base_fragment_info,
            parent_style: text_run.parent_style.clone(),
            font_metrics,
            font_key: ifc_font_info.key,
            text_decoration_line: self.current_inline_container_state().text_decoration_line,
        }));
    }

    fn update_unbreakable_segment_for_new_content(
        &mut self,
        block_sizes_of_content: &LineBlockSizes,
        inline_size: Length,
        flags: SegmentContentFlags,
    ) {
        if flags.is_collapsible_whitespace() || flags.is_wrappable_whitespace() {
            self.current_line_segment.trailing_whitespace_size = inline_size;
        } else {
            self.current_line_segment.trailing_whitespace_size = Length::zero();
        }
        if !flags.is_collapsible_whitespace() {
            self.current_line_segment.has_content = true;
            self.had_inflow_content = true;
        }

        // This may or may not include the size of the strut depending on the quirks mode setting.
        let container_max_block_size = &self
            .current_inline_container_state()
            .nested_strut_block_sizes
            .clone();
        self.current_line_segment
            .max_block_size
            .max_assign(container_max_block_size);
        self.current_line_segment
            .max_block_size
            .max_assign(block_sizes_of_content);

        self.current_line_segment.inline_size += inline_size;

        // Propagate the whitespace setting to the current nesting level.
        let current_nesting_level = self.current_inline_container_state_mut();
        current_nesting_level.has_content = true;
        self.propagate_current_nesting_level_white_space_style();
    }

    fn process_line_break(&mut self, forced_line_break: bool) {
        self.current_line_segment
            .prepare_for_placement_on_empty_line(
                &self.current_line,
                self.inline_box_state_stack.len(),
            );
        self.finish_current_line_and_reset(forced_line_break);
    }

    /// Process a soft wrap opportunity. This will either commit the current unbreakble
    /// segment to the current line, if it fits within the containing block and float
    /// placement boundaries, or do a line break and then commit the segment.
    pub(super) fn process_soft_wrap_opportunity(&mut self) {
        if self.current_line_segment.line_items.is_empty() {
            return;
        }
        if !self.white_space.allow_wrap() {
            return;
        }

        let potential_line_size = LogicalVec2 {
            inline: self.current_line.inline_position + self.current_line_segment.inline_size -
                self.current_line_segment.trailing_whitespace_size,
            block: self
                .current_line_max_block_size_including_nested_containers()
                .max(&self.current_line_segment.max_block_size)
                .resolve(),
        };

        if self.new_potential_line_size_causes_line_break(&potential_line_size) {
            self.process_line_break(false /* forced_line_break */);
        }
        self.commit_current_segment_to_line();
    }

    /// Commit the current unbrekable segment to the current line. In addition, this will
    /// place all floats in the unbreakable segment and expand the line dimensions.
    fn commit_current_segment_to_line(&mut self) {
        // The line segments might have no items and have content after processing a forced
        // linebreak on an empty line.
        if self.current_line_segment.line_items.is_empty() && !self.current_line_segment.has_content
        {
            return;
        }

        if !self.current_line.has_content {
            self.current_line_segment.trim_leading_whitespace();
        }

        self.current_line.inline_position += self.current_line_segment.inline_size;
        self.current_line.max_block_size = self
            .current_line_max_block_size_including_nested_containers()
            .max(&self.current_line_segment.max_block_size);
        let line_inline_size_without_trailing_whitespace =
            self.current_line.inline_position - self.current_line_segment.trailing_whitespace_size;

        // Place all floats in this unbreakable segment.
        let mut segment_items = mem::take(&mut self.current_line_segment.line_items);
        for item in segment_items.iter_mut() {
            if let LineItem::Float(float_item) = item {
                self.place_float_line_item_for_commit_to_line(
                    float_item,
                    line_inline_size_without_trailing_whitespace,
                );
            }
        }

        // If the current line was never placed among floats, we need to do that now based on the
        // new size. Calling `new_potential_line_size_causes_line_break()` here triggers the
        // new line to be positioned among floats. This should never ask for a line
        // break because it is the first content on the line.
        if self.current_line.line_items.is_empty() {
            let will_break = self.new_potential_line_size_causes_line_break(&LogicalVec2 {
                inline: line_inline_size_without_trailing_whitespace,
                block: self.current_line_segment.max_block_size.resolve(),
            });
            assert!(!will_break);
        }

        // Try to merge all TextRuns in the line.
        let to_skip = match (
            self.current_line.line_items.last_mut(),
            segment_items.first_mut(),
        ) {
            (
                Some(LineItem::TextRun(last_line_item)),
                Some(LineItem::TextRun(first_segment_item)),
            ) if last_line_item.font_key == first_segment_item.font_key => {
                last_line_item.text.append(&mut first_segment_item.text);
                1
            },
            _ => 0,
        };

        self.current_line
            .line_items
            .extend(segment_items.into_iter().skip(to_skip));
        self.current_line.has_content |= self.current_line_segment.has_content;

        self.current_line_segment.reset();
    }
}

bitflags! {
    pub struct SegmentContentFlags: u8 {
        const COLLAPSIBLE_WHITESPACE = 0b00000001;
        const WRAPPABLE_WHITESPACE = 0b00000010;
    }
}

impl SegmentContentFlags {
    fn is_collapsible_whitespace(&self) -> bool {
        self.contains(Self::COLLAPSIBLE_WHITESPACE)
    }

    fn is_wrappable_whitespace(&self) -> bool {
        self.contains(Self::WRAPPABLE_WHITESPACE)
    }
}

impl From<WhiteSpace> for SegmentContentFlags {
    fn from(white_space: WhiteSpace) -> Self {
        let mut flags = Self::empty();
        if !white_space.preserve_spaces() {
            flags.insert(Self::COLLAPSIBLE_WHITESPACE);
        }
        if white_space.allow_wrap() {
            flags.insert(Self::WRAPPABLE_WHITESPACE);
        }
        flags
    }
}

enum InlineFormattingContextIterItem<'a> {
    Item(&'a mut InlineLevelBox),
    EndInlineBox,
}

impl InlineFormattingContext {
    pub(super) fn new(
        text_decoration_line: TextDecorationLine,
        has_first_formatted_line: bool,
    ) -> InlineFormattingContext {
        InlineFormattingContext {
            inline_level_boxes: Default::default(),
            font_metrics: Vec::new(),
            text_decoration_line,
            has_first_formatted_line,
            contains_floats: false,
        }
    }

    fn foreach(&self, mut func: impl FnMut(InlineFormattingContextIterItem)) {
        // TODO(mrobinson): Using OwnedRef here we could maybe avoid the second borrow when
        // iterating through members of each inline box.
        struct InlineFormattingContextChildBoxIter {
            inline_level_box: ArcRefCell<InlineLevelBox>,
            next_child_index: usize,
        }

        impl InlineFormattingContextChildBoxIter {
            fn next(&mut self) -> Option<ArcRefCell<InlineLevelBox>> {
                let borrowed_item = self.inline_level_box.borrow();
                let inline_box = match *borrowed_item {
                    InlineLevelBox::InlineBox(ref inline_box) => inline_box,
                    _ => unreachable!(
                        "InlineFormattingContextChildBoxIter created for non-InlineBox."
                    ),
                };

                let item = inline_box.children.get(self.next_child_index).cloned();
                if item.is_some() {
                    self.next_child_index += 1;
                }
                item
            }
        }

        let mut inline_box_iterator_stack: Vec<InlineFormattingContextChildBoxIter> = Vec::new();
        let mut root_iterator = self.inline_level_boxes.iter();
        loop {
            let mut item = None;

            // First try to get the next item in the current inline box.
            if !inline_box_iterator_stack.is_empty() {
                item = inline_box_iterator_stack
                    .last_mut()
                    .and_then(|iter| iter.next());
                if item.is_none() {
                    func(InlineFormattingContextIterItem::EndInlineBox);
                    inline_box_iterator_stack.pop();
                    continue;
                }
            }

            // If there is no current inline box, then try to get the next item from the root of the IFC.
            item = item.or_else(|| root_iterator.next().cloned());

            // If there is no item left, we are done iterating.
            let item = match item {
                Some(item) => item,
                None => return,
            };

            let borrowed_item = &mut *item.borrow_mut();
            func(InlineFormattingContextIterItem::Item(borrowed_item));
            if matches!(borrowed_item, InlineLevelBox::InlineBox(_)) {
                inline_box_iterator_stack.push(InlineFormattingContextChildBoxIter {
                    inline_level_box: item.clone(),
                    next_child_index: 0,
                })
            }
        }
    }

    // This works on an already-constructed `InlineFormattingContext`,
    // Which would have to change if/when
    // `BlockContainer::construct` parallelize their construction.
    pub(super) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        containing_block_writing_mode: WritingMode,
    ) -> ContentSizes {
        ContentSizesComputation::compute(self, layout_context, containing_block_writing_mode)
    }

    pub(super) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
    ) -> FlowLayout {
        let first_line_inline_start = if self.has_first_formatted_line {
            containing_block
                .style
                .get_inherited_text()
                .text_indent
                .to_used_value(containing_block.inline_size)
                .into()
        } else {
            Length::zero()
        };

        let style = containing_block.style;

        // It's unfortunate that it isn't possible to get this during IFC text processing, but in
        // that situation the style of the containing block is unknown.
        let default_font_metrics =
            crate::context::with_thread_local_font_context(layout_context, |font_context| {
                get_font_for_first_font_for_style(style, font_context)
                    .map(|font| font.borrow().metrics.clone())
            });

        let mut ifc = InlineFormattingContextState {
            positioning_context,
            containing_block,
            sequential_layout_state,
            layout_context,
            fonts: &self.font_metrics,
            fragments: Vec::new(),
            current_line: LineUnderConstruction::new(LogicalVec2 {
                inline: first_line_inline_start,
                block: Length::zero(),
            }),
            root_nesting_level: InlineContainerState::new(
                style.to_arc(),
                None, /* parent_container */
                self.text_decoration_line,
                default_font_metrics.as_ref(),
                inline_container_needs_strut(style, layout_context, None),
            ),
            inline_box_state_stack: Vec::new(),
            current_line_segment: UnbreakableSegmentUnderConstruction::new(),
            linebreak_before_new_content: false,
            have_deferred_soft_wrap_opportunity: false,
            prevent_soft_wrap_opportunity_before_next_atomic: false,
            had_inflow_content: false,
            white_space: containing_block.style.get_inherited_text().white_space,
            baselines: Baselines::default(),
        };

        // FIXME(pcwalton): This assumes that margins never collapse through inline formatting
        // contexts (i.e. that inline formatting contexts are never empty). Is that right?
        // FIXME(mrobinson): This should not happen if the IFC collapses through.
        if let Some(ref mut sequential_layout_state) = ifc.sequential_layout_state {
            sequential_layout_state.collapse_margins();
            // FIXME(mrobinson): Collapse margins in the containing block offsets as well??
        }

        self.foreach(|item| match item {
            InlineFormattingContextIterItem::Item(item) => {
                // Any new box should flush a pending hard line break.
                ifc.possibly_flush_deferred_forced_line_break();

                match item {
                    InlineLevelBox::InlineBox(ref inline_box) => {
                        ifc.start_inline_box(inline_box);
                    },
                    InlineLevelBox::TextRun(ref run) => run.layout_into_line_items(&mut ifc),
                    InlineLevelBox::Atomic(ref mut atomic_formatting_context) => {
                        atomic_formatting_context.layout_into_line_items(layout_context, &mut ifc);
                    },
                    InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(ref positioned_box) => {
                        ifc.push_line_item_to_unbreakable_segment(LineItem::AbsolutelyPositioned(
                            AbsolutelyPositionedLineItem {
                                absolutely_positioned_box: positioned_box.clone(),
                            },
                        ));
                    },
                    InlineLevelBox::OutOfFlowFloatBox(ref mut float_box) => {
                        float_box.layout_into_line_items(layout_context, &mut ifc);
                    },
                }
            },
            InlineFormattingContextIterItem::EndInlineBox => {
                ifc.finish_inline_box();
            },
        });

        ifc.finish_last_line();

        let mut collapsible_margins_in_children = CollapsedBlockMargins::zero();
        let content_block_size = ifc.current_line.start_position.block;
        collapsible_margins_in_children.collapsed_through = !ifc.had_inflow_content &&
            content_block_size == Length::zero() &&
            collapsible_with_parent_start_margin.0;

        FlowLayout {
            fragments: ifc.fragments,
            content_block_size,
            collapsible_margins_in_children,
            baselines: ifc.baselines,
        }
    }

    /// Return true if this [InlineFormattingContext] is empty for the purposes of ignoring
    /// during box tree construction. An IFC is empty if it only contains TextRuns with
    /// completely collapsible whitespace. When that happens it can be ignored completely.
    pub fn is_empty(&self) -> bool {
        fn inline_level_boxes_are_empty(boxes: &[ArcRefCell<InlineLevelBox>]) -> bool {
            boxes
                .iter()
                .all(|inline_level_box| inline_level_box_is_empty(&inline_level_box.borrow()))
        }

        fn inline_level_box_is_empty(inline_level_box: &InlineLevelBox) -> bool {
            match inline_level_box {
                InlineLevelBox::InlineBox(_) => false,
                InlineLevelBox::TextRun(text_run) => !text_run.has_uncollapsible_content(),
                InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => false,
                InlineLevelBox::OutOfFlowFloatBox(_) => false,
                InlineLevelBox::Atomic(_) => false,
            }
        }

        inline_level_boxes_are_empty(&self.inline_level_boxes)
    }

    /// Break and shape text of this InlineFormattingContext's TextRun's, which requires doing
    /// all font matching and FontMetrics collection.
    pub(crate) fn break_and_shape_text(&mut self, layout_context: &LayoutContext) {
        let mut ifc_fonts = Vec::new();

        // Whether the last processed node ended with whitespace. This is used to
        // implement rule 4 of <https://www.w3.org/TR/css-text-3/#collapse>:
        //
        // > Any collapsible space immediately following another collapsible space—even one
        // > outside the boundary of the inline containing that space, provided both spaces are
        // > within the same inline formatting context—is collapsed to have zero advance width.
        // > (It is invisible, but retains its soft wrap opportunity, if any.)
        let mut last_inline_box_ended_with_white_space = false;

        // For the purposes of `text-transform: capitalize` the start of the IFC is a word boundary.
        let mut on_word_boundary = true;

        crate::context::with_thread_local_font_context(layout_context, |font_context| {
            let mut linebreaker = None;
            self.foreach(|iter_item| match iter_item {
                InlineFormattingContextIterItem::Item(InlineLevelBox::TextRun(
                    ref mut text_run,
                )) => {
                    text_run.break_and_shape(
                        font_context,
                        &mut linebreaker,
                        &mut ifc_fonts,
                        &mut last_inline_box_ended_with_white_space,
                        &mut on_word_boundary,
                    );
                },
                InlineFormattingContextIterItem::Item(InlineLevelBox::InlineBox(inline_box)) => {
                    if let Some(font) =
                        get_font_for_first_font_for_style(&inline_box.style, font_context)
                    {
                        inline_box.default_font_index =
                            Some(add_or_get_font(&font, &mut ifc_fonts));
                    }
                },
                InlineFormattingContextIterItem::Item(InlineLevelBox::Atomic(_)) => {
                    last_inline_box_ended_with_white_space = false;
                    on_word_boundary = true;
                },
                _ => {},
            });
        });

        self.font_metrics = ifc_fonts;
    }
}

impl InlineContainerState {
    fn new(
        style: Arc<ComputedValues>,
        parent_container: Option<&InlineContainerState>,
        parent_text_decoration_line: TextDecorationLine,
        font_metrics: Option<&FontMetrics>,
        create_strut: bool,
    ) -> Self {
        let text_decoration_line = parent_text_decoration_line | style.clone_text_decoration_line();
        let font_metrics = font_metrics.cloned().unwrap_or_else(FontMetrics::empty);
        let line_height = line_height(&style, &font_metrics);

        let mut baseline_offset = Au::zero();
        let mut strut_block_sizes = Self::get_block_sizes_with_style(
            effective_vertical_align(&style, parent_container),
            &style,
            &font_metrics,
            line_height,
        );
        if let Some(parent_container) = parent_container {
            // The baseline offset from `vertical-align` might adjust where our block size contribution is
            // within the line.
            baseline_offset = parent_container.get_cumulative_baseline_offset_for_child(
                style.clone_vertical_align(),
                &strut_block_sizes,
            );
            strut_block_sizes.adjust_for_baseline_offset(baseline_offset);
        }

        let mut nested_block_sizes = parent_container
            .map(|container| container.nested_strut_block_sizes.clone())
            .unwrap_or_else(LineBlockSizes::zero);
        if create_strut {
            nested_block_sizes.max_assign(&strut_block_sizes);
        }

        Self {
            style,
            has_content: false,
            text_decoration_line,
            nested_strut_block_sizes: nested_block_sizes,
            strut_block_sizes,
            baseline_offset,
            font_metrics,
        }
    }

    fn get_block_sizes_with_style(
        vertical_align: VerticalAlign,
        style: &ComputedValues,
        font_metrics: &FontMetrics,
        line_height: Length,
    ) -> LineBlockSizes {
        if !is_baseline_relative(vertical_align) {
            return LineBlockSizes {
                line_height,
                baseline_relative_size_for_line_height: None,
                size_for_baseline_positioning: BaselineRelativeSize::zero(),
            };
        }

        // From https://drafts.csswg.org/css-inline/#inline-height
        // > If line-height computes to `normal` and either `text-box-edge` is `leading` or this
        // > is the root inline box, the font’s line gap metric may also be incorporated
        // > into A and D by adding half to each side as half-leading.
        //
        // `text-box-edge` isn't implemented (and this is a draft specification), so it's
        // always effectively `leading`, which means we always take into account the line gap
        // when `line-height` is normal.
        let mut ascent = font_metrics.ascent;
        let mut descent = font_metrics.descent;
        if style.get_inherited_text().line_height == LineHeight::Normal {
            let half_leading_from_line_gap =
                (font_metrics.line_gap - descent - ascent).scale_by(0.5);
            ascent += half_leading_from_line_gap;
            descent += half_leading_from_line_gap;
        }

        // The ascent and descent we use for computing the line's final line height isn't
        // the same the ascent and descent we use for finding the baseline. For finding
        // the baseline we want the content rect.
        let size_for_baseline_positioning = BaselineRelativeSize { ascent, descent };

        // From https://drafts.csswg.org/css-inline/#inline-height
        // > When its computed line-height is not normal, its layout bounds are derived solely
        // > from metrics of its first available font (ignoring glyphs from other fonts), and
        // > leading is used to adjust the effective A and D to add up to the used line-height.
        // > Calculate the leading L as L = line-height - (A + D). Half the leading (its
        // > half-leading) is added above A of the first available font, and the other half
        // > below D of the first available font, giving an effective ascent above the baseline
        // > of A′ = A + L/2, and an effective descent of D′ = D + L/2.
        //
        // Note that leading might be negative here and the line-height might be zero. In
        // the case where the height is zero, ascent and descent will move to the same
        // point in the block axis.  Even though the contribution to the line height is
        // zero in this case, the line may get some height when taking them into
        // considering with other zero line height boxes that converge on other block axis
        // locations when using the above formula.
        if style.get_inherited_text().line_height != LineHeight::Normal {
            let half_leading =
                (Au::from_f32_px(line_height.px()) - (ascent + descent)).scale_by(0.5);
            ascent += half_leading;
            descent += half_leading;
        }

        LineBlockSizes {
            line_height,
            baseline_relative_size_for_line_height: Some(BaselineRelativeSize { ascent, descent }),
            size_for_baseline_positioning,
        }
    }

    fn get_block_size_contribution(
        &self,
        vertical_align: VerticalAlign,
        font_metrics: &FontMetrics,
    ) -> LineBlockSizes {
        Self::get_block_sizes_with_style(
            vertical_align,
            &self.style,
            font_metrics,
            line_height(&self.style, font_metrics),
        )
    }

    fn get_cumulative_baseline_offset_for_child(
        &self,
        child_vertical_align: VerticalAlign,
        child_block_size: &LineBlockSizes,
    ) -> Au {
        let block_size =
            self.get_block_size_contribution(child_vertical_align.clone(), &self.font_metrics);
        self.baseline_offset +
            match child_vertical_align {
                // `top` and `bottom are not actually relative to the baseline, but this value is unused
                // in those cases.
                // TODO: We should distinguish these from `baseline` in order to implement "aligned subtrees" properly.
                // See https://drafts.csswg.org/css2/#aligned-subtree.
                VerticalAlign::Keyword(VerticalAlignKeyword::Baseline) |
                VerticalAlign::Keyword(VerticalAlignKeyword::Top) |
                VerticalAlign::Keyword(VerticalAlignKeyword::Bottom) => Au::zero(),
                VerticalAlign::Keyword(VerticalAlignKeyword::Sub) => Au::from_f32_px(
                    block_size
                        .resolve()
                        .scale_by(FONT_SUBSCRIPT_OFFSET_RATIO)
                        .px(),
                ),
                VerticalAlign::Keyword(VerticalAlignKeyword::Super) => -Au::from_f32_px(
                    block_size
                        .resolve()
                        .scale_by(FONT_SUPERSCRIPT_OFFSET_RATIO)
                        .px(),
                ),
                VerticalAlign::Keyword(VerticalAlignKeyword::TextTop) => {
                    child_block_size.size_for_baseline_positioning.ascent - self.font_metrics.ascent
                },
                VerticalAlign::Keyword(VerticalAlignKeyword::Middle) => {
                    // "Align the vertical midpoint of the box with the baseline of the parent
                    // box plus half the x-height of the parent."
                    (child_block_size.size_for_baseline_positioning.ascent -
                        child_block_size.size_for_baseline_positioning.descent -
                        self.font_metrics.x_height)
                        .scale_by(0.5)
                },
                VerticalAlign::Keyword(VerticalAlignKeyword::TextBottom) => {
                    self.font_metrics.descent -
                        child_block_size.size_for_baseline_positioning.descent
                },
                VerticalAlign::Length(length_percentage) => {
                    Au::from_f32_px(-length_percentage.resolve(child_block_size.line_height).px())
                },
            }
    }
}

impl InlineBoxContainerState {
    fn new(
        inline_box: &InlineBox,
        containing_block: &ContainingBlock,
        layout_context: &LayoutContext,
        parent_container: &InlineContainerState,
        is_last_fragment: bool,
        font_metrics: Option<&FontMetrics>,
    ) -> Self {
        let style = inline_box.style.clone();
        let pbm = style.padding_border_margin(containing_block);
        let create_strut = inline_container_needs_strut(&style, layout_context, Some(&pbm));
        Self {
            base: InlineContainerState::new(
                style,
                Some(parent_container),
                parent_container.text_decoration_line,
                font_metrics,
                create_strut,
            ),
            base_fragment_info: inline_box.base_fragment_info,
            pbm,
            is_last_fragment,
        }
    }

    fn layout_into_line_item(
        &mut self,
        is_first_fragment: bool,
        is_last_fragment_of_ib_split: bool,
    ) -> InlineBoxLineItem {
        InlineBoxLineItem {
            base_fragment_info: self.base_fragment_info,
            style: self.base.style.clone(),
            pbm: self.pbm.clone(),
            is_first_fragment,
            is_last_fragment_of_ib_split,
            font_metrics: self.base.font_metrics.clone(),
            baseline_offset: self.base.baseline_offset,
        }
    }
}

impl IndependentFormattingContext {
    fn layout_into_line_items(
        &mut self,
        layout_context: &LayoutContext,
        ifc: &mut InlineFormattingContextState,
    ) {
        let style = self.style();
        let pbm = style.padding_border_margin(ifc.containing_block);
        let margin = pbm.margin.auto_is(Au::zero);
        let pbm_sums = &(&pbm.padding + &pbm.border) + &margin.clone();
        let mut child_positioning_context = None;

        // We need to know the inline size of the atomic before deciding whether to do the line break.
        let fragment = match self {
            IndependentFormattingContext::Replaced(replaced) => {
                let size = replaced.contents.used_size_as_if_inline_element(
                    ifc.containing_block,
                    &replaced.style,
                    None,
                    &pbm,
                );
                let fragments = replaced.contents.make_fragments(&replaced.style, size);
                let content_rect = LogicalRect {
                    start_corner: pbm_sums.start_offset(),
                    size,
                };

                BoxFragment::new(
                    replaced.base_fragment_info,
                    replaced.style.clone(),
                    fragments,
                    content_rect.into(),
                    pbm.padding.into(),
                    pbm.border.into(),
                    margin.into(),
                    None, /* clearance */
                    CollapsedBlockMargins::zero(),
                )
            },
            IndependentFormattingContext::NonReplaced(non_replaced) => {
                let box_size = non_replaced
                    .style
                    .content_box_size(ifc.containing_block, &pbm);
                let max_box_size = non_replaced
                    .style
                    .content_max_box_size(ifc.containing_block, &pbm);
                let min_box_size = non_replaced
                    .style
                    .content_min_box_size(ifc.containing_block, &pbm)
                    .auto_is(Length::zero);

                // https://drafts.csswg.org/css2/visudet.html#inlineblock-width
                let tentative_inline_size = box_size.inline.auto_is(|| {
                    let available_size = ifc.containing_block.inline_size - pbm_sums.inline_sum();
                    non_replaced
                        .inline_content_sizes(layout_context)
                        .shrink_to_fit(available_size)
                        .into()
                });

                // https://drafts.csswg.org/css2/visudet.html#min-max-widths
                // In this case “applying the rules above again” with a non-auto inline-size
                // always results in that size.
                let inline_size = tentative_inline_size
                    .clamp_between_extremums(min_box_size.inline, max_box_size.inline);

                let containing_block_for_children = ContainingBlock {
                    inline_size: inline_size.into(),
                    block_size: box_size.block.map(|t| t.into()),
                    style: &non_replaced.style,
                };
                assert_eq!(
                    ifc.containing_block.style.writing_mode,
                    containing_block_for_children.style.writing_mode,
                    "Mixed writing modes are not supported yet"
                );

                // This always collects for the nearest positioned ancestor even if the parent positioning
                // context doesn't. The thing is we haven't kept track up to this point and there isn't
                // any harm in keeping the hoisted boxes separate.
                child_positioning_context = Some(PositioningContext::new_for_subtree(
                    true, /* collects_for_nearest_positioned_ancestor */
                ));
                let independent_layout = non_replaced.layout(
                    layout_context,
                    child_positioning_context.as_mut().unwrap(),
                    &containing_block_for_children,
                    ifc.containing_block,
                );
                let (inline_size, block_size) =
                    match independent_layout.content_inline_size_for_table {
                        Some(inline) => (inline, independent_layout.content_block_size),
                        None => {
                            // https://drafts.csswg.org/css2/visudet.html#block-root-margin
                            let tentative_block_size = box_size
                                .block
                                .auto_is(|| independent_layout.content_block_size.into());

                            // https://drafts.csswg.org/css2/visudet.html#min-max-heights
                            // In this case “applying the rules above again” with a non-auto block-size
                            // always results in that size.
                            let block_size = tentative_block_size
                                .clamp_between_extremums(min_box_size.block, max_box_size.block);

                            (inline_size.into(), block_size.into())
                        },
                    };

                let content_rect = LogicalRect {
                    start_corner: pbm_sums.start_offset(),
                    size: LogicalVec2 {
                        block: block_size,
                        inline: inline_size,
                    },
                };

                BoxFragment::new(
                    non_replaced.base_fragment_info,
                    non_replaced.style.clone(),
                    independent_layout.fragments,
                    content_rect.into(),
                    pbm.padding.into(),
                    pbm.border.into(),
                    margin.into(),
                    None,
                    CollapsedBlockMargins::zero(),
                )
                .with_baselines(independent_layout.baselines)
            },
        };

        let soft_wrap_opportunity_prevented = mem::replace(
            &mut ifc.prevent_soft_wrap_opportunity_before_next_atomic,
            false,
        );
        if ifc.white_space.allow_wrap() && !soft_wrap_opportunity_prevented {
            ifc.process_soft_wrap_opportunity();
        }

        let size = &pbm_sums.sum().into() + &fragment.content_rect.size;
        let baseline_offset = fragment
            .baselines
            .last
            .map(|baseline| pbm_sums.block_start + baseline)
            .unwrap_or(size.block.into());

        let (block_sizes, baseline_offset_in_parent) =
            self.get_block_sizes_and_baseline_offset(ifc, size.block, baseline_offset);
        ifc.update_unbreakable_segment_for_new_content(
            &block_sizes,
            size.inline,
            SegmentContentFlags::empty(),
        );
        ifc.push_line_item_to_unbreakable_segment(LineItem::Atomic(AtomicLineItem {
            fragment,
            size,
            positioning_context: child_positioning_context,
            baseline_offset_in_parent,
            baseline_offset_in_item: baseline_offset,
        }));

        // Defer a soft wrap opportunity for when we next process text content.
        ifc.have_deferred_soft_wrap_opportunity = true;
    }

    fn get_block_sizes_and_baseline_offset(
        &self,
        ifc: &InlineFormattingContextState,
        block_size: Length,
        baseline_offset_in_content_area: Au,
    ) -> (LineBlockSizes, Au) {
        let mut contribution = if !is_baseline_relative(self.style().clone_vertical_align()) {
            LineBlockSizes {
                line_height: block_size,
                baseline_relative_size_for_line_height: None,
                size_for_baseline_positioning: BaselineRelativeSize::zero(),
            }
        } else {
            let baseline_relative_size = BaselineRelativeSize {
                ascent: baseline_offset_in_content_area,
                descent: Au::from_f32_px(block_size.px()) - baseline_offset_in_content_area,
            };
            LineBlockSizes {
                line_height: block_size,
                baseline_relative_size_for_line_height: Some(baseline_relative_size.clone()),
                size_for_baseline_positioning: baseline_relative_size,
            }
        };

        let baseline_offset = ifc
            .current_inline_container_state()
            .get_cumulative_baseline_offset_for_child(
                self.style().clone_vertical_align(),
                &contribution,
            );
        contribution.adjust_for_baseline_offset(baseline_offset);

        (contribution, baseline_offset)
    }
}

impl FloatBox {
    fn layout_into_line_items(
        &mut self,
        layout_context: &LayoutContext,
        ifc: &mut InlineFormattingContextState,
    ) {
        let fragment = self.layout(
            layout_context,
            ifc.positioning_context,
            ifc.containing_block,
        );
        ifc.push_line_item_to_unbreakable_segment(LineItem::Float(FloatLineItem {
            fragment,
            needs_placement: true,
        }));
    }
}

fn place_pending_floats(ifc: &mut InlineFormattingContextState, line_items: &mut [LineItem]) {
    for item in line_items.iter_mut() {
        if let LineItem::Float(float_line_item) = item {
            if float_line_item.needs_placement {
                ifc.place_float_fragment(&mut float_line_item.fragment);
            }
        }
    }
}

fn line_height(parent_style: &ComputedValues, font_metrics: &FontMetrics) -> Length {
    let font_size = parent_style.get_font().font_size.computed_size();
    match parent_style.get_inherited_text().line_height {
        LineHeight::Normal => Length::from(font_metrics.line_gap),
        LineHeight::Number(number) => font_size * number.0,
        LineHeight::Length(length) => length.0,
    }
}

fn effective_vertical_align(
    style: &ComputedValues,
    container: Option<&InlineContainerState>,
) -> VerticalAlign {
    if container.is_none() {
        // If we are at the root of the inline formatting context, we shouldn't use the
        // computed `vertical-align`, since it has no effect on the contents of this IFC
        // (it can just affect how the block container is aligned within the parent IFC).
        VerticalAlign::Keyword(VerticalAlignKeyword::Baseline)
    } else {
        style.clone_vertical_align()
    }
}

fn is_baseline_relative(vertical_align: VerticalAlign) -> bool {
    !matches!(
        vertical_align,
        VerticalAlign::Keyword(VerticalAlignKeyword::Top) |
            VerticalAlign::Keyword(VerticalAlignKeyword::Bottom)
    )
}

/// Whether or not a strut should be created for an inline container. Normally
/// all inline containers get struts. In quirks mode this isn't always the case
/// though.
///
/// From <https://quirks.spec.whatwg.org/#the-line-height-calculation-quirk>
///
/// > ### § 3.3. The line height calculation quirk
/// > In quirks mode and limited-quirks mode, an inline box that matches the following
/// > conditions, must, for the purpose of line height calculation, act as if the box had a
/// > line-height of zero.
/// >
/// >  - The border-top-width, border-bottom-width, padding-top and padding-bottom
/// >    properties have a used value of zero and the box has a vertical writing mode, or the
/// >    border-right-width, border-left-width, padding-right and padding-left properties have
/// >    a used value of zero and the box has a horizontal writing mode.
/// >  - It either contains no text or it contains only collapsed whitespace.
/// >
/// > ### § 3.4. The blocks ignore line-height quirk
/// > In quirks mode and limited-quirks mode, for a block container element whose content is
/// > composed of inline-level elements, the element’s line-height must be ignored for the
/// > purpose of calculating the minimal height of line boxes within the element.
///
/// Since we incorporate the size of the strut into the line-height calculation when
/// adding text, we can simply not incorporate the strut at the start of inline box
/// processing. This also works the same for the root of the IFC.
fn inline_container_needs_strut(
    style: &ComputedValues,
    layout_context: &LayoutContext,
    pbm: Option<&PaddingBorderMargin>,
) -> bool {
    if layout_context.style_context.quirks_mode() == QuirksMode::NoQuirks {
        return true;
    }

    // This is not in a standard yet, but all browsers disable this quirk for list items.
    // See https://github.com/whatwg/quirks/issues/38.
    if style.get_box().display.is_list_item() {
        return true;
    }

    pbm.map(|pbm| !pbm.padding_border_sums.inline.is_zero())
        .unwrap_or(false)
}

/// A struct which takes care of computing [`ContentSizes`] for an [`InlineFormattingContext`].
struct ContentSizesComputation<'a> {
    layout_context: &'a LayoutContext<'a>,
    containing_block_writing_mode: WritingMode,
    paragraph: ContentSizes,
    current_line: ContentSizes,
    /// Size for whitepsace pending to be added to this line.
    pending_whitespace: Au,
    /// Whether or not this IFC has seen any content, excluding collapsed whitespace.
    had_content_yet: bool,
    /// Stack of ending padding, margin, and border to add to the length
    /// when an inline box finishes.
    ending_inline_pbm_stack: Vec<Length>,
}

impl<'a> ContentSizesComputation<'a> {
    fn traverse(mut self, inline_formatting_context: &InlineFormattingContext) -> ContentSizes {
        inline_formatting_context.foreach(|iter_item| match iter_item {
            InlineFormattingContextIterItem::Item(InlineLevelBox::InlineBox(inline_box)) => {
                // For margins and paddings, a cyclic percentage is resolved against zero
                // for determining intrinsic size contributions.
                // https://drafts.csswg.org/css-sizing-3/#min-percentage-contribution
                let zero = Length::zero();
                let padding = inline_box
                    .style
                    .padding(self.containing_block_writing_mode)
                    .percentages_relative_to(zero);
                let border = inline_box
                    .style
                    .border_width(self.containing_block_writing_mode);
                let margin = inline_box
                    .style
                    .margin(self.containing_block_writing_mode)
                    .percentages_relative_to(zero)
                    .auto_is(Length::zero);

                let pbm = &(&margin + &padding) + &border;
                if inline_box.is_first_fragment {
                    self.add_length(pbm.inline_start);
                }
                if inline_box.is_last_fragment {
                    self.ending_inline_pbm_stack.push(pbm.inline_end);
                } else {
                    self.ending_inline_pbm_stack.push(Length::zero());
                }
            },
            InlineFormattingContextIterItem::EndInlineBox => {
                let length = self
                    .ending_inline_pbm_stack
                    .pop()
                    .unwrap_or_else(Length::zero);
                self.add_length(length);
            },
            InlineFormattingContextIterItem::Item(InlineLevelBox::TextRun(text_run)) => {
                for segment in text_run.shaped_text.iter() {
                    // TODO: This should take account whether or not the first and last character prevent
                    // linebreaks after atomics as in layout.
                    if segment.break_at_start {
                        self.line_break_opportunity()
                    }

                    for run in segment.runs.iter() {
                        let advance = run.glyph_store.total_advance();

                        if run.glyph_store.is_whitespace() {
                            // If this run is a forced line break, we *must* break the line
                            // and start measuring from the inline origin once more.
                            if text_run.glyph_run_is_preserved_newline(run) {
                                self.had_content_yet = true;
                                self.forced_line_break();
                                self.current_line = ContentSizes::zero();
                                continue;
                            }

                            let white_space =
                                text_run.parent_style.get_inherited_text().white_space;
                            // TODO: need to handle white_space.allow_wrap() too.
                            if !white_space.preserve_spaces() {
                                // Discard any leading whitespace in the IFC. This will always be trimmed.
                                if self.had_content_yet {
                                    // Wait to take into account other whitespace until we see more content.
                                    // Whitespace at the end of the IFC will always be trimmed.
                                    self.line_break_opportunity();
                                    self.pending_whitespace += advance;
                                }
                                continue;
                            }
                        }

                        self.had_content_yet = true;
                        self.current_line.min_content += advance;
                        self.current_line.max_content += self.pending_whitespace + advance;
                        self.pending_whitespace = Au::zero();
                    }
                }
            },
            InlineFormattingContextIterItem::Item(InlineLevelBox::Atomic(atomic)) => {
                let outer = atomic.outer_inline_content_sizes(
                    self.layout_context,
                    self.containing_block_writing_mode,
                );

                self.current_line.min_content += self.pending_whitespace + outer.min_content;
                self.current_line.max_content += self.pending_whitespace + outer.max_content;
                self.pending_whitespace = Au::zero();
                self.had_content_yet = true;
            },
            _ => {},
        });

        self.forced_line_break();
        self.paragraph
    }

    fn add_length(&mut self, l: Length) {
        self.current_line.min_content += l.into();
        self.current_line.max_content += l.into();
    }

    fn line_break_opportunity(&mut self) {
        self.paragraph.min_content =
            std::cmp::max(self.paragraph.min_content, self.current_line.min_content);
        self.current_line.min_content = Au::zero();
    }

    fn forced_line_break(&mut self) {
        self.line_break_opportunity();
        self.paragraph.max_content =
            std::cmp::max(self.paragraph.max_content, self.current_line.max_content);
        self.current_line.max_content = Au::zero();
    }

    /// Compute the [`ContentSizes`] of the given [`InlineFormattingContext`].
    fn compute(
        inline_formatting_context: &InlineFormattingContext,
        layout_context: &'a LayoutContext,
        containing_block_writing_mode: WritingMode,
    ) -> ContentSizes {
        Self {
            layout_context,
            containing_block_writing_mode,
            paragraph: ContentSizes::zero(),
            current_line: ContentSizes::zero(),
            pending_whitespace: Au::zero(),
            had_content_yet: false,
            ending_inline_pbm_stack: Vec::new(),
        }
        .traverse(inline_formatting_context)
    }
}
