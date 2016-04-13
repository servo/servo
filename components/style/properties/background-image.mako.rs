<%page args="helpers"/>

<%helpers:longhand name="background-image">
    use cssparser::ToCss;
    use std::fmt;
    use values::specified::Image;
    use values::LocalToCss;

    pub mod computed_value {
        use values::computed;
        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct T(pub Option<computed::Image>);
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                None => dest.write_str("none"),
                Some(computed::Image::Url(ref url)) => url.to_css(dest),
                Some(computed::Image::LinearGradient(ref gradient)) =>
                    gradient.to_css(dest)
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
    pub struct SpecifiedValue(pub Option<Image>);

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue(Some(ref image)) => image.to_css(dest),
                SpecifiedValue(None) => dest.write_str("none"),
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(None)
    }
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            Ok(SpecifiedValue(None))
        } else {
            Ok(SpecifiedValue(Some(try!(Image::parse(context, input)))))
        }
    }
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            match *self {
                SpecifiedValue(None) => computed_value::T(None),
                SpecifiedValue(Some(ref image)) =>
                    computed_value::T(Some(image.to_computed_value(context))),
            }
        }
    }
</%helpers:longhand>
