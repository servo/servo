/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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

    pub fn serialize<'a, W, I>(dest: &mut W, declarations: I) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

        let mut flex_direction = None;
        let mut flex_wrap = None;

        for decl in declarations {
            match *decl {
                PropertyDeclaration::FlexDirection(ref value) => { flex_direction = Some(value); },
                PropertyDeclaration::FlexWrap(ref value) => { flex_wrap = Some(value); },
                _ => return Err(fmt::Error)
            }
        }

        let (flex_direction, flex_wrap) = try_unwrap_longhands!(flex_direction, flex_wrap);

        match *flex_direction {
            DeclaredValue::Initial => try!(write!(dest, "row")),
            _ => try!(flex_direction.to_css(dest))
        };

        try!(write!(dest, " "));

        match *flex_wrap {
            DeclaredValue::Initial => write!(dest, "nowrap"),
            _ => flex_wrap.to_css(dest)
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

    pub fn serialize<'a, W, I>(dest: &mut W, declarations: I) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

        let mut flex_grow = None;
        let mut flex_shrink = None;
        let mut flex_basis = None;

        for decl in declarations {
            match *decl {
                PropertyDeclaration::FlexGrow(ref value) => { flex_grow = Some(value); },
                PropertyDeclaration::FlexShrink(ref value) => { flex_shrink = Some(value); },
                PropertyDeclaration::FlexBasis(ref value) => { flex_basis = Some(value); },
                _ => return Err(fmt::Error)
            }
        }

        let (flex_grow, flex_shrink, flex_basis) = try_unwrap_longhands!(flex_grow, flex_shrink, flex_basis);


        try!(flex_grow.to_css(dest));
        try!(write!(dest, " "));

        try!(flex_shrink.to_css(dest));
        try!(write!(dest, " "));

        flex_basis.to_css(dest)
    }
</%helpers:shorthand>
