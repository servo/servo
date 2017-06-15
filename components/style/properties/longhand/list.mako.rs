/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("List", inherited=True) %>

${helpers.single_keyword("list-style-position", "outside inside", animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-lists/#propdef-list-style-position")}

// TODO(pcwalton): Implement the full set of counter styles per CSS-COUNTER-STYLES [1] 6.1:
//
//     decimal-leading-zero, armenian, upper-armenian, lower-armenian, georgian, lower-roman,
//     upper-roman
//
// TODO(bholley): Missing quite a few gecko properties here as well.
//
// In gecko, {upper,lower}-{roman,alpha} are implemented as @counter-styles in the
// UA, however they can also be set from pres attrs. When @counter-style is supported
// we may need to look into this and handle these differently.
//
// [1]: http://dev.w3.org/csswg/css-counter-styles/
% if product == "servo":
    ${helpers.single_keyword("list-style-type", """
        disc none circle square decimal disclosure-open disclosure-closed lower-alpha upper-alpha
        arabic-indic bengali cambodian cjk-decimal devanagari gujarati gurmukhi kannada khmer lao
        malayalam mongolian myanmar oriya persian telugu thai tibetan cjk-earthly-branch
        cjk-heavenly-stem lower-greek hiragana hiragana-iroha katakana katakana-iroha""",
        needs_conversion="True",
        animation_value_type="none",
        spec="https://drafts.csswg.org/css-lists/#propdef-list-style-type")}
% else:
    <%helpers:longhand name="list-style-type" animation_value_type="none" boxed="True"
                       spec="https://drafts.csswg.org/css-lists/#propdef-list-style-type">
        use values::CustomIdent;
        use values::computed::ComputedValueAsSpecified;
        use values::generics::CounterStyleOrNone;

        pub use self::computed_value::T as SpecifiedValue;

        pub mod computed_value {
            use values::generics::CounterStyleOrNone;

            /// <counter-style> | <string> | none
            #[derive(Debug, Clone, Eq, PartialEq, ToCss)]
            pub enum T {
                CounterStyle(CounterStyleOrNone),
                String(String),
            }
        }

        impl ComputedValueAsSpecified for SpecifiedValue {}
        no_viewport_percentage!(SpecifiedValue);

        #[cfg(feature = "gecko")]
        impl SpecifiedValue {
            /// Convert from gecko keyword to list-style-type.
            ///
            /// This should only be used for mapping type attribute to
            /// list-style-type, and thus only values possible in that
            /// attribute is considered here.
            pub fn from_gecko_keyword(value: u32) -> Self {
                use gecko_bindings::structs;
                SpecifiedValue::CounterStyle(if value == structs::NS_STYLE_LIST_STYLE_NONE {
                    CounterStyleOrNone::None
                } else {
                    <%
                        values = """disc circle square decimal lower-roman
                                    upper-roman lower-alpha upper-alpha""".split()
                    %>
                    CounterStyleOrNone::Name(CustomIdent(match value {
                        % for style in values:
                        structs::NS_STYLE_LIST_STYLE_${style.replace('-', '_').upper()} => atom!("${style}"),
                        % endfor
                        _ => unreachable!("Unknown counter style keyword value"),
                    }))
                })
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::CounterStyle(CounterStyleOrNone::disc())
        }

        #[inline]
        pub fn get_initial_specified_value() -> SpecifiedValue {
            SpecifiedValue::CounterStyle(CounterStyleOrNone::disc())
        }

        pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<SpecifiedValue, ParseError<'i>> {
            Ok(if let Ok(style) = input.try(|i| CounterStyleOrNone::parse(context, i)) {
                SpecifiedValue::CounterStyle(style)
            } else {
                SpecifiedValue::String(input.expect_string()?.into_owned())
            })
        }
    </%helpers:longhand>
% endif

<%helpers:longhand name="list-style-image" animation_value_type="none"
                   boxed="${product == 'gecko'}"
                   spec="https://drafts.csswg.org/css-lists/#propdef-list-style-image">
    use values::computed::ComputedValueAsSpecified;
    use values::specified::UrlOrNone;
    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        use values::specified::UrlOrNone;

        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        #[derive(Debug, Clone, PartialEq, ToCss)]
        pub struct T(pub UrlOrNone);
    }


    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(Either::Second(None_))
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(Either::Second(None_))
    }
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue,ParseError<'i>> {
        % if product == "gecko":
        let mut value = input.try(|input| UrlOrNone::parse(context, input))?;
        if let Either::First(ref mut url) = value {
            url.build_image_value();
        }
        % else :
        let value = input.try(|input| UrlOrNone::parse(context, input))?;
        % endif

        return Ok(SpecifiedValue(value));
    }
</%helpers:longhand>

<%helpers:longhand name="quotes" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-content/#propdef-quotes">
    use cssparser::serialize_string;
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<(String,String)>);
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.0.is_empty() {
                return dest.write_str("none")
            }

            let mut first = true;
            for pair in &self.0 {
                if !first {
                    dest.write_str(" ")?;
                }
                first = false;
                serialize_string(&*pair.0, dest)?;
                dest.write_str(" ")?;
                serialize_string(&*pair.1, dest)?;
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

    pub fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue,ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue(Vec::new()))
        }

        let mut quotes = Vec::new();
        loop {
            let first = match input.next() {
                Ok(Token::QuotedString(value)) => value.into_owned(),
                Ok(t) => return Err(BasicParseError::UnexpectedToken(t).into()),
                Err(_) => break,
            };
            let second = match input.next() {
                Ok(Token::QuotedString(value)) => value.into_owned(),
                Ok(t) => return Err(BasicParseError::UnexpectedToken(t).into()),
                Err(e) => return Err(e.into()),
            };
            quotes.push((first, second))
        }
        if !quotes.is_empty() {
            Ok(SpecifiedValue(quotes))
        } else {
            Err(StyleParseError::UnspecifiedError.into())
        }
    }
</%helpers:longhand>

${helpers.predefined_type("-moz-image-region",
                          "ClipRectOrAuto",
                          "computed::ClipRectOrAuto::auto()",
                          animation_value_type="ComputedValue",
                          products="gecko",
                          boxed="True",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-image-region)")}
