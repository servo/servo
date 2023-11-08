/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import SYSTEM_FONT_LONGHANDS %>

<%helpers:shorthand
    name="font"
    engines="gecko servo"
    sub_properties="
        font-style
        font-variant-caps
        font-weight
        font-stretch
        font-size
        line-height
        font-family
        ${'font-size-adjust' if engine == 'gecko' else ''}
        ${'font-kerning' if engine == 'gecko' else ''}
        ${'font-optical-sizing' if engine == 'gecko' else ''}
        ${'font-variant-alternates' if engine == 'gecko' else ''}
        ${'font-variant-east-asian' if engine == 'gecko' else ''}
        ${'font-variant-emoji' if engine == 'gecko' else ''}
        ${'font-variant-ligatures' if engine == 'gecko' else ''}
        ${'font-variant-numeric' if engine == 'gecko' else ''}
        ${'font-variant-position' if engine == 'gecko' else ''}
        ${'font-language-override' if engine == 'gecko' else ''}
        ${'font-feature-settings' if engine == 'gecko' else ''}
        ${'font-variation-settings' if engine == 'gecko' else ''}
    "
    derive_value_info="False"
    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font"
>
    use crate::computed_values::font_variant_caps::T::SmallCaps;
    use crate::parser::Parse;
    use crate::properties::longhands::{font_family, font_style, font_weight, font_stretch};
    #[cfg(feature = "gecko")]
    use crate::properties::longhands::font_size;
    use crate::properties::longhands::font_variant_caps;
    use crate::values::specified::text::LineHeight;
    use crate::values::specified::{FontSize, FontWeight};
    use crate::values::specified::font::{FontStretch, FontStretchKeyword};
    #[cfg(feature = "gecko")]
    use crate::values::specified::font::SystemFont;

    <%
        gecko_sub_properties = "kerning language_override size_adjust \
                                variant_alternates variant_east_asian \
                                variant_emoji variant_ligatures \
                                variant_numeric variant_position \
                                feature_settings variation_settings \
                                optical_sizing".split()
    %>
    % if engine == "gecko":
        % for prop in gecko_sub_properties:
            use crate::properties::longhands::font_${prop};
        % endfor
    % endif
    use self::font_family::SpecifiedValue as FontFamily;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let mut nb_normals = 0;
        let mut style = None;
        let mut variant_caps = None;
        let mut weight = None;
        let mut stretch = None;
        let size;
        % if engine == "gecko":
            if let Ok(sys) = input.try_parse(|i| SystemFont::parse(context, i)) {
                return Ok(expanded! {
                     % for name in SYSTEM_FONT_LONGHANDS:
                        ${name}: ${name}::SpecifiedValue::system_font(sys),
                     % endfor
                     line_height: LineHeight::normal(),
                     % for name in gecko_sub_properties + ["variant_caps"]:
                         font_${name}: font_${name}::get_initial_specified_value(),
                     % endfor
                 })
            }
        % endif
        loop {
            // Special-case 'normal' because it is valid in each of
            // font-style, font-weight, font-variant and font-stretch.
            // Leaves the values to None, 'normal' is the initial value for each of them.
            if input.try_parse(|input| input.expect_ident_matching("normal")).is_ok() {
                nb_normals += 1;
                continue;
            }
            if style.is_none() {
                if let Ok(value) = input.try_parse(|input| font_style::parse(context, input)) {
                    style = Some(value);
                    continue
                }
            }
            if weight.is_none() {
                if let Ok(value) = input.try_parse(|input| font_weight::parse(context, input)) {
                    weight = Some(value);
                    continue
                }
            }
            if variant_caps.is_none() {
                // The only variant-caps value allowed is small-caps (from CSS2); the added values
                // defined by CSS Fonts 3 and later are not accepted.
                // https://www.w3.org/TR/css-fonts-4/#font-prop
                if input.try_parse(|input| input.expect_ident_matching("small-caps")).is_ok() {
                    variant_caps = Some(SmallCaps);
                    continue
                }
            }
            if stretch.is_none() {
                if let Ok(value) = input.try_parse(FontStretchKeyword::parse) {
                    stretch = Some(FontStretch::Keyword(value));
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

        let line_height = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
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

        let family = FontFamily::parse(context, input)?;
        Ok(expanded! {
            % for name in "style weight stretch variant_caps".split():
                font_${name}: unwrap_or_initial!(font_${name}, ${name}),
            % endfor
            font_size: size,
            line_height: line_height.unwrap_or(LineHeight::normal()),
            font_family: family,
            % if engine == "gecko":
                % for name in gecko_sub_properties:
                    font_${name}: font_${name}::get_initial_specified_value(),
                % endfor
            % endif
        })
    }

    % if engine == "gecko":
        enum CheckSystemResult {
            AllSystem(SystemFont),
            SomeSystem,
            None
        }
    % endif

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            % if engine == "gecko":
                match self.check_system() {
                    CheckSystemResult::AllSystem(sys) => return sys.to_css(dest),
                    CheckSystemResult::SomeSystem => return Ok(()),
                    CheckSystemResult::None => {}
                }
            % endif

            % if engine == "gecko":
            if let Some(v) = self.font_optical_sizing {
                if v != &font_optical_sizing::get_initial_specified_value() {
                    return Ok(());
                }
            }
            if let Some(v) = self.font_variation_settings {
                if v != &font_variation_settings::get_initial_specified_value() {
                    return Ok(());
                }
            }
            if let Some(v) = self.font_variant_emoji {
                if v != &font_variant_emoji::get_initial_specified_value() {
                    return Ok(());
                }
            }

            % for name in gecko_sub_properties:
            % if name != "optical_sizing" and name != "variation_settings" and name != "variant_emoji":
            if self.font_${name} != &font_${name}::get_initial_specified_value() {
                return Ok(());
            }
            % endif
            % endfor
            % endif

            // Only font-stretch keywords are allowed as part as the font
            // shorthand.
            let font_stretch = match *self.font_stretch {
                FontStretch::Keyword(kw) => kw,
                FontStretch::Stretch(percentage) => {
                    match FontStretchKeyword::from_percentage(percentage.0.get()) {
                        Some(kw) => kw,
                        None => return Ok(()),
                    }
                }
                FontStretch::System(..) => return Ok(()),
            };

            // The only variant-caps value allowed in the shorthand is small-caps (from CSS2);
            // the added values defined by CSS Fonts 3 and later are not supported.
            // https://www.w3.org/TR/css-fonts-4/#font-prop
            if self.font_variant_caps != &font_variant_caps::get_initial_specified_value() &&
                *self.font_variant_caps != SmallCaps {
                return Ok(());
            }

            % for name in "style variant_caps".split():
                if self.font_${name} != &font_${name}::get_initial_specified_value() {
                    self.font_${name}.to_css(dest)?;
                    dest.write_char(' ')?;
                }
            % endfor

            // The initial specified font-weight value of 'normal' computes as a number (400),
            // not to the keyword, so we need to check for that as well in order to properly
            // serialize the computed style.
            if self.font_weight != &FontWeight::normal() &&
               self.font_weight != &FontWeight::from_gecko_keyword(400)  {
                self.font_weight.to_css(dest)?;
                dest.write_char(' ')?;
            }

            if font_stretch != FontStretchKeyword::Normal {
                font_stretch.to_css(dest)?;
                dest.write_char(' ')?;
            }

            self.font_size.to_css(dest)?;

            if *self.line_height != LineHeight::normal() {
                dest.write_str(" / ")?;
                self.line_height.to_css(dest)?;
            }

            dest.write_char(' ')?;
            self.font_family.to_css(dest)?;

            Ok(())
        }
    }

    impl<'a> LonghandsToSerialize<'a> {
        % if engine == "gecko":
        /// Check if some or all members are system fonts
        fn check_system(&self) -> CheckSystemResult {
            let mut sys = None;
            let mut all = true;

            % for prop in SYSTEM_FONT_LONGHANDS:
            % if prop == "font_optical_sizing" or prop == "font_variation_settings":
            if let Some(value) = self.${prop} {
            % else:
            {
                let value = self.${prop};
            % endif
                match value.get_system() {
                    Some(s) => {
                        debug_assert!(sys.is_none() || s == sys.unwrap());
                        sys = Some(s);
                    }
                    None => {
                        all = false;
                    }
                }
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
        % endif
    }

    <%
        subprops_for_value_info = ["font_style", "font_weight", "font_stretch",
                                   "font_variant_caps", "font_size", "font_family"]
        subprops_for_value_info = [
            "<longhands::{}::SpecifiedValue as SpecifiedValueInfo>".format(p)
            for p in subprops_for_value_info
        ]
    %>
    impl SpecifiedValueInfo for Longhands {
        const SUPPORTED_TYPES: u8 = 0
            % for p in subprops_for_value_info:
            | ${p}::SUPPORTED_TYPES
            % endfor
            ;

        fn collect_completion_keywords(f: KeywordsCollectFn) {
            % for p in subprops_for_value_info:
            ${p}::collect_completion_keywords(f);
            % endfor
            % if engine == "gecko":
            <SystemFont as SpecifiedValueInfo>::collect_completion_keywords(f);
            % endif
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="font-variant"
                    engines="gecko servo"
                    servo_pref="layout.legacy_layout",
                    flags="SHORTHAND_IN_GETCS"
                    sub_properties="font-variant-caps
                                    ${'font-variant-alternates' if engine == 'gecko' else ''}
                                    ${'font-variant-east-asian' if engine == 'gecko' else ''}
                                    ${'font-variant-emoji' if engine == 'gecko' else ''}
                                    ${'font-variant-ligatures' if engine == 'gecko' else ''}
                                    ${'font-variant-numeric' if engine == 'gecko' else ''}
                                    ${'font-variant-position' if engine == 'gecko' else ''}"
                    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-variant">
% if engine == 'gecko':
    <% sub_properties = "ligatures caps alternates numeric east_asian position emoji".split() %>
% else:
    <% sub_properties = ["caps"] %>
% endif

% for prop in sub_properties:
    use crate::properties::longhands::font_variant_${prop};
% endfor
    #[allow(unused_imports)]
    use crate::values::specified::FontVariantLigatures;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
    % for prop in sub_properties:
        let mut ${prop} = None;
    % endfor

        if input.try_parse(|input| input.expect_ident_matching("normal")).is_ok() {
            // Leave the values to None, 'normal' is the initial value for all the sub properties.
        } else if input.try_parse(|input| input.expect_ident_matching("none")).is_ok() {
            // The 'none' value sets 'font-variant-ligatures' to 'none' and resets all other sub properties
            // to their initial value.
        % if engine == "gecko":
            ligatures = Some(FontVariantLigatures::NONE);
        % endif
        } else {
            let mut has_custom_value: bool = false;
            loop {
                if input.try_parse(|input| input.expect_ident_matching("normal")).is_ok() ||
                   input.try_parse(|input| input.expect_ident_matching("none")).is_ok() {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                }
            % for prop in sub_properties:
                if ${prop}.is_none() {
                    if let Ok(value) = input.try_parse(|i| font_variant_${prop}::parse(context, i)) {
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
            % if engine == "gecko":
                self.font_variant_ligatures == &FontVariantLigatures::NONE;
            % else:
                false;
            % endif

            const TOTAL_SUBPROPS: usize = ${len(sub_properties)};
            let mut nb_normals = 0;
        % for prop in sub_properties:
        % if prop == "emoji":
            if let Some(value) = self.font_variant_${prop} {
        % else:
            {
                let value = self.font_variant_${prop};
        % endif
                if value == &font_variant_${prop}::get_initial_specified_value() {
                   nb_normals += 1;
                }
            }
        % if prop == "emoji":
            else {
                // The property was disabled, so we count it as 'normal' for the purpose
                // of deciding how the shorthand can be serialized.
                nb_normals += 1;
            }
        % endif
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
            % if prop == "emoji":
                if let Some(value) = self.font_variant_${prop} {
            % else:
                {
                    let value = self.font_variant_${prop};
            % endif
                    if value != &font_variant_${prop}::get_initial_specified_value() {
                        if has_any {
                            dest.write_char(' ')?;
                        }
                        has_any = true;
                        value.to_css(dest)?;
                    }
                }
            % endfor
            }

            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="font-synthesis"
                    engines="gecko"
                    flags="SHORTHAND_IN_GETCS"
                    sub_properties="font-synthesis-weight font-synthesis-style font-synthesis-small-caps"
                    derive_value_info="False"
                    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-variant">
    <% sub_properties = ["weight", "style", "small_caps"] %>

    use crate::values::specified::FontSynthesis;

    pub fn parse_value<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
    % for prop in sub_properties:
        let mut ${prop} = FontSynthesis::None;
    % endfor

        if input.try_parse(|input| input.expect_ident_matching("none")).is_ok() {
            // Leave all the individual values as None
        } else {
            let mut has_custom_value = false;
            while !input.is_exhausted() {
                try_match_ident_ignore_ascii_case! { input,
                % for prop in sub_properties:
                    "${prop.replace('_', '-')}" if ${prop} == FontSynthesis::None => {
                        has_custom_value = true;
                        ${prop} = FontSynthesis::Auto;
                        continue;
                    },
                % endfor
                }
            }
            if !has_custom_value {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
        }

        Ok(expanded! {
        % for prop in sub_properties:
            font_synthesis_${prop}: ${prop},
        % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            let mut has_any = false;

        % for prop in sub_properties:
            if self.font_synthesis_${prop} == &FontSynthesis::Auto {
                if has_any {
                    dest.write_char(' ')?;
                }
                has_any = true;
                dest.write_str("${prop.replace('_', '-')}")?;
            }
        % endfor

            if !has_any {
                dest.write_str("none")?;
            }

            Ok(())
        }
    }

    // The shorthand takes the sub-property names of the longhands, and not the
    // 'auto' keyword like they do, so we can't automatically derive this.
    impl SpecifiedValueInfo for Longhands {
        fn collect_completion_keywords(f: KeywordsCollectFn) {
            f(&[
                "none",
            % for prop in sub_properties:
                "${prop.replace('_', '-')}",
            % endfor
            ]);
        }
    }
</%helpers:shorthand>
