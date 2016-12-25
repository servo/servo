/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import to_rust_ident, ALL_SIDES %>

${helpers.four_sides_shorthand("border-color", "border-%s-color", "specified::CSSColor::parse")}

${helpers.four_sides_shorthand("border-style", "border-%s-style",
                               "specified::BorderStyle::parse",
                               needs_context=False)}

<%helpers:shorthand name="border-width" sub_properties="${
        ' '.join('border-%s-width' % side
                 for side in ['top', 'right', 'bottom', 'left'])}">
    use super::parse_four_sides;
    use parser::Parse;
    use values::specified;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let (top, right, bottom, left) = try!(parse_four_sides(input, |i| specified::BorderWidth::parse(context, i)));
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
                    &DeclaredValue::Unset => DeclaredValue::Unset,
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
            if let Ok(value) = input.try(|i| specified::CSSColor::parse(context, i)) {
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
            if let Ok(value) = input.try(|i| specified::BorderWidth::parse(context, i)) {
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
    use parser::Parse;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let radii = try!(BorderRadius::parse(context, input));
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

// https://drafts.csswg.org/css-backgrounds-3/#border-image
<%helpers:shorthand name="border-image" products="gecko" sub_properties="border-image-outset
    border-image-repeat border-image-slice border-image-source border-image-width">
    use properties::longhands::{border_image_outset, border_image_repeat, border_image_slice};
    use properties::longhands::{border_image_source, border_image_width};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        % for name in "outset repeat slice source width".split():
            let mut border_image_${name} = border_image_${name}::get_initial_specified_value();
        % endfor

        try!(input.try(|input| {
            % for name in "outset repeat slice source width".split():
                let mut ${name} = None;
            % endfor
            loop {
                if slice.is_none() {
                    if let Ok(value) = input.try(|input| border_image_slice::parse(context, input)) {
                        slice = Some(value);
                        // Parse border image width and outset, if applicable.
                        let maybe_width_outset: Result<_, ()> = input.try(|input| {
                            try!(input.expect_delim('/'));

                            // Parse border image width, if applicable.
                            let w = input.try(|input|
                                border_image_width::parse(context, input)).ok();

                            // Parse border image outset if applicable.
                            let o = input.try(|input| {
                                try!(input.expect_delim('/'));
                                border_image_outset::parse(context, input)
                            }).ok();
                            Ok((w, o))
                        });
                        if let Ok((w, o)) = maybe_width_outset {
                            width = w;
                            outset = o;
                        }

                        continue
                    }
                }
                % for name in "source repeat".split():
                    if ${name}.is_none() {
                        if let Ok(value) = input.try(|input| border_image_${name}::parse(context, input)) {
                            ${name} = Some(value);
                            continue
                        }
                    }
                % endfor
                break
            }
            let mut any = false;
            % for name in "outset repeat slice source width".split():
                any = any || ${name}.is_some();
            % endfor
            if any {
                % for name in "outset repeat slice source width".split():
                    if let Some(b_${name}) = ${name} {
                        border_image_${name} = b_${name};
                    }
                % endfor
                Ok(())
            } else {
                Err(())
            }
        }));

        Ok(Longhands {
            % for name in "outset repeat slice source width".split():
                border_image_${name}: Some(border_image_${name}),
            % endfor
         })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            % for name in "outset repeat slice source width".split():
                let ${name} = if let DeclaredValue::Value(ref value) = *self.border_image_${name} {
                    Some(value)
                } else {
                    None
                };
            % endfor

            if let Some(source) = source {
                try!(source.to_css(dest));
            } else {
                try!(write!(dest, "none"));
            }

            try!(write!(dest, " "));

            if let Some(slice) = slice {
                try!(slice.to_css(dest));
            } else {
                try!(write!(dest, "100%"));
            }

            try!(write!(dest, " / "));

            if let Some(width) = width {
                try!(width.to_css(dest));
            } else {
                try!(write!(dest, "1"));
            }

            try!(write!(dest, " / "));

            if let Some(outset) = outset {
                try!(outset.to_css(dest));
            } else {
                try!(write!(dest, "0"));
            }

            try!(write!(dest, " "));

            if let Some(repeat) = repeat {
                try!(repeat.to_css(dest));
            } else {
                try!(write!(dest, "stretch"));
            }

            Ok(())
        }
    }
</%helpers:shorthand>
