/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import to_rust_ident, ALL_SIDES, PHYSICAL_SIDES, maybe_moz_logical_alias %>

${helpers.four_sides_shorthand(
    "border-color",
    "border-%s-color",
    "specified::Color::parse",
    engines="gecko servo",
    spec="https://drafts.csswg.org/css-backgrounds/#border-color",
    allow_quirks="Yes",
)}

${helpers.four_sides_shorthand(
    "border-style",
    "border-%s-style",
    engines="gecko servo",
    spec="https://drafts.csswg.org/css-backgrounds/#border-style",
)}

<%helpers:shorthand
    name="border-width"
    engines="gecko servo"
    sub_properties="${
        ' '.join('border-%s-width' % side
                 for side in PHYSICAL_SIDES)}"
    spec="https://drafts.csswg.org/css-backgrounds/#border-width">
    use crate::values::generics::rect::Rect;
    use crate::values::specified::{AllowQuirks, BorderSideWidth};

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let rect = Rect::parse_with(context, input, |_, i| {
            BorderSideWidth::parse_quirky(context, i, AllowQuirks::Yes)
        })?;
        Ok(expanded! {
            border_top_width: rect.0,
            border_right_width: rect.1,
            border_bottom_width: rect.2,
            border_left_width: rect.3,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            % for side in PHYSICAL_SIDES:
            let ${side} = &self.border_${side}_width;
            % endfor
            Rect::new(top, right, bottom, left).to_css(dest)
        }
    }
</%helpers:shorthand>


pub fn parse_border<'i, 't>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
) -> Result<(specified::Color, specified::BorderStyle, specified::BorderSideWidth), ParseError<'i>> {
    use crate::values::specified::{Color, BorderStyle, BorderSideWidth};
    let _unused = context;
    let mut color = None;
    let mut style = None;
    let mut width = None;
    let mut any = false;
    loop {
        if width.is_none() {
            if let Ok(value) = input.try_parse(|i| BorderSideWidth::parse(context, i)) {
                width = Some(value);
                any = true;
            }
        }
        if style.is_none() {
            if let Ok(value) = input.try_parse(BorderStyle::parse) {
                style = Some(value);
                any = true;
                continue
            }
        }
        if color.is_none() {
            if let Ok(value) = input.try_parse(|i| Color::parse(context, i)) {
                color = Some(value);
                any = true;
                continue
            }
        }
        break
    }
    if !any {
        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
    Ok((color.unwrap_or(Color::CurrentColor), style.unwrap_or(BorderStyle::None), width.unwrap_or(BorderSideWidth::medium())))
}

% for side, logical in ALL_SIDES:
    <%
        spec = "https://drafts.csswg.org/css-backgrounds/#border-%s" % side
        if logical:
            spec = "https://drafts.csswg.org/css-logical-props/#propdef-border-%s" % side
    %>
    <%helpers:shorthand
        name="border-${side}"
        engines="gecko servo"
        sub_properties="${' '.join(
            'border-%s-%s' % (side, prop)
            for prop in ['width', 'style', 'color']
        )}"
        aliases="${maybe_moz_logical_alias(engine, (side, logical), '-moz-border-%s')}"
        spec="${spec}">

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let (color, style, width) = super::parse_border(context, input)?;
        Ok(expanded! {
            border_${to_rust_ident(side)}_color: color,
            border_${to_rust_ident(side)}_style: style,
            border_${to_rust_ident(side)}_width: width
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            crate::values::specified::border::serialize_directional_border(
                dest,
                self.border_${to_rust_ident(side)}_width,
                self.border_${to_rust_ident(side)}_style,
                self.border_${to_rust_ident(side)}_color
            )
        }
    }

    </%helpers:shorthand>
% endfor

<%helpers:shorthand name="border"
    engines="gecko servo"
    sub_properties="${' '.join('border-%s-%s' % (side, prop)
        for side in PHYSICAL_SIDES for prop in ['width', 'style', 'color']
        )}
        ${' '.join('border-image-%s' % name
        for name in ['outset', 'repeat', 'slice', 'source', 'width'])}"
    derive_value_info="False"
    spec="https://drafts.csswg.org/css-backgrounds/#border">

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        use crate::properties::longhands::{border_image_outset, border_image_repeat, border_image_slice};
        use crate::properties::longhands::{border_image_source, border_image_width};

        let (color, style, width) = super::parse_border(context, input)?;
        Ok(expanded! {
            % for side in PHYSICAL_SIDES:
                border_${side}_color: color.clone(),
                border_${side}_style: style,
                border_${side}_width: width.clone(),
            % endfor

            // The ‘border’ shorthand resets ‘border-image’ to its initial value.
            // See https://drafts.csswg.org/css-backgrounds-3/#the-border-shorthands
            % for name in "outset repeat slice source width".split():
                border_image_${name}: border_image_${name}::get_initial_specified_value(),
            % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            use crate::properties::longhands;

            // If any of the border-image longhands differ from their initial specified values we should not
            // invoke serialize_directional_border(), so there is no point in continuing on to compute all_equal.
            % for name in "outset repeat slice source width".split():
                if *self.border_image_${name} != longhands::border_image_${name}::get_initial_specified_value() {
                    return Ok(());
                }
            % endfor

            let all_equal = {
                % for side in PHYSICAL_SIDES:
                  let border_${side}_width = self.border_${side}_width;
                  let border_${side}_style = self.border_${side}_style;
                  let border_${side}_color = self.border_${side}_color;
                % endfor

                border_top_width == border_right_width &&
                border_right_width == border_bottom_width &&
                border_bottom_width == border_left_width &&

                border_top_style == border_right_style &&
                border_right_style == border_bottom_style &&
                border_bottom_style == border_left_style &&

                border_top_color == border_right_color &&
                border_right_color == border_bottom_color &&
                border_bottom_color == border_left_color
            };

            // If all longhands are all present, then all sides should be the same,
            // so we can just one set of color/style/width
            if !all_equal {
                return Ok(())
            }
            crate::values::specified::border::serialize_directional_border(
                dest,
                self.border_${side}_width,
                self.border_${side}_style,
                self.border_${side}_color
            )
        }
    }

    // Just use the same as border-left. The border shorthand can't accept
    // any value that the sub-shorthand couldn't.
    <%
        border_left = "<crate::properties::shorthands::border_left::Longhands as SpecifiedValueInfo>"
    %>
    impl SpecifiedValueInfo for Longhands {
        const SUPPORTED_TYPES: u8 = ${border_left}::SUPPORTED_TYPES;
        fn collect_completion_keywords(f: KeywordsCollectFn) {
            ${border_left}::collect_completion_keywords(f);
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    name="border-radius"
    engines="gecko servo"
    sub_properties="${' '.join(
        'border-%s-radius' % (corner)
         for corner in ['top-left', 'top-right', 'bottom-right', 'bottom-left']
    )}"
    extra_prefixes="webkit"
    spec="https://drafts.csswg.org/css-backgrounds/#border-radius"
>
    use crate::values::generics::rect::Rect;
    use crate::values::generics::border::BorderCornerRadius;
    use crate::values::specified::border::BorderRadius;
    use crate::parser::Parse;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let radii = BorderRadius::parse(context, input)?;
        Ok(expanded! {
            border_top_left_radius: radii.top_left,
            border_top_right_radius: radii.top_right,
            border_bottom_right_radius: radii.bottom_right,
            border_bottom_left_radius: radii.bottom_left,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            let LonghandsToSerialize {
                border_top_left_radius: &BorderCornerRadius(ref tl),
                border_top_right_radius: &BorderCornerRadius(ref tr),
                border_bottom_right_radius: &BorderCornerRadius(ref br),
                border_bottom_left_radius: &BorderCornerRadius(ref bl),
            } = *self;


            let widths = Rect::new(tl.width(), tr.width(), br.width(), bl.width());
            let heights = Rect::new(tl.height(), tr.height(), br.height(), bl.height());

            BorderRadius::serialize_rects(widths, heights, dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    name="border-image"
    engines="gecko servo"
    servo_pref="layout.legacy_layout",
    sub_properties="border-image-outset
        border-image-repeat border-image-slice border-image-source border-image-width"
    extra_prefixes="moz:layout.css.prefixes.border-image webkit"
    spec="https://drafts.csswg.org/css-backgrounds-3/#border-image"
>
    use crate::properties::longhands::{border_image_outset, border_image_repeat, border_image_slice};
    use crate::properties::longhands::{border_image_source, border_image_width};

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        % for name in "outset repeat slice source width".split():
        let mut ${name} = border_image_${name}::get_initial_specified_value();
        % endfor
        let mut any = false;
        let mut parsed_slice = false;
        let mut parsed_source = false;
        let mut parsed_repeat = false;
        loop {
            if !parsed_slice {
                if let Ok(value) = input.try_parse(|input| border_image_slice::parse(context, input)) {
                    parsed_slice = true;
                    any = true;
                    slice = value;
                    // Parse border image width and outset, if applicable.
                    let maybe_width_outset: Result<_, ParseError> = input.try_parse(|input| {
                        input.expect_delim('/')?;

                        // Parse border image width, if applicable.
                        let w = input.try_parse(|input| border_image_width::parse(context, input)).ok();

                        // Parse border image outset if applicable.
                        let o = input.try_parse(|input| {
                            input.expect_delim('/')?;
                            border_image_outset::parse(context, input)
                        }).ok();
                        if w.is_none() && o.is_none() {
                            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                        }
                        Ok((w, o))
                    });
                    if let Ok((w, o)) = maybe_width_outset {
                        if let Some(w) = w {
                            width = w;
                        }
                        if let Some(o) = o {
                            outset = o;
                        }
                    }
                    continue;
                }
            }
            % for name in "source repeat".split():
                if !parsed_${name} {
                    if let Ok(value) = input.try_parse(|input| border_image_${name}::parse(context, input)) {
                        ${name} = value;
                        parsed_${name} = true;
                        any = true;
                        continue
                    }
                }
            % endfor
            break
        }
        if !any {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
        Ok(expanded! {
            % for name in "outset repeat slice source width".split():
                border_image_${name}: ${name},
            % endfor
         })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            let mut has_any = false;
            % for name in "source slice outset width repeat".split():
            let has_${name} = *self.border_image_${name} != border_image_${name}::get_initial_specified_value();
            has_any = has_any || has_${name};
            % endfor
            if has_source || !has_any {
                self.border_image_source.to_css(dest)?;
                if !has_any {
                    return Ok(());
                }
            }
            let needs_slice = has_slice || has_width || has_outset;
            if needs_slice {
                if has_source {
                    dest.write_char(' ')?;
                }
                self.border_image_slice.to_css(dest)?;
                if has_width || has_outset {
                    dest.write_str(" /")?;
                    if has_width {
                        dest.write_char(' ')?;
                        self.border_image_width.to_css(dest)?;
                    }
                    if has_outset {
                        dest.write_str(" / ")?;
                        self.border_image_outset.to_css(dest)?;
                    }
                }
            }
            if has_repeat {
                if has_source || needs_slice {
                    dest.write_char(' ')?;
                }
                self.border_image_repeat.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

% for axis in ["block", "inline"]:
    % for prop in ["width", "style", "color"]:
        <%
            spec = "https://drafts.csswg.org/css-logical/#propdef-border-%s-%s" % (axis, prop)
        %>
        <%helpers:shorthand
            engines="gecko servo"
            name="border-${axis}-${prop}"
            sub_properties="${' '.join(
                'border-%s-%s-%s' % (axis, side, prop)
                for side in ['start', 'end']
            )}"
            spec="${spec}">

            use crate::properties::longhands::border_${axis}_start_${prop};
            pub fn parse_value<'i, 't>(
                context: &ParserContext,
                input: &mut Parser<'i, 't>,
            ) -> Result<Longhands, ParseError<'i>> {
                let start_value = border_${axis}_start_${prop}::parse(context, input)?;
                let end_value =
                    input.try_parse(|input| border_${axis}_start_${prop}::parse(context, input))
                        .unwrap_or_else(|_| start_value.clone());

                Ok(expanded! {
                    border_${axis}_start_${prop}: start_value,
                    border_${axis}_end_${prop}: end_value,
                })
            }

            impl<'a> ToCss for LonghandsToSerialize<'a>  {
                fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
                    self.border_${axis}_start_${prop}.to_css(dest)?;

                    if self.border_${axis}_end_${prop} != self.border_${axis}_start_${prop} {
                        dest.write_char(' ')?;
                        self.border_${axis}_end_${prop}.to_css(dest)?;
                    }

                    Ok(())
                }
            }
        </%helpers:shorthand>
    % endfor
% endfor

% for axis in ["block", "inline"]:
    <%
        spec = "https://drafts.csswg.org/css-logical/#propdef-border-%s" % (axis)
    %>
    <%helpers:shorthand
        name="border-${axis}"
        engines="gecko servo"
        sub_properties="${' '.join(
            'border-%s-%s-width' % (axis, side)
            for side in ['start', 'end']
        )} ${' '.join(
            'border-%s-%s-style' % (axis, side)
            for side in ['start', 'end']
        )} ${' '.join(
            'border-%s-%s-color' % (axis, side)
            for side in ['start', 'end']
        )}"
        spec="${spec}">

        use crate::properties::shorthands::border_${axis}_start;
        pub fn parse_value<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<Longhands, ParseError<'i>> {
            let start_value = border_${axis}_start::parse_value(context, input)?;
            Ok(expanded! {
                border_${axis}_start_width: start_value.border_${axis}_start_width.clone(),
                border_${axis}_end_width: start_value.border_${axis}_start_width,
                border_${axis}_start_style: start_value.border_${axis}_start_style.clone(),
                border_${axis}_end_style: start_value.border_${axis}_start_style,
                border_${axis}_start_color: start_value.border_${axis}_start_color.clone(),
                border_${axis}_end_color: start_value.border_${axis}_start_color,
            })
        }

        impl<'a> ToCss for LonghandsToSerialize<'a>  {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
                crate::values::specified::border::serialize_directional_border(
                    dest,
                    self.border_${axis}_start_width,
                    self.border_${axis}_start_style,
                    self.border_${axis}_start_color
                )
            }
        }
    </%helpers:shorthand>
% endfor
