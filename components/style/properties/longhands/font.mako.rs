/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method, to_camel_case, to_rust_ident, to_camel_case_lower, SYSTEM_FONT_LONGHANDS %>

<% data.new_style_struct("Font", inherited=True) %>

${helpers.predefined_type(
    "font-family",
    "FontFamily",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_value="computed::FontFamily::serif()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-family",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "font-style",
    "FontStyle",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_value="computed::FontStyle::normal()",
    initial_specified_value="specified::FontStyle::normal()",
    animation_value_type="FontStyle",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-style",
    servo_restyle_damage="rebuild_and_reflow",
)}

<% font_variant_caps_custom_consts= { "small-caps": "SMALLCAPS",
                                      "all-small-caps": "ALLSMALL",
                                      "petite-caps": "PETITECAPS",
                                      "all-petite-caps": "ALLPETITE",
                                      "titling-caps": "TITLING" } %>

${helpers.single_keyword_system(
    "font-variant-caps",
    "normal small-caps",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    extra_gecko_values="all-small-caps petite-caps all-petite-caps unicase titling-caps",
    gecko_constant_prefix="NS_FONT_VARIANT_CAPS",
    gecko_ffi_name="mFont.variantCaps",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-caps",
    custom_consts=font_variant_caps_custom_consts,
    animation_value_type="discrete",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "font-weight",
    "FontWeight",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_value="computed::FontWeight::normal()",
    initial_specified_value="specified::FontWeight::normal()",
    animation_value_type="Number",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-weight",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "font-size",
    "FontSize",
    engines="gecko servo-2013 servo-2020",
    initial_value="computed::FontSize::medium()",
    initial_specified_value="specified::FontSize::medium()",
    animation_value_type="NonNegativeLength",
    allow_quirks="Yes",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-size",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "font-size-adjust",
    "FontSizeAdjust",
    engines="gecko",
    initial_value="computed::FontSizeAdjust::none()",
    initial_specified_value="specified::FontSizeAdjust::none()",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-size-adjust",
)}

${helpers.predefined_type(
    "font-synthesis",
    "FontSynthesis",
    engines="gecko",
    initial_value="specified::FontSynthesis::get_initial_value()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-synthesis",
)}

${helpers.predefined_type(
    "font-stretch",
    "FontStretch",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_value="computed::FontStretch::hundred()",
    initial_specified_value="specified::FontStretch::normal()",
    animation_value_type="Percentage",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-stretch",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword_system(
    "font-kerning",
    "auto none normal",
    engines="gecko",
    gecko_ffi_name="mFont.kerning",
    gecko_constant_prefix="NS_FONT_KERNING",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-kerning",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "font-variant-alternates",
    "FontVariantAlternates",
    engines="gecko",
    initial_value="computed::FontVariantAlternates::get_initial_value()",
    initial_specified_value="specified::FontVariantAlternates::get_initial_specified_value()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-alternates",
)}

${helpers.predefined_type(
    "font-variant-east-asian",
    "FontVariantEastAsian",
    engines="gecko",
    initial_value="computed::FontVariantEastAsian::empty()",
    initial_specified_value="specified::FontVariantEastAsian::empty()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-east-asian",
)}

${helpers.predefined_type(
    "font-variant-ligatures",
    "FontVariantLigatures",
    engines="gecko",
    initial_value="computed::FontVariantLigatures::empty()",
    initial_specified_value="specified::FontVariantLigatures::empty()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-ligatures",
)}

${helpers.predefined_type(
    "font-variant-numeric",
    "FontVariantNumeric",
    engines="gecko",
    initial_value="computed::FontVariantNumeric::empty()",
    initial_specified_value="specified::FontVariantNumeric::empty()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-numeric",
)}

${helpers.single_keyword_system(
    "font-variant-position",
    "normal sub super",
    engines="gecko",
    gecko_ffi_name="mFont.variantPosition",
    gecko_constant_prefix="NS_FONT_VARIANT_POSITION",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-position",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "font-feature-settings",
    "FontFeatureSettings",
    engines="gecko",
    initial_value="computed::FontFeatureSettings::normal()",
    initial_specified_value="specified::FontFeatureSettings::normal()",
    extra_prefixes="moz:layout.css.prefixes.font-features",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-fonts/#propdef-font-feature-settings",
)}

${helpers.predefined_type(
    "font-variation-settings",
    "FontVariationSettings",
    engines="gecko",
    gecko_pref="layout.css.font-variations.enabled",
    has_effect_on_gecko_scrollbars=False,
    initial_value="computed::FontVariationSettings::normal()",
    initial_specified_value="specified::FontVariationSettings::normal()",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-fonts-4/#propdef-font-variation-settings"
)}

${helpers.predefined_type(
    "font-language-override",
    "FontLanguageOverride",
    engines="gecko",
    initial_value="computed::FontLanguageOverride::zero()",
    initial_specified_value="specified::FontLanguageOverride::normal()",
    animation_value_type="discrete",
    extra_prefixes="moz:layout.css.prefixes.font-features",
    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-language-override",
)}

${helpers.single_keyword_system(
    "font-optical-sizing",
    "auto none",
    engines="gecko",
    gecko_pref="layout.css.font-variations.enabled",
    has_effect_on_gecko_scrollbars=False,
    gecko_ffi_name="mFont.opticalSizing",
    gecko_constant_prefix="NS_FONT_OPTICAL_SIZING",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/css-fonts-4/#font-optical-sizing-def",
)}

${helpers.predefined_type(
    "-x-lang",
    "XLang",
    engines="gecko",
    initial_value="computed::XLang::get_initial_value()",
    animation_value_type="none",
    enabled_in="",
    spec="Internal (not web-exposed)",
)}

${helpers.predefined_type(
    "-moz-script-size-multiplier",
    "MozScriptSizeMultiplier",
    engines="gecko",
    initial_value="computed::MozScriptSizeMultiplier::get_initial_value()",
    animation_value_type="none",
    gecko_ffi_name="mScriptSizeMultiplier",
    enabled_in="",
    spec="Internal (not web-exposed)",
)}

${helpers.predefined_type(
    "-moz-script-level",
    "MozScriptLevel",
    0,
    engines="gecko",
    animation_value_type="none",
    enabled_in="ua",
    gecko_ffi_name="mScriptLevel",
    spec="Internal (not web-exposed)",
)}

${helpers.single_keyword(
    "-moz-math-display",
    "inline block",
    engines="gecko",
    gecko_constant_prefix="NS_MATHML_DISPLAYSTYLE",
    gecko_ffi_name="mMathDisplay",
    enabled_in="ua",
    spec="Internal (not web-exposed)",
    animation_value_type="none",
)}

${helpers.single_keyword(
    "-moz-math-variant",
    """none normal bold italic bold-italic script bold-script
    fraktur double-struck bold-fraktur sans-serif
    bold-sans-serif sans-serif-italic sans-serif-bold-italic
    monospace initial tailed looped stretched""",
    engines="gecko",
    gecko_constant_prefix="NS_MATHML_MATHVARIANT",
    gecko_ffi_name="mMathVariant",
    spec="Internal (not web-exposed)",
    animation_value_type="none",
    enabled_in="",
    needs_conversion=True,
)}

${helpers.predefined_type(
    "-moz-script-min-size",
    "MozScriptMinSize",
    "specified::MozScriptMinSize::get_initial_value()",
    engines="gecko",
    animation_value_type="none",
    enabled_in="",
    gecko_ffi_name="mScriptMinSize",
    spec="Internal (not web-exposed)",
)}

${helpers.predefined_type(
    "-x-text-zoom",
    "XTextZoom",
    "computed::XTextZoom(true)",
    engines="gecko",
    animation_value_type="none",
    enabled_in="",
    spec="Internal (not web-exposed)",
)}

% if engine == "gecko":
    pub mod system_font {
        //! We deal with system fonts here
        //!
        //! System fonts can only be set as a group via the font shorthand.
        //! They resolve at compute time (not parse time -- this lets the
        //! browser respond to changes to the OS font settings).
        //!
        //! While Gecko handles these as a separate property and keyword
        //! values on each property indicating that the font should be picked
        //! from the -x-system-font property, we avoid this. Instead,
        //! each font longhand has a special SystemFont variant which contains
        //! the specified system font. When the cascade function (in helpers)
        //! detects that a value has a system font, it will resolve it, and
        //! cache it on the ComputedValues. After this, it can be just fetched
        //! whenever a font longhand on the same element needs the system font.
        //!
        //! When a longhand property is holding a SystemFont, it's serialized
        //! to an empty string as if its value comes from a shorthand with
        //! variable reference. We may want to improve this behavior at some
        //! point. See also https://github.com/w3c/csswg-drafts/issues/1586.

        use app_units::Au;
        use cssparser::{Parser, ToCss};
        use crate::values::computed::font::GenericFontFamily;
        use crate::properties::longhands;
        use std::fmt;
        use std::hash::{Hash, Hasher};
        use style_traits::ParseError;
        use crate::values::computed::{ToComputedValue, Context};

        <%
            system_fonts = """caption icon menu message-box small-caption status-bar
                              -moz-window -moz-document -moz-workspace -moz-desktop
                              -moz-info -moz-dialog -moz-button -moz-pull-down-menu
                              -moz-list -moz-field""".split()
            kw_font_props = """font_variant_caps
                               font_kerning font_variant_position font_variant_ligatures
                               font_variant_east_asian font_variant_numeric
                               font_optical_sizing""".split()
            kw_cast = """font_variant_caps font_kerning font_variant_position
                         font_optical_sizing""".split()
        %>
        #[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq,
                 SpecifiedValueInfo, ToCss, ToShmem)]
        pub enum SystemFont {
            % for font in system_fonts:
                ${to_camel_case(font)},
            % endfor
        }

        // ComputedValues are compared at times
        // so we need these impls. We don't want to
        // add Eq to Number (which contains a float)
        // so instead we have an eq impl which skips the
        // cached values
        impl PartialEq for ComputedSystemFont {
            fn eq(&self, other: &Self) -> bool {
                self.system_font == other.system_font
            }
        }
        impl Eq for ComputedSystemFont {}

        impl Hash for ComputedSystemFont {
            fn hash<H: Hasher>(&self, hasher: &mut H) {
                self.system_font.hash(hasher)
            }
        }

        impl ToComputedValue for SystemFont {
            type ComputedValue = ComputedSystemFont;

            fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
                use crate::gecko_bindings::bindings;
                use crate::gecko_bindings::structs::{LookAndFeel_FontID, nsFont};
                use std::mem;
                use crate::values::computed::Percentage;
                use crate::values::computed::font::{FontFamily, FontSize, FontStretch, FontStyle, FontFamilyList};
                use crate::values::generics::NonNegative;

                let id = match *self {
                    % for font in system_fonts:
                        SystemFont::${to_camel_case(font)} => {
                            LookAndFeel_FontID::eFont_${to_camel_case(font.replace("-moz-", ""))}
                        }
                    % endfor
                };

                let mut system = mem::MaybeUninit::<nsFont>::uninit();
                let system = unsafe {
                    bindings::Gecko_nsFont_InitSystem(
                        system.as_mut_ptr(),
                        id as i32,
                        cx.style().get_font().gecko(),
                        cx.device().document()
                    );
                    &mut *system.as_mut_ptr()
                };
                let font_weight = longhands::font_weight::computed_value::T::from_gecko_weight(system.weight);
                let font_stretch = FontStretch(NonNegative(Percentage(unsafe {
                    bindings::Gecko_FontStretch_ToFloat(system.stretch)
                })));
                let font_style = FontStyle::from_gecko(system.style);
                let ret = ComputedSystemFont {
                    font_family: FontFamily {
                        families: FontFamilyList::SharedFontList(
                            unsafe { system.fontlist.mFontlist.mBasePtr.to_safe() }
                        ),
                        is_system_font: true,
                    },
                    font_size: FontSize {
                        size: NonNegative(cx.maybe_zoom_text(Au(system.size).into())),
                        keyword_info: None
                    },
                    font_weight,
                    font_stretch,
                    font_style,
                    font_size_adjust: longhands::font_size_adjust::computed_value
                                               ::T::from_gecko_adjust(system.sizeAdjust),
                    % for kwprop in kw_font_props:
                        ${kwprop}: longhands::${kwprop}::computed_value::T::from_gecko_keyword(
                            system.${to_camel_case_lower(kwprop.replace('font_', ''))}
                            % if kwprop in kw_cast:
                                as u32
                            % endif
                        ),
                    % endfor
                    font_language_override: longhands::font_language_override::computed_value
                                                     ::T(system.languageOverride),
                    font_feature_settings: longhands::font_feature_settings::get_initial_value(),
                    font_variation_settings: longhands::font_variation_settings::get_initial_value(),
                    font_variant_alternates: longhands::font_variant_alternates::get_initial_value(),
                    system_font: *self,
                    default_font_type: system.fontlist.mDefaultFontType,
                };
                unsafe { bindings::Gecko_nsFont_Destroy(system); }
                ret
            }

            fn from_computed_value(_: &ComputedSystemFont) -> Self {
                unreachable!()
            }
        }

        #[inline]
        /// Compute and cache a system font
        ///
        /// Must be called before attempting to compute a system font
        /// specified value
        pub fn resolve_system_font(system: SystemFont, context: &mut Context) {
            // Checking if context.cached_system_font.is_none() isn't enough,
            // if animating from one system font to another the cached system font
            // may change
            if Some(system) != context.cached_system_font.as_ref().map(|x| x.system_font) {
                let computed = system.to_computed_value(context);
                context.cached_system_font = Some(computed);
            }
        }

        #[derive(Clone, Debug)]
        pub struct ComputedSystemFont {
            % for name in SYSTEM_FONT_LONGHANDS:
                pub ${name}: longhands::${name}::computed_value::T,
            % endfor
            pub system_font: SystemFont,
            pub default_font_type: GenericFontFamily,
        }

        impl SystemFont {
            pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
                try_match_ident_ignore_ascii_case! { input,
                    % for font in system_fonts:
                        "${font}" => Ok(SystemFont::${to_camel_case(font)}),
                    % endfor
                }
            }
        }

        impl ToCss for SystemFont {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                // We may want to do something better in the future, see
                // w3c/csswg-drafts#1586.
                dest.write_str("-moz-use-system-font")
            }
        }
    }
% else:
    pub mod system_font {
        use cssparser::Parser;

        // We don't parse system fonts, but in the interest of not littering
        // a lot of code with `if engine == "gecko"` conditionals, we have a
        // dummy system font module that does nothing

        #[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
        /// void enum for system font, can never exist
        pub enum SystemFont {}
        impl SystemFont {
            pub fn parse(_: &mut Parser) -> Result<Self, ()> {
                Err(())
            }
        }
    }
% endif

${helpers.single_keyword(
    "-moz-osx-font-smoothing",
    "auto grayscale",
    engines="gecko",
    gecko_constant_prefix="NS_FONT_SMOOTHING",
    gecko_ffi_name="mFont.smoothing",
    gecko_pref="layout.css.osx-font-smoothing.enabled",
    has_effect_on_gecko_scrollbars=False,
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/font-smooth)",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "-moz-font-smoothing-background-color",
    "color::MozFontSmoothingBackgroundColor",
    "computed::color::MozFontSmoothingBackgroundColor::transparent()",
    engines="gecko",
    animation_value_type="none",
    gecko_ffi_name="mFont.fontSmoothingBackgroundColor",
    enabled_in="chrome",
    spec="None (Nonstandard internal property)",
)}

${helpers.predefined_type(
    "-moz-min-font-size-ratio",
    "Percentage",
    "computed::Percentage::hundred()",
    engines="gecko",
    animation_value_type="none",
    enabled_in="ua",
    spec="Nonstandard (Internal-only)",
)}
