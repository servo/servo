/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="text-emphasis" products="gecko" sub_properties="text-emphasis-color
    text-emphasis-style"
    spec="https://drafts.csswg.org/css-text-decor-3/#text-emphasis-property">
    use properties::longhands::{text_emphasis_color, text_emphasis_style};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut color = None;
        let mut style = None;

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

            Ok(Longhands {
                text_emphasis_color: color,
                text_emphasis_style: style,
            })
        } else {
            Err(())
        }
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut style_present = false;
            if let DeclaredValue::Value(ref value) = *self.text_emphasis_style {
                style_present = true;
                try!(value.to_css(dest));
            }

            if let DeclaredValue::Value(ref color) = *self.text_emphasis_color {
                if style_present {
                    try!(write!(dest, " "));
                }
                try!(color.to_css(dest));
            }
            Ok(())
        }
    }
</%helpers:shorthand>

// CSS Compatibility
// https://compat.spec.whatwg.org/
<%helpers:shorthand name="-webkit-text-stroke"
                    sub_properties="-webkit-text-stroke-color
                                    -webkit-text-stroke-width"
                    products="gecko"
                    spec="https://compat.spec.whatwg.org/#the-webkit-text-stroke">
    use cssparser::Color as CSSParserColor;
    use properties::longhands::{_webkit_text_stroke_color, _webkit_text_stroke_width};
    use values::specified::CSSColor;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        use values::specified::{BorderWidth, Length};
        use app_units::Au;

        let (mut color, mut width, mut any) = (None, None, false);
        % for value in "color width".split():
            if ${value}.is_none() {
                if let Ok(value) = input.try(|input| _webkit_text_stroke_${value}::parse(context, input)) {
                    ${value} = Some(value);
                    any = true;
                }
            }
        % endfor

        if !any {
            return Err(());
        }

        Ok(Longhands {
            _webkit_text_stroke_color: color.or(Some(CSSColor { parsed: CSSParserColor::CurrentColor,
                                                                authored: None })),
            _webkit_text_stroke_width: width.or(Some(BorderWidth::from_length(Length::Absolute(Au::from_px(0))))),
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut style_present = false;
            if let DeclaredValue::Value(ref width) = *self._webkit_text_stroke_width {
                style_present = true;
                try!(width.to_css(dest));
            }

            if let DeclaredValue::Value(ref color) = *self._webkit_text_stroke_color {
                if style_present {
                    try!(write!(dest, " "));
                }
                try!(color.to_css(dest));
            }

            Ok(())
        }
    }
</%helpers:shorthand>
