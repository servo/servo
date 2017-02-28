/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="font" sub_properties="font-style font-variant font-weight font-stretch
                                                font-size line-height font-family
                                                ${'font-size-adjust' if product == 'gecko' else ''}
                                                ${'font-kerning' if product == 'gecko' else ''}
                                                ${'font-variant-caps' if product == 'gecko' else ''}
                                                ${'font-variant-position' if product == 'gecko' else ''}
                                                ${'font-language-override' if product == 'none' else ''}"
                    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font">
    use properties::longhands::{font_style, font_variant, font_weight, font_stretch};
    use properties::longhands::{font_size, line_height};
    % if product == "gecko":
    use properties::longhands::{font_size_adjust, font_kerning, font_variant_caps, font_variant_position};
    % endif
    % if product == "none":
    use properties::longhands::font_language_override;
    % endif
    use properties::longhands::font_family::SpecifiedValue as FontFamily;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut nb_normals = 0;
        let mut style = None;
        let mut variant = None;
        let mut weight = None;
        let mut stretch = None;
        let size;
        loop {
            // Special-case 'normal' because it is valid in each of
            // font-style, font-weight, font-variant and font-stretch.
            // Leaves the values to None, 'normal' is the initial value for each of them.
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                nb_normals += 1;
                continue;
            }
            if style.is_none() {
                if let Ok(value) = input.try(|input| font_style::parse(context, input)) {
                    style = Some(value);
                    continue
                }
            }
            if weight.is_none() {
                if let Ok(value) = input.try(|input| font_weight::parse(context, input)) {
                    weight = Some(value);
                    continue
                }
            }
            if variant.is_none() {
                if let Ok(value) = input.try(|input| font_variant::parse(context, input)) {
                    variant = Some(value);
                    continue
                }
            }
            if stretch.is_none() {
                if let Ok(value) = input.try(|input| font_stretch::parse(context, input)) {
                    stretch = Some(value);
                    continue
                }
            }
            size = Some(try!(font_size::parse(context, input)));
            break
        }
        #[inline]
        fn count<T>(opt: &Option<T>) -> u8 {
            if opt.is_some() { 1 } else { 0 }
        }
        if size.is_none() || (count(&style) + count(&weight) + count(&variant) + count(&stretch) + nb_normals) > 4 {
            return Err(())
        }
        let line_height = if input.try(|input| input.expect_delim('/')).is_ok() {
            Some(try!(line_height::parse(context, input)))
        } else {
            None
        };
        let family = FontFamily::parse(input)?;
        Ok(Longhands {
            % for name in "style variant weight stretch size".split():
                font_${name}: unwrap_or_initial!(font_${name}, ${name}),
            % endfor
            line_height: unwrap_or_initial!(line_height),
            font_family: family,
            % if product == "gecko":
                % for name in "size_adjust kerning variant_caps variant_position".split():
                    font_${name}: font_${name}::get_initial_specified_value(),
                % endfor
            % endif
            % if product == "none":
                font_language_override: font_language_override::get_initial_specified_value(),
            % endif
        })
    }

    // This may be a bit off, unsure, possibly needs changes
    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let DeclaredValue::Value(ref style) = *self.font_style {
                try!(style.to_css(dest));
                try!(write!(dest, " "));
            }

            if let DeclaredValue::Value(ref variant) = *self.font_variant {
                try!(variant.to_css(dest));
                try!(write!(dest, " "));
            }

            if let DeclaredValue::Value(ref weight) = *self.font_weight {
                try!(weight.to_css(dest));
                try!(write!(dest, " "));
            }

            if let DeclaredValue::Value(ref stretch) = *self.font_stretch {
                try!(stretch.to_css(dest));
                try!(write!(dest, " "));
            }

            try!(self.font_size.to_css(dest));
            if let DeclaredValue::Value(ref height) = *self.line_height {
                match *height {
                    line_height::SpecifiedValue::Normal => {},
                    _ => {
                        try!(write!(dest, "/"));
                        try!(height.to_css(dest));
                    }
                }
            }

            try!(write!(dest, " "));

            self.font_family.to_css(dest)
        }
    }
</%helpers:shorthand>
