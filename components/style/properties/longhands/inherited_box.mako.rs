/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedBox", inherited=True, gecko_name="Visibility") %>

// TODO: collapse. Well, do tables first.
${helpers.single_keyword(
    "visibility",
    "visible hidden",
    extra_gecko_values="collapse",
    gecko_ffi_name="mVisible",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-box/#propdef-visibility",
)}

// CSS Writing Modes Level 3
// https://drafts.csswg.org/css-writing-modes-3
${helpers.single_keyword(
    "writing-mode",
    "horizontal-tb vertical-rl vertical-lr",
    extra_gecko_values="sideways-rl sideways-lr",
    extra_gecko_aliases="lr=horizontal-tb lr-tb=horizontal-tb \
                         rl=horizontal-tb rl-tb=horizontal-tb \
                         tb=vertical-rl   tb-rl=vertical-rl",
    servo_pref="layout.writing-mode.enabled",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes/#propdef-writing-mode",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "direction",
    "ltr rtl",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes/#propdef-direction",
    needs_conversion=True,
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "text-orientation",
    "mixed upright sideways",
    extra_gecko_aliases="sideways-right=sideways",
    products="gecko",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes/#propdef-text-orientation",
)}

// CSS Color Module Level 4
// https://drafts.csswg.org/css-color/
${helpers.single_keyword(
    "color-adjust",
    "economy exact", products="gecko",
    gecko_pref="layout.css.color-adjust.enabled",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-color/#propdef-color-adjust",
)}

// According to to CSS-IMAGES-3, `optimizespeed` and `optimizequality` are synonyms for `auto`
// And, firefox doesn't support `pixelated` yet (https://bugzilla.mozilla.org/show_bug.cgi?id=856337)
${helpers.single_keyword(
    "image-rendering",
    "auto crisp-edges",
    extra_gecko_values="optimizespeed optimizequality",
    extra_servo_values="pixelated",
    extra_gecko_aliases="-moz-crisp-edges=crisp-edges",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-images/#propdef-image-rendering",
)}

${helpers.single_keyword(
    "image-orientation",
    "none from-image",
    products="gecko",
    gecko_enum_prefix="StyleImageOrientation",
    animation_value_type="discrete",
    gecko_pref="layout.css.image-orientation.enabled",
    spec="https://drafts.csswg.org/css-images/#propdef-image-orientation",
)}
