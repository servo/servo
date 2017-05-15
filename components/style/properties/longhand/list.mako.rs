/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("List", inherited=True) %>

${helpers.single_keyword("list-style-position", "outside inside", animation_value_type="none",
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
${helpers.single_keyword("list-style-type", """
    disc none circle square decimal disclosure-open disclosure-closed lower-alpha upper-alpha
""", extra_servo_values="""arabic-indic bengali cambodian cjk-decimal devanagari
                           gujarati gurmukhi kannada khmer lao malayalam mongolian
                           myanmar oriya persian telugu thai tibetan cjk-earthly-branch
                           cjk-heavenly-stem lower-greek hiragana hiragana-iroha katakana
                           katakana-iroha""",
    extra_gecko_values="""japanese-informal japanese-formal korean-hangul-formal
    korean-hanja-formal korean-hanja-informal simp-chinese-informal simp-chinese-formal
    trad-chinese-informal trad-chinese-formal ethiopic-numeric upper-roman lower-roman
    """,
    gecko_constant_prefix="NS_STYLE_LIST_STYLE",
    needs_conversion="True",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-lists/#propdef-list-style-type")}

<%helpers:longhand name="list-style-image" animation_value_type="none"
                   boxed="${product == 'gecko'}"
                   spec="https://drafts.csswg.org/css-lists/#propdef-list-style-image">
    use std::fmt;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    use values::specified::UrlOrNone;
    pub use self::computed_value::T as SpecifiedValue;
    use style_traits::ToCss;

    pub mod computed_value {
        use values::specified::UrlOrNone;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub UrlOrNone);
    }


    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(Either::Second(None_))
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(Either::Second(None_))
    }
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
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
    use cssparser::Token;
    use std::borrow::Cow;
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;
    use values::HasViewportPercentage;

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

${helpers.predefined_type("-moz-image-region",
                          "ClipRectOrAuto",
                          "computed::ClipRectOrAuto::auto()",
                          animation_value_type="none",
                          products="gecko",
                          boxed="True",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-image-region)")}
