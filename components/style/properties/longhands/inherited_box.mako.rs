/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedBox", inherited=True, gecko_name="Visibility") %>

// TODO: collapse. Well, do tables first.
${helpers.single_keyword(
    "visibility",
    "visible hidden collapse",
    engines="gecko servo-2013 servo-2020",
    gecko_ffi_name="mVisible",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-box/#propdef-visibility",
    gecko_enum_prefix="StyleVisibility",
)}

// CSS Writing Modes Level 3
// https://drafts.csswg.org/css-writing-modes-3
${helpers.single_keyword(
    "writing-mode",
    "horizontal-tb vertical-rl vertical-lr",
    engines="gecko servo-2013 servo-2020",
    extra_gecko_values="sideways-rl sideways-lr",
    gecko_aliases="lr=horizontal-tb lr-tb=horizontal-tb \
                         rl=horizontal-tb rl-tb=horizontal-tb \
                         tb=vertical-rl   tb-rl=vertical-rl",
    servo_2013_pref="layout.writing-mode.enabled",
    servo_2020_pref="layout.writing-mode.enabled",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes/#propdef-writing-mode",
    gecko_enum_prefix="StyleWritingModeProperty",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "direction",
    "ltr rtl",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes/#propdef-direction",
    gecko_enum_prefix="StyleDirection",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "text-orientation",
    "mixed upright sideways",
    engines="gecko",
    gecko_aliases="sideways-right=sideways",
    gecko_enum_prefix="StyleTextOrientation",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes/#propdef-text-orientation",
)}

${helpers.predefined_type(
    "print-color-adjust",
    "PrintColorAdjust",
    "computed::PrintColorAdjust::Economy",
    engines="gecko",
    aliases="color-adjust",
    spec="https://drafts.csswg.org/css-color-adjust/#print-color-adjust",
    animation_value_type="discrete",
)}

// According to to CSS-IMAGES-3, `optimizespeed` and `optimizequality` are synonyms for `auto`
// And, firefox doesn't support `pixelated` yet (https://bugzilla.mozilla.org/show_bug.cgi?id=856337)
${helpers.predefined_type(
    "image-rendering",
    "ImageRendering",
    "computed::ImageRendering::Auto",
    engines="gecko servo-2013 servo-2020",
    spec="https://drafts.csswg.org/css-images/#propdef-image-rendering",
    animation_value_type="discrete",
)}

${helpers.single_keyword(
    "image-orientation",
    "from-image none",
    engines="gecko",
    gecko_enum_prefix="StyleImageOrientation",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-images/#propdef-image-orientation",
)}
