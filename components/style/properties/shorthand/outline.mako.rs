/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="outline" sub_properties="outline-color outline-style outline-width">
    use properties::longhands::outline_width;
    use values::specified;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
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
                if let Ok(value) = input.try(|input| outline_width::parse(context, input)) {
                    width = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }
        if any {
            Ok(Longhands {
                outline_color: color,
                outline_style: style,
                outline_width: width,
            })
        } else {
            Err(())
        }
    }

    pub fn serialize<'a, W, I>(dest: &mut W, declarations: I) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

        let mut width = None;
        let mut style = None;
        let mut color = None;

        for decl in declarations {
            match *decl {
                PropertyDeclaration::OutlineWidth(ref value) => { width = Some(value); },
                PropertyDeclaration::OutlineStyle(ref value) => { style = Some(value); },
                PropertyDeclaration::OutlineColor(ref value) => { color = Some(value); },
                _ => return Err(fmt::Error)
            }
        }

        let (width, style, color) = try_unwrap_longhands!(width, style, color);

        try!(width.to_css(dest));
        try!(write!(dest, " "));

        match *style {
            DeclaredValue::Initial => try!(write!(dest, "none")),
            _ => try!(style.to_css(dest))
        };

        match *color {
            DeclaredValue::Initial => Ok(()),
            _ => {
                try!(write!(dest, " "));
                color.to_css(dest)
            }
        }
    }
</%helpers:shorthand>

// The -moz-outline-radius shorthand is non-standard and not on a standards track.
<%helpers:shorthand name="-moz-outline-radius" sub_properties="${' '.join(
    '-moz-outline-radius-%s' % corner
    for corner in ['topleft', 'topright', 'bottomright', 'bottomleft']
)}" products="gecko">
    use properties::shorthands;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        // Re-use border-radius parsing.
        shorthands::border_radius::parse_value(context, input).map(|longhands| {
            Longhands {
                % for corner in ["top_left", "top_right", "bottom_right", "bottom_left"]:
                _moz_outline_radius_${corner.replace("_", "")}: longhands.border_${corner}_radius,
                % endfor
            }
        })
    }

    pub fn serialize<'a, W, I>(dest: &mut W, declarations: I) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
        // Border radius for the regular radius shorthand is not implemented yet

        let mut top_left = None;
        let mut top_right = None;
        let mut bottom_right = None;
        let mut bottom_left = None;

        for decl in declarations {
            match *decl {
                PropertyDeclaration::MozOutlineTopLeftRadius(ref value) => { top_left = Some(value); },
                PropertyDeclaration::MozOutlineTopRightRadius(ref value) => { top_right = Some(value); },
                PropertyDeclaration::MozOutlineBottomRightRadius(ref value) => { bottom_right  = Some(value); },
                PropertyDeclaration::MozOutlineBottomLeftRadius(ref value) => { bottom_left = Some(value); },
                _ => return Err(fmt::Error)
            }
        }

        let (top_left, top_right, bottom_right, bottom_left) =
            try_unwrap_longhands!(top_left, top_right, bottom_right, bottom_left);


        try!(top_left.to_css(dest));
        try!(write!(dest, " "));

        try!(top_right.to_css(dest));
        try!(write!(dest, " "));

        try!(bottom_right.to_css(dest));
        try!(write!(dest, " "));

        bottom_left.to_css(dest)
    }
</%helpers:shorthand>
