/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="outline" sub_properties="outline-color outline-style outline-width"
                    spec="https://drafts.csswg.org/css-ui/#propdef-outline">
    use properties::longhands::{outline_color, outline_width, outline_style};
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
                if let Ok(value) = input.try(|i| specified::CSSColor::parse(context, i)) {
                    color = Some(value);
                    any = true;
                    continue
                }
            }
            if style.is_none() {
                if let Ok(value) = input.try(|input| outline_style::parse(context, input)) {
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
            Ok(expanded! {
                outline_color: unwrap_or_initial!(outline_color, color),
                outline_style: unwrap_or_initial!(outline_style, style),
                outline_width: unwrap_or_initial!(outline_width, width),
            })
        } else {
            Err(())
        }
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.outline_width.to_css(dest));
            try!(write!(dest, " "));
            try!(self.outline_style.to_css(dest));
            try!(write!(dest, " "));
            self.outline_color.to_css(dest)
        }
    }
</%helpers:shorthand>

// The -moz-outline-radius shorthand is non-standard and not on a standards track.
<%helpers:shorthand name="-moz-outline-radius" sub_properties="${' '.join(
    '-moz-outline-radius-%s' % corner
    for corner in ['topleft', 'topright', 'bottomright', 'bottomleft']
)}" products="gecko" spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-outline-radius)">
    use values::generics::rect::Rect;
    use values::specified::border::BorderRadius;
    use parser::Parse;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let radii = try!(BorderRadius::parse(context, input));
        Ok(expanded! {
            _moz_outline_radius_topleft: radii.top_left,
            _moz_outline_radius_topright: radii.top_right,
            _moz_outline_radius_bottomright: radii.bottom_right,
            _moz_outline_radius_bottomleft: radii.bottom_left,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let LonghandsToSerialize {
                _moz_outline_radius_topleft: ref tl,
                _moz_outline_radius_topright: ref tr,
                _moz_outline_radius_bottomright: ref br,
                _moz_outline_radius_bottomleft: ref bl,
            } = *self;

            let widths = Rect::new(&tl.0.width, &tr.0.width, &br.0.width, &bl.0.width);
            let heights = Rect::new(&tl.0.height, &tr.0.height, &br.0.height, &bl.0.height);

            BorderRadius::serialize_rects(widths, heights, dest)
        }
    }
</%helpers:shorthand>
