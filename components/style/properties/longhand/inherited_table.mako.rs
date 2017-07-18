/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedTable", inherited=True, gecko_name="TableBorder") %>

${helpers.single_keyword("border-collapse", "separate collapse",
                         gecko_constant_prefix="NS_STYLE_BORDER",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-tables/#propdef-border-collapse")}
${helpers.single_keyword("empty-cells", "show hide",
                         gecko_constant_prefix="NS_STYLE_TABLE_EMPTY_CELLS",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-tables/#propdef-empty-cells")}
${helpers.single_keyword("caption-side", "top bottom",
                         extra_gecko_values="right left top-outside bottom-outside",
                         needs_conversion="True",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-tables/#propdef-caption-side")}

<%helpers:longhand name="border-spacing" animation_value_type="ComputedValue" boxed="True"
                   spec="https://drafts.csswg.org/css-tables/#propdef-border-spacing">
    use app_units::Au;
    use values::specified::{AllowQuirks, Length};

    pub mod computed_value {
        use app_units::Au;
        use properties::animated_properties::Animatable;
        use values::animated::ToAnimatedZero;

        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        #[derive(Clone, Copy, Debug, PartialEq, ToCss)]
        pub struct T {
            pub horizontal: Au,
            pub vertical: Au,
        }

        /// https://drafts.csswg.org/css-transitions/#animtype-simple-list
        impl Animatable for T {
            #[inline]
            fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
                -> Result<Self, ()> {
                Ok(T {
                    horizontal: self.horizontal.add_weighted(&other.horizontal,
                                                             self_portion, other_portion)?,
                    vertical: self.vertical.add_weighted(&other.vertical,
                                                         self_portion, other_portion)?,
                })
            }

            #[inline]
            fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
                self.compute_squared_distance(other).map(|sd| sd.sqrt())
            }

            #[inline]
            fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
                Ok(self.horizontal.compute_squared_distance(&other.horizontal)? +
                   self.vertical.compute_squared_distance(&other.vertical)?)
            }
        }

        impl ToAnimatedZero for T {
            #[inline]
            fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
        }
    }

    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
    pub struct SpecifiedValue {
        pub horizontal: Length,
        pub vertical: Option<Length>,
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            horizontal: Au(0),
            vertical: Au(0),
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let horizontal = self.horizontal.to_computed_value(context);
            computed_value::T {
                horizontal: horizontal,
                vertical: self.vertical.as_ref().map_or(horizontal, |v| v.to_computed_value(context)),
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue {
                horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
                vertical: Some(ToComputedValue::from_computed_value(&computed.vertical)),
            }
        }
    }

    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue,ParseError<'i>> {
        let mut first = None;
        let mut second = None;
        match Length::parse_non_negative_quirky(context, input, AllowQuirks::Yes) {
            Err(_) => (),
            Ok(length) => {
                first = Some(length);
                if let Ok(len) = input.try(|i| Length::parse_non_negative_quirky(context, i, AllowQuirks::Yes)) {
                    second = Some(len);
                }
            }
        }
        match (first, second) {
            (None, None) => Err(StyleParseError::UnspecifiedError.into()),
            (Some(length), None) => {
                Ok(SpecifiedValue {
                    horizontal: length,
                    vertical: None,
                })
            }
            (Some(horizontal), Some(vertical)) => {
                Ok(SpecifiedValue {
                    horizontal: horizontal,
                    vertical: Some(vertical),
                })
            }
            (None, Some(_)) => unreachable!(),
        }
    }
</%helpers:longhand>
