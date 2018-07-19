/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, PHYSICAL_SIDES, ALL_SIDES, maybe_moz_logical_alias %>

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
        alias=maybe_moz_logical_alias(product, side, "-moz-border-%s-color"),
        spec=maybe_logical_spec(side, "color"),
        animation_value_type="AnimatedColor",
        logical=is_logical,
        logical_group="border-color",
        allow_quirks=not is_logical,
        flags="APPLIES_TO_FIRST_LETTER",
        ignored_when_colors_disabled=True,
    )}

    ${helpers.predefined_type(
        "border-%s-style" % side_name, "BorderStyle",
        "specified::BorderStyle::None",
        alias=maybe_moz_logical_alias(product, side, "-moz-border-%s-style"),
        spec=maybe_logical_spec(side, "style"),
        flags="APPLIES_TO_FIRST_LETTER",
        animation_value_type="discrete" if not is_logical else "none",
        logical=is_logical,
        logical_group="border-style",
        needs_context=False,
    )}

    ${helpers.predefined_type(
        "border-%s-width" % side_name,
        "BorderSideWidth",
        "::values::computed::NonNegativeLength::new(3.)",
        computed_type="::values::computed::NonNegativeLength",
        alias=maybe_moz_logical_alias(product, side, "-moz-border-%s-width"),
        spec=maybe_logical_spec(side, "width"),
        animation_value_type="NonNegativeLength",
        logical=is_logical,
        logical_group="border-width",
        flags="APPLIES_TO_FIRST_LETTER GETCS_NEEDS_LAYOUT_FLUSH",
        allow_quirks=not is_logical,
        servo_restyle_damage="reflow rebuild_and_reflow_inline"
    )}
% endfor

${helpers.gecko_keyword_conversion(Keyword('border-style',
                                   "none solid double dotted dashed hidden groove ridge inset outset"),
                                   type="::values::specified::BorderStyle")}

// FIXME(#4126): when gfx supports painting it, make this Size2D<LengthOrPercentage>
% for corner in ["top-left", "top-right", "bottom-right", "bottom-left"]:
    ${helpers.predefined_type("border-" + corner + "-radius", "BorderCornerRadius",
                              "computed::BorderCornerRadius::zero()",
                              "parse", extra_prefixes="webkit",
                              spec="https://drafts.csswg.org/css-backgrounds/#border-%s-radius" % corner,
                              boxed=True,
                              flags="APPLIES_TO_FIRST_LETTER",
                              animation_value_type="BorderCornerRadius")}
% endfor

${helpers.single_keyword("box-decoration-break", "slice clone",
                         gecko_enum_prefix="StyleBoxDecorationBreak",
                         gecko_pref="layout.css.box-decoration-break.enabled",
                         spec="https://drafts.csswg.org/css-break/#propdef-box-decoration-break",
                         products="gecko", animation_value_type="discrete")}

${helpers.single_keyword("-moz-float-edge", "content-box margin-box",
                         gecko_ffi_name="mFloatEdge",
                         gecko_enum_prefix="StyleFloatEdge",
                         products="gecko",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-float-edge)",
                         animation_value_type="discrete")}

${helpers.predefined_type("border-image-source", "ImageLayer",
    initial_value="Either::First(None_)",
    initial_specified_value="Either::First(None_)",
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-image",
    vector=False,
    animation_value_type="discrete",
    flags="APPLIES_TO_FIRST_LETTER",
    boxed=True)}

${helpers.predefined_type("border-image-outset", "LengthOrNumberRect",
    parse_method="parse_non_negative",
    initial_value="computed::LengthOrNumberRect::all(computed::LengthOrNumber::zero())",
    initial_specified_value="specified::LengthOrNumberRect::all(specified::LengthOrNumber::zero())",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-outset",
    animation_value_type="discrete",
    flags="APPLIES_TO_FIRST_LETTER",
    boxed=True)}

${helpers.predefined_type(
    "border-image-repeat",
    "BorderImageRepeat",
    "computed::BorderImageRepeat::stretch()",
    initial_specified_value="specified::BorderImageRepeat::stretch()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-backgrounds/#the-border-image-repeat",
    flags="APPLIES_TO_FIRST_LETTER",
)}

${helpers.predefined_type("border-image-width", "BorderImageWidth",
    initial_value="computed::BorderImageWidth::all(computed::BorderImageSideWidth::one())",
    initial_specified_value="specified::BorderImageWidth::all(specified::BorderImageSideWidth::one())",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-width",
    animation_value_type="discrete",
    flags="APPLIES_TO_FIRST_LETTER",
    boxed=True)}

${helpers.predefined_type("border-image-slice", "BorderImageSlice",
    initial_value="computed::NumberOrPercentage::Percentage(computed::Percentage(1.)).into()",
    initial_specified_value="specified::NumberOrPercentage::Percentage(specified::Percentage::new(1.)).into()",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-slice",
    animation_value_type="discrete",
    flags="APPLIES_TO_FIRST_LETTER",
    boxed=True)}

#[cfg(feature = "gecko")]
impl ::values::computed::BorderImageWidth {
    pub fn to_gecko_rect(&self, sides: &mut ::gecko_bindings::structs::nsStyleSides) {
        use gecko_bindings::sugar::ns_style_coord::{CoordDataMut, CoordDataValue};
        use gecko::values::GeckoStyleCoordConvertible;
        use values::generics::border::BorderImageSideWidth;

        % for i in range(0, 4):
        match self.${i} {
            BorderImageSideWidth::Auto => {
                sides.data_at_mut(${i}).set_value(CoordDataValue::Auto)
            },
            BorderImageSideWidth::Length(l) => {
                l.to_gecko_style_coord(&mut sides.data_at_mut(${i}))
            },
            BorderImageSideWidth::Number(n) => {
                sides.data_at_mut(${i}).set_value(CoordDataValue::Factor(n))
            },
        }
        % endfor
    }

    pub fn from_gecko_rect(sides: &::gecko_bindings::structs::nsStyleSides)
                           -> Option<::values::computed::BorderImageWidth> {
        use gecko_bindings::structs::nsStyleUnit::{eStyleUnit_Factor, eStyleUnit_Auto};
        use gecko_bindings::sugar::ns_style_coord::CoordData;
        use gecko::values::GeckoStyleCoordConvertible;
        use values::computed::{LengthOrPercentage, Number};
        use values::generics::border::BorderImageSideWidth;

        Some(
            ::values::computed::BorderImageWidth::new(
                % for i in range(0, 4):
                match sides.data_at(${i}).unit() {
                    eStyleUnit_Auto => {
                        BorderImageSideWidth::Auto
                    },
                    eStyleUnit_Factor => {
                        BorderImageSideWidth::Number(
                            Number::from_gecko_style_coord(&sides.data_at(${i}))
                                .expect("sides[${i}] could not convert to Number"))
                    },
                    _ => {
                        BorderImageSideWidth::Length(
                            LengthOrPercentage::from_gecko_style_coord(&sides.data_at(${i}))
                                .expect("sides[${i}] could not convert to LengthOrPercentager"))
                    },
                },
                % endfor
            )
        )
    }
}
