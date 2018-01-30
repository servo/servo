/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Counters", inherited=False, gecko_name="Content") %>

<%helpers:longhand name="content" boxed="True" animation_value_type="discrete"
                   spec="https://drafts.csswg.org/css-content/#propdef-content">
    #[cfg(feature = "gecko")]
    use values::generics::CounterStyleOrNone;
    #[cfg(feature = "gecko")]
    use values::specified::url::SpecifiedUrl;
    #[cfg(feature = "gecko")]
    use values::specified::Attr;

    #[cfg(feature = "servo")]
    use super::list_style_type;

    pub use self::computed_value::T as SpecifiedValue;
    pub use self::computed_value::ContentItem;

    pub mod computed_value {
        use cssparser;
        use std::fmt::{self, Write};
        use style_traits::{CssWriter, ToCss};
        #[cfg(feature = "gecko")]
        use values::specified::url::SpecifiedUrl;

        #[cfg(feature = "servo")]
        type CounterStyleType = super::super::list_style_type::computed_value::T;
        #[cfg(feature = "gecko")]
        type CounterStyleType = ::values::generics::CounterStyleOrNone;

        #[cfg(feature = "gecko")]
        use values::specified::Attr;

        #[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
        pub enum ContentItem {
            /// Literal string content.
            String(String),
            /// `counter(name, style)`.
            Counter(String, CounterStyleType),
            /// `counters(name, separator, style)`.
            Counters(String, String, CounterStyleType),
            /// `open-quote`.
            OpenQuote,
            /// `close-quote`.
            CloseQuote,
            /// `no-open-quote`.
            NoOpenQuote,
            /// `no-close-quote`.
            NoCloseQuote,

            % if product == "gecko":
                /// `attr([namespace? `|`]? ident)`
                Attr(Attr),
                /// `url(url)`
                Url(SpecifiedUrl),
            % endif
        }

        impl ToCss for ContentItem {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                match *self {
                    ContentItem::String(ref s) => s.to_css(dest),
                    ContentItem::Counter(ref s, ref counter_style) => {
                        dest.write_str("counter(")?;
                        cssparser::serialize_identifier(&**s, dest)?;
                        dest.write_str(", ")?;
                        counter_style.to_css(dest)?;
                        dest.write_str(")")
                    }
                    ContentItem::Counters(ref s, ref separator, ref counter_style) => {
                        dest.write_str("counters(")?;
                        cssparser::serialize_identifier(&**s, dest)?;
                        dest.write_str(", ")?;
                        separator.to_css(dest)?;
                        dest.write_str(", ")?;
                        counter_style.to_css(dest)?;
                        dest.write_str(")")
                    }
                    ContentItem::OpenQuote => dest.write_str("open-quote"),
                    ContentItem::CloseQuote => dest.write_str("close-quote"),
                    ContentItem::NoOpenQuote => dest.write_str("no-open-quote"),
                    ContentItem::NoCloseQuote => dest.write_str("no-close-quote"),

                    % if product == "gecko":
                        ContentItem::Attr(ref attr) => {
                            attr.to_css(dest)
                        }
                        ContentItem::Url(ref url) => url.to_css(dest),
                    % endif
                }
            }
        }

        #[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
        pub enum T {
            Normal,
            None,
            #[cfg(feature = "gecko")]
            MozAltContent,
            Items(Vec<ContentItem>),
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                match *self {
                    T::Normal => dest.write_str("normal"),
                    T::None => dest.write_str("none"),
                    % if product == "gecko":
                        T::MozAltContent => dest.write_str("-moz-alt-content"),
                    % endif
                    T::Items(ref content) => {
                        let mut iter = content.iter();
                        iter.next().unwrap().to_css(dest)?;
                        for c in iter {
                            dest.write_str(" ")?;
                            c.to_css(dest)?;
                        }
                        Ok(())
                    }
                }
            }
        }
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Normal
    }

    #[cfg(feature = "servo")]
    fn parse_counter_style(context: &ParserContext, input: &mut Parser) -> list_style_type::computed_value::T {
        input.try(|input| {
            input.expect_comma()?;
            list_style_type::parse(context, input)
        }).unwrap_or(list_style_type::computed_value::T::Decimal)
    }

    #[cfg(feature = "gecko")]
    fn parse_counter_style(context: &ParserContext, input: &mut Parser) -> CounterStyleOrNone {
        input.try(|input| {
            input.expect_comma()?;
            CounterStyleOrNone::parse(context, input)
        }).unwrap_or(CounterStyleOrNone::decimal())
    }

    // normal | none | [ <string> | <counter> | open-quote | close-quote | no-open-quote |
    // no-close-quote ]+
    // TODO: <uri>, attr(<identifier>)
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Normal)
        }
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::None)
        }
        % if product == "gecko":
            if input.try(|input| input.expect_ident_matching("-moz-alt-content")).is_ok() {
                return Ok(SpecifiedValue::MozAltContent)
            }
        % endif

        let mut content = vec![];
        loop {
            % if product == "gecko":
                if let Ok(mut url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
                    url.build_image_value();
                    content.push(ContentItem::Url(url));
                    continue;
                }
            % endif
            // FIXME: remove clone() when lifetimes are non-lexical
            match input.next().map(|t| t.clone()) {
                Ok(Token::QuotedString(ref value)) => {
                    content.push(ContentItem::String(value.as_ref().to_owned()))
                }
                Ok(Token::Function(ref name)) => {
                    let result = match_ignore_ascii_case! { &name,
                        "counter" => Some(input.parse_nested_block(|input| {
                            let name = input.expect_ident()?.as_ref().to_owned();
                            let style = parse_counter_style(context, input);
                            Ok(ContentItem::Counter(name, style))
                        })),
                        "counters" => Some(input.parse_nested_block(|input| {
                            let name = input.expect_ident()?.as_ref().to_owned();
                            input.expect_comma()?;
                            let separator = input.expect_string()?.as_ref().to_owned();
                            let style = parse_counter_style(context, input);
                            Ok(ContentItem::Counters(name, separator, style))
                        })),
                        % if product == "gecko":
                            "attr" => Some(input.parse_nested_block(|input| {
                                Ok(ContentItem::Attr(Attr::parse_function(context, input)?))
                            })),
                        % endif
                        _ => None
                    };
                    match result {
                        Some(result) => content.push(result?),
                        None => return Err(input.new_custom_error(
                            StyleParseErrorKind::UnexpectedFunction(name.clone())
                        ))
                    }
                }
                Ok(Token::Ident(ref ident)) => {
                    let valid = match_ignore_ascii_case! { &ident,
                        "open-quote" => { content.push(ContentItem::OpenQuote); true },
                        "close-quote" => { content.push(ContentItem::CloseQuote); true },
                        "no-open-quote" => { content.push(ContentItem::NoOpenQuote); true },
                        "no-close-quote" => { content.push(ContentItem::NoCloseQuote); true },

                        _ => false,
                    };
                    if !valid {
                        return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())))
                    }
                }
                Err(_) => break,
                Ok(t) => return Err(input.new_unexpected_token_error(t))
            }
        }
        if content.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(SpecifiedValue::Items(content))
    }
</%helpers:longhand>

${helpers.predefined_type("counter-increment",
                          "CounterIncrement",
                          initial_value="computed::CounterIncrement::none()",
                          animation_value_type="discrete",
                          spec="https://drafts.csswg.org/css-lists/#propdef-counter-increment")}

${helpers.predefined_type("counter-reset",
                          "CounterReset",
                          initial_value="computed::CounterReset::none()",
                          animation_value_type="discrete",
                          spec="https://drafts.csswg.org/css-lists-3/#propdef-counter-reset")}
