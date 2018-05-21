/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

// CSS Basic User Interface Module Level 1
// https://drafts.csswg.org/css-ui-3/
<% data.new_style_struct("UI", inherited=False, gecko_name="UIReset") %>

// TODO spec says that UAs should not support this
// we should probably remove from gecko (https://bugzilla.mozilla.org/show_bug.cgi?id=1328331)
${helpers.single_keyword("ime-mode", "auto normal active disabled inactive",
                         products="gecko", gecko_ffi_name="mIMEMode",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-ui/#input-method-editor")}

${helpers.single_keyword("-moz-user-select", "auto text none all element elements" +
                            " toggle tri-state -moz-all -moz-text",
                         products="gecko",
                         alias="-webkit-user-select",
                         gecko_ffi_name="mUserSelect",
                         gecko_enum_prefix="StyleUserSelect",
                         gecko_strip_moz_prefix=False,
                         aliases="-moz-none=none",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-ui-4/#propdef-user-select")}

// TODO(emilio): This probably should be hidden from content.
${helpers.single_keyword("-moz-window-dragging", "default drag no-drag", products="gecko",
                         gecko_ffi_name="mWindowDragging",
                         gecko_enum_prefix="StyleWindowDragging",
                         animation_value_type="discrete",
                         spec="None (Nonstandard Firefox-only property)")}

${helpers.single_keyword("-moz-window-shadow", "none default menu tooltip sheet", products="gecko",
                         gecko_ffi_name="mWindowShadow",
                         gecko_constant_prefix="NS_STYLE_WINDOW_SHADOW",
                         animation_value_type="discrete",
                         enabled_in="chrome",
                         spec="None (Nonstandard internal property)")}

// TODO(bug 1419695) This should be hidden from content.
${helpers.predefined_type("-moz-window-opacity", "Opacity", "1.0", products="gecko",
                          gecko_ffi_name="mWindowOpacity",
                          animation_value_type="ComputedValue",
                          spec="None (Nonstandard internal property)")}

// TODO(bug 1419695) This should be hidden from content.
${helpers.predefined_type(
    "-moz-window-transform",
    "Transform",
    "generics::transform::Transform::none()",
    products="gecko",
    gecko_ffi_name="mSpecifiedWindowTransform",
    flags="GETCS_NEEDS_LAYOUT_FLUSH",
    animation_value_type="ComputedValue",
    spec="None (Nonstandard internal property)"
)}

// TODO(bug 1419695) This should be hidden from content.
${helpers.predefined_type(
    "-moz-window-transform-origin",
    "TransformOrigin",
    "computed::TransformOrigin::initial_value()",
    animation_value_type="ComputedValue",
    gecko_ffi_name="mWindowTransformOrigin",
    products="gecko",
    boxed=True,
    flags="GETCS_NEEDS_LAYOUT_FLUSH",
    spec="None (Nonstandard internal property)"
)}

// TODO(emilio): Probably also should be hidden from content.
${helpers.predefined_type("-moz-force-broken-image-icon",
                          "MozForceBrokenImageIcon",
                          "computed::MozForceBrokenImageIcon::false_value()",
                          animation_value_type="discrete",
                          products="gecko",
                          spec="None (Nonstandard Firefox-only property)")}
