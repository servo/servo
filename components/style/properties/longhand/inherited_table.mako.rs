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

<%helpers:longhand name="border-spacing" animation_value_type="BorderSpacing" boxed="True"
                   spec="https://drafts.csswg.org/css-tables/#propdef-border-spacing">
    use values::specified::{AllowQuirks, Length};
    use values::specified::length::NonNegativeLength;

    pub mod computed_value {
        use values::animated::{ToAnimatedValue, ToAnimatedZero};
        use values::computed::NonNegativeAu;

        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        #[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, PartialEq, ToCss)]
        pub struct T {
            pub horizontal: NonNegativeAu,
            pub vertical: NonNegativeAu,
        }

        impl ToAnimatedZero for T {
            #[inline]
            fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
        }

        impl ToAnimatedValue for T {
            type AnimatedValue = Self;

            #[inline]
            fn to_animated_value(self) -> Self {
                self
            }

            #[inline]
            fn from_animated_value(animated: Self::AnimatedValue) -> Self {
                T {
                    horizontal: ToAnimatedValue::from_animated_value(animated.horizontal),
                    vertical: ToAnimatedValue::from_animated_value(animated.vertical)
                }
            }
        }
    }

    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Clone, Debug, PartialEq, ToCss)]
    pub struct SpecifiedValue {
        pub horizontal: NonNegativeLength,
        pub vertical: Option<NonNegativeLength>,
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        use values::computed::NonNegativeAu;
        computed_value::T {
            horizontal: NonNegativeAu::zero(),
            vertical: NonNegativeAu::zero(),
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
                    horizontal: length.into(),
                    vertical: None,
                })
            }
            (Some(horizontal), Some(vertical)) => {
                Ok(SpecifiedValue {
                    horizontal: horizontal.into(),
                    vertical: Some(vertical.into()),
                })
            }
            (None, Some(_)) => unreachable!(),
        }
    }
</%helpers:longhand>
