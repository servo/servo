/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword %>
<% data.new_style_struct("InheritedText", inherited=True, gecko_name="Text") %>

${helpers.predefined_type(
    "color",
    "ColorPropertyValue",
    "::cssparser::RGBA::new(0, 0, 0, 255)",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="AnimatedRGBA",
    ignored_when_colors_disabled="True",
    spec="https://drafts.csswg.org/css-color/#color",
)}

${helpers.predefined_type(
    "line-height",
    "LineHeight",
    "computed::LineHeight::normal()",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="LineHeight",
    spec="https://drafts.csswg.org/css2/visudet.html#propdef-line-height",
    servo_restyle_damage="reflow"
)}

// CSS Text Module Level 3

${helpers.predefined_type(
    "text-transform",
    "TextTransform",
    "computed::TextTransform::none()",
    engines="gecko servo-2013",
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
    alias="-webkit-text-size-adjust",
)}

${helpers.predefined_type(
    "text-indent",
    "LengthPercentage",
    "computed::LengthPercentage::zero()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
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
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-overflow-wrap",
    alias="word-wrap",
    needs_context=False,
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "word-break",
    "WordBreak",
    "computed::WordBreak::Normal",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-word-break",
    needs_context=False,
    servo_restyle_damage="rebuild_and_reflow",
)}

// TODO(pcwalton): Support `text-justify: distribute`.
<%helpers:single_keyword
    name="text-justify"
    values="auto none inter-word"
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    extra_gecko_values="inter-character"
    extra_specified="${'distribute' if engine == 'gecko' else ''}"
    gecko_enum_prefix="StyleTextJustify"
    animation_value_type="discrete"
    gecko_pref="layout.css.text-justify.enabled"
    has_effect_on_gecko_scrollbars="False"
    spec="https://drafts.csswg.org/css-text/#propdef-text-justify"
    servo_restyle_damage="rebuild_and_reflow"
>
    % if engine == 'gecko':
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _: &Context) -> computed_value::T {
            match *self {
                % for value in "Auto None InterCharacter InterWord".split():
                SpecifiedValue::${value} => computed_value::T::${value},
                % endfor
                // https://drafts.csswg.org/css-text-3/#valdef-text-justify-distribute
                SpecifiedValue::Distribute => computed_value::T::InterCharacter,
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> SpecifiedValue {
            match *computed {
                % for value in "Auto None InterCharacter InterWord".split():
                computed_value::T::${value} => SpecifiedValue::${value},
                % endfor
            }
        }
    }
    % endif
</%helpers:single_keyword>

${helpers.predefined_type(
    "text-align-last",
    "TextAlignLast",
    "computed::text::TextAlignLast::Auto",
    needs_context=False,
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-text-align-last",
)}

// TODO make this a shorthand and implement text-align-last/text-align-all
${helpers.predefined_type(
    "text-align",
    "TextAlign",
    "computed::TextAlign::Start",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text/#propdef-text-align",
    servo_restyle_damage = "reflow",
)}

${helpers.predefined_type(
    "letter-spacing",
    "LetterSpacing",
    "computed::LetterSpacing::normal()",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-text/#propdef-letter-spacing",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "word-spacing",
    "WordSpacing",
    "computed::WordSpacing::zero()",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-text/#propdef-word-spacing",
    servo_restyle_damage="rebuild_and_reflow",
)}

<%helpers:single_keyword
    name="white-space"
    values="normal pre nowrap pre-wrap pre-line"
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    extra_gecko_values="break-spaces -moz-pre-space"
    gecko_enum_prefix="StyleWhiteSpace"
    needs_conversion="True"
    animation_value_type="discrete"
    spec="https://drafts.csswg.org/css-text/#propdef-white-space"
    servo_restyle_damage="rebuild_and_reflow"
>
    % if engine in ["servo-2013", "servo-2020"]:
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
    engines="gecko servo-2013",
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
    "computed::TextEmphasisPosition::over_right()",
    engines="gecko",
    initial_specified_value="specified::TextEmphasisPosition::over_right()",
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
    "-moz-tab-size",
    "NonNegativeLengthOrNumber",
    "generics::length::LengthOrNumber::Number(From::from(8.0))",
    engines="gecko",
    animation_value_type="LengthOrNumber",
    spec="https://drafts.csswg.org/css-text-3/#tab-size-property",
)}

${helpers.predefined_type(
    "line-break",
    "LineBreak",
    "computed::LineBreak::Auto",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text-3/#line-break-property",
    needs_context=False,
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
    "BorderSideWidth",
    "crate::values::computed::NonNegativeLength::new(0.)",
    engines="gecko",
    initial_specified_value="specified::BorderSideWidth::zero()",
    computed_type="crate::values::computed::NonNegativeLength",
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

${helpers.single_keyword(
    "ruby-position",
    "over under",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-ruby/#ruby-position-property",
    gecko_enum_prefix="StyleRubyPosition",
)}

// CSS Writing Modes Module Level 3
// https://drafts.csswg.org/css-writing-modes-3/

${helpers.single_keyword(
    "text-combine-upright",
    "none all",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-writing-modes-3/#text-combine-upright",
)}

// SVG 1.1: Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword(
    "text-rendering",
    "auto optimizespeed optimizelegibility geometricprecision",
    engines="gecko servo-2013 servo-2020",
    gecko_enum_prefix="StyleTextRendering",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/painting.html#TextRenderingProperty",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "-moz-control-character-visibility",
    "hidden visible",
    engines="gecko",
    gecko_enum_prefix="StyleControlCharacterVisibility",
    gecko_pref_controlled_initial_value="layout.css.control-characters.visible=visible",
    animation_value_type="none",
    gecko_ffi_name="mControlCharacterVisibility",
    spec="Nonstandard",
)}

// text underline offset
${helpers.predefined_type(
    "text-underline-offset",
    "LengthPercentageOrAuto",
    "computed::LengthPercentageOrAuto::auto()",
    engines="gecko",
    animation_value_type="ComputedValue",
    gecko_pref="layout.css.text-underline-offset.enabled",
    has_effect_on_gecko_scrollbars=False,
    spec="https://drafts.csswg.org/css-text-decor-4/#underline-offset",
)}

// text underline position
${helpers.predefined_type(
    "text-underline-position",
    "TextUnderlinePosition",
    "computed::TextUnderlinePosition::AUTO",
    engines="gecko",
    animation_value_type="discrete",
    gecko_pref="layout.css.text-underline-position.enabled",
    has_effect_on_gecko_scrollbars=False,
    spec="https://drafts.csswg.org/css-text-decor-3/#text-underline-position-property",
)}

// text decoration skip ink
${helpers.predefined_type(
    "text-decoration-skip-ink",
    "TextDecorationSkipInk",
    "computed::TextDecorationSkipInk::Auto",
    engines="gecko",
    needs_context=False,
    animation_value_type="discrete",
    gecko_pref="layout.css.text-decoration-skip-ink.enabled",
    has_effect_on_gecko_scrollbars=False,
    spec="https://drafts.csswg.org/css-text-decor-4/#text-decoration-skip-ink-property",
)}
