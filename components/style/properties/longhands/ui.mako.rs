/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

// CSS Basic User Interface Module Level 1
// https://drafts.csswg.org/css-ui-3/
<% data.new_style_struct("UI", inherited=False, gecko_name="UIReset") %>

// TODO spec says that UAs should not support this
// we should probably remove from gecko (https://bugzilla.mozilla.org/show_bug.cgi?id=1328331)
${helpers.single_keyword(
    "ime-mode",
    "auto normal active disabled inactive",
    engines="gecko",
    gecko_enum_prefix="StyleImeMode",
    gecko_ffi_name="mIMEMode",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-ui/#input-method-editor",
)}

${helpers.single_keyword(
    "scrollbar-width",
    "auto thin none",
    engines="gecko",
    gecko_enum_prefix="StyleScrollbarWidth",
    animation_value_type="discrete",
    gecko_pref="layout.css.scrollbar-width.enabled",
    enabled_in="chrome",
    spec="https://drafts.csswg.org/css-scrollbars-1/#scrollbar-width"
)}

${helpers.predefined_type(
    "user-select",
    "UserSelect",
    "computed::UserSelect::Auto",
    engines="gecko",
    extra_prefixes="moz webkit",
    animation_value_type="discrete",
    needs_context=False,
    spec="https://drafts.csswg.org/css-ui-4/#propdef-user-select",
)}

// TODO(emilio): This probably should be hidden from content.
${helpers.single_keyword(
    "-moz-window-dragging",
    "default drag no-drag",
    engines="gecko",
    gecko_ffi_name="mWindowDragging",
    gecko_enum_prefix="StyleWindowDragging",
    animation_value_type="discrete",
    spec="None (Nonstandard Firefox-only property)",
)}

${helpers.single_keyword(
    "-moz-window-shadow",
    "default none menu tooltip sheet",
    engines="gecko",
    gecko_ffi_name="mWindowShadow",
    gecko_enum_prefix="StyleWindowShadow",
    animation_value_type="discrete",
    enabled_in="chrome",
    spec="None (Nonstandard internal property)",
)}

${helpers.predefined_type(
    "-moz-window-opacity",
    "Opacity",
    "1.0",
    engines="gecko",
    gecko_ffi_name="mWindowOpacity",
    animation_value_type="ComputedValue",
    spec="None (Nonstandard internal property)",
    enabled_in="chrome",
)}

${helpers.predefined_type(
    "-moz-window-transform",
    "Transform",
    "generics::transform::Transform::none()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="None (Nonstandard internal property)",
    enabled_in="chrome",
)}

${helpers.predefined_type(
    "-moz-window-transform-origin",
    "TransformOrigin",
    "computed::TransformOrigin::initial_value()",
    engines="gecko",
    animation_value_type="ComputedValue",
    gecko_ffi_name="mWindowTransformOrigin",
    boxed=True,
    spec="None (Nonstandard internal property)",
    enabled_in="chrome",
)}

// TODO(emilio): Probably also should be hidden from content.
${helpers.predefined_type(
    "-moz-force-broken-image-icon",
    "MozForceBrokenImageIcon",
    "computed::MozForceBrokenImageIcon::false_value()",
    engines="gecko",
    animation_value_type="discrete",
    spec="None (Nonstandard Firefox-only property)",
)}
