/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%! from data import to_rust_ident %>

<%namespace name="helpers" file="/helpers.mako.rs" />

// https://drafts.csswg.org/css-flexbox/#flex-flow-property
<%helpers:shorthand name="flex-flow" sub_properties="flex-direction flex-wrap"
                                     experimental="True">
    use properties::longhands::{flex_direction, flex_wrap};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut direction = None;
        let mut wrap = None;
        loop {
            if direction.is_none() {
                if let Ok(value) = input.try(|input| flex_direction::parse(context, input)) {
                    direction = Some(value);
                    continue
                }
            }
            if wrap.is_none() {
                if let Ok(value) = input.try(|input| flex_wrap::parse(context, input)) {
                    wrap = Some(value);
                    continue
                }
            }
            break
        }

        if direction.is_none() && wrap.is_none() {
            return Err(())
        }
        Ok(Longhands {
            flex_direction: direction,
            flex_wrap: wrap,
        })
    }


    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self.flex_direction {
                DeclaredValue::Initial => try!(write!(dest, "row")),
                _ => try!(self.flex_direction.to_css(dest))
            };

            try!(write!(dest, " "));

            match *self.flex_wrap {
                DeclaredValue::Initial => write!(dest, "nowrap"),
                _ => self.flex_wrap.to_css(dest)
            }
        }
    }
</%helpers:shorthand>

// https://drafts.csswg.org/css-flexbox/#flex-property
<%helpers:shorthand name="flex" sub_properties="flex-grow flex-shrink flex-basis"
                                experimental="True">
    use app_units::Au;
    use values::specified::{Number, Length, LengthOrPercentageOrAutoOrContent};

    pub fn parse_flexibility(input: &mut Parser)
                             -> Result<(Number, Option<Number>),()> {
        let grow = try!(Number::parse_non_negative(input));
        let shrink = input.try(Number::parse_non_negative).ok();
        Ok((grow, shrink))
    }

    pub fn parse_value(_: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut grow = None;
        let mut shrink = None;
        let mut basis = None;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(Longhands {
                flex_grow: Some(Number(0.0)),
                flex_shrink: Some(Number(0.0)),
                flex_basis: Some(LengthOrPercentageOrAutoOrContent::Auto)
            })
        }
        loop {
            if grow.is_none() {
                if let Ok((flex_grow, flex_shrink)) = input.try(parse_flexibility) {
                    grow = Some(flex_grow);
                    shrink = flex_shrink;
                    continue
                }
            }
            if basis.is_none() {
                if let Ok(value) = input.try(LengthOrPercentageOrAutoOrContent::parse) {
                    basis = Some(value);
                    continue
                }
            }
            break
        }

        if grow.is_none() && basis.is_none() {
            return Err(())
        }
        Ok(Longhands {
            flex_grow: grow.or(Some(Number(1.0))),
            flex_shrink: shrink.or(Some(Number(1.0))),
            flex_basis: basis.or(Some(LengthOrPercentageOrAutoOrContent::Length(Length::Absolute(Au(0)))))
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.flex_grow.to_css(dest));
            try!(write!(dest, " "));

            try!(self.flex_shrink.to_css(dest));
            try!(write!(dest, " "));

            self.flex_basis.to_css(dest)
        }
    }
</%helpers:shorthand>

// https://drafts.csswg.org/css-grid/#propdef-grid-row
% for prop in ["grid-row", "grid-column"]:
<%helpers:shorthand name="${prop}" products="gecko" sub_properties="${prop}-start ${prop}-end">
    use values::specified::grid::GridLine;

    pub fn parse_value(_: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut start = GridLine::default();
        let mut end = GridLine::default();

        if let Ok(line) = input.try(GridLine::parse) {
            start = line;
        }

        if let Ok(_) = input.try(|i| i.expect_delim('/')) {
            end = try!(GridLine::parse(input));
        }

        Ok(Longhands {
            ${to_rust_ident(prop)}_start: Some(start),
            ${to_rust_ident(prop)}_end: Some(end),
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.${to_rust_ident(prop)}_start.to_css(dest));
            try!(write!(dest, " / "));
            self.${to_rust_ident(prop)}_end.to_css(dest)
        }
    }
</%helpers:shorthand>
% endfor
