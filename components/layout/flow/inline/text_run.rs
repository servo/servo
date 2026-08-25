/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::mem;
use std::ops::Range;
use std::sync::Arc;

use app_units::Au;
use fonts::font_feature_values::ResolvedFontVariantAlternates;
use fonts::{FontContext, FontRef, ShapedText, ShapedTextSlice, ShapingFlags, ShapingOptions};
use icu_locid::subtags::Language;
use icu_properties::{self, LineBreak};
use layout_api::SharedSelection;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use servo_arc::Arc as ServoArc;
use servo_base::text::{Utf32CodeUnits, is_bidi_control};
use smallvec::SmallVec;
use style::Zero;
use style::computed_values::font_kerning::T as FontKerning;
use style::computed_values::font_variant_position::T as FontVariantPosition;
use style::computed_values::text_rendering::T as TextRendering;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::font_face::FontLanguageOverride;
use style::properties::ComputedValues;
use style::values::computed::{
    FontFeatureSettings, FontVariantEastAsian, FontVariantLigatures, FontVariantNumeric,
};
use unicode_bidi::Level;
use unicode_script::Script;

use super::{InlineFormattingContextLayout, SharedInlineStyles};
use crate::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::WeakLayoutBox;
use crate::flow::inline::shaping_queue::ShapingQueueEntry;
use crate::flow::inline::text_transform::OffsetMap;
use crate::flow::inline::{BidiLevels, LineBlockSizes, LineItem, SegmentContentFlags};
use crate::fragment_tree::BaseFragmentInfo;

// There are two reasons why we might want to break at the start:
//
//  1. The line breaker told us that a break was necessary between two separate
//     instances of sending text to it.
//  2. We are following replaced content ie `have_deferred_soft_wrap_opportunity`.
//
// In both cases, we don't want to do this if the first character prevents a
// soft wrap opportunity.
#[derive(PartialEq)]
enum SegmentStartSoftWrapPolicy {
    Force,
    FollowLinebreaker,
}

/// A data structure which contains information used when shaping a [`TextRunSegment`].
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct FontAndScriptInfo {
    /// The script used when shaping a [`TextRunSegment`].
    pub script: Script,
    /// The rest of the font information which is never modified.
    #[conditional_malloc_size_of]
    pub font_info: Arc<FontInfo>,
}

impl FontAndScriptInfo {
    /// Creates a minimal [`FontAndScriptInfo`] for a single font, with generic language settings
    /// and the default shaping configuration. This is only used to generate placeholders for
    /// text carets on otherwise empty lines.
    pub(crate) fn simple_for_font(font: FontRef) -> Self {
        Self {
            script: Script::Common,
            font_info: Arc::new(FontInfo::simple_for_font(font)),
        }
    }
}

/// A data structure which contains information used when shaping a [`TextRunSegment`].
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct FontInfo {
    /// The font used when shaping a [`TextRunSegment`].
    pub font: FontRef,
    /// The BiDi [`Level`] used when shaping a [`TextRunSegment`].
    pub bidi_level: Level,
    /// The [`Language`] used when shaping a [`TextRunSegment`].
    pub language: Language,
    /// Spacing to add between each letter. Corresponds to the CSS 2.1 `letter-spacing` property.
    ///
    /// Letter spacing is not applied to all characters. Use [Self::letter_spacing_for_character] to
    /// determine the amount of spacing to apply.
    pub letter_spacing: Option<Au>,
    /// Spacing to add between each word. Corresponds to the CSS 2.1 `word-spacing` property.
    pub word_spacing: Option<Au>,
    /// The [`TextRendering`] value from the original style.
    pub text_rendering: TextRendering,
    /// The value of the `font-kerning` property from the original style.
    pub kerning: FontKerning,
    /// The value of the `font-variant-ligatures` property from the original style.
    pub ligatures: FontVariantLigatures,
    /// The value of the `font-variant-numeric` property from the original style.
    pub numeric: FontVariantNumeric,
    /// The value of the `font-variant-east-asian` property from the original style.
    pub east_asian: FontVariantEastAsian,
    /// The value of the `font-feature-settings` property from the original style.
    pub feature_settings: FontFeatureSettings,
    /// The value of the `font-variant-position` property from the original style.
    pub position: FontVariantPosition,
    /// The value of the `font-variant-alternates` property from the original style.
    ///
    /// Any alternate names are already resolved at this point.
    pub alternates: ResolvedFontVariantAlternates,
}

impl FontInfo {
    fn simple_for_font(font: FontRef) -> Self {
        Self {
            font,
            bidi_level: Level::ltr(),
            language: Language::UND,
            letter_spacing: None,
            word_spacing: None,
            text_rendering: TextRendering::Auto,
            kerning: FontKerning::Auto,
            ligatures: FontVariantLigatures::NORMAL,
            numeric: FontVariantNumeric::NORMAL,
            east_asian: FontVariantEastAsian::NORMAL,
            feature_settings: FontFeatureSettings::normal(),
            position: FontVariantPosition::Normal,
            alternates: Default::default(),
        }
    }
}

impl From<&FontAndScriptInfo> for ShapingOptions {
    fn from(info: &FontAndScriptInfo) -> Self {
        let mut ligatures = info.font_info.ligatures;
        let mut flags = ShapingFlags::empty();
        if info.font_info.bidi_level.is_rtl() {
            flags.insert(ShapingFlags::RTL_FLAG);
        }

        // From https://www.w3.org/TR/css-text-3/#cursive-script:
        // Cursive scripts do not admit gaps between their letters for either
        // justification or letter-spacing.
        let letter_spacing = info
            .font_info
            .letter_spacing
            .filter(|_| !is_cursive_script(info.script));
        if letter_spacing.is_some() {
            ligatures = FontVariantLigatures::NONE;
        };
        if info.font_info.text_rendering == TextRendering::Optimizespeed {
            ligatures = FontVariantLigatures::NONE;
            flags.insert(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG)
        }

        // We currently always leave kerning enabled for "font-kerning: auto".
        if info.font_info.kerning == FontKerning::None {
            flags.insert(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG);
        }

        Self {
            letter_spacing,
            word_spacing: info.font_info.word_spacing,
            script: info.script,
            language: info.font_info.language,
            ligatures,
            numeric: info.font_info.numeric,
            east_asian: info.font_info.east_asian,
            feature_settings: info.font_info.feature_settings.clone(),
            position: info.font_info.position,
            flags,
            alternates: info.font_info.alternates.clone(),
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct TextRunSegment {
    /// Information about the font and language used in this text run. This is produced by
    /// segmenting the inline formatting context's text content by font, script, and bidi level.
    pub info: FontAndScriptInfo,

    /// The range of bytes in the parent [`super::InlineFormattingContext`]'s text content.
    pub byte_range: Range<usize>,

    /// The range of characters in the parent [`super::InlineFormattingContext`]'s text content.
    pub character_range: Range<usize>,

    /// Whether or not the linebreaker said that we should allow a line break at the start of this
    /// segment.
    pub break_at_start: bool,

    /// The shaped runs within this segment.
    #[conditional_malloc_size_of]
    pub runs: Vec<Arc<ShapedTextSlice>>,

    /// The shaped text that was used to produce this segment. [`Self::runs`] are slices
    /// of this shaped text.
    #[conditional_malloc_size_of]
    pub shaped_text: Option<Arc<ShapedText>>,
}

impl TextRunSegment {
    fn new(
        info: FontAndScriptInfo,
        byte_range: Range<usize>,
        character_range: Range<usize>,
    ) -> Self {
        Self {
            info,
            byte_range,
            character_range,
            runs: Vec::new(),
            break_at_start: false,
            shaped_text: None,
        }
    }

    /// Returns true if the new `Font`, `Script` and BiDi `Level` are compatible with this segment
    /// or false otherwise.
    fn is_compatible(
        &self,
        new_font: &Option<FontRef>,
        new_script: Script,
        new_bidi_level: Level,
    ) -> bool {
        if self.info.font_info.bidi_level != new_bidi_level {
            return false;
        }
        if new_font
            .as_ref()
            .is_some_and(|new_font| !Arc::ptr_eq(&self.info.font_info.font, new_font))
        {
            return false;
        }

        !script_is_specific(self.info.script) ||
            !script_is_specific(new_script) ||
            self.info.script == new_script
    }

    /// Update this segment to end at the given byte and character index. The update will only ever
    /// make the Script specific and will not change it otherwise.
    fn update(&mut self, next_byte_index: usize, next_character_index: usize, new_script: Script) {
        if !script_is_specific(self.info.script) && script_is_specific(new_script) {
            self.info = FontAndScriptInfo {
                script: new_script,
                font_info: self.info.font_info.clone(),
            };
        }
        self.character_range.end = next_character_index;
        self.byte_range.end = next_byte_index;
    }

    fn layout_into_line_items(
        &self,
        text_run: &TextRun,
        mut soft_wrap_policy: SegmentStartSoftWrapPolicy,
        ifc: &mut InlineFormattingContextLayout,
    ) {
        if self.break_at_start && soft_wrap_policy == SegmentStartSoftWrapPolicy::FollowLinebreaker
        {
            soft_wrap_policy = SegmentStartSoftWrapPolicy::Force;
        }

        let mut character_range_start = self.character_range.start;
        for (run_index, run) in self.runs.iter().enumerate() {
            let new_character_range_end = character_range_start + run.character_count();

            // Break before each unbreakable run in this TextRun, except the first unless the
            // linebreaker was set to break before the first run.
            if run_index != 0 || soft_wrap_policy == SegmentStartSoftWrapPolicy::Force {
                ifc.process_soft_wrap_opportunity();
            }

            let run_start = text_run.run_data.character_range_in_ifc_text.start;
            ifc.push_glyph_store_to_unbreakable_segment(
                run.clone(),
                text_run,
                &self.info,
                Utf32CodeUnits(character_range_start - run_start)..
                    Utf32CodeUnits(new_character_range_end - run_start),
            );

            character_range_start = new_character_range_end;
        }
    }

    pub(crate) fn is_compatible_with_old_shaping_result(&self, old_segment: &Self) -> bool {
        old_segment.info == self.info && self.byte_range == old_segment.byte_range
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct CaretPlaceholder {
    /// The [`TextFragmentRunData`] of the [`TextRun`] that contains this caret placeholder.
    #[conditional_malloc_size_of]
    pub run_data: Arc<SharedTextRunData>,
    /// The `BaseFragmentInfo` of the originating text node that this caret placeholder is in.
    pub base_fragment_info: BaseFragmentInfo,
    /// Character index of the preserved newline in the IFC's transformed text, relative
    /// to the start of the DOM node.
    pub character_index: usize,
}

/// A single item in a [`TextRun`].
#[derive(Debug, MallocSizeOf)]
pub(crate) enum TextRunItem {
    /// A hard line break i.e. a "\n" as other types line breaks are normalized to "\n".
    LineBreak(Option<CaretPlaceholder>),
    /// A preserved tab character that should advance the line to a tab stop.
    Tab { bidi_level: Level },
    /// Any other text for which a font can be matched. We store a `Box` here as [`TextRunSegment`]
    /// is quite a bit larger than the other enum variants.
    TextSegment(Box<TextRunSegment>),
}

/// A data structure that holds per-`TextRun` data used on `TextFragment`s.
/// This ensures that the data is not duplicated between fragments.
#[derive(Debug, MallocSizeOf)]
pub(crate) struct SharedTextRunData {
    /// The [`crate::SharedStyle`] from this `TextRun`'s parent element. This is
    /// shared so that incremental layout can simply update the parent element and
    /// this [`TextRun`] will be updated automatically.
    pub inline_styles: SharedInlineStyles,
    /// The range of characters in this text in `InlineFormattingContext::text_content`
    /// of the `InlineFormattingContext` that owns this `TextRun`. These are counting
    /// `char`s, *not* UTF-8 offsets.
    pub character_range_in_ifc_text: Range<usize>,
    /// The original offset of this `TextRun` in the `InlineFormattingContext`'s input
    /// text (untransformed by white space collapse and `text-transform`).
    pub original_offset: Utf32CodeUnits,
    /// The selected text in this `TextRun`. This may either be document selection or form control
    /// selection.
    #[conditional_malloc_size_of]
    pub selection: Option<SharedSelection>,
    /// The [`OffsetMap`] used when creating this `TextRun`'s `InlineFormattingContext`. This
    /// is used for mapping between DOM text offsets and layout text offsets (and vice-versa).
    pub offset_map: ArcRefCell<OffsetMap>,
}

impl SharedTextRunData {
    /// Map a range in the originating `TextRun`'s DOM node text into the range in the
    /// `TextRun`'s layout transformed (by white space collapse and `text-transform`)
    /// text.
    pub(crate) fn map_dom_range_to_transformed_range(
        &self,
        range: Range<Utf32CodeUnits>,
    ) -> Range<Utf32CodeUnits> {
        let offset_map = self.offset_map.borrow();
        let offset_in_ifc_text = Utf32CodeUnits(self.character_range_in_ifc_text.start);
        offset_map.map(range.start + self.original_offset) - offset_in_ifc_text..
            offset_map.map(range.end + self.original_offset) - offset_in_ifc_text
    }

    /// Map an offset in the originating `TextRun`s DOM node's transformed text (by white
    /// space collapse and `text-transform`) to untransformed text for use by the DOM.
    pub(crate) fn map_transformed_offset_to_dom_offset(
        &self,
        offset: Utf32CodeUnits,
    ) -> Utf32CodeUnits {
        let offset_map = self.offset_map.borrow();
        let offset_in_ifc_text = Utf32CodeUnits(self.character_range_in_ifc_text.start);
        offset_map.reverse_map(offset + offset_in_ifc_text) - self.original_offset
    }
}

/// A single [`TextRun`] for the box tree. These are all descendants of
/// [`super::InlineBox`] or the root of the [`super::InlineFormattingContext`].  During
/// box tree construction, text is split into [`TextRun`]s based on their font, script,
/// etc. When these are created text is already shaped.
///
/// <https://www.w3.org/TR/css-display-3/#css-text-run>
#[derive(Debug, MallocSizeOf)]
pub(crate) struct TextRun {
    /// The [`BaseFragmentInfo`] for this [`TextRun`]. Usually this comes from the
    /// original text node in the DOM for the text.
    pub base_fragment_info: BaseFragmentInfo,

    /// Data to be used by all [`TextFragment`]s spawned by this [`TextRun`] to avoid
    /// having to clone the data into each fragment.
    #[conditional_malloc_size_of]
    pub run_data: Arc<SharedTextRunData>,

    /// A weak reference to the parent of this layout box. This becomes valid as soon
    /// as the *parent* of this box is added to the tree.
    pub parent_box: Option<WeakLayoutBox>,

    /// The range of text in [`super::InlineFormattingContext::text_content`] of the
    /// [`super::InlineFormattingContext`] that owns this [`TextRun`]. These are UTF-8 offsets.
    pub text_range: Range<usize>,

    /// The [`TextRunItem`]s of this text run. This is produced by segmenting the incoming text
    /// by things such as font and script as well as separating out hard line breaks.
    /// segments, and shaped.
    pub items: Vec<TextRunItem>,
}

impl TextRun {
    pub(crate) fn new(
        base_fragment_info: BaseFragmentInfo,
        run_data: Arc<SharedTextRunData>,
        text_range: Range<usize>,
        old_text_run: Option<ArcRefCell<TextRun>>,
    ) -> Self {
        // If there was a previous box tree layout of this text run, try to preserve the old shaped text.
        let items = old_text_run
            .map(|old_text_run| std::mem::take(&mut old_text_run.borrow_mut().items))
            .unwrap_or_default();
        Self {
            base_fragment_info,
            run_data,
            parent_box: None,
            text_range,
            items,
        }
    }

    pub(super) fn inline_styles(&self) -> &SharedInlineStyles {
        &self.run_data.inline_styles
    }

    pub(super) fn segment(
        &mut self,
        self_arc_ref_cell: ArcRefCell<TextRun>,
        formatting_context_text: &str,
        layout_context: &LayoutContext,
        bidi_levels: &BidiLevels,
    ) -> SmallVec<[ShapingQueueEntry; 1]> {
        let parent_style = self.inline_styles().style.borrow().clone();
        let items = self.segment_text_by_font(
            layout_context,
            formatting_context_text,
            bidi_levels,
            &parent_style,
        );

        // If a previous box tree layout seeded this [`TextRun`] with old shaping results, use those
        // to try to prevent re-shaping.
        let mut old_text_run_items = std::mem::replace(&mut self.items, items).into_iter();

        self.items
            .iter()
            .enumerate()
            .map(move |(index, text_run_item)| {
                let old_text_run_item = old_text_run_items.next();
                ShapingQueueEntry::new(
                    self_arc_ref_cell.clone(),
                    text_run_item,
                    index,
                    old_text_run_item,
                )
            })
            .collect()
    }

    /// Take the [`TextRun`]'s text and turn it into [`TextRunSegment`]s. Each segment has a matched
    /// font and script. Fonts may differ when glyphs are found in fallback fonts.
    /// [`super::InlineFormattingContext`].
    fn segment_text_by_font(
        &mut self,
        layout_context: &LayoutContext,
        formatting_context_text: &str,
        bidi_levels: &BidiLevels,
        parent_style: &ServoArc<ComputedValues>,
    ) -> Vec<TextRunItem> {
        let font_style = parent_style.clone_font();
        let language = font_style._x_lang.0.parse().unwrap_or(Language::UND);
        let language_for_shaping = Some(font_style.font_language_override)
            .filter(|language_override| *language_override != FontLanguageOverride::normal())
            .and_then(|language_override| {
                // FIXME: ICU4x limits language tags to three bytes as that is limit
                // defined by BCP 47. But OpenType defines a couple four-letter
                // languages, and stylo correctly stores a four-byte value for the computed
                // value of the property.
                //
                // https://www.w3.org/TR/css-fonts-4/#font-language-override-string-value
                //
                // For now we need to truncate the language tag ):
                Language::try_from_bytes(&language_override.0.to_be_bytes()[..3]).ok()
            })
            .unwrap_or(language);
        let font_size = font_style.font_size.computed_size().into();
        let kerning = font_style.font_kerning;
        let ligatures = font_style.font_variant_ligatures;
        let numeric = font_style.font_variant_numeric;
        let east_asian = font_style.font_variant_east_asian;
        let feature_settings = font_style.font_feature_settings.clone();
        let position = font_style.font_variant_position;
        let alternates = font_style.font_variant_alternates.clone();

        let font_group = layout_context.font_context.font_group(font_style);
        let inherited_text_style = parent_style.get_inherited_text();
        let word_spacing = Some(inherited_text_style.word_spacing.to_used_value(font_size));
        let letter_spacing = inherited_text_style
            .letter_spacing
            .0
            .to_used_value(font_size);
        let letter_spacing = if !letter_spacing.is_zero() {
            Some(letter_spacing)
        } else {
            None
        };
        let text_rendering = inherited_text_style.text_rendering;

        let mut current: Option<TextRunSegment> = None;
        let mut results = Vec::new();
        let finish_current_segment =
            |current: &mut Option<TextRunSegment>, results: &mut Vec<TextRunItem>| {
                if let Some(current) = current.take() {
                    results.push(TextRunItem::TextSegment(Box::new(current)));
                }
            };

        let text_run_text = &formatting_context_text[self.text_range.clone()];
        let char_iterator = TwoCharsAtATimeIterator::new(text_run_text.chars());
        // The next bytes index of the character within the entire inline formatting context's text.
        let mut next_byte_index = self.text_range.start;
        for (relative_character_index, (character, next_character)) in char_iterator.enumerate() {
            // The current character index within the entire inline formatting context's text.
            let current_character_index =
                self.run_data.character_range_in_ifc_text.start + relative_character_index;

            let current_byte_index = next_byte_index;
            next_byte_index += character.len_utf8();

            if character == '\n' {
                finish_current_segment(&mut current, &mut results);
                results.push(TextRunItem::LineBreak(
                    self.run_data.selection.is_some().then(|| CaretPlaceholder {
                        run_data: self.run_data.clone(),
                        base_fragment_info: self.base_fragment_info,
                        // The placeholder that is placed after a newline is for the index after that newline.
                        // The newline itself is at the end of the previous line.
                        character_index: relative_character_index + 1,
                    }),
                ));
                continue;
            }

            if character == '\t' {
                finish_current_segment(&mut current, &mut results);
                results.push(TextRunItem::Tab {
                    bidi_level: bidi_levels.level(current_byte_index),
                });
                continue;
            }

            let (font, script, bidi_level) = if character_cannot_change_font(character) {
                (None, Script::Common, bidi_levels.level(current_byte_index))
            } else {
                (
                    font_group.find_by_codepoint(
                        &layout_context.font_context,
                        character,
                        next_character,
                        language,
                    ),
                    Script::from(character),
                    bidi_levels.level(current_byte_index),
                )
            };

            // If the existing segment is compatible with the character, just merge the character into it.
            if let Some(current) = current.as_mut() &&
                current.is_compatible(&font, script, bidi_level)
            {
                current.update(next_byte_index, current_character_index + 1, script);
                continue;
            }

            let Some(font) = font.or_else(|| font_group.first(&layout_context.font_context)) else {
                continue;
            };

            let alternates = layout_context
                .font_context
                .resolve_font_variant_alternate_identifiers_for(
                    &font,
                    &alternates,
                    layout_context.style_context.stylist,
                );
            let info = FontAndScriptInfo {
                script,
                font_info: Arc::new(FontInfo {
                    font,
                    bidi_level,
                    language: language_for_shaping,
                    word_spacing,
                    letter_spacing,
                    text_rendering,
                    kerning,
                    ligatures,
                    numeric,
                    east_asian,
                    feature_settings: feature_settings.clone(),
                    alternates,
                    position,
                }),
            };

            finish_current_segment(&mut current, &mut results);
            assert!(current.is_none());

            current = Some(TextRunSegment::new(
                info,
                current_byte_index..next_byte_index,
                current_character_index..current_character_index + 1,
            ));
        }

        finish_current_segment(&mut current, &mut results);
        results
    }

    pub(super) fn layout_into_line_items(&self, ifc: &mut InlineFormattingContextLayout) {
        if self.text_range.is_empty() {
            return;
        }

        // If we are following replaced content, we should have a soft wrap opportunity, unless the
        // first character of this `TextRun` prevents that soft wrap opportunity. If we see such a
        // character it should also override the LineBreaker's indication to break at the start.
        let have_deferred_soft_wrap_opportunity =
            mem::replace(&mut ifc.have_deferred_soft_wrap_opportunity, false);
        let mut soft_wrap_policy = match have_deferred_soft_wrap_opportunity {
            true => SegmentStartSoftWrapPolicy::Force,
            false => SegmentStartSoftWrapPolicy::FollowLinebreaker,
        };

        for item in self.items.iter() {
            ifc.possibly_flush_deferred_forced_line_break();

            match item {
                // If this whitespace forces a line break, queue up a hard line break the next time we
                // see any content. We don't line break immediately, because we'd like to finish processing
                // any ongoing inline boxes before ending the line.
                TextRunItem::LineBreak(caret_placeholder) => {
                    ifc.defer_forced_line_break_at_character_offset(caret_placeholder);
                },
                TextRunItem::Tab { bidi_level } => self.process_preserved_tab(ifc, *bidi_level),
                TextRunItem::TextSegment(segment) => {
                    segment.layout_into_line_items(self, soft_wrap_policy, ifc)
                },
            }
            soft_wrap_policy = SegmentStartSoftWrapPolicy::FollowLinebreaker;
        }
    }

    fn process_preserved_tab(
        &self,
        ifc_layout: &mut InlineFormattingContextLayout,
        bidi_level: Level,
    ) {
        let advance = ifc_layout.ifc.next_tab_stop_after_inline_advance(
            &self.inline_styles().style.borrow(),
            ifc_layout.potential_line_size().inline,
        );
        if advance.is_zero() {
            return;
        }

        ifc_layout.update_unbreakable_segment_for_new_content(
            &LineBlockSizes::zero(),
            advance,
            SegmentContentFlags::empty(),
        );
        ifc_layout.push_line_item_to_unbreakable_segment(LineItem::Tab {
            inline_box_identifier: ifc_layout.current_inline_box_identifier(),
            advance,
            bidi_level,
        });

        if ifc_layout
            .current_inline_container_state()
            .style
            .get_inherited_text()
            .white_space_collapse ==
            WhiteSpaceCollapse::BreakSpaces
        {
            ifc_layout.process_soft_wrap_opportunity();
        }
    }
}

/// From <https://www.w3.org/TR/css-text-3/#cursive-script>:
/// Cursive scripts do not admit gaps between their letters for either justification
/// or letter-spacing. The following Unicode scripts are included: Arabic, Hanifi
/// Rohingya, Mandaic, Mongolian, N’Ko, Phags Pa, Syriac
fn is_cursive_script(script: Script) -> bool {
    matches!(
        script,
        Script::Arabic |
            Script::Hanifi_Rohingya |
            Script::Mandaic |
            Script::Mongolian |
            Script::Nko |
            Script::Phags_Pa |
            Script::Syriac
    )
}

/// Whether or not this character should be able to change the font during segmentation.  Certain
/// character are not rendered at all, so it doesn't matter what font we use to render them. They
/// should just be added to the current segment.
fn character_cannot_change_font(character: char) -> bool {
    if character.is_control() {
        return true;
    }
    if character == '\u{00A0}' {
        return true;
    }
    if is_bidi_control(character) {
        return false;
    }

    matches!(
        icu_properties::maps::line_break().get(character),
        LineBreak::CombiningMark |
            LineBreak::Glue |
            LineBreak::ZWSpace |
            LineBreak::WordJoiner |
            LineBreak::ZWJ
    )
}

pub(super) fn get_font_for_first_font_for_style(
    style: &ComputedValues,
    font_context: &FontContext,
) -> Option<FontRef> {
    let font = font_context
        .font_group(style.clone_font())
        .first(font_context);
    if font.is_none() {
        warn!("Could not find font for style: {:?}", style.clone_font());
    }
    font
}
pub(crate) struct TwoCharsAtATimeIterator<InputIterator> {
    /// The input character iterator.
    iterator: InputIterator,
    /// The first character to produce in the next run of the iterator.
    next_character: Option<char>,
}

impl<InputIterator> TwoCharsAtATimeIterator<InputIterator> {
    fn new(iterator: InputIterator) -> Self {
        Self {
            iterator,
            next_character: None,
        }
    }
}

impl<InputIterator> Iterator for TwoCharsAtATimeIterator<InputIterator>
where
    InputIterator: Iterator<Item = char>,
{
    type Item = (char, Option<char>);

    fn next(&mut self) -> Option<Self::Item> {
        // If the iterator isn't initialized do that now.
        if self.next_character.is_none() {
            self.next_character = self.iterator.next();
        }
        let character = self.next_character?;
        self.next_character = self.iterator.next();
        Some((character, self.next_character))
    }
}

pub(crate) fn script_is_specific(script: Script) -> bool {
    script != Script::Common && script != Script::Inherited
}
