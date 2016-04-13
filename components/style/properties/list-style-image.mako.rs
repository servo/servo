<%page args="helpers"/>

<%helpers:longhand name="list-style-image">
    use cssparser::{ToCss, Token};
    use std::fmt;
    use url::Url;
    use values::LocalToCss;

    #[derive(Debug, Clone, PartialEq, Eq, HeapSizeOf)]
    pub enum SpecifiedValue {
        None,
        Url(Url),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::None => dest.write_str("none"),
                SpecifiedValue::Url(ref url) => url.to_css(dest),
            }
        }
    }

    pub mod computed_value {
        use cssparser::{ToCss, Token};
        use std::fmt;
        use url::Url;
        use values::LocalToCss;

        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct T(pub Option<Url>);

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("none"),
                    Some(ref url) => url.to_css(dest),
                }
            }
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, _context: &Cx) -> computed_value::T {
            match *self {
                SpecifiedValue::None => computed_value::T(None),
                SpecifiedValue::Url(ref url) => computed_value::T(Some(url.clone())),
            }
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            Ok(SpecifiedValue::None)
        } else {
            Ok(SpecifiedValue::Url(context.parse_url(&*try!(input.expect_url()))))
        }
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(None)
    }
</%helpers:longhand>
