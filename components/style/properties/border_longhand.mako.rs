<%page args="helpers" />

${helpers.new_style_struct("Border", is_inherited=False, gecko_name="nsStyleBorder",
                           additional_methods=[helpers.new_method("border_" + side + "_is_none_or_hidden_and_has_nonzero_width",
                                                                  "bool") for side in ["top", "right", "bottom", "left"]])}

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("border-%s-color" % side, "CSSColor", "::cssparser::Color::CurrentColor")}
% endfor

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("border-%s-style" % side, "BorderStyle", "specified::BorderStyle::none")}
% endfor

% for side in ["top", "right", "bottom", "left"]:
    <%helpers:longhand name="border-${side}-width">
        use app_units::Au;
        use cssparser::ToCss;
        use std::fmt;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        #[inline]
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                               -> Result<SpecifiedValue, ()> {
            specified::parse_border_width(input).map(SpecifiedValue)
        }
        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(pub specified::Length);
        pub mod computed_value {
            use app_units::Au;
            pub type T = Au;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(3)  // medium
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                self.0.to_computed_value(context)
            }
        }
    </%helpers:longhand>
% endfor

// FIXME(#4126): when gfx supports painting it, make this Size2D<LengthOrPercentage>
% for corner in ["top-left", "top-right", "bottom-right", "bottom-left"]:
    ${helpers.predefined_type("border-" + corner + "-radius", "BorderRadiusSize",
                              "computed::BorderRadiusSize::zero()",
                              "parse")}
% endfor
