/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::mem;

use app_units::Au;
use gfx::font::FontMetrics;
use gfx::text::text_run::GlyphRun;
use serde::Serialize;
use servo_arc::Arc;
use style::properties::ComputedValues;
use webrender_api::FontInstanceKey;
use xi_unicode::{linebreak_property, LineBreakLeafIter};

use super::inline::InlineFormattingContextState;
use crate::context::LayoutContext;
use crate::fragment_tree::BaseFragmentInfo;

// These constants are the xi-unicode line breaking classes that are defined in
// `table.rs`. Unfortunately, they are only identified by number.
const XI_LINE_BREAKING_CLASS_GL: u8 = 12;
const XI_LINE_BREAKING_CLASS_WJ: u8 = 30;
const XI_LINE_BREAKING_CLASS_ZWJ: u8 = 40;

/// https://www.w3.org/TR/css-display-3/#css-text-run
#[derive(Debug, Serialize)]
pub(crate) struct TextRun {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub parent_style: Arc<ComputedValues>,
    pub text: String,
    pub has_uncollapsible_content: bool,
    pub shaped_text: Option<BreakAndShapeResult>,
}

#[derive(Debug, Serialize)]
pub(crate) struct BreakAndShapeResult {
    pub font_metrics: FontMetrics,
    pub font_key: FontInstanceKey,
    pub runs: Vec<GlyphRun>,
    pub break_at_start: bool,
}

impl TextRun {
    pub(super) fn break_and_shape(
        &mut self,
        layout_context: &LayoutContext,
        linebreaker: &mut Option<LineBreakLeafIter>,
    ) {
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

        self.shaped_text =
            crate::context::with_thread_local_font_context(layout_context, |font_context| {
                let font_group = font_context.font_group(font_style);
                let font = match font_group.borrow_mut().first(font_context) {
                    Some(font) => font,
                    None => return Err("Could not find find for TextRun."),
                };
                let mut font = font.borrow_mut();

                let word_spacing = &inherited_text_style.word_spacing;
                let word_spacing =
                    word_spacing
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
                    linebreaker,
                );

                Ok(BreakAndShapeResult {
                    font_metrics: font.metrics.clone(),
                    font_key: font.font_key,
                    runs,
                    break_at_start,
                })
            })
            .ok();
    }

    pub(super) fn glyph_run_is_whitespace_ending_with_preserved_newline(
        &self,
        run: &GlyphRun,
    ) -> bool {
        if !run.glyph_store.is_whitespace() {
            return false;
        }
        if !self
            .parent_style
            .get_inherited_text()
            .white_space
            .preserve_newlines()
        {
            return false;
        }

        let last_byte = self.text.as_bytes().get(run.range.end().to_usize() - 1);
        last_byte == Some(&b'\n')
    }

    pub(super) fn layout_into_line_items(&self, ifc: &mut InlineFormattingContextState) {
        let broken = match self.shaped_text.as_ref() {
            Some(broken) => broken,
            None => return,
        };

        // We either have a soft wrap opportunity if specified by the breaker or if we are
        // following replaced content.
        let have_deferred_soft_wrap_opportunity =
            mem::replace(&mut ifc.have_deferred_soft_wrap_opportunity, false);
        let mut break_at_start = broken.break_at_start || have_deferred_soft_wrap_opportunity;

        if have_deferred_soft_wrap_opportunity {
            if let Some(first_character) = self.text.chars().nth(0) {
                break_at_start = break_at_start &&
                    !char_prevents_soft_wrap_opportunity_when_before_or_after_atomic(
                        first_character,
                    )
            }
        }

        if let Some(last_character) = self.text.chars().last() {
            ifc.prevent_soft_wrap_opportunity_before_next_atomic =
                char_prevents_soft_wrap_opportunity_when_before_or_after_atomic(last_character);
        }

        for (run_index, run) in broken.runs.iter().enumerate() {
            ifc.possibly_flush_deferred_forced_line_break();

            // If this whitespace forces a line break, queue up a hard line break the next time we
            // see any content. We don't line break immediately, because we'd like to finish processing
            // any ongoing inline boxes before ending the line.
            if self.glyph_run_is_whitespace_ending_with_preserved_newline(run) {
                ifc.defer_forced_line_break();
                continue;
            }

            // Break before each unbrekable run in this TextRun, except the first unless the
            // linebreaker was set to break before the first run.
            if run_index != 0 || break_at_start {
                ifc.process_soft_wrap_opportunity();
            }

            ifc.push_glyph_store_to_unbreakable_segment(
                run.glyph_store.clone(),
                self.base_fragment_info,
                &self.parent_style,
                &broken.font_metrics,
                broken.font_key,
            );
        }
    }
}

/// comes before or after an atomic inline element.
///
/// From https://www.w3.org/TR/css-text-3/#line-break-details:
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
