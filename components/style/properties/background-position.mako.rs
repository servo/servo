<%page args="helpers"/>

<%helpers:longhand name="background-position">
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;

        pub mod computed_value {
            use values::computed::LengthOrPercentage;

            #[derive(PartialEq, Copy, Clone, Debug, HeapSizeOf)]
            pub struct T {
                pub horizontal: LengthOrPercentage,
                pub vertical: LengthOrPercentage,
            }
        }

        #[derive(Debug, Clone, PartialEq, Copy, HeapSizeOf)]
        pub struct SpecifiedValue {
            pub horizontal: specified::LengthOrPercentage,
            pub vertical: specified::LengthOrPercentage,
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.vertical.to_css(dest));
                Ok(())
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.vertical.to_css(dest));
                Ok(())
            }
        }

        impl SpecifiedValue {
            fn new(first: specified::PositionComponent, second: specified::PositionComponent)
                    -> Result<SpecifiedValue, ()> {
                let (horiz, vert) = match (category(first), category(second)) {
                    // Don't allow two vertical keywords or two horizontal keywords.
                    (PositionCategory::HorizontalKeyword, PositionCategory::HorizontalKeyword) |
                    (PositionCategory::VerticalKeyword, PositionCategory::VerticalKeyword) => return Err(()),

                    // Swap if both are keywords and vertical precedes horizontal.
                    (PositionCategory::VerticalKeyword, PositionCategory::HorizontalKeyword) |
                    (PositionCategory::VerticalKeyword, PositionCategory::OtherKeyword) |
                    (PositionCategory::OtherKeyword, PositionCategory::HorizontalKeyword) => (second, first),

                    // By default, horizontal is first.
                    _ => (first, second),
                };
                Ok(SpecifiedValue {
                    horizontal: horiz.to_length_or_percentage(),
                    vertical: vert.to_length_or_percentage(),
                })
            }
        }

        // Collapse `Position` into a few categories to simplify the above `match` expression.
        enum PositionCategory {
            HorizontalKeyword,
            VerticalKeyword,
            OtherKeyword,
            LengthOrPercentage,
        }
        fn category(p: specified::PositionComponent) -> PositionCategory {
            match p {
                specified::PositionComponent::Left |
                specified::PositionComponent::Right =>
                    PositionCategory::HorizontalKeyword,
                specified::PositionComponent::Top |
                specified::PositionComponent::Bottom =>
                    PositionCategory::VerticalKeyword,
                specified::PositionComponent::Center =>
                    PositionCategory::OtherKeyword,
                specified::PositionComponent::LengthOrPercentage(_) =>
                    PositionCategory::LengthOrPercentage,
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                computed_value::T {
                    horizontal: self.horizontal.to_computed_value(context),
                    vertical: self.vertical.to_computed_value(context),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T {
                horizontal: computed::LengthOrPercentage::Percentage(0.0),
                vertical: computed::LengthOrPercentage::Percentage(0.0),
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            let first = try!(specified::PositionComponent::parse(input));
            let second = input.try(specified::PositionComponent::parse)
                .unwrap_or(specified::PositionComponent::Center);
            SpecifiedValue::new(first, second)
        }
</%helpers:longhand>
