/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("List", inherited=True) %>

${helpers.single_keyword(
    "list-style-position",
    "outside inside",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-lists/#propdef-list-style-position",
    servo_restyle_damage="rebuild_and_reflow",
)}

// TODO(pcwalton): Implement the full set of counter styles per CSS-COUNTER-STYLES [1] 6.1:
//
//     decimal-leading-zero, armenian, upper-armenian, lower-armenian, georgian, lower-roman,
//     upper-roman
//
// [1]: http://dev.w3.org/csswg/css-counter-styles/
% if engine in ["servo-2013", "servo-2020"]:
    ${helpers.single_keyword(
        "list-style-type",
        "disc none circle square disclosure-open disclosure-closed",
        extra_servo_2013_values="""
            decimal lower-alpha upper-alpha arabic-indic bengali cambodian cjk-decimal devanagari
            gujarati gurmukhi kannada khmer lao malayalam mongolian myanmar oriya persian telugu
            thai tibetan cjk-earthly-branch cjk-heavenly-stem lower-greek hiragana hiragana-iroha
            katakana katakana-iroha
        """,
        engines="servo-2013 servo-2020",
        animation_value_type="discrete",
        spec="https://drafts.csswg.org/css-lists/#propdef-list-style-type",
        servo_restyle_damage="rebuild_and_reflow",
    )}
% endif
% if engine == "gecko":
    ${helpers.predefined_type(
        "list-style-type",
        "ListStyleType",
        "computed::ListStyleType::disc()",
        engines="gecko",
        initial_specified_value="specified::ListStyleType::disc()",
        animation_value_type="discrete",
        boxed=True,
        spec="https://drafts.csswg.org/css-lists/#propdef-list-style-type",
        servo_restyle_damage="rebuild_and_reflow",
    )}
% endif

${helpers.predefined_type(
    "list-style-image",
    "Image",
    engines="gecko servo-2013 servo-2020",
    initial_value="computed::Image::None",
    initial_specified_value="specified::Image::None",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-lists/#propdef-list-style-image",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "quotes",
    "Quotes",
    "computed::Quotes::get_initial_value()",
    engines="gecko servo-2013",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-content/#propdef-quotes",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "-moz-image-region",
    "ClipRectOrAuto",
    "computed::ClipRectOrAuto::auto()",
    engines="gecko",
    gecko_ffi_name="mImageRegion",
    animation_value_type="ComputedValue",
    boxed=True,
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-image-region)",
)}

${helpers.predefined_type(
    "-moz-list-reversed",
    "MozListReversed",
    "computed::MozListReversed::False",
    engines="gecko",
    animation_value_type="discrete",
    enabled_in="ua",
    spec="Internal implementation detail for <ol reversed>",
    servo_restyle_damage="rebuild_and_reflow",
)}
