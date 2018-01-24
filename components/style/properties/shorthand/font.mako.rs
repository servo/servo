/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import SYSTEM_FONT_LONGHANDS %>

<%helpers:shorthand name="font"
                    sub_properties="font-style font-variant-caps font-weight font-stretch
                                    font-size line-height font-family
                                    ${'font-size-adjust' if product == 'gecko' else ''}
                                    ${'font-kerning' if product == 'gecko' else ''}
                                    ${'font-variant-alternates' if product == 'gecko' else ''}
                                    ${'font-variant-east-asian' if product == 'gecko' else ''}
                                    ${'font-variant-ligatures' if product == 'gecko' else ''}
                                    ${'font-variant-numeric' if product == 'gecko' else ''}
                                    ${'font-variant-position' if product == 'gecko' else ''}
                                    ${'font-language-override' if product == 'gecko' else ''}
                                    ${'font-feature-settings' if product == 'gecko' else ''}"
                    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font">
    use parser::Parse;
    use properties::longhands::{font_family, font_style, font_weight, font_stretch};
    use properties::longhands::font_variant_caps;
    #[cfg(feature = "gecko")]
    use properties::longhands::system_font::SystemFont;
    use values::specified::text::LineHeight;
    use values::specified::FontSize;

    <%
        gecko_sub_properties = "kerning language_override size_adjust \
                                variant_alternates variant_east_asian \
                                variant_ligatures variant_numeric \
                                variant_position feature_settings".split()
    %>
    % if product == "gecko":
        % for prop in gecko_sub_properties:
            use properties::longhands::font_${prop};
        % endfor
    % endif
    use self::font_family::SpecifiedValue as FontFamily;

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        let mut nb_normals = 0;
        let mut style = None;
        let mut variant_caps = None;
        let mut weight = None;
        let mut stretch = None;
        let size;
        % if product == "gecko":
            if let Ok(sys) = input.try(SystemFont::parse) {
                return Ok(expanded! {
                     % for name in SYSTEM_FONT_LONGHANDS:
                         % if name == "font_size":
                             ${name}: FontSize::system_font(sys),
                         % else:
                             ${name}: ${name}::SpecifiedValue::system_font(sys),
                         % endif
                     % endfor
                     // line-height is just reset to initial
                     line_height: LineHeight::normal(),
                 })
            }
        % endif
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
            if variant_caps.is_none() {
                if let Ok(value) = input.try(|input| font_variant_caps::parse(context, input)) {
                    variant_caps = Some(value);
                    continue
                }
            }
            if stretch.is_none() {
                if let Ok(value) = input.try(|input| font_stretch::parse(context, input)) {
                    stretch = Some(value);
                    continue
                }
            }
            size = Some(FontSize::parse(context, input)?);
            break
        }

        let size = match size {
            Some(s) => s,
            None => {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            }
        };

        let line_height = if input.try(|input| input.expect_delim('/')).is_ok() {
            Some(LineHeight::parse(context, input)?)
        } else {
            None
        };

        #[inline]
        fn count<T>(opt: &Option<T>) -> u8 {
            if opt.is_some() { 1 } else { 0 }
        }

        if (count(&style) + count(&weight) + count(&variant_caps) + count(&stretch) + nb_normals) > 4 {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }

        let family = FontFamily::parse_specified(input)?;
        Ok(expanded! {
            % for name in "style weight stretch variant_caps".split():
                font_${name}: unwrap_or_initial!(font_${name}, ${name}),
            % endfor
            font_size: size,
            line_height: line_height.unwrap_or(LineHeight::normal()),
            font_family: family,
            % if product == "gecko":
                % for name in gecko_sub_properties:
                    font_${name}: font_${name}::get_initial_specified_value(),
                % endfor
            % endif
        })
    }

    % if product == "gecko":
        enum CheckSystemResult {
            AllSystem(SystemFont),
            SomeSystem,
            None
        }
    % endif
    enum SerializeFor {
        Normal,
    % if product == "gecko":
        Canvas,
    % endif
    }

    impl<'a> LonghandsToSerialize<'a> {
        fn to_css_for<W>(
            &self,
            serialize_for: SerializeFor,
            dest: &mut CssWriter<W>,
        ) -> fmt::Result
        where
            W: Write,
        {
            % if product == "gecko":
                match self.check_system() {
                    CheckSystemResult::AllSystem(sys) => return sys.to_css(dest),
                    CheckSystemResult::SomeSystem => return Ok(()),
                    CheckSystemResult::None => ()
                }
            % endif

            % if product == "gecko":
                % for name in gecko_sub_properties:
                    if self.font_${name} != &font_${name}::get_initial_specified_value() {
                        return Ok(());
                    }
                % endfor
            % endif

            // In case of serialization for canvas font, we need to drop
            // initial values of properties other than size and family.
            % for name in "style variant_caps weight stretch".split():
                let needs_this_property = match serialize_for {
                    SerializeFor::Normal => true,
                % if product == "gecko":
                    SerializeFor::Canvas =>
                        self.font_${name} != &font_${name}::get_initial_specified_value(),
                % endif
                };
                if needs_this_property {
                    self.font_${name}.to_css(dest)?;
                    dest.write_str(" ")?;
                }
            % endfor

            self.font_size.to_css(dest)?;

            if *self.line_height != LineHeight::normal() {
                dest.write_str("/")?;
                self.line_height.to_css(dest)?;
            }

            dest.write_str(" ")?;
            self.font_family.to_css(dest)?;

            Ok(())
        }

        % if product == "gecko":
            /// Check if some or all members are system fonts
            fn check_system(&self) -> CheckSystemResult {
                let mut sys = None;
                let mut all = true;

                % for prop in SYSTEM_FONT_LONGHANDS:
                    if let Some(s) = self.${prop}.get_system() {
                        debug_assert!(sys.is_none() || s == sys.unwrap());
                        sys = Some(s);
                    } else {
                        all = false;
                    }
                % endfor
                if self.line_height != &LineHeight::normal() {
                    all = false
                }
                if all {
                    CheckSystemResult::AllSystem(sys.unwrap())
                } else if sys.is_some() {
                    CheckSystemResult::SomeSystem
                } else {
                    CheckSystemResult::None
                }
            }

            /// Serialize the shorthand value for canvas font attribute.
            pub fn to_css_for_canvas<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
                self.to_css_for(SerializeFor::Canvas, dest)
            }
        % endif
    }

    // This may be a bit off, unsure, possibly needs changes
    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.to_css_for(SerializeFor::Normal, dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="font-variant"
                    sub_properties="font-variant-caps
                                    ${'font-variant-alternates' if product == 'gecko' else ''}
                                    ${'font-variant-east-asian' if product == 'gecko' else ''}
                                    ${'font-variant-ligatures' if product == 'gecko' else ''}
                                    ${'font-variant-numeric' if product == 'gecko' else ''}
                                    ${'font-variant-position' if product == 'gecko' else ''}"
                    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-variant">
    <% gecko_sub_properties = "alternates east_asian ligatures numeric position".split() %>
    <%
        sub_properties = ["caps"]
        if product == "gecko":
            sub_properties += gecko_sub_properties
    %>

% for prop in sub_properties:
    use properties::longhands::font_variant_${prop};
% endfor
    #[allow(unused_imports)]
    use values::specified::FontVariantLigatures;

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
    % for prop in sub_properties:
        let mut ${prop} = None;
    % endfor

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            // Leave the values to None, 'normal' is the initial value for all the sub properties.
        } else if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            // The 'none' value sets 'font-variant-ligatures' to 'none' and resets all other sub properties
            // to their initial value.
        % if product == "gecko":
            ligatures = Some(FontVariantLigatures::none());
        % endif
        } else {
            let mut has_custom_value: bool = false;
            loop {
                if input.try(|input| input.expect_ident_matching("normal")).is_ok() ||
                   input.try(|input| input.expect_ident_matching("none")).is_ok() {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                }
            % for prop in sub_properties:
                if ${prop}.is_none() {
                    if let Ok(value) = input.try(|i| font_variant_${prop}::parse(context, i)) {
                        has_custom_value = true;
                        ${prop} = Some(value);
                        continue
                    }
                }
            % endfor

                break
            }

            if !has_custom_value {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            }
        }

        Ok(expanded! {
        % for prop in sub_properties:
            font_variant_${prop}: unwrap_or_initial!(font_variant_${prop}, ${prop}),
        % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        #[allow(unused_assignments)]
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {

            let has_none_ligatures =
            % if product == "gecko":
                self.font_variant_ligatures == &FontVariantLigatures::none();
            % else:
                false;
            % endif

            const TOTAL_SUBPROPS: usize = ${len(sub_properties)};
            let mut nb_normals = 0;
        % for prop in sub_properties:
            if self.font_variant_${prop} == &font_variant_${prop}::get_initial_specified_value() {
                nb_normals += 1;
            }
        % endfor


            if nb_normals > 0 && nb_normals == TOTAL_SUBPROPS {
                dest.write_str("normal")?;
            } else if has_none_ligatures {
                if nb_normals == TOTAL_SUBPROPS - 1 {
                    // Serialize to 'none' if 'font-variant-ligatures' is set to 'none' and all other
                    // font feature properties are reset to their initial value.
                    dest.write_str("none")?;
                } else {
                    return Ok(())
                }
            } else {
                let mut has_any = false;
            % for prop in sub_properties:
                if self.font_variant_${prop} != &font_variant_${prop}::get_initial_specified_value() {
                    if has_any {
                        dest.write_str(" ")?;
                    }
                    has_any = true;
                    self.font_variant_${prop}.to_css(dest)?;
                }
            % endfor
            }

            Ok(())
        }
    }
</%helpers:shorthand>
