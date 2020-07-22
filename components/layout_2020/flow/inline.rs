/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::flow::float::{FloatBox, FloatContext};
use crate::flow::FlowLayout;
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::{
    AbsoluteOrFixedPositionedFragment, AnonymousFragment, BoxFragment, CollapsedBlockMargins,
    DebugId, FontMetrics, Fragment, Tag, TextFragment,
};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::positioned::{
    relative_adjustement, AbsolutelyPositionedBox, HoistedAbsolutelyPositionedBox,
    PositioningContext,
};
use crate::sizing::ContentSizes;
use crate::style_ext::{ComputedValuesExt, Display, DisplayGeneratingBox, DisplayOutside};
use crate::ContainingBlock;
use app_units::Au;
use atomic_refcell::AtomicRef;
use gfx::text::text_run::GlyphRun;
use servo_arc::Arc;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage, Percentage};
use style::values::specified::text::TextAlignKeyword;
use style::values::specified::text::TextDecorationLine;
use style::Zero;
use webrender_api::FontInstanceKey;

#[derive(Debug, Default, Serialize)]
pub(crate) struct InlineFormattingContext {
    pub(super) inline_level_boxes: Vec<ArcRefCell<InlineLevelBox>>,
    pub(super) text_decoration_line: TextDecorationLine,
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
    pub tag: Tag,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    pub first_fragment: bool,
    pub last_fragment: bool,
    pub children: Vec<ArcRefCell<InlineLevelBox>>,
}

/// https://www.w3.org/TR/css-display-3/#css-text-run
#[derive(Debug, Serialize)]
pub(crate) struct TextRun {
    pub tag: Tag,
    #[serde(skip_serializing)]
    pub parent_style: Arc<ComputedValues>,
    pub text: String,
}

struct InlineNestingLevelState<'box_tree> {
    remaining_boxes: InlineBoxChildIter<'box_tree>,
    fragments_so_far: Vec<Fragment>,
    inline_start: Length,
    max_block_size_of_fragments_so_far: Length,
    positioning_context: Option<PositioningContext>,
    /// Indicates whether this nesting level have text decorations in effect.
    /// From https://drafts.csswg.org/css-text-decor/#line-decoration
    // "When specified on or propagated to a block container that establishes
    //  an IFC..."
    text_decoration_line: TextDecorationLine,
}

struct PartialInlineBoxFragment<'box_tree> {
    tag: Tag,
    style: Arc<ComputedValues>,
    start_corner: Vec2<Length>,
    padding: Sides<Length>,
    border: Sides<Length>,
    margin: Sides<Length>,
    last_box_tree_fragment: bool,
    parent_nesting_level: InlineNestingLevelState<'box_tree>,
}

struct InlineFormattingContextState<'box_tree, 'a, 'b> {
    positioning_context: &'a mut PositioningContext,
    containing_block: &'b ContainingBlock<'b>,
    lines: Lines,
    inline_position: Length,
    partial_inline_boxes_stack: Vec<PartialInlineBoxFragment<'box_tree>>,
    current_nesting_level: InlineNestingLevelState<'box_tree>,
    float_context: Option<&'a mut FloatContext>,
}

impl<'box_tree, 'a, 'b> InlineFormattingContextState<'box_tree, 'a, 'b> {
    fn push_hoisted_box_to_positioning_context(
        &mut self,
        hoisted_box: HoistedAbsolutelyPositionedBox,
    ) {
        if let Some(context) = self.current_nesting_level.positioning_context.as_mut() {
            context.push(hoisted_box);
            return;
        }

        for nesting_level in self.partial_inline_boxes_stack.iter_mut().rev() {
            if let Some(context) = nesting_level
                .parent_nesting_level
                .positioning_context
                .as_mut()
            {
                context.push(hoisted_box);
                return;
            }
        }

        self.positioning_context.push(hoisted_box);
    }
}

struct Lines {
    // One anonymous fragment per line
    fragments: Vec<Fragment>,
    next_line_block_position: Length,
}

impl InlineFormattingContext {
    pub(super) fn new(text_decoration_line: TextDecorationLine) -> InlineFormattingContext {
        InlineFormattingContext {
            inline_level_boxes: Default::default(),
            text_decoration_line,
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
        struct Computation<'a> {
            layout_context: &'a LayoutContext<'a>,
            containing_block_writing_mode: WritingMode,
            paragraph: ContentSizes,
            current_line: ContentSizes,
            current_line_percentages: Percentage,
        }
        impl Computation<'_> {
            fn traverse(&mut self, inline_level_boxes: &[ArcRefCell<InlineLevelBox>]) {
                for inline_level_box in inline_level_boxes {
                    match &mut *inline_level_box.borrow_mut() {
                        InlineLevelBox::InlineBox(inline_box) => {
                            let padding =
                                inline_box.style.padding(self.containing_block_writing_mode);
                            let border = inline_box
                                .style
                                .border_width(self.containing_block_writing_mode);
                            let margin =
                                inline_box.style.margin(self.containing_block_writing_mode);
                            macro_rules! add {
                                ($condition: ident, $side: ident) => {
                                    if inline_box.$condition {
                                        self.add_lengthpercentage(padding.$side);
                                        self.add_length(border.$side);
                                        if let Some(lp) = margin.$side.non_auto() {
                                            self.add_lengthpercentage(lp)
                                        }
                                    }
                                };
                            }

                            add!(first_fragment, inline_start);
                            self.traverse(&inline_box.children);
                            add!(last_fragment, inline_end);
                        },
                        InlineLevelBox::TextRun(text_run) => {
                            let BreakAndShapeResult {
                                runs,
                                break_at_start,
                                ..
                            } = text_run.break_and_shape(self.layout_context);
                            if break_at_start {
                                self.line_break_opportunity()
                            }
                            for run in &runs {
                                let advance = Length::from(run.glyph_store.total_advance());
                                if run.glyph_store.is_whitespace() {
                                    self.line_break_opportunity()
                                } else {
                                    self.current_line.min_content += advance
                                }
                                self.current_line.max_content += advance
                            }
                        },
                        InlineLevelBox::Atomic(atomic) => {
                            let (outer, pc) = atomic.outer_inline_content_sizes_and_percentages(
                                self.layout_context,
                                self.containing_block_writing_mode,
                            );
                            self.current_line.min_content += outer.min_content;
                            self.current_line.max_content += outer.max_content;
                            self.current_line_percentages += pc;
                        },
                        InlineLevelBox::OutOfFlowFloatBox(_) |
                        InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => {},
                    }
                }
            }

            fn add_lengthpercentage(&mut self, lp: &LengthPercentage) {
                if let Some(l) = lp.to_length() {
                    self.add_length(l);
                }
                if let Some(p) = lp.to_percentage() {
                    self.current_line_percentages += p;
                }
            }

            fn add_length(&mut self, l: Length) {
                self.current_line.min_content += l;
                self.current_line.max_content += l;
            }

            fn line_break_opportunity(&mut self) {
                self.paragraph
                    .min_content
                    .max_assign(take(&mut self.current_line.min_content));
            }

            fn forced_line_break(&mut self) {
                self.line_break_opportunity();
                self.current_line
                    .adjust_for_pbm_percentages(take(&mut self.current_line_percentages));
                self.paragraph
                    .max_content
                    .max_assign(take(&mut self.current_line.max_content));
            }
        }
        fn take<T: Zero>(x: &mut T) -> T {
            std::mem::replace(x, T::zero())
        }
        let mut computation = Computation {
            layout_context,
            containing_block_writing_mode,
            paragraph: ContentSizes::zero(),
            current_line: ContentSizes::zero(),
            current_line_percentages: Percentage::zero(),
        };
        computation.traverse(&self.inline_level_boxes);
        computation.forced_line_break();
        computation.paragraph
    }

    pub(super) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        float_context: Option<&mut FloatContext>,
    ) -> FlowLayout {
        let mut ifc = InlineFormattingContextState {
            positioning_context,
            containing_block,
            partial_inline_boxes_stack: Vec::new(),
            lines: Lines {
                fragments: Vec::new(),
                next_line_block_position: Length::zero(),
            },
            inline_position: Length::zero(),
            current_nesting_level: InlineNestingLevelState {
                remaining_boxes: InlineBoxChildIter::from_formatting_context(self),
                fragments_so_far: Vec::with_capacity(self.inline_level_boxes.len()),
                inline_start: Length::zero(),
                max_block_size_of_fragments_so_far: Length::zero(),
                positioning_context: None,
                text_decoration_line: self.text_decoration_line,
            },
            float_context,
        };

        loop {
            if let Some(child) = ifc.current_nesting_level.remaining_boxes.next() {
                match &mut *child.borrow_mut() {
                    InlineLevelBox::InlineBox(inline) => {
                        let partial = inline.start_layout(child.clone(), &mut ifc);
                        ifc.partial_inline_boxes_stack.push(partial)
                    },
                    InlineLevelBox::TextRun(run) => run.layout(layout_context, &mut ifc),
                    InlineLevelBox::Atomic(a) => layout_atomic(layout_context, &mut ifc, a),
                    InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                        let style = AtomicRef::map(box_.borrow(), |box_| box_.context.style());
                        let initial_start_corner =
                            match Display::from(style.get_box().original_display) {
                                Display::GeneratingBox(DisplayGeneratingBox::OutsideInside {
                                    outside,
                                    inside: _,
                                }) => Vec2 {
                                    inline: match outside {
                                        DisplayOutside::Inline => ifc.inline_position,
                                        DisplayOutside::Block => Length::zero(),
                                    },
                                    block: ifc.lines.next_line_block_position,
                                },
                                Display::Contents => {
                                    panic!("display:contents does not generate an abspos box")
                                },
                                Display::None => {
                                    panic!("display:none does not generate an abspos box")
                                },
                            };
                        let hoisted_box = AbsolutelyPositionedBox::to_hoisted(
                            box_.clone(),
                            initial_start_corner,
                            tree_rank,
                            ifc.containing_block,
                        );
                        let hoisted_fragment = hoisted_box.fragment.clone();
                        ifc.push_hoisted_box_to_positioning_context(hoisted_box);
                        ifc.current_nesting_level.fragments_so_far.push(
                            Fragment::AbsoluteOrFixedPositioned(
                                AbsoluteOrFixedPositionedFragment {
                                    hoisted_fragment,
                                    position: style.clone_position(),
                                },
                            ),
                        );
                    },
                    InlineLevelBox::OutOfFlowFloatBox(box_) => {
                        ifc.current_nesting_level
                            .fragments_so_far
                            .push(Fragment::Float(box_.layout(
                                layout_context,
                                ifc.positioning_context,
                                containing_block,
                                ifc.float_context.as_mut().map(|c| &mut **c),
                            )));
                    },
                }
            } else
            // Reached the end of ifc.remaining_boxes
            if let Some(mut partial) = ifc.partial_inline_boxes_stack.pop() {
                partial.finish_layout(
                    layout_context,
                    &mut ifc.current_nesting_level,
                    &mut ifc.inline_position,
                    false,
                );
                ifc.current_nesting_level = partial.parent_nesting_level
            } else {
                ifc.lines.finish_line(
                    &mut ifc.current_nesting_level,
                    containing_block,
                    ifc.float_context,
                    ifc.inline_position,
                );
                return FlowLayout {
                    fragments: ifc.lines.fragments,
                    content_block_size: ifc.lines.next_line_block_position,
                    collapsible_margins_in_children: CollapsedBlockMargins::zero(),
                };
            }
        }
    }
}

impl Lines {
    fn finish_line(
        &mut self,
        top_nesting_level: &mut InlineNestingLevelState,
        containing_block: &ContainingBlock,
        mut float_context: Option<&mut FloatContext>,
        line_content_inline_size: Length,
    ) {
        let mut line_contents = std::mem::take(&mut top_nesting_level.fragments_so_far);
        let line_block_size = std::mem::replace(
            &mut top_nesting_level.max_block_size_of_fragments_so_far,
            Length::zero(),
        );
        enum TextAlign {
            Start,
            Center,
            End,
        }
        let line_left_is_inline_start = containing_block
            .style
            .writing_mode
            .line_left_is_inline_start();
        let text_align = match containing_block.style.clone_text_align() {
            TextAlignKeyword::Start => TextAlign::Start,
            TextAlignKeyword::Center => TextAlign::Center,
            TextAlignKeyword::End => TextAlign::End,
            TextAlignKeyword::Left => {
                if line_left_is_inline_start {
                    TextAlign::Start
                } else {
                    TextAlign::End
                }
            },
            TextAlignKeyword::Right => {
                if line_left_is_inline_start {
                    TextAlign::End
                } else {
                    TextAlign::Start
                }
            },
        };
        let move_by = match text_align {
            TextAlign::Start => Length::zero(),
            TextAlign::Center => (containing_block.inline_size - line_content_inline_size) / 2.,
            TextAlign::End => containing_block.inline_size - line_content_inline_size,
        };
        if move_by > Length::zero() {
            for fragment in &mut line_contents {
                fragment.offset_inline(&move_by);
            }
        }
        let start_corner = Vec2 {
            inline: Length::zero(),
            block: self.next_line_block_position,
        };
        let size = Vec2 {
            inline: containing_block.inline_size,
            block: line_block_size,
        };
        self.next_line_block_position += size.block;

        if let Some(ref mut float_context) = float_context {
            // NB(pcwalton): This sets the "containing block position" of the float context to the
            // position of the line box, not that of its real containing block, because inline
            // fragment positions are relative to the line boxes. I think this is ultimately
            // harmless, but it is a bit questionable.
            float_context.containing_block_position += size.block;
            float_context.lower_ceiling(float_context.containing_block_position);
        }

        self.fragments
            .push(Fragment::Anonymous(AnonymousFragment::new(
                Rect { start_corner, size },
                line_contents,
                containing_block.style.writing_mode,
            )))
    }
}

impl InlineBox {
    fn start_layout<'box_tree>(
        &self,
        this_inline_level_box: ArcRefCell<InlineLevelBox>,
        ifc: &mut InlineFormattingContextState<'box_tree, '_, '_>,
    ) -> PartialInlineBoxFragment<'box_tree> {
        let style = self.style.clone();
        let pbm = style.padding_border_margin(&ifc.containing_block);
        let mut padding = pbm.padding;
        let mut border = pbm.border;
        let mut margin = pbm.margin.auto_is(Length::zero);

        if self.first_fragment {
            ifc.inline_position += padding.inline_start + border.inline_start + margin.inline_start;
        } else {
            padding.inline_start = Length::zero();
            border.inline_start = Length::zero();
            margin.inline_start = Length::zero();
        }
        let mut start_corner = Vec2 {
            block: padding.block_start + border.block_start + margin.block_start,
            inline: ifc.inline_position - ifc.current_nesting_level.inline_start,
        };
        if style.clone_position().is_relative() {
            start_corner += &relative_adjustement(&style, ifc.containing_block)
        }
        let positioning_context = PositioningContext::new_for_style(&style);
        let text_decoration_line =
            ifc.current_nesting_level.text_decoration_line | style.clone_text_decoration_line();
        PartialInlineBoxFragment {
            tag: self.tag,
            style,
            start_corner,
            padding,
            border,
            margin,
            last_box_tree_fragment: self.last_fragment,
            parent_nesting_level: std::mem::replace(
                &mut ifc.current_nesting_level,
                InlineNestingLevelState {
                    remaining_boxes: InlineBoxChildIter::from_inline_level_box(
                        this_inline_level_box,
                    ),
                    fragments_so_far: Vec::with_capacity(self.children.len()),
                    inline_start: ifc.inline_position,
                    max_block_size_of_fragments_so_far: Length::zero(),
                    positioning_context,
                    text_decoration_line: text_decoration_line,
                },
            ),
        }
    }
}

impl<'box_tree> PartialInlineBoxFragment<'box_tree> {
    fn finish_layout(
        &mut self,
        layout_context: &LayoutContext,
        nesting_level: &mut InlineNestingLevelState,
        inline_position: &mut Length,
        at_line_break: bool,
    ) {
        let content_rect = Rect {
            size: Vec2 {
                inline: *inline_position - self.start_corner.inline,
                block: nesting_level.max_block_size_of_fragments_so_far,
            },
            start_corner: self.start_corner.clone(),
        };

        let mut fragment = BoxFragment::new(
            self.tag,
            self.style.clone(),
            std::mem::take(&mut nesting_level.fragments_so_far),
            content_rect,
            self.padding.clone(),
            self.border.clone(),
            self.margin.clone(),
            CollapsedBlockMargins::zero(),
        );
        let last_fragment = self.last_box_tree_fragment && !at_line_break;
        if last_fragment {
            *inline_position += fragment.padding.inline_end +
                fragment.border.inline_end +
                fragment.margin.inline_end;
        } else {
            fragment.padding.inline_end = Length::zero();
            fragment.border.inline_end = Length::zero();
            fragment.margin.inline_end = Length::zero();
        }
        self.parent_nesting_level
            .max_block_size_of_fragments_so_far
            .max_assign(
                fragment.content_rect.size.block +
                    fragment.padding.block_sum() +
                    fragment.border.block_sum() +
                    fragment.margin.block_sum(),
            );

        if let Some(context) = nesting_level.positioning_context.as_mut() {
            context.layout_collected_children(layout_context, &mut fragment);
        }

        self.parent_nesting_level
            .fragments_so_far
            .push(Fragment::Box(fragment));
    }
}

fn layout_atomic(
    layout_context: &LayoutContext,
    ifc: &mut InlineFormattingContextState,
    atomic: &mut IndependentFormattingContext,
) {
    let style = atomic.style();
    let pbm = style.padding_border_margin(&ifc.containing_block);
    let margin = pbm.margin.auto_is(Length::zero);
    let pbm_sums = &(&pbm.padding + &pbm.border) + &margin;
    ifc.inline_position += pbm_sums.inline_start;
    let mut start_corner = Vec2 {
        block: pbm_sums.block_start,
        inline: ifc.inline_position - ifc.current_nesting_level.inline_start,
    };
    if style.clone_position().is_relative() {
        start_corner += &relative_adjustement(&style, ifc.containing_block)
    }

    let fragment = match atomic {
        IndependentFormattingContext::Replaced(replaced) => {
            let size = replaced.contents.used_size_as_if_inline_element(
                ifc.containing_block,
                &replaced.style,
                &pbm,
            );
            let fragments = replaced
                .contents
                .make_fragments(&replaced.style, size.clone());
            let content_rect = Rect { start_corner, size };
            BoxFragment::new(
                replaced.tag,
                replaced.style.clone(),
                fragments,
                content_rect,
                pbm.padding,
                pbm.border,
                margin,
                CollapsedBlockMargins::zero(),
            )
        },
        IndependentFormattingContext::NonReplaced(non_replaced) => {
            let box_size = non_replaced
                .style
                .content_box_size(&ifc.containing_block, &pbm);
            let max_box_size = non_replaced
                .style
                .content_max_box_size(&ifc.containing_block, &pbm);
            let min_box_size = non_replaced
                .style
                .content_min_box_size(&ifc.containing_block, &pbm)
                .auto_is(Length::zero);

            // https://drafts.csswg.org/css2/visudet.html#inlineblock-width
            let tentative_inline_size = box_size.inline.auto_is(|| {
                let available_size = ifc.containing_block.inline_size - pbm_sums.inline_sum();
                non_replaced
                    .inline_content_sizes(layout_context)
                    .shrink_to_fit(available_size)
            });

            // https://drafts.csswg.org/css2/visudet.html#min-max-widths
            // In this case “applying the rules above again” with a non-auto inline-size
            // always results in that size.
            let inline_size = tentative_inline_size
                .clamp_between_extremums(min_box_size.inline, max_box_size.inline);

            let containing_block_for_children = ContainingBlock {
                inline_size,
                block_size: box_size.block,
                style: &non_replaced.style,
            };
            assert_eq!(
                ifc.containing_block.style.writing_mode,
                containing_block_for_children.style.writing_mode,
                "Mixed writing modes are not supported yet"
            );
            // FIXME is this correct?
            let dummy_tree_rank = 0;
            // FIXME: Do we need to call `adjust_static_positions` somewhere near here?
            let independent_layout = non_replaced.layout(
                layout_context,
                ifc.positioning_context,
                &containing_block_for_children,
                dummy_tree_rank,
            );

            // https://drafts.csswg.org/css2/visudet.html#block-root-margin
            let tentative_block_size = box_size
                .block
                .auto_is(|| independent_layout.content_block_size);

            // https://drafts.csswg.org/css2/visudet.html#min-max-heights
            // In this case “applying the rules above again” with a non-auto block-size
            // always results in that size.
            let block_size = tentative_block_size
                .clamp_between_extremums(min_box_size.block, max_box_size.block);

            let content_rect = Rect {
                start_corner,
                size: Vec2 {
                    block: block_size,
                    inline: inline_size,
                },
            };
            BoxFragment::new(
                non_replaced.tag,
                non_replaced.style.clone(),
                independent_layout.fragments,
                content_rect,
                pbm.padding,
                pbm.border,
                margin,
                CollapsedBlockMargins::zero(),
            )
        },
    };

    ifc.inline_position += pbm_sums.inline_end + fragment.content_rect.size.inline;
    ifc.current_nesting_level
        .max_block_size_of_fragments_so_far
        .max_assign(pbm_sums.block_sum() + fragment.content_rect.size.block);
    ifc.current_nesting_level
        .fragments_so_far
        .push(Fragment::Box(fragment));
}

struct BreakAndShapeResult {
    font_metrics: FontMetrics,
    font_key: FontInstanceKey,
    runs: Vec<GlyphRun>,
    break_at_start: bool,
}

impl TextRun {
    fn break_and_shape(&self, layout_context: &LayoutContext) -> BreakAndShapeResult {
        use gfx::font::ShapingFlags;
        use style::computed_values::text_rendering::T as TextRendering;
        use style::computed_values::word_break::T as WordBreak;

        let font_style = self.parent_style.clone_font();
        let inherited_text_style = self.parent_style.get_inherited_text();
        let letter_spacing = if inherited_text_style.letter_spacing.0.px() != 0. {
            Some(app_units::Au::from(inherited_text_style.letter_spacing.0))
        } else {
            None
        };

        let mut flags = ShapingFlags::empty();
        if letter_spacing.is_some() {
            flags.insert(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG);
        }
        if inherited_text_style.text_rendering == TextRendering::Optimizespeed {
            flags.insert(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG);
            flags.insert(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG)
        }
        if inherited_text_style.word_break == WordBreak::KeepAll {
            flags.insert(ShapingFlags::KEEP_ALL_FLAG);
        }

        crate::context::with_thread_local_font_context(layout_context, |font_context| {
            let font_group = font_context.font_group(font_style);
            let font = font_group
                .borrow_mut()
                .first(font_context)
                .expect("could not find font");
            let mut font = font.borrow_mut();

            let word_spacing = &inherited_text_style.word_spacing;
            let word_spacing = word_spacing
                .to_length()
                .map(|l| l.into())
                .unwrap_or_else(|| {
                    let space_width = font
                        .glyph_index(' ')
                        .map(|glyph_id| font.glyph_h_advance(glyph_id))
                        .unwrap_or(gfx::font::LAST_RESORT_GLYPH_ADVANCE);
                    word_spacing.to_used_value(Au::from_f64_px(space_width))
                });

            let shaping_options = gfx::font::ShapingOptions {
                letter_spacing,
                word_spacing,
                script: unicode_script::Script::Common,
                flags,
            };

            let (runs, break_at_start) = gfx::text::text_run::TextRun::break_and_shape(
                &mut font,
                &self.text,
                &shaping_options,
                &mut None,
            );

            BreakAndShapeResult {
                font_metrics: (&font.metrics).into(),
                font_key: font.font_key,
                runs,
                break_at_start,
            }
        })
    }

    fn layout(&self, layout_context: &LayoutContext, ifc: &mut InlineFormattingContextState) {
        use style::values::generics::text::LineHeight;

        let BreakAndShapeResult {
            font_metrics,
            font_key,
            runs,
            break_at_start: _,
        } = self.break_and_shape(layout_context);
        let font_size = self.parent_style.get_font().font_size.size.0;
        let mut runs = runs.iter();
        loop {
            let mut glyphs = vec![];
            let mut advance_width = Length::zero();
            let mut last_break_opportunity = None;
            let mut force_line_break = false;
            // Fit as many glyphs within a single line as possible.
            loop {
                let next = runs.next();
                // If there are no more text runs we still need to check if the last
                // run was a forced line break
                if next
                    .as_ref()
                    .map_or(true, |run| run.glyph_store.is_whitespace())
                {
                    // If this run exceeds the bounds of the containing block, then
                    // we need to attempt to break the line.
                    if advance_width > ifc.containing_block.inline_size - ifc.inline_position {
                        // Reset the text run iterator to the last whitespace if possible,
                        // to attempt to re-layout the most recent glyphs on a new line.
                        if let Some((len, width, iter)) = last_break_opportunity.take() {
                            glyphs.truncate(len);
                            advance_width = width;
                            runs = iter;
                        }
                        break;
                    }
                }
                if let Some(run) = next {
                    if run.glyph_store.is_whitespace() {
                        last_break_opportunity = Some((glyphs.len(), advance_width, runs.clone()));
                        // If this whitespace ends with a newline, we need to check if
                        // it's meaningful within the current style. If so, we force
                        // a line break immediately.
                        let last_byte = self.text.as_bytes().get(run.range.end().to_usize() - 1);
                        if last_byte == Some(&b'\n') &&
                            self.parent_style
                                .get_inherited_text()
                                .white_space
                                .preserve_newlines()
                        {
                            force_line_break = true;
                            break;
                        }
                    }
                    glyphs.push(run.glyph_store.clone());
                    advance_width += Length::from(run.glyph_store.total_advance());
                } else {
                    // No more runs, so we can end the line.
                    break;
                }
            }
            let line_height = match self.parent_style.get_inherited_text().line_height {
                LineHeight::Normal => font_metrics.line_gap,
                LineHeight::Number(n) => font_size * n.0,
                LineHeight::Length(l) => l.0,
            };
            let rect = Rect {
                start_corner: Vec2 {
                    block: Length::zero(),
                    inline: ifc.inline_position - ifc.current_nesting_level.inline_start,
                },
                size: Vec2 {
                    block: line_height,
                    inline: advance_width,
                },
            };
            ifc.inline_position += advance_width;
            ifc.current_nesting_level
                .max_block_size_of_fragments_so_far
                .max_assign(line_height);
            ifc.current_nesting_level
                .fragments_so_far
                .push(Fragment::Text(TextFragment {
                    tag: self.tag,
                    debug_id: DebugId::new(),
                    parent_style: self.parent_style.clone(),
                    rect,
                    font_metrics,
                    font_key,
                    glyphs,
                    text_decoration_line: ifc.current_nesting_level.text_decoration_line,
                }));
            // If this line is being broken because of a trailing newline, we can't ignore it.
            if runs.as_slice().is_empty() && !force_line_break {
                break;
            } else {
                // New line
                ifc.current_nesting_level.inline_start = Length::zero();
                let mut nesting_level = &mut ifc.current_nesting_level;
                for partial in ifc.partial_inline_boxes_stack.iter_mut().rev() {
                    partial.finish_layout(
                        layout_context,
                        nesting_level,
                        &mut ifc.inline_position,
                        true,
                    );
                    partial.start_corner.inline = Length::zero();
                    partial.padding.inline_start = Length::zero();
                    partial.border.inline_start = Length::zero();
                    partial.margin.inline_start = Length::zero();
                    partial.parent_nesting_level.inline_start = Length::zero();
                    nesting_level = &mut partial.parent_nesting_level;
                }
                ifc.lines.finish_line(
                    nesting_level,
                    ifc.containing_block,
                    ifc.float_context.as_mut().map(|c| &mut **c),
                    ifc.inline_position,
                );
                ifc.inline_position = Length::zero();
            }
        }
    }
}

enum InlineBoxChildIter<'box_tree> {
    InlineFormattingContext(std::slice::Iter<'box_tree, ArcRefCell<InlineLevelBox>>),
    InlineBox {
        inline_level_box: ArcRefCell<InlineLevelBox>,
        child_index: usize,
    },
}

impl<'box_tree> InlineBoxChildIter<'box_tree> {
    fn from_formatting_context(
        inline_formatting_context: &'box_tree InlineFormattingContext,
    ) -> InlineBoxChildIter<'box_tree> {
        InlineBoxChildIter::InlineFormattingContext(
            inline_formatting_context.inline_level_boxes.iter(),
        )
    }

    fn from_inline_level_box(
        inline_level_box: ArcRefCell<InlineLevelBox>,
    ) -> InlineBoxChildIter<'box_tree> {
        InlineBoxChildIter::InlineBox {
            inline_level_box,
            child_index: 0,
        }
    }
}

impl<'box_tree> Iterator for InlineBoxChildIter<'box_tree> {
    type Item = ArcRefCell<InlineLevelBox>;
    fn next(&mut self) -> Option<ArcRefCell<InlineLevelBox>> {
        match *self {
            InlineBoxChildIter::InlineFormattingContext(ref mut iter) => iter.next().cloned(),
            InlineBoxChildIter::InlineBox {
                ref inline_level_box,
                ref mut child_index,
            } => match *inline_level_box.borrow() {
                InlineLevelBox::InlineBox(ref inline_box) => {
                    if *child_index >= inline_box.children.len() {
                        return None;
                    }

                    let kid = inline_box.children[*child_index].clone();
                    *child_index += 1;
                    Some(kid)
                },
                _ => unreachable!(),
            },
        }
    }
}
