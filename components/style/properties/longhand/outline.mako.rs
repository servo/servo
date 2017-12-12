/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Outline",
                         inherited=False,
                         additional_methods=[Method("outline_has_nonzero_width", "bool")]) %>

// TODO(pcwalton): `invert`
${helpers.predefined_type(
    "outline-color",
    "Color",
    "computed_value::T::currentcolor()",
    initial_specified_value="specified::Color::currentcolor()",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    spec="https://drafts.csswg.org/css-ui/#propdef-outline-color",
)}

<%helpers:longhand name="outline-style" animation_value_type="discrete"
                   spec="https://drafts.csswg.org/css-ui/#propdef-outline-style">
    use values::specified::BorderStyle;

    pub type SpecifiedValue = Either<Auto, BorderStyle>;

    impl SpecifiedValue {
        #[inline]
        pub fn none_or_hidden(&self) -> bool {
            match *self {
                Either::First(ref _auto) => false,
                Either::Second(ref border_style) => border_style.none_or_hidden()
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        Either::Second(BorderStyle::None)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        Either::Second(BorderStyle::None)
    }

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        SpecifiedValue::parse(context, input)
            .and_then(|result| {
                if let Either::Second(BorderStyle::Hidden) = result {
                    // The outline-style property accepts the same values as
                    // border-style, except that 'hidden' is not a legal outline
                    // style.
                    Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent("hidden".into())))
                } else {
                    Ok(result)
                }
            })
    }
</%helpers:longhand>

${helpers.predefined_type("outline-width",
                          "BorderSideWidth",
                          "::values::computed::NonNegativeLength::new(3.)",
                          initial_specified_value="specified::BorderSideWidth::Medium",
                          computed_type="::values::computed::NonNegativeLength",
                          animation_value_type="NonNegativeLength",
                          spec="https://drafts.csswg.org/css-ui/#propdef-outline-width")}

// The -moz-outline-radius-* properties are non-standard and not on a standards track.
% for corner in ["topleft", "topright", "bottomright", "bottomleft"]:
    ${helpers.predefined_type("-moz-outline-radius-" + corner, "BorderCornerRadius",
        "computed::BorderCornerRadius::zero()",
        products="gecko",
        boxed=True,
        animation_value_type="BorderCornerRadius",
        spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-outline-radius)")}
% endfor

${helpers.predefined_type("outline-offset", "Length", "::values::computed::Length::new(0.)",
                          products="servo gecko", animation_value_type="ComputedValue",
                          spec="https://drafts.csswg.org/css-ui/#propdef-outline-offset")}
