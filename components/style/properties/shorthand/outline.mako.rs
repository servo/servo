/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="outline" sub_properties="outline-color outline-style outline-width">
    use properties::longhands::outline_width;
    use values::specified;
    use parser::Parse;

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

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.outline_width.to_css(dest));
            try!(write!(dest, " "));

            match *self.outline_style {
                DeclaredValue::Initial => try!(write!(dest, "none")),
                _ => try!(self.outline_style.to_css(dest))
            };

            match *self.outline_color {
                DeclaredValue::Initial => Ok(()),
                _ => {
                    try!(write!(dest, " "));
                    self.outline_color.to_css(dest)
                }
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

    // TODO: Border radius for the radius shorthand is not implemented correctly yet
    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self._moz_outline_radius_topleft.to_css(dest));
            try!(write!(dest, " "));

            try!(self._moz_outline_radius_topright.to_css(dest));
            try!(write!(dest, " "));

            try!(self._moz_outline_radius_bottomright.to_css(dest));
            try!(write!(dest, " "));

            self._moz_outline_radius_bottomleft.to_css(dest)
        }
    }
</%helpers:shorthand>
