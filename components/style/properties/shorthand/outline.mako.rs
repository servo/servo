/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="outline"
                    sub_properties="outline-color outline-style outline-width"
                    derive_serialize="True"
                    spec="https://drafts.csswg.org/css-ui/#propdef-outline">
    use properties::longhands::{outline_color, outline_width, outline_style};
    use values::specified;
    use parser::Parse;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let _unused = context;
        let mut color = None;
        let mut style = None;
        let mut width = None;
        let mut any = false;
        loop {
            if color.is_none() {
                if let Ok(value) = input.try(|i| specified::Color::parse(context, i)) {
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
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
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

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let radii = BorderRadius::parse(context, input)?;
        Ok(expanded! {
            _moz_outline_radius_topleft: radii.top_left,
            _moz_outline_radius_topright: radii.top_right,
            _moz_outline_radius_bottomright: radii.bottom_right,
            _moz_outline_radius_bottomleft: radii.bottom_left,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            use values::generics::border::BorderCornerRadius;

            let LonghandsToSerialize {
                _moz_outline_radius_topleft: &BorderCornerRadius(ref tl),
                _moz_outline_radius_topright: &BorderCornerRadius(ref tr),
                _moz_outline_radius_bottomright: &BorderCornerRadius(ref br),
                _moz_outline_radius_bottomleft: &BorderCornerRadius(ref bl),
            } = *self;

            let widths = Rect::new(tl.width(), tr.width(), br.width(), bl.width());
            let heights = Rect::new(tl.height(), tr.height(), br.height(), bl.height());

            BorderRadius::serialize_rects(widths, heights, dest)
        }
    }
</%helpers:shorthand>
