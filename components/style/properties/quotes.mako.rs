<%page args="helpers"/>

<%helpers:longhand name="quotes">
    use std::borrow::Cow;
    use std::fmt;
    use values::computed::ComputedValueAsSpecified;

    use cssparser::{ToCss, Token};

    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct T(pub Vec<(String,String)>);
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
                try!(Token::QuotedString(Cow::from(&*pair.0)).to_css(dest));
                try!(dest.write_str(" "));
                try!(Token::QuotedString(Cow::from(&*pair.1)).to_css(dest));
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![
            ("\u{201c}".to_owned(), "\u{201d}".to_owned()),
            ("\u{2018}".to_owned(), "\u{2019}".to_owned()),
        ])
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue(Vec::new()))
        }

        let mut quotes = Vec::new();
        loop {
            let first = match input.next() {
                Ok(Token::QuotedString(value)) => value.into_owned(),
                Ok(_) => return Err(()),
                Err(()) => break,
            };
            let second = match input.next() {
                Ok(Token::QuotedString(value)) => value.into_owned(),
                _ => return Err(()),
            };
            quotes.push((first, second))
        }
        if !quotes.is_empty() {
            Ok(SpecifiedValue(quotes))
        } else {
            Err(())
        }
    }
</%helpers:longhand>
