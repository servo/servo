<%page args="helpers"/>

<%helpers:longhand name="counter-increment">
    use std::fmt;
    use super::content;
    use values::computed::ComputedValueAsSpecified;

    use cssparser::{ToCss, Token, serialize_identifier};
    use std::borrow::{Cow, ToOwned};

    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct T(pub Vec<(String,i32)>);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(Vec::new())
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut first = true;
            for pair in &self.0 {
                if !first {
                    try!(dest.write_str(" "));
                }
                first = false;
                try!(serialize_identifier(&pair.0, dest));
                try!(write!(dest, " {}", pair.1));
            }
            Ok(())
        }
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        parse_common(1, input)
    }

    pub fn parse_common(default_value: i32, input: &mut Parser) -> Result<SpecifiedValue,()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue(Vec::new()))
        }

        let mut counters = Vec::new();
        loop {
            let counter_name = match input.next() {
                Ok(Token::Ident(ident)) => (*ident).to_owned(),
                Ok(_) => return Err(()),
                Err(_) => break,
            };
            if content::counter_name_is_illegal(&counter_name) {
                return Err(())
            }
            let counter_delta =
                input.try(|input| specified::parse_integer(input)).unwrap_or(default_value);
            counters.push((counter_name, counter_delta))
        }

        if !counters.is_empty() {
            Ok(SpecifiedValue(counters))
        } else {
            Err(())
        }
    }
</%helpers:longhand>
