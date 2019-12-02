/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::flow::float::FloatBox;
use crate::flow::FlowLayout;
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::CollapsedBlockMargins;
use crate::fragments::{AnonymousFragment, BoxFragment, Fragment, TextFragment};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::positioned::{AbsolutelyPositionedBox, AbsolutelyPositionedFragment};
use crate::sizing::{outer_inline_content_sizes, ContentSizes};
use crate::style_ext::{ComputedValuesExt, Display, DisplayGeneratingBox, DisplayOutside};
use crate::{relative_adjustement, ContainingBlock};
use app_units::Au;
use gfx::text::text_run::GlyphRun;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::values::computed::{Length, Percentage};
use style::Zero;
use webrender_api::FontInstanceKey;

#[derive(Debug, Default)]
pub(crate) struct InlineFormattingContext {
    pub(super) inline_level_boxes: Vec<Arc<InlineLevelBox>>,
}

#[derive(Debug)]
pub(crate) enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(TextRun),
    OutOfFlowAbsolutelyPositionedBox(AbsolutelyPositionedBox),
    OutOfFlowFloatBox(FloatBox),
    Atomic(IndependentFormattingContext),
}

#[derive(Debug)]
pub(crate) struct InlineBox {
    pub style: Arc<ComputedValues>,
    pub first_fragment: bool,
    pub last_fragment: bool,
    pub children: Vec<Arc<InlineLevelBox>>,
}

/// https://www.w3.org/TR/css-display-3/#css-text-run
#[derive(Debug)]
pub(crate) struct TextRun {
    pub parent_style: Arc<ComputedValues>,
    pub text: String,
}

struct InlineNestingLevelState<'box_tree> {
    remaining_boxes: std::slice::Iter<'box_tree, Arc<InlineLevelBox>>,
    fragments_so_far: Vec<Fragment>,
    inline_start: Length,
    max_block_size_of_fragments_so_far: Length,
}

struct PartialInlineBoxFragment<'box_tree> {
    style: Arc<ComputedValues>,
    start_corner: Vec2<Length>,
    padding: Sides<Length>,
    border: Sides<Length>,
    margin: Sides<Length>,
    last_box_tree_fragment: bool,
    parent_nesting_level: InlineNestingLevelState<'box_tree>,
}

struct InlineFormattingContextState<'box_tree, 'cb> {
    containing_block: &'cb ContainingBlock,
    line_boxes: LinesBoxes,
    inline_position: Length,
    partial_inline_boxes_stack: Vec<PartialInlineBoxFragment<'box_tree>>,
    current_nesting_level: InlineNestingLevelState<'box_tree>,
}

struct LinesBoxes {
    boxes: Vec<Fragment>,
    next_line_block_position: Length,
}

impl InlineFormattingContext {
    // This works on an already-constructed `InlineFormattingContext`,
    // Which would have to change if/when
    // `BlockContainer::construct` parallelize their construction.
    #[allow(unused)]
    pub(super) fn content_sizes(&self, layout_context: &LayoutContext) -> ContentSizes {
        struct Computation {
            paragraph: ContentSizes,
            current_line: ContentSizes,
            current_line_percentages: Percentage,
        }
        impl Computation {
            fn traverse(
                &mut self,
                layout_context: &LayoutContext,
                inline_level_boxes: &[Arc<InlineLevelBox>],
            ) {
                for inline_level_box in inline_level_boxes {
                    match &**inline_level_box {
                        InlineLevelBox::InlineBox(inline_box) => {
                            let padding = inline_box.style.padding();
                            let border = inline_box.style.border_width();
                            let margin = inline_box.style.margin();
                            macro_rules! add_length {
                                ($x: expr) => {{
                                    let length = $x;
                                    self.current_line.min_content += length;
                                    self.current_line.max_content += length;
                                }};
                            }
                            macro_rules! add_lengthpercentage {
                                ($x: expr) => {{
                                    add_length!($x.length_component());
                                    self.current_line_percentages += $x.percentage_component();
                                }};
                            }
                            macro_rules! add {
                                ($condition: ident, $side: ident) => {
                                    if inline_box.$condition {
                                        add_lengthpercentage!(padding.$side);
                                        add_length!(border.$side);
                                        margin.$side.non_auto().map(|x| add_lengthpercentage!(x));
                                    }
                                };
                            }

                            add!(first_fragment, inline_start);
                            self.traverse(layout_context, &inline_box.children);
                            add!(last_fragment, inline_end);
                        },
                        InlineLevelBox::TextRun(text_run) => {
                            let (_, _, _, runs, break_at_start) =
                                text_run.break_and_shape(layout_context);
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
                            let inner = || {
                                // atomic
                                // .inline_content_sizes
                                // .as_ref()
                                // .expect("Accessing content size that was not requested")
                                // .clone()
                                todo!()
                            };
                            let (outer, pc) = outer_inline_content_sizes(&atomic.style, &inner);
                            self.current_line.min_content += outer.min_content;
                            self.current_line.max_content += outer.max_content;
                            self.current_line_percentages += pc;
                        },
                        InlineLevelBox::OutOfFlowFloatBox(_) |
                        InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => {},
                    }
                }
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
            paragraph: ContentSizes::zero(),
            current_line: ContentSizes::zero(),
            current_line_percentages: Percentage::zero(),
        };
        computation.traverse(layout_context, &self.inline_level_boxes);
        computation.forced_line_break();
        computation.paragraph
    }

    pub(super) fn layout<'a>(
        &'a self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    ) -> FlowLayout {
        let mut ifc = InlineFormattingContextState {
            containing_block,
            partial_inline_boxes_stack: Vec::new(),
            line_boxes: LinesBoxes {
                boxes: Vec::new(),
                next_line_block_position: Length::zero(),
            },
            inline_position: Length::zero(),
            current_nesting_level: InlineNestingLevelState {
                remaining_boxes: self.inline_level_boxes.iter(),
                fragments_so_far: Vec::with_capacity(self.inline_level_boxes.len()),
                inline_start: Length::zero(),
                max_block_size_of_fragments_so_far: Length::zero(),
            },
        };
        loop {
            if let Some(child) = ifc.current_nesting_level.remaining_boxes.next() {
                match &**child {
                    InlineLevelBox::InlineBox(inline) => {
                        let partial = inline.start_layout(&mut ifc);
                        ifc.partial_inline_boxes_stack.push(partial)
                    },
                    InlineLevelBox::TextRun(run) => run.layout(layout_context, &mut ifc),
                    InlineLevelBox::Atomic(_independent) => {
                        // TODO
                        continue;
                    },
                    InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                        let initial_start_corner =
                            match Display::from(box_.contents.style.get_box().original_display) {
                                Display::GeneratingBox(DisplayGeneratingBox::OutsideInside {
                                    outside,
                                    inside: _,
                                }) => Vec2 {
                                    inline: match outside {
                                        DisplayOutside::Inline => ifc.inline_position,
                                        DisplayOutside::Block => Length::zero(),
                                    },
                                    block: ifc.line_boxes.next_line_block_position,
                                },
                                Display::Contents => {
                                    panic!("display:contents does not generate an abspos box")
                                },
                                Display::None => {
                                    panic!("display:none does not generate an abspos box")
                                },
                            };
                        absolutely_positioned_fragments
                            .push(box_.layout(initial_start_corner, tree_rank));
                    },
                    InlineLevelBox::OutOfFlowFloatBox(_box_) => {
                        // TODO
                        continue;
                    },
                }
            } else
            // Reached the end of ifc.remaining_boxes
            if let Some(mut partial) = ifc.partial_inline_boxes_stack.pop() {
                partial.finish_layout(
                    &mut ifc.current_nesting_level,
                    &mut ifc.inline_position,
                    false,
                );
                ifc.current_nesting_level = partial.parent_nesting_level
            } else {
                ifc.line_boxes
                    .finish_line(&mut ifc.current_nesting_level, containing_block);
                return FlowLayout {
                    fragments: ifc.line_boxes.boxes,
                    content_block_size: ifc.line_boxes.next_line_block_position,
                    collapsible_margins_in_children: CollapsedBlockMargins::zero(),
                };
            }
        }
    }
}

impl LinesBoxes {
    fn finish_line(
        &mut self,
        top_nesting_level: &mut InlineNestingLevelState,
        containing_block: &ContainingBlock,
    ) {
        let start_corner = Vec2 {
            inline: Length::zero(),
            block: self.next_line_block_position,
        };
        let size = Vec2 {
            inline: containing_block.inline_size,
            block: std::mem::replace(
                &mut top_nesting_level.max_block_size_of_fragments_so_far,
                Length::zero(),
            ),
        };
        self.next_line_block_position += size.block;
        self.boxes.push(Fragment::Anonymous(AnonymousFragment {
            children: std::mem::take(&mut top_nesting_level.fragments_so_far),
            rect: Rect { start_corner, size },
            mode: containing_block.mode,
        }))
    }
}

impl InlineBox {
    fn start_layout<'box_tree>(
        &'box_tree self,
        ifc: &mut InlineFormattingContextState<'box_tree, '_>,
    ) -> PartialInlineBoxFragment<'box_tree> {
        let style = self.style.clone();
        let cbis = ifc.containing_block.inline_size;
        let mut padding = style.padding().percentages_relative_to(cbis);
        let mut border = style.border_width();
        let mut margin = style
            .margin()
            .percentages_relative_to(cbis)
            .auto_is(Length::zero);
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
        start_corner += &relative_adjustement(
            &style,
            ifc.containing_block.inline_size,
            ifc.containing_block.block_size,
        );
        PartialInlineBoxFragment {
            style,
            start_corner,
            padding,
            border,
            margin,
            last_box_tree_fragment: self.last_fragment,
            parent_nesting_level: std::mem::replace(
                &mut ifc.current_nesting_level,
                InlineNestingLevelState {
                    remaining_boxes: self.children.iter(),
                    fragments_so_far: Vec::with_capacity(self.children.len()),
                    inline_start: ifc.inline_position,
                    max_block_size_of_fragments_so_far: Length::zero(),
                },
            ),
        }
    }
}

impl<'box_tree> PartialInlineBoxFragment<'box_tree> {
    fn finish_layout(
        &mut self,
        nesting_level: &mut InlineNestingLevelState,
        inline_position: &mut Length,
        at_line_break: bool,
    ) {
        let mut fragment = BoxFragment {
            style: self.style.clone(),
            children: std::mem::take(&mut nesting_level.fragments_so_far),
            content_rect: Rect {
                size: Vec2 {
                    inline: *inline_position - self.start_corner.inline,
                    block: nesting_level.max_block_size_of_fragments_so_far,
                },
                start_corner: self.start_corner.clone(),
            },
            padding: self.padding.clone(),
            border: self.border.clone(),
            margin: self.margin.clone(),
            block_margins_collapsed_with_children: CollapsedBlockMargins::zero(),
        };
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
        self.parent_nesting_level
            .fragments_so_far
            .push(Fragment::Box(fragment));
    }
}

impl TextRun {
    fn break_and_shape(
        &self,
        layout_context: &LayoutContext,
    ) -> (Au, Au, FontInstanceKey, Vec<GlyphRun>, bool) {
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

        let shaping_options = gfx::font::ShapingOptions {
            letter_spacing,
            word_spacing: inherited_text_style.word_spacing.to_hash_key(),
            script: unicode_script::Script::Common,
            flags,
        };

        crate::context::with_thread_local_font_context(layout_context, |font_context| {
            let font_group = font_context.font_group(font_style);
            let font = font_group
                .borrow_mut()
                .first(font_context)
                .expect("could not find font");
            let mut font = font.borrow_mut();

            let (runs, break_at_start) = gfx::text::text_run::TextRun::break_and_shape(
                &mut font,
                &self.text,
                &shaping_options,
                &mut None,
            );

            (
                font.metrics.ascent,
                font.metrics.line_gap,
                font.font_key,
                runs,
                break_at_start,
            )
        })
    }

    fn layout(&self, layout_context: &LayoutContext, ifc: &mut InlineFormattingContextState) {
        use style::values::generics::text::LineHeight;

        let (font_ascent, font_line_gap, font_key, runs, _) = self.break_and_shape(layout_context);
        let font_size = self.parent_style.get_font().font_size.size.0;
        let mut runs = runs.iter();
        loop {
            let mut glyphs = vec![];
            let mut advance_width = Length::zero();
            let mut last_break_opportunity = None;
            loop {
                let next = runs.next();
                if next
                    .as_ref()
                    .map_or(true, |run| run.glyph_store.is_whitespace())
                {
                    if advance_width > ifc.containing_block.inline_size - ifc.inline_position {
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
                    }
                    glyphs.push(run.glyph_store.clone());
                    advance_width += Length::from(run.glyph_store.total_advance());
                } else {
                    break;
                }
            }
            let line_height = match self.parent_style.get_inherited_text().line_height {
                LineHeight::Normal => font_line_gap.into(),
                LineHeight::Number(n) => font_size * n.0,
                LineHeight::Length(l) => l.0,
            };
            let content_rect = Rect {
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
                    parent_style: self.parent_style.clone(),
                    content_rect,
                    ascent: font_ascent.into(),
                    font_key,
                    glyphs,
                }));
            if runs.is_empty() {
                break;
            } else {
                // New line
                ifc.current_nesting_level.inline_start = Length::zero();
                let mut nesting_level = &mut ifc.current_nesting_level;
                for partial in ifc.partial_inline_boxes_stack.iter_mut().rev() {
                    partial.finish_layout(nesting_level, &mut ifc.inline_position, true);
                    partial.start_corner.inline = Length::zero();
                    partial.padding.inline_start = Length::zero();
                    partial.border.inline_start = Length::zero();
                    partial.margin.inline_start = Length::zero();
                    partial.parent_nesting_level.inline_start = Length::zero();
                    nesting_level = &mut partial.parent_nesting_level;
                }
                ifc.line_boxes
                    .finish_line(nesting_level, ifc.containing_block);
                ifc.inline_position = Length::zero();
            }
        }
    }
}
