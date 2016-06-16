/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="flex-flow" sub_properties="flex-direction flex-wrap"
                                     experimental="True">
    use properties::longhands::{flex_direction, flex_wrap};

    let mut direction = None;
    let mut wrap = None;
    let mut any = false;
    loop {
        if direction.is_none() {
            if let Ok(value) = input.try(|input| flex_direction::parse(context, input)) {
                direction = Some(value);
                any = true;
                continue
            }
        }
        if wrap.is_none() {
            if let Ok(value) = input.try(|input| flex_wrap::parse(context, input)) {
                wrap = Some(value);
                any = true;
                continue
            }
        }
        break
    }

    if any {
        Ok(Longhands {
            flex_direction: direction,
            flex_wrap: wrap,
        })
    } else {
        Err(())
    }
</%helpers:shorthand>

<%helpers:shorthand name="flex" sub_properties="flex-grow flex-shrink flex-basis"
                                experimental="True">
    use app_units::Au;
    use values::specified::{Number, Length, LengthOrPercentageOrAutoOrContent};

    pub fn parse_flexibility(input: &mut Parser)
                             -> Result<(Option<Number>, Option<Number>),()> {
        let grow = Some(try!(Number::parse_non_negative(input)));
        let shrink = input.try(Number::parse_non_negative).ok();
        Ok((grow, shrink))
    }

    let mut grow = None;
    let mut shrink = None;
    let mut basis = None;
    let mut any = false;

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
                grow = flex_grow;
                shrink = flex_shrink;
                any = true;
                continue
            }
        }
        if basis.is_none() {
            if let Ok(value) = input.try(LengthOrPercentageOrAutoOrContent::parse) {
                basis = Some(value);
                any = true;
                continue
            }
        }
        break
    }
    if any {
        if grow == None {
            grow = Some(Number(1.0));
        }
        if shrink == None && basis == None {
            basis = Some(LengthOrPercentageOrAutoOrContent::Length(Length::Absolute(Au(0))));
        }
        if shrink == None {
            shrink = Some(Number(1.0));
        }
        if basis == None {
            basis = Some(LengthOrPercentageOrAutoOrContent::Auto);
        }
        Ok(Longhands {
            flex_grow: grow,
            flex_shrink: shrink,
            flex_basis: basis,
        })
    } else {
        Err(())
    }
</%helpers:shorthand>
