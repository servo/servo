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
//!  - [`LineItem::InlineStartBoxPaddingBorderMargin`]
//!  - [`LineItem::InlineEndBoxPaddingBorderMargin`]
//!  - [`LineItem::TextRun`]
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

pub mod construct;
pub mod inline_box;
pub mod line;
mod line_breaker;
pub mod text_run;

use std::cell::{OnceCell, RefCell};
use std::mem;
use std::rc::Rc;

use app_units::{Au, MAX_AU};
use bitflags::bitflags;
use construct::InlineFormattingContextBuilder;
use fonts::{FontMetrics, GlyphStore};
use inline_box::{InlineBox, InlineBoxContainerState, InlineBoxIdentifier, InlineBoxes};
use line::{
    AbsolutelyPositionedLineItem, AtomicLineItem, FloatLineItem, LineItem, LineItemLayout,
    TextRunLineItem,
};
use line_breaker::LineBreaker;
use servo_arc::Arc;
use style::Zero;
use style::computed_values::text_wrap_mode::T as TextWrapMode;
use style::computed_values::vertical_align::T as VerticalAlign;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::context::QuirksMode;
use style::properties::ComputedValues;
use style::properties::style_structs::InheritedText;
use style::values::generics::box_::VerticalAlignKeyword;
use style::values::generics::font::LineHeight;
use style::values::specified::box_::BaselineSource;
use style::values::specified::text::{TextAlignKeyword, TextDecorationLine};
use style::values::specified::{TextAlignLast, TextJustify};
use text_run::{
    TextRun, XI_LINE_BREAKING_CLASS_GL, XI_LINE_BREAKING_CLASS_WJ, XI_LINE_BREAKING_CLASS_ZWJ,
    add_or_get_font, get_font_for_first_font_for_style,
};
use unicode_bidi::{BidiInfo, Level};
use webrender_api::FontInstanceKey;
use xi_unicode::linebreak_property;

use super::float::{Clear, PlacementAmongFloats};
use super::{
    CacheableLayoutResult, IndependentFloatOrAtomicLayoutResult,
    IndependentFormattingContextContents,
};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::flow::CollapsibleWithParentStartMargin;
use crate::flow::float::{FloatBox, SequentialLayoutState};
use crate::formatting_contexts::{
    Baselines, IndependentFormattingContext, IndependentNonReplacedContents,
};
use crate::fragment_tree::{
    BoxFragment, CollapsedBlockMargins, CollapsedMargin, Fragment, FragmentFlags,
    PositioningFragment,
};
use crate::geom::{LogicalRect, LogicalVec2, ToLogical};
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext};
use crate::sizing::{ComputeInlineContentSizes, ContentSizes, InlineContentSizesResult};
use crate::style_ext::{ComputedValuesExt, PaddingBorderMargin};
use crate::{ConstraintSpace, ContainingBlock, PropagatedBoxTreeData};

// From gfxFontConstants.h in Firefox.
static FONT_SUBSCRIPT_OFFSET_RATIO: f32 = 0.20;
static FONT_SUPERSCRIPT_OFFSET_RATIO: f32 = 0.34;

#[derive(Debug)]
pub(crate) struct InlineFormattingContext {
    /// All [`InlineItem`]s in this [`InlineFormattingContext`] stored in a flat array.
    /// [`InlineItem::StartInlineBox`] and [`InlineItem::EndInlineBox`] allow representing
    /// the tree of inline boxes within the formatting context, but a flat array allows
    /// easy iteration through all inline items.
    pub(super) inline_items: Vec<ArcRefCell<InlineItem>>,

    /// The tree of inline boxes in this [`InlineFormattingContext`]. These are stored in
    /// a flat array with each being given a [`InlineBoxIdentifier`].
    pub(super) inline_boxes: InlineBoxes,

    /// The text content of this inline formatting context.
    pub(super) text_content: String,

    /// A store of font information for all the shaped segments in this formatting
    /// context in order to avoid duplicating this information.
    pub font_metrics: Vec<FontKeyAndMetrics>,

    pub(super) text_decoration_line: TextDecorationLine,

    /// Whether this IFC contains the 1st formatted line of an element:
    /// <https://www.w3.org/TR/css-pseudo-4/#first-formatted-line>.
    pub(super) has_first_formatted_line: bool,

    /// Whether or not this [`InlineFormattingContext`] contains floats.
    pub(super) contains_floats: bool,

    /// Whether or not this is an [`InlineFormattingContext`] for a single line text input.
    pub(super) is_single_line_text_input: bool,

    /// Whether or not this is an [`InlineFormattingContext`] has right-to-left content, which
    /// will require reordering during layout.
    pub(super) has_right_to_left_content: bool,
}

/// A collection of data used to cache [`FontMetrics`] in the [`InlineFormattingContext`]
#[derive(Debug)]
pub(crate) struct FontKeyAndMetrics {
    pub key: FontInstanceKey,
    pub pt_size: Au,
    pub metrics: FontMetrics,
}

#[derive(Debug)]
pub(crate) enum InlineItem {
    StartInlineBox(ArcRefCell<InlineBox>),
    EndInlineBox,
    TextRun(ArcRefCell<TextRun>),
    OutOfFlowAbsolutelyPositionedBox(
        ArcRefCell<AbsolutelyPositionedBox>,
        usize, /* offset_in_text */
    ),
    OutOfFlowFloatBox(Arc<FloatBox>),
    Atomic(
        Arc<IndependentFormattingContext>,
        usize, /* offset_in_text */
        Level, /* bidi_level */
    ),
}

/// Information about the current line under construction for a particular
/// [`InlineFormattingContextLayout`]. This tracks position and size information while
/// [`LineItem`]s are collected and is used as input when those [`LineItem`]s are
/// converted into [`Fragment`]s during the final phase of line layout. Note that this
/// does not store the [`LineItem`]s themselves, as they are stored as part of the
/// nesting state in the [`InlineFormattingContextLayout`].
struct LineUnderConstruction {
    /// The position where this line will start once it is laid out. This includes any
    /// offset from `text-indent`.
    start_position: LogicalVec2<Au>,

    /// The current inline position in the line being laid out into [`LineItem`]s in this
    /// [`InlineFormattingContext`] independent of the depth in the nesting level.
    inline_position: Au,

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
    placement_among_floats: OnceCell<LogicalRect<Au>>,

    /// The LineItems for the current line under construction that have already
    /// been committed to this line.
    line_items: Vec<LineItem>,
}

impl LineUnderConstruction {
    fn new(start_position: LogicalVec2<Au>) -> Self {
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
            Some(placement_among_floats) => placement_among_floats.start_corner.block,
            None => self.start_position.block,
        }
    }

    fn replace_placement_among_floats(&mut self, new_placement: LogicalRect<Au>) {
        self.placement_among_floats.take();
        let _ = self.placement_among_floats.set(new_placement);
    }

    /// Trim the trailing whitespace in this line and return the width of the whitespace trimmed.
    fn trim_trailing_whitespace(&mut self) -> Au {
        // From <https://www.w3.org/TR/css-text-3/#white-space-phase-2>:
        // > 3. A sequence of collapsible spaces at the end of a line is removed,
        // >    as well as any trailing U+1680   OGHAM SPACE MARK whose white-space
        // >    property is normal, nowrap, or pre-line.
        let mut whitespace_trimmed = Au::zero();
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
                LineItem::TextRun(_, text_run) => Some(
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
    line_height: Au,
    baseline_relative_size_for_line_height: Option<BaselineRelativeSize>,
    size_for_baseline_positioning: BaselineRelativeSize,
}

impl LineBlockSizes {
    fn zero() -> Self {
        LineBlockSizes {
            line_height: Au::zero(),
            baseline_relative_size_for_line_height: None,
            size_for_baseline_positioning: BaselineRelativeSize::zero(),
        }
    }

    fn resolve(&self) -> Au {
        let height_from_ascent_and_descent = self
            .baseline_relative_size_for_line_height
            .as_ref()
            .map(|size| (size.ascent + size.descent).abs())
            .unwrap_or_else(Au::zero);
        self.line_height.max(height_from_ascent_and_descent)
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
                let leading = self.resolve() -
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
    inline_size: Au,

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
    trailing_whitespace_size: Au,
}

impl UnbreakableSegmentUnderConstruction {
    fn new() -> Self {
        Self {
            inline_size: Au::zero(),
            max_block_size: LineBlockSizes {
                line_height: Au::zero(),
                baseline_relative_size_for_line_height: None,
                size_for_baseline_positioning: BaselineRelativeSize::zero(),
            },
            line_items: Vec::new(),
            inline_box_hierarchy_depth: None,
            has_content: false,
            trailing_whitespace_size: Au::zero(),
        }
    }

    /// Reset this segment after its contents have been committed to a line.
    fn reset(&mut self) {
        assert!(self.line_items.is_empty()); // Preserve allocated memory.
        self.inline_size = Au::zero();
        self.max_block_size = LineBlockSizes::zero();
        self.inline_box_hierarchy_depth = None;
        self.has_content = false;
        self.trailing_whitespace_size = Au::zero();
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
        let mut whitespace_trimmed = Au::zero();
        for item in self.line_items.iter_mut() {
            if !item.trim_whitespace_at_start(&mut whitespace_trimmed) {
                break;
            }
        }
        self.inline_size -= whitespace_trimmed;
    }
}

bitflags! {
    pub struct InlineContainerStateFlags: u8 {
        const CREATE_STRUT = 0b0001;
        const IS_SINGLE_LINE_TEXT_INPUT = 0b0010;
    }
}

pub(super) struct InlineContainerState {
    /// The style of this inline container.
    style: Arc<ComputedValues>,

    /// Flags which describe details of this [`InlineContainerState`].
    flags: InlineContainerStateFlags,

    /// Whether or not we have processed any content (an atomic element or text) for
    /// this inline box on the current line OR any previous line.
    has_content: RefCell<bool>,

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
    pub baseline_offset: Au,

    /// The font metrics of the non-fallback font for this container.
    font_metrics: FontMetrics,
}

pub(super) struct InlineFormattingContextLayout<'layout_data> {
    positioning_context: &'layout_data mut PositioningContext,
    containing_block: &'layout_data ContainingBlock<'layout_data>,
    sequential_layout_state: Option<&'layout_data mut SequentialLayoutState>,
    layout_context: &'layout_data LayoutContext<'layout_data>,

    /// The [`InlineFormattingContext`] that we are laying out.
    ifc: &'layout_data InlineFormattingContext,

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
    inline_box_state_stack: Vec<Rc<InlineBoxContainerState>>,

    /// A collection of [`InlineBoxContainerState`] of all the inlines that are present
    /// in this inline formatting context. We keep this as well as the stack, so that we
    /// can access them during line layout, which may happen after relevant [`InlineBoxContainerState`]s
    /// have been popped of the the stack.
    inline_box_states: Vec<Rc<InlineBoxContainerState>>,

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
    /// [`InlineFormattingContextLayout::finish_inline_box()`].
    linebreak_before_new_content: bool,

    /// When a `<br>` element has `clear`, this needs to be applied after the linebreak,
    /// which will be processed *after* the `<br>` element is processed. This member
    /// stores any deferred `clear` to apply after a linebreak.
    deferred_br_clear: Clear,

    /// Whether or not a soft wrap opportunity is queued. Soft wrap opportunities are
    /// queued after replaced content and they are processed when the next text content
    /// is encountered.
    pub have_deferred_soft_wrap_opportunity: bool,

    /// Whether or not this InlineFormattingContext has processed any in flow content at all.
    had_inflow_content: bool,

    /// Whether or not the layout of this InlineFormattingContext depends on the block size
    /// of its container for the purposes of flexbox layout.
    depends_on_block_constraints: bool,

    /// The currently white-space-collapse setting of this line. This is stored on the
    /// [`InlineFormattingContextLayout`] because when a soft wrap opportunity is defined
    /// by the boundary between two characters, the white-space-collapse property of their
    /// nearest common ancestor is used.
    white_space_collapse: WhiteSpaceCollapse,

    /// The currently text-wrap-mode setting of this line. This is stored on the
    /// [`InlineFormattingContextLayout`] because when a soft wrap opportunity is defined
    /// by the boundary between two characters, the text-wrap-mode property of their nearest
    /// common ancestor is used.
    text_wrap_mode: TextWrapMode,

    /// The offset of the first and last baselines in the inline formatting context that we
    /// are laying out. This is used to propagate baselines to the ancestors of
    /// `display: inline-block` elements and table content.
    baselines: Baselines,
}

impl InlineFormattingContextLayout<'_> {
    fn current_inline_container_state(&self) -> &InlineContainerState {
        match self.inline_box_state_stack.last() {
            Some(inline_box_state) => &inline_box_state.base,
            None => &self.root_nesting_level,
        }
    }

    fn current_inline_box_identifier(&self) -> Option<InlineBoxIdentifier> {
        self.inline_box_state_stack
            .last()
            .map(|state| state.identifier)
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
        let style_text = style.get_inherited_text();
        self.white_space_collapse = style_text.white_space_collapse;
        self.text_wrap_mode = style_text.text_wrap_mode;
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
        let inline_box_state = InlineBoxContainerState::new(
            inline_box,
            self.containing_block,
            self.layout_context,
            self.current_inline_container_state(),
            inline_box.is_last_fragment,
            inline_box
                .default_font_index
                .map(|index| &self.ifc.font_metrics[index].metrics),
        );

        self.depends_on_block_constraints |= inline_box
            .style
            .depends_on_block_constraints_due_to_relative_positioning(
                self.containing_block.style.writing_mode,
            );

        // If we are starting a `<br>` element prepare to clear after its deferred linebreak has been
        // processed. Note that a `<br>` is composed of the element itself and the inner pseudo-element
        // with the actual linebreak. Both will have this `FragmentFlag`; that's why this code only
        // sets `deferred_br_clear` if it isn't set yet.
        if inline_box_state
            .base_fragment_info
            .flags
            .contains(FragmentFlags::IS_BR_ELEMENT) &&
            self.deferred_br_clear == Clear::None
        {
            self.deferred_br_clear = Clear::from_style_and_container_writing_mode(
                &inline_box_state.base.style,
                self.containing_block.style.writing_mode,
            );
        }

        if inline_box.is_first_fragment {
            self.current_line_segment.inline_size += inline_box_state.pbm.padding.inline_start +
                inline_box_state.pbm.border.inline_start +
                inline_box_state.pbm.margin.inline_start.auto_is(Au::zero);
            self.current_line_segment
                .line_items
                .push(LineItem::InlineStartBoxPaddingBorderMargin(
                    inline_box.identifier,
                ));
        }

        let inline_box_state = Rc::new(inline_box_state);

        // Push the state onto the IFC-wide collection of states. Inline boxes are numbered in
        // the order that they are encountered, so this should correspond to the order they
        // are pushed onto `self.inline_box_states`.
        assert_eq!(
            self.inline_box_states.len(),
            inline_box.identifier.index_in_inline_boxes as usize
        );
        self.inline_box_states.push(inline_box_state.clone());
        self.inline_box_state_stack.push(inline_box_state);
    }

    /// Finish laying out a particular [`InlineBox`] into line items. This will
    /// pop its state off of [`Self::inline_box_state_stack`].
    fn finish_inline_box(&mut self) {
        let inline_box_state = match self.inline_box_state_stack.pop() {
            Some(inline_box_state) => inline_box_state,
            None => return, // We are at the root.
        };

        self.current_line_segment
            .max_block_size
            .max_assign(&inline_box_state.base.nested_strut_block_sizes);

        // If the inline box that we just finished had any content at all, we want to propagate
        // the `white-space` property of its parent to future inline children. This is because
        // when a soft wrap opportunity is defined by the boundary between two elements, the
        // `white-space` used is that of their nearest common ancestor.
        if *inline_box_state.base.has_content.borrow() {
            self.propagate_current_nesting_level_white_space_style();
        }

        if inline_box_state.is_last_fragment {
            let pbm_end = inline_box_state.pbm.padding.inline_end +
                inline_box_state.pbm.border.inline_end +
                inline_box_state.pbm.margin.inline_end.auto_is(Au::zero);
            self.current_line_segment.inline_size += pbm_end;
            self.current_line_segment
                .line_items
                .push(LineItem::InlineEndBoxPaddingBorderMargin(
                    inline_box_state.identifier,
                ))
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
    /// [`InlineFormattingContextLayout`] preparing it for laying out a new line.
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

        let resolved_block_advance = effective_block_advance.resolve();
        let mut block_end_position = block_start_position + resolved_block_advance;
        if let Some(sequential_layout_state) = self.sequential_layout_state.as_mut() {
            // This amount includes both the block size of the line and any extra space
            // added to move the line down in order to avoid overlapping floats.
            let increment = block_end_position - self.current_line.start_position.block;
            sequential_layout_state.advance_block_position(increment);

            // This newline may have been triggered by a `<br>` with clearance, in which case we
            // want to make sure that we make space not only for the current line, but any clearance
            // from floats.
            if let Some(clearance) = sequential_layout_state
                .calculate_clearance(self.deferred_br_clear, &CollapsedMargin::zero())
            {
                sequential_layout_state.advance_block_position(clearance);
                block_end_position += clearance;
            };
            self.deferred_br_clear = Clear::None;
        }

        // Set up the new line now that we no longer need the old one.
        let mut line_to_layout = std::mem::replace(
            &mut self.current_line,
            LineUnderConstruction::new(LogicalVec2 {
                inline: Au::zero(),
                block: block_end_position,
            }),
        );

        if line_to_layout.has_floats_waiting_to_be_placed {
            place_pending_floats(self, &mut line_to_layout.line_items);
        }

        let start_position = LogicalVec2 {
            block: block_start_position,
            inline: inline_start_position,
        };

        let baseline_offset = effective_block_advance.find_baseline_offset();
        let start_positioning_context_length = self.positioning_context.len();
        let fragments = LineItemLayout::layout_line_items(
            self,
            line_to_layout.line_items,
            start_position,
            &effective_block_advance,
            justification_adjustment,
        );

        // If the line doesn't have any fragments, we don't need to add a containing fragment for it.
        if fragments.is_empty() &&
            self.positioning_context.len() == start_positioning_context_length
        {
            return;
        }

        let baseline = baseline_offset + block_start_position;
        self.baselines.first.get_or_insert(baseline);
        self.baselines.last = Some(baseline);

        // The inline part of this start offset was taken into account when determining
        // the inline start of the line in `calculate_inline_start_for_current_line` so
        // we do not need to include it in the `start_corner` of the line's main Fragment.
        let start_corner = LogicalVec2 {
            inline: Au::zero(),
            block: block_start_position,
        };

        let logical_origin_in_physical_coordinates =
            start_corner.to_physical_vector(self.containing_block.style.writing_mode);
        self.positioning_context
            .adjust_static_position_of_hoisted_fragments_with_offset(
                &logical_origin_in_physical_coordinates,
                start_positioning_context_length,
            );

        let physical_line_rect = LogicalRect {
            start_corner,
            size: LogicalVec2 {
                inline: self.containing_block.size.inline,
                block: effective_block_advance.resolve(),
            },
        }
        .as_physical(Some(self.containing_block));
        self.fragments
            .push(Fragment::Positioning(PositioningFragment::new_anonymous(
                physical_line_rect,
                fragments,
            )));
    }

    /// Given the amount of whitespace trimmed from the line and taking into consideration
    /// the `text-align` property, calculate where the line under construction starts in
    /// the inline axis as well as the adjustment needed for every justification opportunity
    /// to account for `text-align: justify`.
    fn calculate_current_line_inline_start_and_justification_adjustment(
        &self,
        whitespace_trimmed: Au,
        last_line_or_forced_line_break: bool,
    ) -> (Au, Au) {
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
            TextAlignKeyword::Center | TextAlignKeyword::MozCenter => TextAlign::Center,
            TextAlignKeyword::End => TextAlign::End,
            TextAlignKeyword::Left | TextAlignKeyword::MozLeft => {
                if style.writing_mode.line_left_is_inline_start() {
                    TextAlign::Start
                } else {
                    TextAlign::End
                }
            },
            TextAlignKeyword::Right | TextAlignKeyword::MozRight => {
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
            None => (Au::zero(), self.containing_block.size.inline),
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
                TextAlign::Center => (available_space - line_length + text_indent)
                    .scale_by(0.5)
                    .max(text_indent),
            };

        // Calculate the justification adjustment. This is simply the remaining space on the line,
        // dividided by the number of justficiation opportunities that we recorded when building
        // the line.
        let text_justify = self.containing_block.style.clone_text_justify();
        let justification_adjustment = match (text_align_keyword, text_justify) {
            // `text-justify: none` should disable text justification.
            // TODO: Handle more `text-justify` values.
            (TextAlignKeyword::Justify, TextJustify::None) => Au::zero(),
            (TextAlignKeyword::Justify, _) => {
                match self.current_line.count_justification_opportunities() {
                    0 => Au::zero(),
                    num_justification_opportunities => {
                        (available_space - text_indent - line_length)
                            .scale_by(1. / num_justification_opportunities as f32)
                    },
                }
            },
            _ => Au::zero(),
        };

        // If the content overflows the line, then justification adjustment will become negative. In
        // that case, do not make any adjustment for justification.
        let justification_adjustment = justification_adjustment.max(Au::zero());

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
            self.containing_block,
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
        line_inline_size_without_trailing_whitespace: Au,
    ) {
        let logical_margin_rect_size = float_item
            .fragment
            .margin_rect()
            .size
            .to_logical(self.containing_block.style.writing_mode);
        let inline_size = logical_margin_rect_size.inline.max(Au::zero());

        let available_inline_size = match self.current_line.placement_among_floats.get() {
            Some(placement_among_floats) => placement_among_floats.size.inline,
            None => self.containing_block.size.inline,
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
    fn place_line_among_floats(&self, potential_line_size: &LogicalVec2<Au>) -> LogicalRect<Au> {
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
                inline: potential_line_size.inline,
                block: potential_line_size.block,
            },
            &PaddingBorderMargin::zero(),
        );

        let mut placement_rect = placement.place();
        placement_rect.start_corner -= ifc_offset_in_float_container;
        placement_rect
    }

    /// Returns true if a new potential line size for the current line would require a line
    /// break. This takes into account floats and will also update the "placement among
    /// floats" for this line if the potential line size would not cause a line break.
    /// Thus, calling this method has side effects and should only be done while in the
    /// process of laying out line content that is always going to be committed to this
    /// line or the next.
    fn new_potential_line_size_causes_line_break(
        &mut self,
        potential_line_size: &LogicalVec2<Au>,
    ) -> bool {
        let available_line_space = if self.sequential_layout_state.is_some() {
            self.current_line
                .placement_among_floats
                .get_or_init(|| self.place_line_among_floats(potential_line_size))
                .size
        } else {
            LogicalVec2 {
                inline: self.containing_block.size.inline,
                block: MAX_AU,
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
        if potential_line_size.inline > self.containing_block.size.inline {
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
        // If the current portion of the unbreakable segment does not fit on the current line
        // we need to put it on a new line *before* actually triggering the hard line break.
        if !self.unbreakable_segment_fits_on_line() {
            self.process_line_break(false /* forced_line_break */);
        }

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
                Au::zero(),
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
        bidi_level: Level,
    ) {
        let inline_advance = glyph_store.total_advance();
        let flags = if glyph_store.is_whitespace() {
            SegmentContentFlags::from(text_run.parent_style.get_inherited_text())
        } else {
            SegmentContentFlags::empty()
        };

        // If the metrics of this font don't match the default font, we are likely using a fallback
        // font and need to adjust the line size to account for a potentially different font.
        // If somehow the metrics match, the line size won't change.
        let ifc_font_info = &self.ifc.font_metrics[font_index];
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
            let mut block_size = container_state.get_block_size_contribution(
                vertical_align,
                &font_metrics,
                &container_state.font_metrics,
            );
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

        let current_inline_box_identifier = self.current_inline_box_identifier();
        match self.current_line_segment.line_items.last_mut() {
            Some(LineItem::TextRun(inline_box_identifier, line_item))
                if *inline_box_identifier == current_inline_box_identifier &&
                    line_item.can_merge(ifc_font_info.key, bidi_level) =>
            {
                line_item.text.push(glyph_store);
                return;
            },
            _ => {},
        }

        self.push_line_item_to_unbreakable_segment(LineItem::TextRun(
            current_inline_box_identifier,
            TextRunLineItem {
                text: vec![glyph_store],
                base_fragment_info: text_run.base_fragment_info,
                parent_style: text_run.parent_style.clone(),
                font_metrics,
                font_key: ifc_font_info.key,
                text_decoration_line: self.current_inline_container_state().text_decoration_line,
                bidi_level,
            },
        ));
    }

    fn update_unbreakable_segment_for_new_content(
        &mut self,
        block_sizes_of_content: &LineBlockSizes,
        inline_size: Au,
        flags: SegmentContentFlags,
    ) {
        if flags.is_collapsible_whitespace() || flags.is_wrappable_and_hangable() {
            self.current_line_segment.trailing_whitespace_size = inline_size;
        } else {
            self.current_line_segment.trailing_whitespace_size = Au::zero();
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
        *self
            .current_inline_container_state()
            .has_content
            .borrow_mut() = true;
        self.propagate_current_nesting_level_white_space_style();
    }

    fn process_line_break(&mut self, forced_line_break: bool) {
        self.current_line_segment.trim_leading_whitespace();
        self.finish_current_line_and_reset(forced_line_break);
    }

    pub(super) fn unbreakable_segment_fits_on_line(&mut self) -> bool {
        let potential_line_size = LogicalVec2 {
            inline: self.current_line.inline_position + self.current_line_segment.inline_size -
                self.current_line_segment.trailing_whitespace_size,
            block: self
                .current_line_max_block_size_including_nested_containers()
                .max(&self.current_line_segment.max_block_size)
                .resolve(),
        };

        !self.new_potential_line_size_causes_line_break(&potential_line_size)
    }

    /// Process a soft wrap opportunity. This will either commit the current unbreakble
    /// segment to the current line, if it fits within the containing block and float
    /// placement boundaries, or do a line break and then commit the segment.
    pub(super) fn process_soft_wrap_opportunity(&mut self) {
        if self.current_line_segment.line_items.is_empty() {
            return;
        }
        if self.text_wrap_mode == TextWrapMode::Nowrap {
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
            if let LineItem::Float(_, float_item) = item {
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

        self.current_line.line_items.extend(segment_items);
        self.current_line.has_content |= self.current_line_segment.has_content;

        self.current_line_segment.reset();
    }
}

bitflags! {
    pub struct SegmentContentFlags: u8 {
        const COLLAPSIBLE_WHITESPACE = 0b00000001;
        const WRAPPABLE_AND_HANGABLE_WHITESPACE = 0b00000010;
    }
}

impl SegmentContentFlags {
    fn is_collapsible_whitespace(&self) -> bool {
        self.contains(Self::COLLAPSIBLE_WHITESPACE)
    }

    fn is_wrappable_and_hangable(&self) -> bool {
        self.contains(Self::WRAPPABLE_AND_HANGABLE_WHITESPACE)
    }
}

impl From<&InheritedText> for SegmentContentFlags {
    fn from(style_text: &InheritedText) -> Self {
        let mut flags = Self::empty();

        // White-space with `white-space-collapse: break-spaces` or `white-space-collapse: preserve`
        // never collapses.
        if !matches!(
            style_text.white_space_collapse,
            WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces
        ) {
            flags.insert(Self::COLLAPSIBLE_WHITESPACE);
        }

        // White-space with `white-space-collapse: break-spaces` never hangs and always takes up
        // space.
        if style_text.text_wrap_mode == TextWrapMode::Wrap &&
            style_text.white_space_collapse != WhiteSpaceCollapse::BreakSpaces
        {
            flags.insert(Self::WRAPPABLE_AND_HANGABLE_WHITESPACE);
        }
        flags
    }
}

impl InlineFormattingContext {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "InlineFormattingContext::new_with_builder",
            skip_all,
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    pub(super) fn new_with_builder(
        builder: InlineFormattingContextBuilder,
        layout_context: &LayoutContext,
        propagated_data: PropagatedBoxTreeData,
        has_first_formatted_line: bool,
        is_single_line_text_input: bool,
        starting_bidi_level: Level,
    ) -> Self {
        // This is to prevent a double borrow.
        let text_content: String = builder.text_segments.into_iter().collect();
        let mut font_metrics = Vec::new();

        let bidi_info = BidiInfo::new(&text_content, Some(starting_bidi_level));
        let has_right_to_left_content = bidi_info.has_rtl();

        let mut new_linebreaker = LineBreaker::new(text_content.as_str());
        for item in builder.inline_items.iter() {
            match &mut *item.borrow_mut() {
                InlineItem::TextRun(text_run) => {
                    text_run.borrow_mut().segment_and_shape(
                        &text_content,
                        &layout_context.font_context,
                        &mut new_linebreaker,
                        &mut font_metrics,
                        &bidi_info,
                    );
                },
                InlineItem::StartInlineBox(inline_box) => {
                    let inline_box = &mut *inline_box.borrow_mut();
                    if let Some(font) = get_font_for_first_font_for_style(
                        &inline_box.style,
                        &layout_context.font_context,
                    ) {
                        inline_box.default_font_index = Some(add_or_get_font(
                            &font,
                            &mut font_metrics,
                            &layout_context.font_context,
                        ));
                    }
                },
                InlineItem::Atomic(_, index_in_text, bidi_level) => {
                    *bidi_level = bidi_info.levels[*index_in_text];
                },
                InlineItem::OutOfFlowAbsolutelyPositionedBox(..) |
                InlineItem::OutOfFlowFloatBox(_) |
                InlineItem::EndInlineBox => {},
            }
        }

        InlineFormattingContext {
            text_content,
            inline_items: builder.inline_items,
            inline_boxes: builder.inline_boxes,
            font_metrics,
            text_decoration_line: propagated_data.text_decoration,
            has_first_formatted_line,
            contains_floats: builder.contains_floats,
            is_single_line_text_input,
            has_right_to_left_content,
        }
    }

    pub(super) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
    ) -> CacheableLayoutResult {
        let first_line_inline_start = if self.has_first_formatted_line {
            containing_block
                .style
                .get_inherited_text()
                .text_indent
                .length
                .to_used_value(containing_block.size.inline)
        } else {
            Au::zero()
        };

        let style = containing_block.style;

        // It's unfortunate that it isn't possible to get this during IFC text processing, but in
        // that situation the style of the containing block is unknown.
        let default_font_metrics =
            get_font_for_first_font_for_style(style, &layout_context.font_context)
                .map(|font| font.metrics.clone());

        let style_text = containing_block.style.get_inherited_text();
        let mut inline_container_state_flags = InlineContainerStateFlags::empty();
        if inline_container_needs_strut(style, layout_context, None) {
            inline_container_state_flags.insert(InlineContainerStateFlags::CREATE_STRUT);
        }
        if self.is_single_line_text_input {
            inline_container_state_flags
                .insert(InlineContainerStateFlags::IS_SINGLE_LINE_TEXT_INPUT);
        }

        let mut layout = InlineFormattingContextLayout {
            positioning_context,
            containing_block,
            sequential_layout_state,
            layout_context,
            ifc: self,
            fragments: Vec::new(),
            current_line: LineUnderConstruction::new(LogicalVec2 {
                inline: first_line_inline_start,
                block: Au::zero(),
            }),
            root_nesting_level: InlineContainerState::new(
                style.to_arc(),
                inline_container_state_flags,
                None, /* parent_container */
                self.text_decoration_line,
                default_font_metrics.as_ref(),
            ),
            inline_box_state_stack: Vec::new(),
            inline_box_states: Vec::with_capacity(self.inline_boxes.len()),
            current_line_segment: UnbreakableSegmentUnderConstruction::new(),
            linebreak_before_new_content: false,
            deferred_br_clear: Clear::None,
            have_deferred_soft_wrap_opportunity: false,
            had_inflow_content: false,
            depends_on_block_constraints: false,
            white_space_collapse: style_text.white_space_collapse,
            text_wrap_mode: style_text.text_wrap_mode,
            baselines: Baselines::default(),
        };

        // FIXME(pcwalton): This assumes that margins never collapse through inline formatting
        // contexts (i.e. that inline formatting contexts are never empty). Is that right?
        // FIXME(mrobinson): This should not happen if the IFC collapses through.
        if let Some(ref mut sequential_layout_state) = layout.sequential_layout_state {
            sequential_layout_state.collapse_margins();
            // FIXME(mrobinson): Collapse margins in the containing block offsets as well??
        }

        for item in self.inline_items.iter() {
            let item = &*item.borrow();

            // Any new box should flush a pending hard line break.
            if !matches!(item, InlineItem::EndInlineBox) {
                layout.possibly_flush_deferred_forced_line_break();
            }

            match item {
                InlineItem::StartInlineBox(inline_box) => {
                    layout.start_inline_box(&inline_box.borrow());
                },
                InlineItem::EndInlineBox => layout.finish_inline_box(),
                InlineItem::TextRun(run) => run.borrow().layout_into_line_items(&mut layout),
                InlineItem::Atomic(atomic_formatting_context, offset_in_text, bidi_level) => {
                    atomic_formatting_context.layout_into_line_items(
                        &mut layout,
                        *offset_in_text,
                        *bidi_level,
                    );
                },
                InlineItem::OutOfFlowAbsolutelyPositionedBox(positioned_box, _) => {
                    layout.push_line_item_to_unbreakable_segment(LineItem::AbsolutelyPositioned(
                        layout.current_inline_box_identifier(),
                        AbsolutelyPositionedLineItem {
                            absolutely_positioned_box: positioned_box.clone(),
                        },
                    ));
                },
                InlineItem::OutOfFlowFloatBox(float_box) => {
                    float_box.layout_into_line_items(&mut layout);
                },
            }
        }

        layout.finish_last_line();

        let mut collapsible_margins_in_children = CollapsedBlockMargins::zero();
        let content_block_size = layout.current_line.start_position.block;
        collapsible_margins_in_children.collapsed_through = !layout.had_inflow_content &&
            content_block_size.is_zero() &&
            collapsible_with_parent_start_margin.0;

        CacheableLayoutResult {
            fragments: layout.fragments,
            content_block_size,
            collapsible_margins_in_children,
            baselines: layout.baselines,
            depends_on_block_constraints: layout.depends_on_block_constraints,
            content_inline_size_for_table: None,
            specific_layout_info: None,
        }
    }

    fn next_character_prevents_soft_wrap_opportunity(&self, index: usize) -> bool {
        let Some(character) = self.text_content[index..].chars().nth(1) else {
            return false;
        };
        char_prevents_soft_wrap_opportunity_when_before_or_after_atomic(character)
    }

    fn previous_character_prevents_soft_wrap_opportunity(&self, index: usize) -> bool {
        let Some(character) = self.text_content[0..index].chars().next_back() else {
            return false;
        };
        char_prevents_soft_wrap_opportunity_when_before_or_after_atomic(character)
    }
}

impl InlineContainerState {
    fn new(
        style: Arc<ComputedValues>,
        flags: InlineContainerStateFlags,
        parent_container: Option<&InlineContainerState>,
        parent_text_decoration_line: TextDecorationLine,
        font_metrics: Option<&FontMetrics>,
    ) -> Self {
        let text_decoration_line = parent_text_decoration_line | style.clone_text_decoration_line();
        let font_metrics = font_metrics.cloned().unwrap_or_else(FontMetrics::empty);
        let line_height = line_height(
            &style,
            &font_metrics,
            flags.contains(InlineContainerStateFlags::IS_SINGLE_LINE_TEXT_INPUT),
        );

        let mut baseline_offset = Au::zero();
        let mut strut_block_sizes = Self::get_block_sizes_with_style(
            effective_vertical_align(&style, parent_container),
            &style,
            &font_metrics,
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
        if flags.contains(InlineContainerStateFlags::CREATE_STRUT) {
            nested_block_sizes.max_assign(&strut_block_sizes);
        }

        Self {
            style,
            flags,
            has_content: RefCell::new(false),
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
        font_metrics_of_first_font: &FontMetrics,
        line_height: Au,
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
        if style.get_font().line_height == LineHeight::Normal {
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
        if style.get_font().line_height != LineHeight::Normal {
            ascent = font_metrics_of_first_font.ascent;
            descent = font_metrics_of_first_font.descent;
            let half_leading = (line_height - (ascent + descent)).scale_by(0.5);
            // We want the sum of `ascent` and `descent` to equal `line_height`.
            // If we just add `half_leading` to both, then we may not get `line_height`
            // due to precision limitations of `Au`. Instead, we set `descent` to
            // the value that will guarantee the correct sum.
            ascent += half_leading;
            descent = line_height - ascent;
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
        font_metrics_of_first_font: &FontMetrics,
    ) -> LineBlockSizes {
        Self::get_block_sizes_with_style(
            vertical_align,
            &self.style,
            font_metrics,
            font_metrics_of_first_font,
            line_height(
                &self.style,
                font_metrics,
                self.flags
                    .contains(InlineContainerStateFlags::IS_SINGLE_LINE_TEXT_INPUT),
            ),
        )
    }

    fn get_cumulative_baseline_offset_for_child(
        &self,
        child_vertical_align: VerticalAlign,
        child_block_size: &LineBlockSizes,
    ) -> Au {
        let block_size = self.get_block_size_contribution(
            child_vertical_align.clone(),
            &self.font_metrics,
            &self.font_metrics,
        );
        self.baseline_offset +
            match child_vertical_align {
                // `top` and `bottom are not actually relative to the baseline, but this value is unused
                // in those cases.
                // TODO: We should distinguish these from `baseline` in order to implement "aligned subtrees" properly.
                // See https://drafts.csswg.org/css2/#aligned-subtree.
                VerticalAlign::Keyword(VerticalAlignKeyword::Baseline) |
                VerticalAlign::Keyword(VerticalAlignKeyword::Top) |
                VerticalAlign::Keyword(VerticalAlignKeyword::Bottom) => Au::zero(),
                VerticalAlign::Keyword(VerticalAlignKeyword::Sub) => {
                    block_size.resolve().scale_by(FONT_SUBSCRIPT_OFFSET_RATIO)
                },
                VerticalAlign::Keyword(VerticalAlignKeyword::Super) => {
                    -block_size.resolve().scale_by(FONT_SUPERSCRIPT_OFFSET_RATIO)
                },
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
                    -length_percentage.to_used_value(child_block_size.line_height)
                },
            }
    }
}

impl IndependentFormattingContext {
    fn layout_into_line_items(
        &self,
        layout: &mut InlineFormattingContextLayout,
        offset_in_text: usize,
        bidi_level: Level,
    ) {
        // We need to know the inline size of the atomic before deciding whether to do the line break.
        let mut child_positioning_context = PositioningContext::new_for_style(self.style())
            .unwrap_or_else(|| PositioningContext::new_for_subtree(true));
        let IndependentFloatOrAtomicLayoutResult {
            mut fragment,
            baselines,
            pbm_sums,
        } = self.layout_float_or_atomic_inline(
            layout.layout_context,
            &mut child_positioning_context,
            layout.containing_block,
        );

        // If this Fragment's layout depends on the block size of the containing block,
        // then the entire layout of the inline formatting context does as well.
        layout.depends_on_block_constraints |= fragment.base.flags.contains(
            FragmentFlags::SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM,
        );

        // Offset the content rectangle by the physical offset of the padding, border, and margin.
        let container_writing_mode = layout.containing_block.style.writing_mode;
        let pbm_physical_offset = pbm_sums
            .start_offset()
            .to_physical_size(container_writing_mode);
        fragment.content_rect = fragment
            .content_rect
            .translate(pbm_physical_offset.to_vector());

        // Apply baselines if necessary.
        let mut fragment = match baselines {
            Some(baselines) => fragment.with_baselines(baselines),
            None => fragment,
        };

        // Lay out absolutely positioned children if this new atomic establishes a containing block
        // for absolutes.
        let positioning_context = if self.is_replaced() {
            None
        } else {
            if fragment
                .style
                .establishes_containing_block_for_absolute_descendants(fragment.base.flags)
            {
                child_positioning_context
                    .layout_collected_children(layout.layout_context, &mut fragment);
            }
            Some(child_positioning_context)
        };

        if layout.text_wrap_mode == TextWrapMode::Wrap &&
            !layout
                .ifc
                .previous_character_prevents_soft_wrap_opportunity(offset_in_text)
        {
            layout.process_soft_wrap_opportunity();
        }

        let size = pbm_sums.sum() +
            fragment
                .content_rect
                .size
                .to_logical(container_writing_mode);
        let baseline_offset = self
            .pick_baseline(&fragment.baselines(container_writing_mode))
            .map(|baseline| pbm_sums.block_start + baseline)
            .unwrap_or(size.block);

        let (block_sizes, baseline_offset_in_parent) =
            self.get_block_sizes_and_baseline_offset(layout, size.block, baseline_offset);
        layout.update_unbreakable_segment_for_new_content(
            &block_sizes,
            size.inline,
            SegmentContentFlags::empty(),
        );
        layout.push_line_item_to_unbreakable_segment(LineItem::Atomic(
            layout.current_inline_box_identifier(),
            AtomicLineItem {
                fragment,
                size,
                positioning_context,
                baseline_offset_in_parent,
                baseline_offset_in_item: baseline_offset,
                bidi_level,
            },
        ));

        // If there's a soft wrap opportunity following this atomic, defer a soft wrap opportunity
        // for when we next process text content.
        if !layout
            .ifc
            .next_character_prevents_soft_wrap_opportunity(offset_in_text)
        {
            layout.have_deferred_soft_wrap_opportunity = true;
        }
    }

    /// Picks either the first or the last baseline, depending on `baseline-source`.
    /// TODO: clarify that this is not to be used for box alignment in flex/grid
    /// <https://drafts.csswg.org/css-inline/#baseline-source>
    fn pick_baseline(&self, baselines: &Baselines) -> Option<Au> {
        match self.style().clone_baseline_source() {
            BaselineSource::First => baselines.first,
            BaselineSource::Last => baselines.last,
            BaselineSource::Auto => match &self.contents {
                IndependentFormattingContextContents::NonReplaced(
                    IndependentNonReplacedContents::Flow(_),
                ) => baselines.last,
                _ => baselines.first,
            },
        }
    }

    fn get_block_sizes_and_baseline_offset(
        &self,
        ifc: &InlineFormattingContextLayout,
        block_size: Au,
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
                descent: block_size - baseline_offset_in_content_area,
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
    fn layout_into_line_items(&self, layout: &mut InlineFormattingContextLayout) {
        let fragment = self.layout(
            layout.layout_context,
            layout.positioning_context,
            layout.containing_block,
        );
        layout.push_line_item_to_unbreakable_segment(LineItem::Float(
            layout.current_inline_box_identifier(),
            FloatLineItem {
                fragment,
                needs_placement: true,
            },
        ));
    }
}

fn place_pending_floats(ifc: &mut InlineFormattingContextLayout, line_items: &mut [LineItem]) {
    for item in line_items.iter_mut() {
        if let LineItem::Float(_, float_line_item) = item {
            if float_line_item.needs_placement {
                ifc.place_float_fragment(&mut float_line_item.fragment);
            }
        }
    }
}

fn line_height(
    parent_style: &ComputedValues,
    font_metrics: &FontMetrics,
    is_single_line_text_input: bool,
) -> Au {
    let font = parent_style.get_font();
    let font_size = font.font_size.computed_size();
    let mut line_height = match font.line_height {
        LineHeight::Normal => font_metrics.line_gap,
        LineHeight::Number(number) => (font_size * number.0).into(),
        LineHeight::Length(length) => length.0.into(),
    };

    // Single line text inputs line height is clamped to the size of `normal`. See
    // <https://github.com/whatwg/html/pull/5462>.
    if is_single_line_text_input {
        line_height.max_assign(font_metrics.line_gap);
    }

    line_height
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

impl ComputeInlineContentSizes for InlineFormattingContext {
    // This works on an already-constructed `InlineFormattingContext`,
    // Which would have to change if/when
    // `BlockContainer::construct` parallelize their construction.
    fn compute_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        ContentSizesComputation::compute(self, layout_context, constraint_space)
    }
}

/// A struct which takes care of computing [`ContentSizes`] for an [`InlineFormattingContext`].
struct ContentSizesComputation<'layout_data> {
    layout_context: &'layout_data LayoutContext<'layout_data>,
    constraint_space: &'layout_data ConstraintSpace,
    paragraph: ContentSizes,
    current_line: ContentSizes,
    /// Size for whitespace pending to be added to this line.
    pending_whitespace: ContentSizes,
    /// Whether or not the current line has seen any content (excluding collapsed whitespace),
    /// when sizing under a min-content constraint.
    had_content_yet_for_min_content: bool,
    /// Whether or not the current line has seen any content (excluding collapsed whitespace),
    /// when sizing under a max-content constraint.
    had_content_yet_for_max_content: bool,
    /// Stack of ending padding, margin, and border to add to the length
    /// when an inline box finishes.
    ending_inline_pbm_stack: Vec<Au>,
    depends_on_block_constraints: bool,
}

impl<'layout_data> ContentSizesComputation<'layout_data> {
    fn traverse(
        mut self,
        inline_formatting_context: &InlineFormattingContext,
    ) -> InlineContentSizesResult {
        for inline_item in inline_formatting_context.inline_items.iter() {
            self.process_item(&inline_item.borrow(), inline_formatting_context);
        }

        self.forced_line_break();
        InlineContentSizesResult {
            sizes: self.paragraph,
            depends_on_block_constraints: self.depends_on_block_constraints,
        }
    }

    fn process_item(
        &mut self,
        inline_item: &InlineItem,
        inline_formatting_context: &InlineFormattingContext,
    ) {
        match inline_item {
            InlineItem::StartInlineBox(inline_box) => {
                // For margins and paddings, a cyclic percentage is resolved against zero
                // for determining intrinsic size contributions.
                // https://drafts.csswg.org/css-sizing-3/#min-percentage-contribution
                let inline_box = inline_box.borrow();
                let zero = Au::zero();
                let writing_mode = self.constraint_space.writing_mode;
                let layout_style = inline_box.layout_style();
                let padding = layout_style
                    .padding(writing_mode)
                    .percentages_relative_to(zero);
                let border = layout_style.border_width(writing_mode);
                let margin = inline_box
                    .style
                    .margin(writing_mode)
                    .percentages_relative_to(zero)
                    .auto_is(Au::zero);

                let pbm = margin + padding + border;
                if inline_box.is_first_fragment {
                    self.add_inline_size(pbm.inline_start);
                }
                if inline_box.is_last_fragment {
                    self.ending_inline_pbm_stack.push(pbm.inline_end);
                } else {
                    self.ending_inline_pbm_stack.push(Au::zero());
                }
            },
            InlineItem::EndInlineBox => {
                let length = self.ending_inline_pbm_stack.pop().unwrap_or_else(Au::zero);
                self.add_inline_size(length);
            },
            InlineItem::TextRun(text_run) => {
                let text_run = &*text_run.borrow();
                for segment in text_run.shaped_text.iter() {
                    let style_text = text_run.parent_style.get_inherited_text();
                    let can_wrap = style_text.text_wrap_mode == TextWrapMode::Wrap;

                    // TODO: This should take account whether or not the first and last character prevent
                    // linebreaks after atomics as in layout.
                    if can_wrap && segment.break_at_start {
                        self.line_break_opportunity()
                    }

                    for run in segment.runs.iter() {
                        let advance = run.glyph_store.total_advance();
                        if run.glyph_store.is_whitespace() {
                            // If this run is a forced line break, we *must* break the line
                            // and start measuring from the inline origin once more.
                            if run.is_single_preserved_newline() {
                                self.forced_line_break();
                                continue;
                            }
                            if !matches!(
                                style_text.white_space_collapse,
                                WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces
                            ) {
                                if can_wrap {
                                    self.line_break_opportunity();
                                } else if self.had_content_yet_for_min_content {
                                    self.pending_whitespace.min_content += advance;
                                }
                                if self.had_content_yet_for_max_content {
                                    self.pending_whitespace.max_content += advance;
                                }
                                continue;
                            }
                            if can_wrap {
                                self.pending_whitespace.max_content += advance;
                                self.commit_pending_whitespace();
                                self.line_break_opportunity();
                                continue;
                            }
                        }

                        self.commit_pending_whitespace();
                        self.add_inline_size(advance);

                        // Typically whitespace glyphs are placed in a separate store,
                        // but for `white-space: break-spaces` we place the first whitespace
                        // with the preceding text. That prevents a line break before that
                        // first space, but we still need to allow a line break after it.
                        if can_wrap && run.glyph_store.ends_with_whitespace() {
                            self.line_break_opportunity();
                        }
                    }
                }
            },
            InlineItem::Atomic(atomic, offset_in_text, _level) => {
                // TODO: need to handle TextWrapMode::Nowrap.
                if !inline_formatting_context
                    .previous_character_prevents_soft_wrap_opportunity(*offset_in_text)
                {
                    self.line_break_opportunity();
                }

                let InlineContentSizesResult {
                    sizes: outer,
                    depends_on_block_constraints,
                } = atomic.outer_inline_content_sizes(
                    self.layout_context,
                    &self.constraint_space.into(),
                    &LogicalVec2::zero(),
                    false, /* auto_block_size_stretches_to_containing_block */
                );
                self.depends_on_block_constraints |= depends_on_block_constraints;

                if !inline_formatting_context
                    .next_character_prevents_soft_wrap_opportunity(*offset_in_text)
                {
                    self.line_break_opportunity();
                }

                self.commit_pending_whitespace();
                self.current_line += outer;
            },
            _ => {},
        }
    }

    fn add_inline_size(&mut self, l: Au) {
        self.current_line.min_content += l;
        self.current_line.max_content += l;
    }

    fn line_break_opportunity(&mut self) {
        // Clear the pending whitespace, assuming that at the end of the line
        // it needs to either hang or be removed. If that isn't the case,
        // `commit_pending_whitespace()` should be called first.
        self.pending_whitespace.min_content = Au::zero();
        let current_min_content = mem::take(&mut self.current_line.min_content);
        self.paragraph.min_content.max_assign(current_min_content);
        self.had_content_yet_for_min_content = false;
    }

    fn forced_line_break(&mut self) {
        // Handle the line break for min-content sizes.
        self.line_break_opportunity();

        // Repeat the same logic, but now for max-content sizes.
        self.pending_whitespace.max_content = Au::zero();
        let current_max_content = mem::take(&mut self.current_line.max_content);
        self.paragraph.max_content.max_assign(current_max_content);
        self.had_content_yet_for_max_content = false;
    }

    fn commit_pending_whitespace(&mut self) {
        self.current_line += mem::take(&mut self.pending_whitespace);
        self.had_content_yet_for_min_content = true;
        self.had_content_yet_for_max_content = true;
    }

    /// Compute the [`ContentSizes`] of the given [`InlineFormattingContext`].
    fn compute(
        inline_formatting_context: &InlineFormattingContext,
        layout_context: &'layout_data LayoutContext,
        constraint_space: &'layout_data ConstraintSpace,
    ) -> InlineContentSizesResult {
        Self {
            layout_context,
            constraint_space,
            paragraph: ContentSizes::zero(),
            current_line: ContentSizes::zero(),
            pending_whitespace: ContentSizes::zero(),
            had_content_yet_for_min_content: false,
            had_content_yet_for_max_content: false,
            ending_inline_pbm_stack: Vec::new(),
            depends_on_block_constraints: false,
        }
        .traverse(inline_formatting_context)
    }
}

/// Whether or not this character will rpevent a soft wrap opportunity when it
/// comes before or after an atomic inline element.
///
/// From <https://www.w3.org/TR/css-text-3/#line-break-details>:
///
/// > For Web-compatibility there is a soft wrap opportunity before and after each
/// > replaced element or other atomic inline, even when adjacent to a character that
/// > would normally suppress them, including U+00A0 NO-BREAK SPACE. However, with
/// > the exception of U+00A0 NO-BREAK SPACE, there must be no soft wrap opportunity
/// > between atomic inlines and adjacent characters belonging to the Unicode GL, WJ,
/// > or ZWJ line breaking classes.
fn char_prevents_soft_wrap_opportunity_when_before_or_after_atomic(character: char) -> bool {
    if character == '\u{00A0}' {
        return false;
    }
    let class = linebreak_property(character);
    class == XI_LINE_BREAKING_CLASS_GL ||
        class == XI_LINE_BREAKING_CLASS_WJ ||
        class == XI_LINE_BREAKING_CLASS_ZWJ
}
