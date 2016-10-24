/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Border", inherited=False,
                   additional_methods=[Method("border_" + side + "_has_nonzero_width",
                                              "bool") for side in ["top", "right", "bottom", "left"]]) %>

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("border-%s-color" % side, "CSSColor",
                              "::cssparser::Color::CurrentColor",
                              animatable=True)}
% endfor

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("border-%s-style" % side, "BorderStyle",
                              "specified::BorderStyle::none",
                              need_clone=True, animatable=False)}
% endfor

% for side in ["top", "right", "bottom", "left"]:
    <%helpers:longhand name="border-${side}-width" animatable="True">
        use app_units::Au;
        use cssparser::ToCss;
        use std::fmt;
        use values::HasViewportPercentage;
        use values::specified::BorderWidth;

        pub type SpecifiedValue = BorderWidth;

        #[inline]
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            BorderWidth::parse(input)
        }

        pub mod computed_value {
            use app_units::Au;
            pub type T = Au;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(3)  // medium
        }
    </%helpers:longhand>
% endfor

// FIXME(#4126): when gfx supports painting it, make this Size2D<LengthOrPercentage>
% for corner in ["top-left", "top-right", "bottom-right", "bottom-left"]:
    ${helpers.predefined_type("border-" + corner + "-radius", "BorderRadiusSize",
                              "computed::BorderRadiusSize::zero()",
                              "parse",
                              animatable=True)}
% endfor

${helpers.single_keyword("box-decoration-break", "slice clone",
                         gecko_enum_prefix="StyleBoxDecorationBreak",
                         products="gecko", animatable=False)}

${helpers.single_keyword("-moz-float-edge", "content-box margin-box",
                         gecko_ffi_name="mFloatEdge",
                         gecko_enum_prefix="StyleFloatEdge",
                         products="gecko",
                         animatable=False)}
