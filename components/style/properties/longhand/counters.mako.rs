/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Counters", inherited=False, gecko_name="Content") %>

<%helpers:longhand name="content" animatable="False">
    use cssparser::Token;
    use std::ascii::AsciiExt;
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;

    use super::list_style_type;

    pub use self::computed_value::T as SpecifiedValue;
    pub use self::computed_value::ContentItem;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use super::super::list_style_type;

        use cssparser;
        use std::fmt;
        use style_traits::ToCss;

        #[derive(Debug, PartialEq, Eq, Clone)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum ContentItem {
            /// Literal string content.
            String(String),
            /// `counter(name, style)`.
            Counter(String, list_style_type::computed_value::T),
            /// `counters(name, separator, style)`.
            Counters(String, String, list_style_type::computed_value::T),
            /// `open-quote`.
            OpenQuote,
            /// `close-quote`.
            CloseQuote,
            /// `no-open-quote`.
            NoOpenQuote,
            /// `no-close-quote`.
            NoCloseQuote,
        }

        impl ToCss for ContentItem {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    ContentItem::String(ref s) => {
                        cssparser::serialize_string(&**s, dest)
                    }
                    ContentItem::Counter(ref s, ref list_style_type) => {
                        try!(dest.write_str("counter("));
                        try!(cssparser::serialize_identifier(&**s, dest));
                        try!(dest.write_str(", "));
                        try!(list_style_type.to_css(dest));
                        dest.write_str(")")
                    }
                    ContentItem::Counters(ref s, ref separator, ref list_style_type) => {
                        try!(dest.write_str("counter("));
                        try!(cssparser::serialize_identifier(&**s, dest));
                        try!(dest.write_str(", "));
                        try!(cssparser::serialize_string(&**separator, dest));
                        try!(dest.write_str(", "));
                        try!(list_style_type.to_css(dest));
                        dest.write_str(")")
                    }
                    ContentItem::OpenQuote => dest.write_str("open-quote"),
                    ContentItem::CloseQuote => dest.write_str("close-quote"),
                    ContentItem::NoOpenQuote => dest.write_str("no-open-quote"),
                    ContentItem::NoCloseQuote => dest.write_str("no-close-quote"),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, PartialEq, Eq, Clone)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            normal,
            none,
            Content(Vec<ContentItem>),
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::normal => dest.write_str("normal"),
                    T::none => dest.write_str("none"),
                    T::Content(ref content) => {
                        let mut iter = content.iter();
                        try!(iter.next().unwrap().to_css(dest));
                        for c in iter {
                            try!(dest.write_str(" "));
                            try!(c.to_css(dest));
                        }
                        Ok(())
                    }
                }
            }
        }
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T  {
        computed_value::T::normal
    }

    pub fn counter_name_is_illegal(name: &str) -> bool {
        name.eq_ignore_ascii_case("none") || name.eq_ignore_ascii_case("inherit") ||
            name.eq_ignore_ascii_case("initial")
    }

    // normal | none | [ <string> | <counter> | open-quote | close-quote | no-open-quote |
    // no-close-quote ]+
    // TODO: <uri>, attr(<identifier>)
    pub fn parse(context: &ParserContext, input: &mut Parser)
                 -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::normal)
        }
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::none)
        }
        let mut content = vec![];
        loop {
            match input.next() {
                Ok(Token::QuotedString(value)) => {
                    content.push(ContentItem::String(value.into_owned()))
                }
                Ok(Token::Function(name)) => {
                    content.push(try!(match_ignore_ascii_case! { name,
                        "counter" => input.parse_nested_block(|input| {
                            let name = try!(input.expect_ident()).into_owned();
                            let style = input.try(|input| {
                                try!(input.expect_comma());
                                list_style_type::parse(context, input)
                            }).unwrap_or(list_style_type::computed_value::T::decimal);
                            Ok(ContentItem::Counter(name, style))
                        }),
                        "counters" => input.parse_nested_block(|input| {
                            let name = try!(input.expect_ident()).into_owned();
                            try!(input.expect_comma());
                            let separator = try!(input.expect_string()).into_owned();
                            let style = input.try(|input| {
                                try!(input.expect_comma());
                                list_style_type::parse(context, input)
                            }).unwrap_or(list_style_type::computed_value::T::decimal);
                            Ok(ContentItem::Counters(name, separator, style))
                        }),
                        _ => return Err(())
                    }));
                }
                Ok(Token::Ident(ident)) => {
                    match_ignore_ascii_case! { ident,
                        "open-quote" => content.push(ContentItem::OpenQuote),
                        "close-quote" => content.push(ContentItem::CloseQuote),
                        "no-open-quote" => content.push(ContentItem::NoOpenQuote),
                        "no-close-quote" => content.push(ContentItem::NoCloseQuote),
                        _ => return Err(())
                    }
                }
                Err(_) => break,
                _ => return Err(())
            }
        }
        if !content.is_empty() {
            Ok(SpecifiedValue::Content(content))
        } else {
            Err(())
        }
    }
</%helpers:longhand>

<%helpers:longhand name="counter-increment" products="servo" animatable="False">
    use std::fmt;
    use style_traits::ToCss;
    use super::content;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    use cssparser::{Token, serialize_identifier};
    use std::borrow::{Cow, ToOwned};

    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<(String,i32)>);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(Vec::new())
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

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

<%helpers:longhand name="counter-reset" products="servo" animatable="False">
    pub use super::counter_increment::{SpecifiedValue, computed_value, get_initial_value};
    use super::counter_increment::{parse_common};

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        parse_common(0, input)
    }
</%helpers:longhand>
