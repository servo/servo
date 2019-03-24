/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("List", inherited=True) %>

${helpers.single_keyword("list-style-position", "outside inside", animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-lists/#propdef-list-style-position",
                         servo_restyle_damage="rebuild_and_reflow")}

// TODO(pcwalton): Implement the full set of counter styles per CSS-COUNTER-STYLES [1] 6.1:
//
//     decimal-leading-zero, armenian, upper-armenian, lower-armenian, georgian, lower-roman,
//     upper-roman
//
// [1]: http://dev.w3.org/csswg/css-counter-styles/
% if product == "servo":
    ${helpers.single_keyword(
        "list-style-type",
        """disc none circle square decimal disclosure-open disclosure-closed lower-alpha upper-alpha
        arabic-indic bengali cambodian cjk-decimal devanagari gujarati gurmukhi kannada khmer lao
        malayalam mongolian myanmar oriya persian telugu thai tibetan cjk-earthly-branch
        cjk-heavenly-stem lower-greek hiragana hiragana-iroha katakana katakana-iroha""",
        animation_value_type="discrete",
        spec="https://drafts.csswg.org/css-lists/#propdef-list-style-type",
        servo_restyle_damage="rebuild_and_reflow",
    )}
% else:
    ${helpers.predefined_type(
        "list-style-type",
        "ListStyleType",
        "computed::ListStyleType::disc()",
        initial_specified_value="specified::ListStyleType::disc()",
        animation_value_type="discrete",
        boxed=True,
        spec="https://drafts.csswg.org/css-lists/#propdef-list-style-type",
        servo_restyle_damage="rebuild_and_reflow",
    )}
% endif

${helpers.predefined_type(
    "list-style-image",
    "url::ImageUrlOrNone",
    initial_value="computed::url::ImageUrlOrNone::none()",
    initial_specified_value="specified::url::ImageUrlOrNone::none()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-lists/#propdef-list-style-image",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "quotes",
    "Quotes",
    "computed::Quotes::get_initial_value()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-content/#propdef-quotes",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "-moz-image-region",
    "ClipRectOrAuto",
    "computed::ClipRectOrAuto::auto()",
    animation_value_type="ComputedValue",
    products="gecko",
    boxed=True,
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-image-region)",
)}

${helpers.predefined_type(
    "-moz-list-reversed",
    "MozListReversed",
    "computed::MozListReversed::False",
    animation_value_type="discrete",
    products="gecko",
    enabled_in="ua",
    needs_context=False,
    spec="Internal implementation detail for <ol reversed>",
    servo_restyle_damage="rebuild_and_reflow",
)}
