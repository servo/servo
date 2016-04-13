<%page args="helpers"/>

<%helpers:raw_longhand name="color">
    use cssparser::Color as CSSParserColor;
    use cssparser::RGBA;
    use values::specified::{CSSColor, CSSRGBA};

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, _context: &Cx) -> computed_value::T {
            self.parsed
        }
    }

    pub type SpecifiedValue = CSSRGBA;
    pub mod computed_value {
        use cssparser;
        pub type T = cssparser::RGBA;
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        RGBA { red: 0., green: 0., blue: 0., alpha: 1. }  /* black */
    }
    pub fn parse_specified(_context: &ParserContext, input: &mut Parser)
                           -> Result<DeclaredValue<SpecifiedValue>, ()> {
        let value = try!(CSSColor::parse(input));
        let rgba = match value.parsed {
            CSSParserColor::RGBA(rgba) => rgba,
            CSSParserColor::CurrentColor => return Ok(DeclaredValue::Inherit)
        };
        Ok(DeclaredValue::Value(CSSRGBA {
            parsed: rgba,
            authored: value.authored,
        }))
    }
</%helpers:raw_longhand>
