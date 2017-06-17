/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="text-emphasis" products="gecko"
    sub_properties="text-emphasis-style text-emphasis-color"
    derive_serialize="True"
    spec="https://drafts.csswg.org/css-text-decor-3/#text-emphasis-property">
    use properties::longhands::{text_emphasis_color, text_emphasis_style};

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
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
            Ok(expanded! {
                text_emphasis_color: unwrap_or_initial!(text_emphasis_color, color),
                text_emphasis_style: unwrap_or_initial!(text_emphasis_style, style),
            })
        } else {
            Err(StyleParseError::UnspecifiedError.into())
        }
    }
</%helpers:shorthand>

// CSS Compatibility
// https://compat.spec.whatwg.org/
<%helpers:shorthand name="-webkit-text-stroke"
                    sub_properties="-webkit-text-stroke-width
                                    -webkit-text-stroke-color"
                    products="gecko"
                    derive_serialize="True"
                    spec="https://compat.spec.whatwg.org/#the-webkit-text-stroke">
    use properties::longhands::{_webkit_text_stroke_color, _webkit_text_stroke_width};

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        let mut color = None;
        let mut width = None;
        loop {
            if color.is_none() {
                if let Ok(value) = input.try(|input| _webkit_text_stroke_color::parse(context, input)) {
                    color = Some(value);
                    continue
                }
            }

            if width.is_none() {
                if let Ok(value) = input.try(|input| _webkit_text_stroke_width::parse(context, input)) {
                    width = Some(value);
                    continue
                }
            }
            break
        }

        if color.is_some() || width.is_some() {
            Ok(expanded! {
                _webkit_text_stroke_color: unwrap_or_initial!(_webkit_text_stroke_color, color),
                _webkit_text_stroke_width: unwrap_or_initial!(_webkit_text_stroke_width, width),
            })
        } else {
            Err(StyleParseError::UnspecifiedError.into())
        }
    }
</%helpers:shorthand>
