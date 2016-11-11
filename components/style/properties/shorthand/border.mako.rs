/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import to_rust_ident, ALL_SIDES %>

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
        let (top, right, bottom, left) = try!(parse_four_sides(input, specified::BorderWidth::parse));
        Ok(Longhands {
            % for side in ["top", "right", "bottom", "left"]:
                ${to_rust_ident('border-%s-width' % side)}: Some(${side}),
            % endfor
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            % for side in ["top", "right", "bottom", "left"]:
                let ${side} = match self.border_${side}_width {
                    &DeclaredValue::Value(ref value) => DeclaredValue::Value(*value),
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

            super::serialize_four_sides(dest, &top, &right, &bottom, &left)
        }
    }
</%helpers:shorthand>


pub fn parse_border(context: &ParserContext, input: &mut Parser)
                 -> Result<(Option<specified::CSSColor>,
                            Option<specified::BorderStyle>,
                            Option<specified::BorderWidth>), ()> {
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
            if let Ok(value) = input.try(specified::BorderWidth::parse) {
                width = Some(value);
                any = true;
                continue
            }
        }
        break
    }
    if any { Ok((color, style, width)) } else { Err(()) }
}

% for side in map(lambda x: x[0], ALL_SIDES):
    <%helpers:shorthand name="border-${side}" sub_properties="${' '.join(
        'border-%s-%s' % (side, prop)
        for prop in ['color', 'style', 'width']
    )}">

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let (color, style, width) = try!(super::parse_border(context, input));
        Ok(Longhands {
            border_${to_rust_ident(side)}_color: color,
            border_${to_rust_ident(side)}_style: style,
            border_${to_rust_ident(side)}_width: width
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            super::serialize_directional_border(
                dest,
                self.border_${to_rust_ident(side)}_width,
                self.border_${to_rust_ident(side)}_style,
                self.border_${to_rust_ident(side)}_color
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
                border_${side}_width: width,
            % endfor
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            // If all longhands are all present, then all sides should be the same,
            // so we can just one set of color/style/width
            super::serialize_directional_border(
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
    use values::specified::basic_shape::BorderRadius;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let _ignored = context;

        let radii = try!(BorderRadius::parse(input));
        Ok(Longhands {
            border_top_left_radius: Some(radii.top_left),
            border_top_right_radius: Some(radii.top_right),
            border_bottom_right_radius: Some(radii.bottom_right),
            border_bottom_left_radius: Some(radii.bottom_left),
        })
    }

    // TODO: I do not understand how border radius works with respect to the slashes /,
    // so putting a default generic impl for now
    // https://developer.mozilla.org/en-US/docs/Web/CSS/border-radius
    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
