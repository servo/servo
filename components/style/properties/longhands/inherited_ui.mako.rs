/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedUI", inherited=True, gecko_name="UI") %>

${helpers.predefined_type(
    "cursor",
    "Cursor",
    "computed::Cursor::auto()",
    engines="gecko servo-2013 servo-2020",
    initial_specified_value="specified::Cursor::auto()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-ui/#cursor",
)}

// NB: `pointer-events: auto` (and use of `pointer-events` in anything that isn't SVG, in fact)
// is nonstandard, slated for CSS4-UI.
// TODO(pcwalton): SVG-only values.
${helpers.single_keyword(
    "pointer-events",
    "auto none",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="discrete",
    extra_gecko_values="visiblepainted visiblefill visiblestroke visible painted fill stroke all",
    spec="https://www.w3.org/TR/SVG11/interact.html#PointerEventsProperty",
    gecko_enum_prefix="StylePointerEvents",
)}

${helpers.single_keyword(
    "-moz-inert",
    "none inert",
    engines="gecko",
    gecko_ffi_name="mInert",
    gecko_enum_prefix="StyleInert",
    animation_value_type="discrete",
    enabled_in="ua",
    spec="Nonstandard (https://html.spec.whatwg.org/multipage/#inert-subtrees)",
)}

${helpers.single_keyword(
    "-moz-user-input",
    "auto none",
    engines="gecko",
    gecko_ffi_name="mUserInput",
    gecko_enum_prefix="StyleUserInput",
    animation_value_type="discrete",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-user-input)",
)}

${helpers.single_keyword(
    "-moz-user-modify",
    "read-only read-write write-only",
    engines="gecko",
    gecko_ffi_name="mUserModify",
    gecko_enum_prefix="StyleUserModify",
    needs_conversion=True,
    animation_value_type="discrete",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-user-modify)",
)}

${helpers.single_keyword(
    "-moz-user-focus",
    "none ignore normal select-after select-before select-menu select-same select-all",
    engines="gecko",
    gecko_ffi_name="mUserFocus",
    gecko_enum_prefix="StyleUserFocus",
    animation_value_type="discrete",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-user-focus)",
)}

${helpers.predefined_type(
    "caret-color",
    "color::CaretColor",
    "generics::color::CaretColor::auto()",
    engines="gecko",
    spec="https://drafts.csswg.org/css-ui/#caret-color",
    animation_value_type="CaretColor",
    boxed=True,
    ignored_when_colors_disabled=True,
)}

${helpers.predefined_type(
    "accent-color",
    "ColorOrAuto",
    "generics::color::ColorOrAuto::Auto",
    engines="gecko",
    spec="https://drafts.csswg.org/css-ui-4/#widget-accent",
    gecko_pref="layout.css.accent-color.enabled",
    animation_value_type="ColorOrAuto",
    boxed=True,
    ignored_when_colors_disabled=True,
    has_effect_on_gecko_scrollbars=False,
)}

${helpers.predefined_type(
    "scrollbar-color",
    "ui::ScrollbarColor",
    "Default::default()",
    engines="gecko",
    spec="https://drafts.csswg.org/css-scrollbars-1/#scrollbar-color",
    animation_value_type="ScrollbarColor",
    boxed=True,
    ignored_when_colors_disabled=True,
)}
