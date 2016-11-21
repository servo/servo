/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// Per CSS-TEXT 6.2, "for legacy reasons, UAs must treat `word-wrap` as an alternate name for
// the `overflow-wrap` property, as if it were a shorthand of `overflow-wrap`."
<%helpers:shorthand name="word-wrap" sub_properties="overflow-wrap">
    use properties::longhands::overflow_wrap;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        Ok(Longhands {
            overflow_wrap: Some(try!(overflow_wrap::parse(context, input))),
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.overflow_wrap.to_css(dest)
        }
    }
</%helpers:shorthand>

// https://drafts.csswg.org/css-text-decor-3/#text-emphasis-property
<%helpers:shorthand name="text-emphasis" products="gecko" sub_properties="text-emphasis-color
    text-emphasis-style">
    use properties::longhands::{text_emphasis_color, text_emphasis_style};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut color = None;
        let mut style = None;

        try!(input.try(|input| {
            loop {
                if color.is_none() {
                    if let Ok(value) = input.try(|input| text_emphasis_color::parse(context, input)) {
                        color = Some(value);
                        continue
                    }
                }
                if style.is_none() {
                    if let Ok(value) = input.try(|input| text_emphasis_style::parse(context, input)) {
                        style = Some(value);
                        continue
                    }
                }
                break
            }
            if color.is_some() || style.is_some() {
                if style.is_none() {
                    style = Some(text_emphasis_style::get_initial_specified_value());
                }
                Ok(())
            } else {
                Err(())
            }
        }));

        Ok(Longhands {
            text_emphasis_color: color,
            text_emphasis_style: style,
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let DeclaredValue::Value(ref value) = *self.text_emphasis_style {
                try!(value.to_css(dest));
            } else {
                try!(write!(dest, "none"));
            }

            try!(write!(dest, " "));

            if let DeclaredValue::Value(ref color) = *self.text_emphasis_color {
                color.to_css(dest)
            } else {
                write!(dest, "currentColor")
            }
        }
    }
</%helpers:shorthand>
