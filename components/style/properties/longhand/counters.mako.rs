/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Counters", inherited=False, gecko_name="Content") %>

<%helpers:longhand name="content" boxed="True" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-content/#propdef-content">
    use cssparser::Token;
    use std::ascii::AsciiExt;
    use values::computed::ComputedValueAsSpecified;
    use values::specified::url::SpecifiedUrl;
    use values::HasViewportPercentage;

    use super::list_style_type;

    pub use self::computed_value::T as SpecifiedValue;
    pub use self::computed_value::ContentItem;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use super::super::list_style_type;

        use cssparser;
        use std::fmt;
        use style_traits::ToCss;
        use values::specified::url::SpecifiedUrl;

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

            % if product == "gecko":
                /// `-moz-alt-content`
                MozAltContent,
                /// `attr([namespace? `|`]? ident)`
                Attr(Option<String>, String),
                /// `url(url)`
                Url(SpecifiedUrl),
            % endif
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

                    % if product == "gecko":
                        ContentItem::MozAltContent => dest.write_str("-moz-alt-content"),
                        ContentItem::Attr(ref ns, ref attr) => {
                            dest.write_str("attr(")?;
                            if let Some(ref ns) = *ns {
                                cssparser::Token::Ident((&**ns).into()).to_css(dest)?;
                                dest.write_str("|")?;
                            }
                            cssparser::Token::Ident((&**attr).into()).to_css(dest)?;
                            dest.write_str(")")
                        }
                        ContentItem::Url(ref url) => url.to_css(dest),
                    % endif
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
            % if product == "gecko":
                if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
                    content.push(ContentItem::Url(url));
                    continue;
                }
            % endif
            match input.next() {
                Ok(Token::QuotedString(value)) => {
                    content.push(ContentItem::String(value.into_owned()))
                }
                Ok(Token::Function(name)) => {
                    content.push(try!(match_ignore_ascii_case! { &name,
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
                        % if product == "gecko":
                            "attr" => input.parse_nested_block(|input| {
                                // Syntax is `[namespace? `|`]? ident`
                                // no spaces allowed
                                // FIXME (bug 1346693) we should be checking that
                                // this is a valid namespace and encoding it as a namespace
                                // number from the map
                                let first = input.try(|i| i.expect_ident()).ok().map(|i| i.into_owned());
                                if let Ok(token) = input.try(|i| i.next_including_whitespace()) {
                                    match token {
                                        Token::Delim('|') => {
                                            // must be followed by an ident
                                            let tok2 = input.next_including_whitespace()?;
                                            if let Token::Ident(second) = tok2 {
                                                return Ok(ContentItem::Attr(first, second.into_owned()))
                                            } else {
                                                return Err(())
                                            }
                                        }
                                        _ => return Err(())
                                    }
                                }
                                if let Some(first) = first {
                                    Ok(ContentItem::Attr(None, first))
                                } else {
                                    Err(())
                                }
                            }),
                        % endif
                        _ => return Err(())
                    }));
                }
                Ok(Token::Ident(ident)) => {
                    match_ignore_ascii_case! { &ident,
                        "open-quote" => content.push(ContentItem::OpenQuote),
                        "close-quote" => content.push(ContentItem::CloseQuote),
                        "no-open-quote" => content.push(ContentItem::NoOpenQuote),
                        "no-close-quote" => content.push(ContentItem::NoCloseQuote),

                        % if product == "gecko":
                            "-moz-alt-content" => content.push(ContentItem::MozAltContent),
                        % endif

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

<%helpers:longhand name="counter-increment" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-lists/#propdef-counter-increment">
    use std::fmt;
    use style_traits::ToCss;
    use super::content;
    use values::HasViewportPercentage;

    use cssparser::{Token, serialize_identifier};
    use std::borrow::{Cow, ToOwned};

    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecifiedValue(pub Vec<(String, specified::Integer)>);

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<(String, i32)>);

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result
                where W: fmt::Write,
            {
                use cssparser::serialize_identifier;
                if self.0.is_empty() {
                    return dest.write_str("none")
                }

                let mut first = true;
                for pair in &self.0 {
                    if !first {
                        try!(dest.write_str(" "));
                    }
                    first = false;
                    try!(serialize_identifier(&pair.0, dest));
                    try!(dest.write_str(" "));
                    try!(pair.1.to_css(dest));
                }
                Ok(())
            }
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
            computed_value::T(self.0.iter().map(|entry| {
                (entry.0.clone(), entry.1.to_computed_value(context))
            }).collect::<Vec<_>>())
        }

        fn from_computed_value(computed: &Self::ComputedValue) -> Self {
            SpecifiedValue(computed.0.iter().map(|entry| {
                (entry.0.clone(), specified::Integer::from_computed_value(&entry.1))
            }).collect::<Vec<_>>())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(Vec::new())
    }

    no_viewport_percentage!(SpecifiedValue);

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result
            where W: fmt::Write,
        {
            if self.0.is_empty() {
                return dest.write_str("none");
            }
            let mut first = true;
            for pair in &self.0 {
                if !first {
                    try!(dest.write_str(" "));
                }
                first = false;
                try!(serialize_identifier(&pair.0, dest));
                try!(dest.write_str(" "));
                try!(pair.1.to_css(dest));
            }

            Ok(())
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        parse_common(context, 1, input)
    }

    pub fn parse_common(context: &ParserContext, default_value: i32, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use std::ascii::AsciiExt;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue(Vec::new()))
        }

        let mut counters = Vec::new();
        loop {
            let counter_name = match input.next() {
                Ok(Token::Ident(ident)) => {
                    if CSSWideKeyword::from_ident(&ident).is_some() || ident.eq_ignore_ascii_case("none") {
                        // Don't accept CSS-wide keywords or none as the counter name.
                        return Err(());
                    }
                    (*ident).to_owned()
                }
                Ok(_) => return Err(()),
                Err(_) => break,
            };
            let counter_delta = input.try(|input| specified::parse_integer(context, input))
                                     .unwrap_or(specified::Integer::new(default_value));
            counters.push((counter_name, counter_delta))
        }

        if !counters.is_empty() {
            Ok(SpecifiedValue(counters))
        } else {
            Err(())
        }
    }
</%helpers:longhand>

<%helpers:longhand name="counter-reset" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-lists-3/#propdef-counter-reset">
    pub use super::counter_increment::{SpecifiedValue, computed_value, get_initial_value};
    use super::counter_increment::parse_common;

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        parse_common(context, 0, input)
    }
</%helpers:longhand>
