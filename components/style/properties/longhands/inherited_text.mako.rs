/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword %>
<% data.new_style_struct("InheritedText", inherited=True, gecko_name="Text") %>

${helpers.predefined_type(
    "color",
    "ColorPropertyValue",
    "crate::color::AbsoluteColor::black()",
    engines="gecko servo",
    animation_value_type="AbsoluteColor",
    ignored_when_colors_disabled="True",
    spec="https://drafts.csswg.org/css-color/#color",
)}

${helpers.predefined_type(
    "line-height",
    "LineHeight",
    "computed::LineHeight::normal()",
    engines="gecko servo",
    animation_value_type="LineHeight",
    spec="https://drafts.csswg.org/css2/visudet.html#propdef-line-height",
    servo_restyle_damage="reflow"
)}

// CSS Text Module Level 3

${helpers.predefined_type(
    "text-transform",
    "TextTransform",
    "computed::TextTransform::none()",
    engines="gecko servo",
    servo_pref="layout.legacy_layout",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-text-transform",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "hyphens",
    "manual none auto",
    engines="gecko",
    gecko_enum_prefix="StyleHyphens",
    animation_value_type="discrete",
    extra_prefixes="moz",
    spec="https://drafts.csswg.org/css-text/#propdef-hyphens",
)}

// TODO: Support <percentage>
${helpers.single_keyword(
    "-moz-text-size-adjust",
    "auto none",
    engines="gecko",
    gecko_enum_prefix="StyleTextSizeAdjust",
    gecko_ffi_name="mTextSizeAdjust",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-size-adjust/#adjustment-control",
    aliases="-webkit-text-size-adjust",
)}

${helpers.predefined_type(
    "text-indent",
    "LengthPercentage",
    "computed::LengthPercentage::zero()",
    engines="gecko servo",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-text/#propdef-text-indent",
    allow_quirks="Yes",
    servo_restyle_damage = "reflow",
)}

// Also known as "word-wrap" (which is more popular because of IE), but this is
// the preferred name per CSS-TEXT 6.2.
${helpers.predefined_type(
    "overflow-wrap",
    "OverflowWrap",
    "computed::OverflowWrap::Normal",
    engines="gecko servo",
    servo_pref="layout.legacy_layout",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-overflow-wrap",
    aliases="word-wrap",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "word-break",
    "WordBreak",
    "computed::WordBreak::Normal",
    engines="gecko servo",
    servo_pref="layout.legacy_layout",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-word-break",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "text-justify",
    "TextJustify",
    "computed::TextJustify::Auto",
    engines="gecko servo",
    servo_pref="layout.legacy_layout",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-text-justify",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "text-align-last",
    "TextAlignLast",
    "computed::text::TextAlignLast::Auto",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-text-align-last",
)}

// TODO make this a shorthand and implement text-align-last/text-align-all
${helpers.predefined_type(
    "text-align",
    "TextAlign",
    "computed::TextAlign::Start",
    engines="gecko servo",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-text-align",
    servo_restyle_damage = "reflow",
)}

${helpers.predefined_type(
    "letter-spacing",
    "LetterSpacing",
    "computed::LetterSpacing::normal()",
    engines="gecko servo",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-text/#propdef-letter-spacing",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "word-spacing",
    "WordSpacing",
    "computed::WordSpacing::zero()",
    engines="gecko servo",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-text/#propdef-word-spacing",
    servo_restyle_damage="rebuild_and_reflow",
)}

<%helpers:single_keyword
    name="white-space"
    values="normal pre nowrap pre-wrap pre-line"
    engines="gecko servo",
    extra_gecko_values="break-spaces -moz-pre-space"
    gecko_enum_prefix="StyleWhiteSpace"
    needs_conversion="True"
    animation_value_type="discrete"
    spec="https://drafts.csswg.org/css-text/#propdef-white-space"
    servo_restyle_damage="rebuild_and_reflow"
>
    % if engine == "servo":
    impl SpecifiedValue {
        pub fn allow_wrap(&self) -> bool {
            match *self {
                SpecifiedValue::Nowrap |
                SpecifiedValue::Pre => false,
                SpecifiedValue::Normal |
                SpecifiedValue::PreWrap |
                SpecifiedValue::PreLine => true,
            }
        }

        pub fn preserve_newlines(&self) -> bool {
            match *self {
                SpecifiedValue::Normal |
                SpecifiedValue::Nowrap => false,
                SpecifiedValue::Pre |
                SpecifiedValue::PreWrap |
                SpecifiedValue::PreLine => true,
            }
        }

        pub fn preserve_spaces(&self) -> bool {
            match *self {
                SpecifiedValue::Normal |
                SpecifiedValue::Nowrap |
                SpecifiedValue::PreLine => false,
                SpecifiedValue::Pre |
                SpecifiedValue::PreWrap => true,
            }
        }
    }
    % endif
</%helpers:single_keyword>

${helpers.predefined_type(
    "text-shadow",
    "SimpleShadow",
    None,
    engines="gecko servo",
    servo_pref="layout.legacy_layout",
    vector=True,
    vector_animation_type="with_zero",
    animation_value_type="AnimatedTextShadowList",
    ignored_when_colors_disabled=True,
    simple_vector_bindings=True,
    spec="https://drafts.csswg.org/css-text-decor-3/#text-shadow-property",
)}

${helpers.predefined_type(
    "text-emphasis-style",
    "TextEmphasisStyle",
    "computed::TextEmphasisStyle::None",
    engines="gecko",
    initial_specified_value="SpecifiedValue::None",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-emphasis-style",
)}

${helpers.predefined_type(
    "text-emphasis-position",
    "TextEmphasisPosition",
    "computed::TextEmphasisPosition::OVER",
    engines="gecko",
    initial_specified_value="specified::TextEmphasisPosition::OVER",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-emphasis-position",
)}

${helpers.predefined_type(
    "text-emphasis-color",
    "Color",
    "computed_value::T::currentcolor()",
    engines="gecko",
    initial_specified_value="specified::Color::currentcolor()",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-emphasis-color",
)}

${helpers.predefined_type(
    "tab-size",
    "NonNegativeLengthOrNumber",
    "generics::length::LengthOrNumber::Number(From::from(8.0))",
    engines="gecko",
    animation_value_type="LengthOrNumber",
    spec="https://drafts.csswg.org/css-text-3/#tab-size-property",
    aliases="-moz-tab-size",
)}

${helpers.predefined_type(
    "line-break",
    "LineBreak",
    "computed::LineBreak::Auto",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text-3/#line-break-property",
)}

// CSS Compatibility
// https://compat.spec.whatwg.org
${helpers.predefined_type(
    "-webkit-text-fill-color",
    "Color",
    "computed_value::T::currentcolor()",
    engines="gecko",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    spec="https://compat.spec.whatwg.org/#the-webkit-text-fill-color",
)}

${helpers.predefined_type(
    "-webkit-text-stroke-color",
    "Color",
    "computed_value::T::currentcolor()",
    initial_specified_value="specified::Color::currentcolor()",
    engines="gecko",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    spec="https://compat.spec.whatwg.org/#the-webkit-text-stroke-color",
)}

${helpers.predefined_type(
    "-webkit-text-stroke-width",
    "LineWidth",
    "app_units::Au(0)",
    engines="gecko",
    initial_specified_value="specified::LineWidth::zero()",
    spec="https://compat.spec.whatwg.org/#the-webkit-text-stroke-width",
    animation_value_type="discrete",
)}

// CSS Ruby Layout Module Level 1
// https://drafts.csswg.org/css-ruby/
${helpers.single_keyword(
    "ruby-align",
    "space-around start center space-between",
    engines="gecko",
    animation_value_type="discrete",
    gecko_enum_prefix="StyleRubyAlign",
    spec="https://drafts.csswg.org/css-ruby/#ruby-align-property",
)}

${helpers.predefined_type(
    "ruby-position",
    "RubyPosition",
    "computed::RubyPosition::AlternateOver",
    engines="gecko",
    spec="https://drafts.csswg.org/css-ruby/#ruby-position-property",
    animation_value_type="discrete",
)}

// CSS Writing Modes Module Level 3
// https://drafts.csswg.org/css-writing-modes-3/

${helpers.single_keyword(
    "text-combine-upright",
    "none all",
    engines="gecko",
    gecko_enum_prefix="StyleTextCombineUpright",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes-3/#text-combine-upright",
)}

// SVG 1.1: Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword(
    "text-rendering",
    "auto optimizespeed optimizelegibility geometricprecision",
    engines="gecko servo",
    gecko_enum_prefix="StyleTextRendering",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/painting.html#TextRenderingProperty",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "-moz-control-character-visibility",
    "text::MozControlCharacterVisibility",
    "Default::default()",
    engines="gecko",
    enabled_in="chrome",
    gecko_pref="layout.css.moz-control-character-visibility.enabled",
    has_effect_on_gecko_scrollbars=False,
    animation_value_type="none",
    spec="Nonstandard"
)}

// text underline offset
${helpers.predefined_type(
    "text-underline-offset",
    "LengthPercentageOrAuto",
    "computed::LengthPercentageOrAuto::auto()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-text-decor-4/#underline-offset",
)}

// text underline position
${helpers.predefined_type(
    "text-underline-position",
    "TextUnderlinePosition",
    "computed::TextUnderlinePosition::AUTO",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text-decor-3/#text-underline-position-property",
)}

// text decoration skip ink
${helpers.predefined_type(
    "text-decoration-skip-ink",
    "TextDecorationSkipInk",
    "computed::TextDecorationSkipInk::Auto",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text-decor-4/#text-decoration-skip-ink-property",
)}

// hyphenation character
${helpers.predefined_type(
    "hyphenate-character",
    "HyphenateCharacter",
    "computed::HyphenateCharacter::Auto",
    engines="gecko",
    gecko_pref="layout.css.hyphenate-character.enabled",
    has_effect_on_gecko_scrollbars=False,
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/css-text-4/#hyphenate-character",
)}

${helpers.predefined_type(
    "forced-color-adjust",
    "ForcedColorAdjust",
    "computed::ForcedColorAdjust::Auto",
    engines="gecko",
    gecko_pref="layout.css.forced-color-adjust.enabled",
    has_effect_on_gecko_scrollbars=False,
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-color-adjust-1/#forced-color-adjust-prop",
)}

${helpers.single_keyword(
    "-webkit-text-security",
    "none circle disc square",
    engines="gecko",
    gecko_enum_prefix="StyleTextSecurity",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#MISSING",
)}
