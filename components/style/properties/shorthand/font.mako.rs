/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import SYSTEM_FONT_LONGHANDS %>

<%helpers:shorthand name="font"
                    sub_properties="font-style font-variant-caps font-weight font-stretch
                                    font-size line-height font-family
                                    ${'font-size-adjust' if product == 'gecko' or data.testing else ''}
                                    ${'font-kerning' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-alternates' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-east-asian' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-ligatures' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-numeric' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-position' if product == 'gecko' or data.testing else ''}
                                    ${'font-language-override' if product == 'gecko' or data.testing else ''}
                                    ${'font-feature-settings' if product == 'gecko' or data.testing else ''}"
                    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font">
    use properties::longhands::{font_family, font_style, font_weight, font_stretch};
    use properties::longhands::{font_size, line_height, font_variant_caps};
    #[cfg(feature = "gecko")]
    use properties::longhands::system_font::SystemFont;

    <%
        gecko_sub_properties = "kerning language_override size_adjust \
                                variant_alternates variant_east_asian \
                                variant_ligatures variant_numeric \
                                variant_position feature_settings".split()
    %>
    % if product == "gecko" or data.testing:
        % for prop in gecko_sub_properties:
            use properties::longhands::font_${prop};
        % endfor
    % endif
    use self::font_family::SpecifiedValue as FontFamily;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
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
                         ${name}: ${name}::SpecifiedValue::system_font(sys),
                     % endfor
                     // line-height is just reset to initial
                     line_height: line_height::get_initial_specified_value(),
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
            size = Some(try!(font_size::parse(context, input)));
            break
        }
        #[inline]
        fn count<T>(opt: &Option<T>) -> u8 {
            if opt.is_some() { 1 } else { 0 }
        }
        if size.is_none() ||
           (count(&style) + count(&weight) + count(&variant_caps) + count(&stretch) + nb_normals) > 4 {
            return Err(())
        }
        let line_height = if input.try(|input| input.expect_delim('/')).is_ok() {
            Some(try!(line_height::parse(context, input)))
        } else {
            None
        };
        let family = FontFamily::parse(input)?;
        Ok(expanded! {
            % for name in "style weight stretch size variant_caps".split():
                font_${name}: unwrap_or_initial!(font_${name}, ${name}),
            % endfor
            line_height: unwrap_or_initial!(line_height),
            font_family: family,
            % if product == "gecko" or data.testing:
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
        fn to_css_for<W>(&self,
                         serialize_for: SerializeFor,
                         dest: &mut W) -> fmt::Result where W: fmt::Write {
            % if product == "gecko":
                match self.check_system() {
                    CheckSystemResult::AllSystem(sys) => return sys.to_css(dest),
                    CheckSystemResult::SomeSystem => return Ok(()),
                    CheckSystemResult::None => ()
                }
            % endif

            % if product == "gecko" or data.testing:
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

            match *self.line_height {
                line_height::SpecifiedValue::Normal => {},
                _ => {
                    dest.write_str("/")?;
                    self.line_height.to_css(dest)?;
                }
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
                if self.line_height != &line_height::get_initial_specified_value() {
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
            pub fn to_css_for_canvas<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.to_css_for(SerializeFor::Canvas, dest)
            }
        % endif
    }

    // This may be a bit off, unsure, possibly needs changes
    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.to_css_for(SerializeFor::Normal, dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="font-variant"
                    sub_properties="font-variant-caps
                                    ${'font-variant-alternates' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-east-asian' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-ligatures' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-numeric' if product == 'gecko' or data.testing else ''}
                                    ${'font-variant-position' if product == 'gecko' or data.testing else ''}"
                    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-variant">
    use properties::longhands::font_variant_caps;
    <% gecko_sub_properties = "alternates east_asian ligatures numeric position".split() %>
    % if product == "gecko" or data.testing:
        % for prop in gecko_sub_properties:
            use properties::longhands::font_variant_${prop};
        % endfor
    % endif

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut nb_normals = 0;
        let mut caps = None;
        loop {
            // Special-case 'normal' because it is valid in each of
            // all sub properties.
            // Leaves the values to None, 'normal' is the initial value for each of them.
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                nb_normals += 1;
                continue;
            }
            if caps.is_none() {
                if let Ok(value) = input.try(|input| font_variant_caps::parse(context, input)) {
                    caps = Some(value);
                    continue
                }
            }
            break
        }
        #[inline]
        fn count<T>(opt: &Option<T>) -> u8 {
            if opt.is_some() { 1 } else { 0 }
        }
        let count = count(&caps) + nb_normals;
        if count == 0 || count > 1 {
            return Err(())
        }
        Ok(expanded! {
            font_variant_caps: unwrap_or_initial!(font_variant_caps, caps),
            // FIXME: Bug 1356134 - parse all sub properties.
            % if product == "gecko" or data.testing:
                % for name in gecko_sub_properties:
                    font_variant_${name}: font_variant_${name}::get_initial_specified_value(),
                % endfor
            % endif
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {

    % if product == "gecko" or data.testing:
        % for name in gecko_sub_properties:
            // FIXME: Bug 1356134 - handle all sub properties.
            if self.font_variant_${name} != &font_variant_${name}::get_initial_specified_value() {
                return Ok(());
            }
        % endfor
    % endif

            self.font_variant_caps.to_css(dest)?;

            Ok(())
        }
    }
</%helpers:shorthand>
