/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, ALL_CORNERS, PHYSICAL_SIDES, ALL_SIDES, maybe_moz_logical_alias %>

<% data.new_style_struct("Border", inherited=False,
                   additional_methods=[Method("border_" + side + "_has_nonzero_width",
                                              "bool") for side in ["top", "right", "bottom", "left"]]) %>
<%
    def maybe_logical_spec(side, kind):
        if side[1]: # if it is logical
            return "https://drafts.csswg.org/css-logical-props/#propdef-border-%s-%s" % (side[0], kind)
        else:
            return "https://drafts.csswg.org/css-backgrounds/#border-%s-%s" % (side[0], kind)
%>
% for side in ALL_SIDES:
    <%
        side_name = side[0]
        is_logical = side[1]
    %>
    ${helpers.predefined_type(
        "border-%s-color" % side_name, "Color",
        "computed_value::T::currentcolor()",
        engines="gecko servo-2013 servo-2020",
        servo_2020_pref="layout.2020.unimplemented",
        alias=maybe_moz_logical_alias(engine, side, "-moz-border-%s-color"),
        spec=maybe_logical_spec(side, "color"),
        animation_value_type="AnimatedColor",
        logical=is_logical,
        logical_group="border-color",
        allow_quirks="No" if is_logical else "Yes",
        ignored_when_colors_disabled=True,
    )}

    ${helpers.predefined_type(
        "border-%s-style" % side_name, "BorderStyle",
        "specified::BorderStyle::None",
        engines="gecko servo-2013 servo-2020",
        alias=maybe_moz_logical_alias(engine, side, "-moz-border-%s-style"),
        spec=maybe_logical_spec(side, "style"),
        animation_value_type="discrete" if not is_logical else "none",
        logical=is_logical,
        logical_group="border-style",
        needs_context=False,
    )}

    ${helpers.predefined_type(
        "border-%s-width" % side_name,
        "BorderSideWidth",
        "crate::values::computed::NonNegativeLength::new(3.)",
        engines="gecko servo-2013 servo-2020",
        servo_2020_pref="layout.2020.unimplemented",
        computed_type="crate::values::computed::NonNegativeLength",
        alias=maybe_moz_logical_alias(engine, side, "-moz-border-%s-width"),
        spec=maybe_logical_spec(side, "width"),
        animation_value_type="NonNegativeLength",
        logical=is_logical,
        logical_group="border-width",
        allow_quirks="No" if is_logical else "Yes",
        servo_restyle_damage="reflow rebuild_and_reflow_inline"
    )}
% endfor

% for corner in ALL_CORNERS:
    <%
        corner_name = corner[0]
        is_logical = corner[1]
        if is_logical:
            prefixes = None
        else:
            prefixes = "webkit"
    %>
    ${helpers.predefined_type(
        "border-%s-radius" % corner_name,
        "BorderCornerRadius",
        "computed::BorderCornerRadius::zero()",
        "parse",
        engines="gecko servo-2013",
        extra_prefixes=prefixes,
        spec=maybe_logical_spec(corner, "radius"),
        boxed=True,
        animation_value_type="BorderCornerRadius",
        logical_group="border-radius",
        logical=is_logical,
    )}
% endfor

${helpers.single_keyword(
    "box-decoration-break",
    "slice clone",
    engines="gecko",
    gecko_enum_prefix="StyleBoxDecorationBreak",
    spec="https://drafts.csswg.org/css-break/#propdef-box-decoration-break",
    animation_value_type="discrete",
)}

${helpers.single_keyword(
    "-moz-float-edge",
    "content-box margin-box",
    engines="gecko",
    gecko_ffi_name="mFloatEdge",
    gecko_enum_prefix="StyleFloatEdge",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-float-edge)",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "border-image-source",
    "ImageLayer",
    engines="gecko servo-2013",
    initial_value="computed::ImageLayer::none()",
    initial_specified_value="specified::ImageLayer::none()",
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-image",
    vector=False,
    animation_value_type="discrete",
    boxed=engine == "servo-2013",
    ignored_when_colors_disabled=True
)}

${helpers.predefined_type(
    "border-image-outset",
    "NonNegativeLengthOrNumberRect",
    engines="gecko servo-2013",
    initial_value="generics::rect::Rect::all(computed::NonNegativeLengthOrNumber::zero())",
    initial_specified_value="generics::rect::Rect::all(specified::NonNegativeLengthOrNumber::zero())",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-outset",
    animation_value_type="discrete",
    boxed=True,
)}

${helpers.predefined_type(
    "border-image-repeat",
    "BorderImageRepeat",
    "computed::BorderImageRepeat::stretch()",
    engines="gecko servo-2013",
    initial_specified_value="specified::BorderImageRepeat::stretch()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-backgrounds/#the-border-image-repeat",
)}

${helpers.predefined_type(
    "border-image-width",
    "BorderImageWidth",
    engines="gecko servo-2013",
    initial_value="computed::BorderImageWidth::all(computed::BorderImageSideWidth::one())",
    initial_specified_value="specified::BorderImageWidth::all(specified::BorderImageSideWidth::one())",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-width",
    animation_value_type="discrete",
    boxed=True,
)}

${helpers.predefined_type(
    "border-image-slice",
    "BorderImageSlice",
    engines="gecko servo-2013",
    initial_value="computed::BorderImageSlice::hundred_percent()",
    initial_specified_value="specified::BorderImageSlice::hundred_percent()",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-slice",
    animation_value_type="discrete",
    boxed=True,
)}
