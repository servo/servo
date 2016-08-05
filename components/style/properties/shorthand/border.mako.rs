/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import to_rust_ident %>

${helpers.four_sides_shorthand("border-color", "border-%s-color", "specified::CSSColor::parse")}
${helpers.four_sides_shorthand("border-style", "border-%s-style",
                       "specified::BorderStyle::parse")}
<%helpers:shorthand name="border-width" sub_properties="${
        ' '.join('border-%s-width' % side
                 for side in ['top', 'right', 'bottom', 'left'])}">
    use super::parse_four_sides;
    use values::specified;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let _unused = context;
        let (top, right, bottom, left) = try!(parse_four_sides(input, specified::parse_border_width));
        Ok(Longhands {
            % for side in ["top", "right", "bottom", "left"]:
                ${to_rust_ident('border-%s-width' % side)}:
                    Some(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue(${side})),
            % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            // extract tuple container values so that the different border widths
            // can be compared via partial eq
            % for side in ["top", "right", "bottom", "left"]:
                let ${side} = match self.border_${side}_width {
                    &DeclaredValue::Value(ref value) => DeclaredValue::Value(value.0),
                    &DeclaredValue::WithVariables {
                        css: ref a, first_token_type: ref b, base_url: ref c, from_shorthand: ref d
                    } => DeclaredValue::WithVariables {
                        // WithVariables should not be reachable during serialization
                        css: a.clone(), first_token_type: b.clone(), base_url: c.clone(), from_shorthand: d.clone()
                    },
                    &DeclaredValue::Initial => DeclaredValue::Initial,
                    &DeclaredValue::Inherit => DeclaredValue::Inherit,
                };
            % endfor

            super::serialize_four_sides_shorthand(dest, &top, &right, &bottom, &left)
        }
    }
</%helpers:shorthand>


pub fn parse_border(context: &ParserContext, input: &mut Parser)
                 -> Result<(Option<specified::CSSColor>,
                            Option<specified::BorderStyle>,
                            Option<specified::Length>), ()> {
    use values::specified;
    let _unused = context;
    let mut color = None;
    let mut style = None;
    let mut width = None;
    let mut any = false;
    loop {
        if color.is_none() {
            if let Ok(value) = input.try(specified::CSSColor::parse) {
                color = Some(value);
                any = true;
                continue
            }
        }
        if style.is_none() {
            if let Ok(value) = input.try(specified::BorderStyle::parse) {
                style = Some(value);
                any = true;
                continue
            }
        }
        if width.is_none() {
            if let Ok(value) = input.try(specified::parse_border_width) {
                width = Some(value);
                any = true;
                continue
            }
        }
        break
    }
    if any { Ok((color, style, width)) } else { Err(()) }
}

% for side in ["top", "right", "bottom", "left"]:
    <%helpers:shorthand name="border-${side}" sub_properties="${' '.join(
        'border-%s-%s' % (side, prop)
        for prop in ['color', 'style', 'width']
    )}">

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let (color, style, width) = try!(super::parse_border(context, input));
        Ok(Longhands {
            border_${side}_color: color,
            border_${side}_style: style,
            border_${side}_width:
                width.map(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            super::serialize_directional_border_shorthand(
                dest,
                self.border_${side}_width,
                self.border_${side}_style,
                self.border_${side}_color
            )
        }
    }

    </%helpers:shorthand>
% endfor

<%helpers:shorthand name="border" sub_properties="${' '.join(
    'border-%s-%s' % (side, prop)
    for side in ['top', 'right', 'bottom', 'left']
    for prop in ['color', 'style', 'width']
)}">

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let (color, style, width) = try!(super::parse_border(context, input));
        Ok(Longhands {
            % for side in ["top", "right", "bottom", "left"]:
                border_${side}_color: color.clone(),
                border_${side}_style: style,
                border_${side}_width:
                    width.map(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue),
            % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            // If all longhands are all present, then all sides should be the same,
            // so we can just one set of color/style/width
            super::serialize_directional_border_shorthand(
                dest,
                self.border_${side}_width,
                self.border_${side}_style,
                self.border_${side}_color
            )
        }
    }

</%helpers:shorthand>

<%helpers:shorthand name="border-radius" sub_properties="${' '.join(
    'border-%s-radius' % (corner)
     for corner in ['top-left', 'top-right', 'bottom-right', 'bottom-left']
)}">
    use app_units::Au;
    use values::specified::{Length, LengthOrPercentage};
    use values::specified::BorderRadiusSize;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let _ignored = context;

        fn parse_one_set_of_border_values(mut input: &mut Parser)
                                         -> Result<[LengthOrPercentage; 4], ()> {
            let mut count = 0;
            let mut values = [LengthOrPercentage::Length(Length::Absolute(Au(0))); 4];
            while count < 4 {
                if let Ok(value) = input.try(LengthOrPercentage::parse) {
                    values[count] = value;
                    count += 1;
                } else {
                    break
                }
            }

            match count {
                1 => Ok([values[0], values[0], values[0], values[0]]),
                2 => Ok([values[0], values[1], values[0], values[1]]),
                3 => Ok([values[0], values[1], values[2], values[1]]),
                4 => Ok([values[0], values[1], values[2], values[3]]),
                _ => Err(()),
            }
        }

        fn parse_one_set_of_border_radii(mut input: &mut Parser)
                                         -> Result<[BorderRadiusSize; 4], ()> {
            let widths = try!(parse_one_set_of_border_values(input));
            let mut heights = widths.clone();
            let mut radii_values = [BorderRadiusSize::zero(); 4];
            if input.try(|input| input.expect_delim('/')).is_ok() {
                heights = try!(parse_one_set_of_border_values(input));
            }
            for i in 0..radii_values.len() {
                radii_values[i] = BorderRadiusSize::new(widths[i], heights[i]);
            }
            Ok(radii_values)
        }

        let radii = try!(parse_one_set_of_border_radii(input));
        Ok(Longhands {
            border_top_left_radius: Some(radii[0]),
            border_top_right_radius: Some(radii[1]),
            border_bottom_right_radius: Some(radii[2]),
            border_bottom_left_radius: Some(radii[3]),
        })
    }

    // TODO: I do not understand how border radius works with respect to the slashes /,
    // so putting a default generic impl for now
    // https://developer.mozilla.org/en-US/docs/Web/CSS/border-radius
    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.border_top_left_radius.to_css(dest));
            try!(write!(dest, " "));

            try!(self.border_top_right_radius.to_css(dest));
            try!(write!(dest, " "));

            try!(self.border_bottom_right_radius.to_css(dest));
            try!(write!(dest, " "));

            self.border_bottom_left_radius.to_css(dest)
        }
    }
</%helpers:shorthand>
