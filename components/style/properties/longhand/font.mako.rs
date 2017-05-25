/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method, to_camel_case, to_rust_ident, to_camel_case_lower, SYSTEM_FONT_LONGHANDS %>

<% data.new_style_struct("Font",
                         inherited=True) %>

<%def name="nongecko_unreachable()">
    %if product == "gecko":
        ${caller.body()}
    %else:
        unreachable!()
    %endif
</%def>

// Define ToComputedValue, ToCss, and other boilerplate for a specified value
// which is of the form `enum SpecifiedValue {Value(..), System(SystemFont)}`
<%def name="simple_system_boilerplate(name)">
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Value(ref v) => v.to_css(dest),
                SpecifiedValue::System(_) => Ok(())
            }
        }
    }


    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, _context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::Value(ref v) => v.clone(),
                SpecifiedValue::System(_) => {
                    <%self:nongecko_unreachable>
                        _context.cached_system_font.as_ref().unwrap().${name}.clone()
                    </%self:nongecko_unreachable>
                }
            }
        }

        fn from_computed_value(other: &computed_value::T) -> Self {
            SpecifiedValue::Value(other.clone())
        }
    }
</%def>

<%helpers:longhand name="font-family" animation_value_type="none" need_index="True"  boxed="${product == 'gecko'}"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-family">
    use properties::longhands::system_font::SystemFont;
    use self::computed_value::{FontFamily, FamilyName};
    use std::fmt;
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use cssparser::{CssStringWriter, Parser, serialize_identifier};
        use std::fmt::{self, Write};
        use Atom;
        use style_traits::ToCss;
        pub use self::FontFamily as SingleComputedValue;

        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum FontFamily {
            FamilyName(FamilyName),
            Generic(Atom),
        }

        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub struct FamilyName {
            pub name: Atom,
            pub quoted: bool,
        }

        impl FontFamily {
            #[inline]
            pub fn atom(&self) -> &Atom {
                match *self {
                    FontFamily::FamilyName(ref family_name) => &family_name.name,
                    FontFamily::Generic(ref name) => name,
                }
            }

            #[inline]
            #[cfg(not(feature = "gecko"))] // Gecko can't borrow atoms as UTF-8.
            pub fn name(&self) -> &str {
                self.atom()
            }

            #[cfg(not(feature = "gecko"))] // Gecko can't borrow atoms as UTF-8.
            pub fn from_atom(input: Atom) -> FontFamily {
                match input {
                    atom!("serif") |
                    atom!("sans-serif") |
                    atom!("cursive") |
                    atom!("fantasy") |
                    atom!("monospace") => {
                        return FontFamily::Generic(input)
                    }
                    _ => {}
                }
                match_ignore_ascii_case! { &input,
                    "serif" => return FontFamily::Generic(atom!("serif")),
                    "sans-serif" => return FontFamily::Generic(atom!("sans-serif")),
                    "cursive" => return FontFamily::Generic(atom!("cursive")),
                    "fantasy" => return FontFamily::Generic(atom!("fantasy")),
                    "monospace" => return FontFamily::Generic(atom!("monospace")),
                    _ => {}
                }

                // We don't know if it's quoted or not. So we set it to
                // quoted by default.
                FontFamily::FamilyName(FamilyName {
                    name: input,
                    quoted: true,
                })
            }

            /// Parse a font-family value
            pub fn parse(input: &mut Parser) -> Result<Self, ()> {
                if let Ok(value) = input.try(|input| input.expect_string()) {
                    return Ok(FontFamily::FamilyName(FamilyName {
                        name: Atom::from(&*value),
                        quoted: true,
                    }))
                }
                let first_ident = try!(input.expect_ident());

                // FIXME(bholley): The fast thing to do here would be to look up the
                // string (as lowercase) in the static atoms table. We don't have an
                // API to do that yet though, so we do the simple thing for now.
                let mut css_wide_keyword = false;
                match_ignore_ascii_case! { &first_ident,
                    "serif" => return Ok(FontFamily::Generic(atom!("serif"))),
                    "sans-serif" => return Ok(FontFamily::Generic(atom!("sans-serif"))),
                    "cursive" => return Ok(FontFamily::Generic(atom!("cursive"))),
                    "fantasy" => return Ok(FontFamily::Generic(atom!("fantasy"))),
                    "monospace" => return Ok(FontFamily::Generic(atom!("monospace"))),
                    % if product == "gecko":
                        "-moz-fixed" => return Ok(FontFamily::Generic(atom!("-moz-fixed"))),
                    % endif

                    // https://drafts.csswg.org/css-fonts/#propdef-font-family
                    // "Font family names that happen to be the same as a keyword value
                    //  (`inherit`, `serif`, `sans-serif`, `monospace`, `fantasy`, and `cursive`)
                    //  must be quoted to prevent confusion with the keywords with the same names.
                    //  The keywords ‘initial’ and ‘default’ are reserved for future use
                    //  and must also be quoted when used as font names.
                    //  UAs must not consider these keywords as matching the <family-name> type."
                    "inherit" => css_wide_keyword = true,
                    "initial" => css_wide_keyword = true,
                    "unset" => css_wide_keyword = true,
                    "default" => css_wide_keyword = true,
                    _ => {}
                }

                let mut value = first_ident.into_owned();
                // These keywords are not allowed by themselves.
                // The only way this value can be valid with with another keyword.
                if css_wide_keyword {
                    let ident = input.expect_ident()?;
                    value.push_str(" ");
                    value.push_str(&ident);
                }
                while let Ok(ident) = input.try(|input| input.expect_ident()) {
                    value.push_str(" ");
                    value.push_str(&ident);
                }
                Ok(FontFamily::FamilyName(FamilyName {
                    name: Atom::from(value),
                    quoted: false,
                }))
            }
        }

        impl ToCss for FamilyName {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.quoted {
                    dest.write_char('"')?;
                    write!(CssStringWriter::new(dest), "{}", self.name)?;
                    dest.write_char('"')
                } else {
                    serialize_identifier(&*self.name.to_string(), dest)
                }
            }
        }

        impl ToCss for FontFamily {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    FontFamily::FamilyName(ref name) => name.to_css(dest),

                    // All generic values accepted by the parser are known to not require escaping.
                    FontFamily::Generic(ref name) => {
                        % if product == "gecko":
                            // We should treat -moz-fixed as monospace
                            if name == &atom!("-moz-fixed") {
                                return write!(dest, "monospace");
                            }
                        % endif

                        write!(dest, "{}", name)
                    },
                }
            }
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
                try!(iter.next().unwrap().to_css(dest));
                for family in iter {
                    try!(dest.write_str(", "));
                    try!(family.to_css(dest));
                }
                Ok(())
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<FontFamily>);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![FontFamily::Generic(atom!("serif"))])
    }

    /// <family-name>#
    /// <family-name> = <string> | [ <ident>+ ]
    /// TODO: <generic-family>
    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        SpecifiedValue::parse(input)
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SpecifiedValue {
        Values(Vec<FontFamily>),
        System(SystemFont),
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;
        fn to_computed_value(&self, _cx: &Context) -> Self::ComputedValue {
            match *self {
                SpecifiedValue::Values(ref v) => computed_value::T(v.clone()),
                SpecifiedValue::System(_) => {
                    <%self:nongecko_unreachable>
                        _cx.cached_system_font.as_ref().unwrap().font_family.clone()
                    </%self:nongecko_unreachable>
                }
            }
        }
        fn from_computed_value(other: &computed_value::T) -> Self {
            SpecifiedValue::Values(other.0.clone())
        }
    }

    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }

        pub fn parse(input: &mut Parser) -> Result<Self, ()> {
            input.parse_comma_separated(|input| FontFamily::parse(input)).map(SpecifiedValue::Values)
        }
    }


    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Values(ref v) => {
                    let mut iter = v.iter();
                    iter.next().unwrap().to_css(dest)?;
                    for family in iter {
                        dest.write_str(", ")?;
                        family.to_css(dest)?;
                    }
                    Ok(())
                }
                _ => Ok(())
            }
        }
    }

    /// `FamilyName::parse` is based on `FontFamily::parse` and not the other way around
    /// because we want the former to exclude generic family keywords.
    impl Parse for FamilyName {
        fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
            match FontFamily::parse(input) {
                Ok(FontFamily::FamilyName(name)) => Ok(name),
                Ok(FontFamily::Generic(_)) |
                Err(()) => Err(())
            }
        }
    }
</%helpers:longhand>

${helpers.single_keyword_system("font-style",
                                "normal italic oblique",
                                gecko_constant_prefix="NS_FONT_STYLE",
                                gecko_ffi_name="mFont.style",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-style",
                                animation_value_type="none")}


<% font_variant_caps_custom_consts= { "small-caps": "SMALLCAPS",
                                      "all-small-caps": "ALLSMALL",
                                      "petite-caps": "PETITECAPS",
                                      "all-petite-caps": "ALLPETITE",
                                      "titling-caps": "TITLING" } %>

${helpers.single_keyword_system("font-variant-caps",
                               "normal small-caps",
                               extra_gecko_values="all-small-caps petite-caps all-petite-caps unicase titling-caps",
                               gecko_constant_prefix="NS_FONT_VARIANT_CAPS",
                               gecko_ffi_name="mFont.variantCaps",
                               spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-caps",
                               custom_consts=font_variant_caps_custom_consts,
                               animation_value_type="none")}

<%helpers:longhand name="font-weight" need_clone="True" animation_value_type="ComputedValue"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-weight">
    use std::fmt;
    use style_traits::ToCss;
    use properties::longhands::system_font::SystemFont;

    no_viewport_percentage!(SpecifiedValue);

    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Normal,
        Bold,
        Bolder,
        Lighter,
        % for weight in range(100, 901, 100):
            Weight${weight},
        % endfor
        System(SystemFont),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Normal => dest.write_str("normal"),
                SpecifiedValue::Bold => dest.write_str("bold"),
                SpecifiedValue::Bolder => dest.write_str("bolder"),
                SpecifiedValue::Lighter => dest.write_str("lighter"),
                % for weight in range(100, 901, 100):
                    SpecifiedValue::Weight${weight} => dest.write_str("${weight}"),
                % endfor
                SpecifiedValue::System(_) => Ok(())
            }
        }
    }

    /// normal | bold | bolder | lighter | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        input.try(|input| {
            match_ignore_ascii_case! { &try!(input.expect_ident()),
                "normal" => Ok(SpecifiedValue::Normal),
                "bold" => Ok(SpecifiedValue::Bold),
                "bolder" => Ok(SpecifiedValue::Bolder),
                "lighter" => Ok(SpecifiedValue::Lighter),
                _ => Err(())
            }
        }).or_else(|()| {
            SpecifiedValue::from_int(input.expect_integer()?)
        })
    }

    impl SpecifiedValue {
        pub fn from_int(kw: i32) -> Result<Self, ()> {
            match kw {
                % for weight in range(100, 901, 100):
                    ${weight} => Ok(SpecifiedValue::Weight${weight}),
                % endfor
                _ => Err(())
            }
        }

        pub fn from_gecko_keyword(kw: u32) -> Self {
            Self::from_int(kw as i32).expect("Found unexpected value in style
                                              struct for font-weight property")
        }
    }

    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    }

    /// Used in @font-face, where relative keywords are not allowed.
    impl Parse for computed_value::T {
        fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
            match parse(context, input)? {
                % for weight in range(100, 901, 100):
                    SpecifiedValue::Weight${weight} => Ok(computed_value::T::Weight${weight}),
                % endfor
                SpecifiedValue::Normal => Ok(computed_value::T::Weight400),
                SpecifiedValue::Bold => Ok(computed_value::T::Weight700),
                SpecifiedValue::Bolder |
                SpecifiedValue::Lighter => Err(()),
                SpecifiedValue::System(..) => unreachable!(),
            }
        }
    }

    pub mod computed_value {
        #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        #[repr(u16)]
        pub enum T {
            % for weight in range(100, 901, 100):
                Weight${weight} = ${weight},
            % endfor
        }
        impl T {
            #[inline]
            pub fn is_bold(self) -> bool {
                match self {
                    T::Weight900 | T::Weight800 |
                    T::Weight700 | T::Weight600 => true,
                    _ => false
                }
            }

            /// Obtain a Servo computed value from a Gecko computed font-weight
            pub fn from_gecko_weight(weight: u16) -> Self {
                match weight {
                    % for weight in range(100, 901, 100):
                        ${weight} => T::Weight${weight},
                    % endfor
                    _ => panic!("from_gecko_weight: called with invalid weight")
                }
            }
        }
    }
    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                % for weight in range(100, 901, 100):
                    computed_value::T::Weight${weight} => dest.write_str("${weight}"),
                % endfor
            }
        }
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Weight400  // normal
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Normal
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                % for weight in range(100, 901, 100):
                    SpecifiedValue::Weight${weight} => computed_value::T::Weight${weight},
                % endfor
                SpecifiedValue::Normal => computed_value::T::Weight400,
                SpecifiedValue::Bold => computed_value::T::Weight700,
                SpecifiedValue::Bolder => match context.inherited_style().get_font().clone_font_weight() {
                    computed_value::T::Weight100 => computed_value::T::Weight400,
                    computed_value::T::Weight200 => computed_value::T::Weight400,
                    computed_value::T::Weight300 => computed_value::T::Weight400,
                    computed_value::T::Weight400 => computed_value::T::Weight700,
                    computed_value::T::Weight500 => computed_value::T::Weight700,
                    computed_value::T::Weight600 => computed_value::T::Weight900,
                    computed_value::T::Weight700 => computed_value::T::Weight900,
                    computed_value::T::Weight800 => computed_value::T::Weight900,
                    computed_value::T::Weight900 => computed_value::T::Weight900,
                },
                SpecifiedValue::Lighter => match context.inherited_style().get_font().clone_font_weight() {
                    computed_value::T::Weight100 => computed_value::T::Weight100,
                    computed_value::T::Weight200 => computed_value::T::Weight100,
                    computed_value::T::Weight300 => computed_value::T::Weight100,
                    computed_value::T::Weight400 => computed_value::T::Weight100,
                    computed_value::T::Weight500 => computed_value::T::Weight100,
                    computed_value::T::Weight600 => computed_value::T::Weight400,
                    computed_value::T::Weight700 => computed_value::T::Weight400,
                    computed_value::T::Weight800 => computed_value::T::Weight700,
                    computed_value::T::Weight900 => computed_value::T::Weight700,
                },
                SpecifiedValue::System(_) => {
                    <%self:nongecko_unreachable>
                        context.cached_system_font.as_ref().unwrap().font_weight.clone()
                    </%self:nongecko_unreachable>
                }
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                % for weight in range(100, 901, 100):
                    computed_value::T::Weight${weight} => SpecifiedValue::Weight${weight},
                % endfor
            }
        }
    }
</%helpers:longhand>

<%helpers:longhand name="font-size" need_clone="True" animation_value_type="ComputedValue"
                   allow_quirks="True" spec="https://drafts.csswg.org/css-fonts/#propdef-font-size">
    use app_units::Au;
    use properties::longhands::system_font::SystemFont;
    use properties::style_structs::Font;
    use std::fmt;
    use style_traits::{HasViewportPercentage, ToCss};
    use values::FONT_MEDIUM_PX;
    use values::specified::{AllowQuirks, FontRelativeLength, LengthOrPercentage};
    use values::specified::{NoCalcLength, Percentage};
    use values::specified::length::FontBaseSize;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Length(ref lop) => lop.to_css(dest),
                SpecifiedValue::Keyword(kw, _) => kw.to_css(dest),
                SpecifiedValue::Smaller => dest.write_str("smaller"),
                SpecifiedValue::Larger => dest.write_str("larger"),
                SpecifiedValue::System(_) => Ok(()),
            }
        }
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::Length(ref lop) => lop.has_viewport_percentage(),
                _ => false
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Length(specified::LengthOrPercentage),
        /// A keyword value, along with a ratio.
        /// The ratio in any specified keyword value
        /// will be 1, but we cascade keywordness even
        /// after font-relative (percent and em) values
        /// have been applied, which is where the keyword
        /// comes in. See bug 1355707
        Keyword(KeywordSize, f32),
        Smaller,
        Larger,
        System(SystemFont)
    }

    impl From<specified::LengthOrPercentage> for SpecifiedValue {
        fn from(other: specified::LengthOrPercentage) -> Self {
            SpecifiedValue::Length(other)
        }
    }

    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
    }

    /// CSS font keywords
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum KeywordSize {
        XXSmall = 0,
        XSmall = 1,
        Small = 2,
        Medium = 3,
        Large = 4,
        XLarge = 5,
        XXLarge = 6,
        // This is not a real font keyword and will not parse
        // HTML font-size 7 corresponds to this value
        XXXLarge = 7,
    }

    pub use self::KeywordSize::*;

    impl KeywordSize {
        pub fn parse(input: &mut Parser) -> Result<Self, ()> {
            Ok(match_ignore_ascii_case! {&*input.expect_ident()?,
                "xx-small" => XXSmall,
                "x-small" => XSmall,
                "small" => Small,
                "medium" => Medium,
                "large" => Large,
                "x-large" => XLarge,
                "xx-large" => XXLarge,
                _ => return Err(())
            })
        }
    }

    impl Default for KeywordSize {
        fn default() -> Self {
            Medium
        }
    }

    impl ToCss for KeywordSize {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            dest.write_str(match *self {
                XXSmall => "xx-small",
                XSmall => "x-small",
                Small => "small",
                Medium => "medium",
                Large => "large",
                XLarge => "x-large",
                XXLarge => "xx-large",
                XXXLarge => unreachable!("We should never serialize \
                                          specified values set via
                                          HTML presentation attributes"),
            })
        }
    }

    % if product == "servo":
        impl ToComputedValue for KeywordSize {
            type ComputedValue = Au;
            #[inline]
            fn to_computed_value(&self, _: &Context) -> computed_value::T {
                // https://drafts.csswg.org/css-fonts-3/#font-size-prop
                use values::FONT_MEDIUM_PX;
                match *self {
                    XXSmall => Au::from_px(FONT_MEDIUM_PX) * 3 / 5,
                    XSmall => Au::from_px(FONT_MEDIUM_PX) * 3 / 4,
                    Small => Au::from_px(FONT_MEDIUM_PX) * 8 / 9,
                    Medium => Au::from_px(FONT_MEDIUM_PX),
                    Large => Au::from_px(FONT_MEDIUM_PX) * 6 / 5,
                    XLarge => Au::from_px(FONT_MEDIUM_PX) * 3 / 2,
                    XXLarge => Au::from_px(FONT_MEDIUM_PX) * 2,
                    XXXLarge => Au::from_px(FONT_MEDIUM_PX) * 3,
                }
            }

            #[inline]
            fn from_computed_value(_: &computed_value::T) -> Self {
                unreachable!()
            }
        }
    % else:
        impl ToComputedValue for KeywordSize {
            type ComputedValue = Au;
            #[inline]
            fn to_computed_value(&self, cx: &Context) -> computed_value::T {
                use gecko_bindings::structs::nsIAtom;
                use values::specified::length::au_to_int_px;
                // Data from nsRuleNode.cpp in Gecko
                // Mapping from base size and HTML size to pixels
                // The first index is (base_size - 9), the second is the
                // HTML size. "0" is CSS keyword xx-small, not HTML size 0,
                // since HTML size 0 is the same as 1.
                //
                //  xxs   xs      s      m     l      xl     xxl   -
                //  -     0/1     2      3     4      5      6     7
                static FONT_SIZE_MAPPING: [[i32; 8]; 8] = [
                    [9,    9,     9,     9,    11,    14,    18,    27],
                    [9,    9,     9,    10,    12,    15,    20,    30],
                    [9,    9,    10,    11,    13,    17,    22,    33],
                    [9,    9,    10,    12,    14,    18,    24,    36],
                    [9,   10,    12,    13,    16,    20,    26,    39],
                    [9,   10,    12,    14,    17,    21,    28,    42],
                    [9,   10,    13,    15,    18,    23,    30,    45],
                    [9,   10,    13,    16,    18,    24,    32,    48]
                ];

                static FONT_SIZE_FACTORS: [i32; 8] = [60, 75, 89, 100, 120, 150, 200, 300];

                // XXXManishearth handle quirks mode

                let ref gecko_font = cx.style().get_font().gecko();
                let base_size = unsafe { Atom::with(gecko_font.mLanguage.raw::<nsIAtom>(), |atom| {
                    cx.font_metrics_provider.get_size(atom, gecko_font.mGenericID).0
                }) };

                let base_size_px = au_to_int_px(base_size as f32);
                let html_size = *self as usize;
                if base_size_px >= 9 && base_size_px <= 16 {
                    Au::from_px(FONT_SIZE_MAPPING[(base_size_px - 9) as usize][html_size])
                } else {
                    Au(FONT_SIZE_FACTORS[html_size] * base_size / 100)
                }
            }

            #[inline]
            fn from_computed_value(_: &computed_value::T) -> Self {
                unreachable!()
            }
        }
    % endif

    /// This is the ratio applied for font-size: larger
    /// and smaller by both Firefox and Chrome
    const LARGER_FONT_SIZE_RATIO: f32 = 1.2;

    impl SpecifiedValue {
        /// https://html.spec.whatwg.org/multipage/#rules-for-parsing-a-legacy-font-size
        pub fn from_html_size(size: u8) -> Self {
            SpecifiedValue::Keyword(match size {
                // If value is less than 1, let it be 1.
                0 | 1 => XSmall,
                2 => Small,
                3 => Medium,
                4 => Large,
                5 => XLarge,
                6 => XXLarge,
                // If value is greater than 7, let it be 7.
                _ => XXXLarge,
            }, 1.)
        }

        /// If this value is specified as a ratio of the parent font (em units or percent)
        /// return the ratio
        pub fn as_font_ratio(&self) -> Option<f32> {
            if let SpecifiedValue::Length(ref lop) = *self {
                if let LengthOrPercentage::Percentage(pc) = *lop {
                    return Some(pc.0)
                } else if let LengthOrPercentage::Length(ref nocalc) = *lop {
                    if let NoCalcLength::FontRelative(FontRelativeLength::Em(em)) = *nocalc {
                        return Some(em)
                    }
                }
            } else if let SpecifiedValue::Larger = *self {
                return Some(LARGER_FONT_SIZE_RATIO)
            } else if let SpecifiedValue::Smaller = *self {
                return Some(1. / LARGER_FONT_SIZE_RATIO)
            }
            None
        }

        /// Compute it against a given base font size
        pub fn to_computed_value_against(&self, context: &Context, base_size: FontBaseSize) -> Au {
            use values::specified::length::FontRelativeLength;
            match *self {
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        NoCalcLength::FontRelative(value))) => {
                    value.to_computed_value(context, base_size)
                }
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        NoCalcLength::ServoCharacterWidth(value))) => {
                    value.to_computed_value(base_size.resolve(context))
                }
                SpecifiedValue::Length(LengthOrPercentage::Length(ref l)) => {
                    l.to_computed_value(context)
                }
                SpecifiedValue::Length(LengthOrPercentage::Percentage(Percentage(value))) => {
                    base_size.resolve(context).scale_by(value)
                }
                SpecifiedValue::Length(LengthOrPercentage::Calc(ref calc)) => {
                    let calc = calc.to_computed_value(context);
                    calc.to_used_value(Some(base_size.resolve(context))).unwrap()
                }
                SpecifiedValue::Keyword(ref key, fraction) => {
                    key.to_computed_value(context).scale_by(fraction)
                }
                SpecifiedValue::Smaller => {
                    FontRelativeLength::Em(1. / LARGER_FONT_SIZE_RATIO)
                        .to_computed_value(context, base_size)
                }
                SpecifiedValue::Larger => {
                    FontRelativeLength::Em(LARGER_FONT_SIZE_RATIO)
                        .to_computed_value(context, base_size)
                }

                SpecifiedValue::System(_) => {
                    <%self:nongecko_unreachable>
                        context.cached_system_font.as_ref().unwrap().font_size
                    </%self:nongecko_unreachable>
                }
            }
        }
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_value() -> computed_value::T {
        Au::from_px(FONT_MEDIUM_PX)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Keyword(Medium, 1.)
    }


    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            self.to_computed_value_against(context, FontBaseSize::InheritedStyle)
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        ToComputedValue::from_computed_value(computed)
                ))
        }
    }

    /// <length> | <percentage> | <absolute-size> | <relative-size>
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        parse_quirky(context, input, AllowQuirks::No)
    }

    /// Parses a font-size, with quirks.
    pub fn parse_quirky(context: &ParserContext,
                        input: &mut Parser,
                        allow_quirks: AllowQuirks)
                        -> Result<SpecifiedValue, ()> {
        use self::specified::LengthOrPercentage;
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative_quirky(context, i, allow_quirks)) {
            return Ok(SpecifiedValue::Length(lop))
        }

        if let Ok(kw) = input.try(KeywordSize::parse) {
            return Ok(SpecifiedValue::Keyword(kw, 1.))
        }

        match_ignore_ascii_case! {&*input.expect_ident()?,
            "smaller" => Ok(SpecifiedValue::Smaller),
            "larger" => Ok(SpecifiedValue::Larger),
            _ => Err(())
        }
    }

    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    }

    #[allow(unused_mut)]
    pub fn cascade_specified_font_size(context: &mut Context,
                                      specified_value: &SpecifiedValue,
                                      mut computed: Au,
                                      parent: &Font) {
        if let SpecifiedValue::Keyword(kw, fraction)
                            = *specified_value {
            context.mutate_style().font_size_keyword = Some((kw, fraction));
        } else if let Some(ratio) = specified_value.as_font_ratio() {
            // In case a font-size-relative value was applied to a keyword
            // value, we must preserve this fact in case the generic font family
            // changes. relative values (em and %) applied to keywords must be
            // recomputed from the base size for the keyword and the relative size.
            //
            // See bug 1355707
            if let Some((kw, fraction)) = context.inherited_style().font_computation_data.font_size_keyword {
                context.mutate_style().font_size_keyword = Some((kw, fraction * ratio));
            } else {
                context.mutate_style().font_size_keyword = None;
            }
        } else {
            context.mutate_style().font_size_keyword = None;
        }

        // we could use clone_language and clone_font_family() here but that's
        // expensive. Do it only in gecko mode for now.
        % if product == "gecko":
            use gecko_bindings::structs::nsIAtom;
            // if the language or generic changed, we need to recalculate
            // the font size from the stored font-size origin information.
            if context.style().get_font().gecko().mLanguage.raw::<nsIAtom>() !=
               context.inherited_style().get_font().gecko().mLanguage.raw::<nsIAtom>() ||
               context.style().get_font().gecko().mGenericID !=
               context.inherited_style().get_font().gecko().mGenericID {
                if let Some((kw, ratio)) = context.style().font_size_keyword {
                    computed = kw.to_computed_value(context).scale_by(ratio);
                }
            }
        % endif

        let parent_unconstrained = context.mutate_style()
                                       .mutate_font()
                                       .apply_font_size(computed,
                                                        parent);

        if let Some(parent) = parent_unconstrained {
            let new_unconstrained = specified_value
                        .to_computed_value_against(context, FontBaseSize::Custom(parent));
            context.mutate_style()
                   .mutate_font()
                   .apply_unconstrained_font_size(new_unconstrained);
        }
    }

    pub fn cascade_inherit_font_size(context: &mut Context, parent: &Font) {
        // If inheriting, we must recompute font-size in case of language changes
        // using the font_size_keyword. We also need to do this to handle
        // mathml scriptlevel changes
        let kw_inherited_size = context.style().font_size_keyword.map(|(kw, ratio)| {
            SpecifiedValue::Keyword(kw, ratio).to_computed_value(context)
        });
        let used_kw = context.mutate_style().mutate_font()
               .inherit_font_size_from(parent, kw_inherited_size);
        if used_kw {
            context.mutate_style().font_size_keyword =
                context.inherited_style.font_computation_data.font_size_keyword;
        } else {
            context.mutate_style().font_size_keyword = None;
        }
    }

    pub fn cascade_initial_font_size(context: &mut Context) {
        // font-size's default ("medium") does not always
        // compute to the same value and depends on the font
        let computed = longhands::font_size::get_initial_specified_value()
                            .to_computed_value(context);
        context.mutate_style().mutate_${data.current_style_struct.name_lower}()
               .set_font_size(computed);
        context.mutate_style().font_size_keyword = Some((Default::default(), 1.));
    }
</%helpers:longhand>

<%helpers:longhand products="gecko" name="font-size-adjust" animation_value_type="ComputedValue"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-size-adjust">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        None,
        Number(specified::Number),
        System(SystemFont),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result
            where W: fmt::Write,
        {
            match *self {
                SpecifiedValue::None => dest.write_str("none"),
                SpecifiedValue::Number(number) => number.to_css(dest),
                SpecifiedValue::System(_) => Ok(()),
            }
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
            match *self {
                SpecifiedValue::None => computed_value::T::None,
                SpecifiedValue::Number(ref n) => computed_value::T::Number(n.to_computed_value(context)),
                SpecifiedValue::System(_) => {
                    <%self:nongecko_unreachable>
                        context.cached_system_font.as_ref().unwrap().font_size_adjust
                    </%self:nongecko_unreachable>
                }
            }
        }

        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::None => SpecifiedValue::None,
                computed_value::T::Number(ref v) => SpecifiedValue::Number(specified::Number::from_computed_value(v)),
            }
        }
    }

    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    }

    pub mod computed_value {
        use properties::animated_properties::Animatable;
        use std::fmt;
        use style_traits::ToCss;
        use values::CSSFloat;

        #[derive(Copy, Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            None,
            Number(CSSFloat),
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result
                where W: fmt::Write,
            {
                match *self {
                    T::None => dest.write_str("none"),
                    T::Number(number) => number.to_css(dest),
                }
            }
        }

        impl T {
            pub fn from_gecko_adjust(gecko: f32) -> Self {
                if gecko == -1.0 {
                    T::None
                } else {
                    T::Number(gecko)
                }
            }
        }

        impl Animatable for T {
            fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
                -> Result<Self, ()> {
                match (*self, *other) {
                    (T::Number(ref number), T::Number(ref other)) =>
                        Ok(T::Number(try!(number.add_weighted(other,
                                                              self_portion, other_portion)))),
                    _ => Err(()),
                }
            }

            #[inline]
            fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
                match (*self, *other) {
                    (T::Number(ref number), T::Number(ref other)) =>
                        number.compute_distance(other),
                    _ => Err(()),
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::None
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::None
    }

    /// none | <number>
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use values::specified::Number;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::None);
        }

        Ok(SpecifiedValue::Number(try!(Number::parse_non_negative(context, input))))
    }
</%helpers:longhand>

<%helpers:longhand products="gecko" name="font-synthesis" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-synthesis">
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub weight: bool,
        pub style: bool,
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.weight && self.style {
                dest.write_str("weight style")
            } else if self.style {
                dest.write_str("style")
            } else if self.weight {
                dest.write_str("weight")
            } else {
                dest.write_str("none")
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        SpecifiedValue { weight: true, style: true }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = SpecifiedValue { weight: false, style: false };
        match_ignore_ascii_case! { &try!(input.expect_ident()),
            "none" => Ok(result),
            "weight" => {
                result.weight = true;
                if input.try(|input| input.expect_ident_matching("style")).is_ok() {
                    result.style = true;
                }
                Ok(result)
            },
            "style" => {
                result.style = true;
                if input.try(|input| input.expect_ident_matching("weight")).is_ok() {
                    result.weight = true;
                }
                Ok(result)
            },
            _ => Err(())
        }
    }
</%helpers:longhand>

${helpers.single_keyword_system("font-stretch",
                                "normal ultra-condensed extra-condensed condensed \
                                 semi-condensed semi-expanded expanded extra-expanded \
                                 ultra-expanded",
                                gecko_ffi_name="mFont.stretch",
                                gecko_constant_prefix="NS_FONT_STRETCH",
                                cast_type='i16',
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-stretch",
                                animation_value_type="ComputedValue")}

${helpers.single_keyword_system("font-kerning",
                                "auto none normal",
                                products="gecko",
                                gecko_ffi_name="mFont.kerning",
                                gecko_constant_prefix="NS_FONT_KERNING",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-kerning",
                                animation_value_type="none")}

/// FIXME: Implement proper handling of each values.
/// https://github.com/servo/servo/issues/15957
<%helpers:longhand name="font-variant-alternates" products="gecko" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-alternates">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    bitflags! {
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub flags VariantAlternates: u8 {
            const NORMAL = 0,
            const HISTORICAL_FORMS = 0x01,
            const STYLISTIC = 0x02,
            const STYLESET = 0x04,
            const CHARACTER_VARIANT = 0x08,
            const SWASH = 0x10,
            const ORNAMENTS = 0x20,
            const ANNOTATION = 0x40,
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum SpecifiedValue {
        Value(VariantAlternates),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_variant_alternates"></%self:simple_system_boilerplate>

    <% font_variant_alternates_map = { "HISTORICAL_FORMS": "HISTORICAL",
                                       "STYLISTIC": "STYLISTIC",
                                       "STYLESET": "STYLESET",
                                       "CHARACTER_VARIANT": "CHARACTER_VARIANT",
                                       "SWASH": "SWASH",
                                       "ORNAMENTS": "ORNAMENTS",
                                       "ANNOTATION": "ANNOTATION" } %>
    ${helpers.gecko_bitflags_conversion(font_variant_alternates_map, 'NS_FONT_VARIANT_ALTERNATES_',
                                        'VariantAlternates', kw_type='u16')}

    impl ToCss for VariantAlternates {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("normal")
            }

            let mut has_any = false;

            macro_rules! write_value {
                ($ident:ident => $str:expr) => {
                    if self.intersects($ident) {
                        if has_any {
                            try!(dest.write_str(" "));
                        }
                        has_any = true;
                        try!(dest.write_str($str));
                    }
                }
            }

            write_value!(HISTORICAL_FORMS => "historical-forms");
            write_value!(STYLISTIC => "stylistic");
            write_value!(STYLESET => "styleset");
            write_value!(CHARACTER_VARIANT => "character-variant");
            write_value!(SWASH => "swash");
            write_value!(ORNAMENTS => "ornaments");
            write_value!(ANNOTATION => "annotation");

            debug_assert!(has_any);
            Ok(())
        }
    }

    pub mod computed_value {
        pub type T = super::VariantAlternates;
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::empty()
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(VariantAlternates::empty())
    }

    /// normal |
    ///  [ stylistic(<feature-value-name>)           ||
    ///    historical-forms                          ||
    ///    styleset(<feature-value-name> #)          ||
    ///    character-variant(<feature-value-name> #) ||
    ///    swash(<feature-value-name>)               ||
    ///    ornaments(<feature-value-name>)           ||
    ///    annotation(<feature-value-name>) ]
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = VariantAlternates::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Value(result))
        }

        while let Ok(ident) = input.try(|input| input.expect_ident()) {
            let flag = match_ignore_ascii_case! { &ident,
                "stylistic" => STYLISTIC,
                "historical-forms" => HISTORICAL_FORMS,
                "styleset" => STYLESET,
                "character-variant" => CHARACTER_VARIANT,
                "swash" => SWASH,
                "ornaments" => ORNAMENTS,
                "annotation" => ANNOTATION,
                _ => return Err(()),
            };
            if result.intersects(flag) {
                return Err(())
            }
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(SpecifiedValue::Value(result))
        } else {
            Err(())
        }
    }
</%helpers:longhand>

#[cfg(any(feature = "gecko", feature = "testing"))]
macro_rules! exclusive_value {
    (($value:ident, $set:expr) => $ident:ident) => {
        if $value.intersects($set) {
            return Err(())
        } else {
            $ident
        }
    }
}

<%helpers:longhand name="font-variant-east-asian" products="gecko" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-east-asian">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    bitflags! {
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub flags VariantEastAsian: u16 {
            const NORMAL = 0,
            const JIS78 = 0x01,
            const JIS83 = 0x02,
            const JIS90 = 0x04,
            const JIS04 = 0x08,
            const SIMPLIFIED = 0x10,
            const TRADITIONAL = 0x20,
            const FULL_WIDTH = 0x40,
            const PROPORTIONAL_WIDTH = 0x80,
            const RUBY = 0x100,
        }
    }


    #[derive(Debug, Clone, PartialEq)]
    pub enum SpecifiedValue {
        Value(VariantEastAsian),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_variant_east_asian"></%self:simple_system_boilerplate>

    //                                 servo_bit: gecko_bit
    <% font_variant_east_asian_map = { "JIS78": "JIS78",
                                       "JIS83": "JIS83",
                                       "JIS90": "JIS90",
                                       "JIS04": "JIS04",
                                       "SIMPLIFIED": "SIMPLIFIED",
                                       "TRADITIONAL": "TRADITIONAL",
                                       "FULL_WIDTH": "FULL_WIDTH",
                                       "PROPORTIONAL_WIDTH": "PROP_WIDTH",
                                       "RUBY": "RUBY" } %>

    ${helpers.gecko_bitflags_conversion(font_variant_east_asian_map, 'NS_FONT_VARIANT_EAST_ASIAN_',
                                        'VariantEastAsian', kw_type='u16')}


    impl ToCss for VariantEastAsian {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("normal")
            }

            let mut has_any = false;

            macro_rules! write_value {
                ($ident:ident => $str:expr) => {
                    if self.intersects($ident) {
                        if has_any {
                            try!(dest.write_str(" "));
                        }
                        has_any = true;
                        try!(dest.write_str($str));
                    }
                }
            }

            write_value!(JIS78 => "jis78");
            write_value!(JIS83 => "jis83");
            write_value!(JIS90 => "jis90");
            write_value!(JIS04 => "jis04");
            write_value!(SIMPLIFIED => "simplified");
            write_value!(TRADITIONAL => "traditional");
            write_value!(FULL_WIDTH => "full-width");
            write_value!(PROPORTIONAL_WIDTH => "proportional-width");
            write_value!(RUBY => "ruby");

            debug_assert!(has_any);
            Ok(())
        }
    }

    pub mod computed_value {
        pub type T = super::VariantEastAsian;
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::empty()
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(VariantEastAsian::empty())
    }

    /// normal | [ <east-asian-variant-values> || <east-asian-width-values> || ruby ]
    /// <east-asian-variant-values> = [ jis78 | jis83 | jis90 | jis04 | simplified | traditional ]
    /// <east-asian-width-values>   = [ full-width | proportional-width ]
    <% east_asian_variant_values = "JIS78 | JIS83 | JIS90 | JIS04 | SIMPLIFIED | TRADITIONAL" %>
    <% east_asian_width_values = "FULL_WIDTH | PROPORTIONAL_WIDTH" %>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = VariantEastAsian::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Value(result))
        }

        while let Ok(ident) = input.try(|input| input.expect_ident()) {
            let flag = match_ignore_ascii_case! { &ident,
                "jis78" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => JIS78),
                "jis83" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => JIS83),
                "jis90" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => JIS90),
                "jis04" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => JIS04),
                "simplified" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => SIMPLIFIED),
                "traditional" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => TRADITIONAL),
                "full-width" =>
                    exclusive_value!((result, ${east_asian_width_values}) => FULL_WIDTH),
                "proportional-width" =>
                    exclusive_value!((result, ${east_asian_width_values}) => PROPORTIONAL_WIDTH),
                "ruby" =>
                    exclusive_value!((result, RUBY) => RUBY),
                _ => return Err(()),
            };
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(SpecifiedValue::Value(result))
        } else {
            Err(())
        }
    }
</%helpers:longhand>

<%helpers:longhand name="font-variant-ligatures" products="gecko" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-ligatures">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    bitflags! {
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub flags VariantLigatures: u16 {
            const NORMAL = 0,
            const NONE = 0x01,
            const COMMON_LIGATURES = 0x02,
            const NO_COMMON_LIGATURES = 0x04,
            const DISCRETIONARY_LIGATURES = 0x08,
            const NO_DISCRETIONARY_LIGATURES = 0x10,
            const HISTORICAL_LIGATURES = 0x20,
            const NO_HISTORICAL_LIGATURES = 0x40,
            const CONTEXTUAL = 0x80,
            const NO_CONTEXTUAL = 0x100,
        }
    }


    #[derive(Debug, Clone, PartialEq)]
    pub enum SpecifiedValue {
        Value(VariantLigatures),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_variant_ligatures"></%self:simple_system_boilerplate>

    //                                 servo_bit: gecko_bit
    <% font_variant_ligatures_map = { "NONE": "NONE",
                                      "COMMON_LIGATURES": "COMMON",
                                      "NO_COMMON_LIGATURES": "NO_COMMON",
                                      "DISCRETIONARY_LIGATURES": "DISCRETIONARY",
                                      "NO_DISCRETIONARY_LIGATURES": "NO_DISCRETIONARY",
                                      "HISTORICAL_LIGATURES": "HISTORICAL",
                                      "NO_HISTORICAL_LIGATURES": "NO_HISTORICAL",
                                      "CONTEXTUAL": "CONTEXTUAL",
                                      "NO_CONTEXTUAL": "NO_CONTEXTUAL" } %>

    ${helpers.gecko_bitflags_conversion(font_variant_ligatures_map, 'NS_FONT_VARIANT_LIGATURES_',
                                        'VariantLigatures', kw_type='u16')}

    impl ToCss for VariantLigatures {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("normal")
            }
            if self.contains(NONE) {
                return dest.write_str("none")
            }

            let mut has_any = false;

            macro_rules! write_value {
                ($ident:ident => $str:expr) => {
                    if self.intersects($ident) {
                        if has_any {
                            try!(dest.write_str(" "));
                        }
                        has_any = true;
                        try!(dest.write_str($str));
                    }
                }
            }

            write_value!(COMMON_LIGATURES => "common-ligatures");
            write_value!(NO_COMMON_LIGATURES => "no-common-ligatures");
            write_value!(DISCRETIONARY_LIGATURES => "discretionary-ligatures");
            write_value!(NO_DISCRETIONARY_LIGATURES => "no-discretionary-ligatures");
            write_value!(HISTORICAL_LIGATURES => "historical-ligatures");
            write_value!(NO_HISTORICAL_LIGATURES => "no-historical-ligatures");
            write_value!(CONTEXTUAL => "contextual");
            write_value!(NO_CONTEXTUAL => "no-contextual");

            debug_assert!(has_any);
            Ok(())
        }
    }

    pub mod computed_value {
        pub type T = super::VariantLigatures;
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::empty()
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(VariantLigatures::empty())
    }

    /// normal | none |
    /// [ <common-lig-values> ||
    ///   <discretionary-lig-values> ||
    ///   <historical-lig-values> ||
    ///   <contextual-alt-values> ]
    /// <common-lig-values>        = [ common-ligatures | no-common-ligatures ]
    /// <discretionary-lig-values> = [ discretionary-ligatures | no-discretionary-ligatures ]
    /// <historical-lig-values>    = [ historical-ligatures | no-historical-ligatures ]
    /// <contextual-alt-values>    = [ contextual | no-contextual ]
    <% common_lig_values = "COMMON_LIGATURES | NO_COMMON_LIGATURES" %>
    <% discretionary_lig_values = "DISCRETIONARY_LIGATURES | NO_DISCRETIONARY_LIGATURES" %>
    <% historical_lig_values = "HISTORICAL_LIGATURES | NO_HISTORICAL_LIGATURES" %>
    <% contextual_alt_values = "CONTEXTUAL | NO_CONTEXTUAL" %>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = VariantLigatures::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Value(result))
        }
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::Value(NONE))
        }

        while let Ok(ident) = input.try(|input| input.expect_ident()) {
            let flag = match_ignore_ascii_case! { &ident,
                "common-ligatures" =>
                    exclusive_value!((result, ${common_lig_values}) => COMMON_LIGATURES),
                "no-common-ligatures" =>
                    exclusive_value!((result, ${common_lig_values}) => NO_COMMON_LIGATURES),
                "discretionary-ligatures" =>
                    exclusive_value!((result, ${discretionary_lig_values}) => DISCRETIONARY_LIGATURES),
                "no-discretionary-ligatures" =>
                    exclusive_value!((result, ${discretionary_lig_values}) => NO_DISCRETIONARY_LIGATURES),
                "historical-ligatures" =>
                    exclusive_value!((result, ${historical_lig_values}) => HISTORICAL_LIGATURES),
                "no-historical-ligatures" =>
                    exclusive_value!((result, ${historical_lig_values}) => NO_HISTORICAL_LIGATURES),
                "contextual" =>
                    exclusive_value!((result, ${contextual_alt_values}) => CONTEXTUAL),
                "no-contextual" =>
                    exclusive_value!((result, ${contextual_alt_values}) => NO_CONTEXTUAL),
                _ => return Err(()),
            };
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(SpecifiedValue::Value(result))
        } else {
            Err(())
        }
    }
</%helpers:longhand>

<%helpers:longhand name="font-variant-numeric" products="gecko" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-numeric">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    bitflags! {
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub flags VariantNumeric: u8 {
            const NORMAL = 0,
            const LINING_NUMS = 0x01,
            const OLDSTYLE_NUMS = 0x02,
            const PROPORTIONAL_NUMS = 0x04,
            const TABULAR_NUMS = 0x08,
            const DIAGONAL_FRACTIONS = 0x10,
            const STACKED_FRACTIONS = 0x20,
            const SLASHED_ZERO = 0x40,
            const ORDINAL = 0x80,
        }
    }



    #[derive(Debug, Clone, PartialEq)]
    pub enum SpecifiedValue {
        Value(VariantNumeric),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_variant_numeric"></%self:simple_system_boilerplate>


    //                              servo_bit: gecko_bit
    <% font_variant_numeric_map = { "LINING_NUMS": "LINING",
                                    "OLDSTYLE_NUMS": "OLDSTYLE",
                                    "PROPORTIONAL_NUMS": "PROPORTIONAL",
                                    "TABULAR_NUMS": "TABULAR",
                                    "DIAGONAL_FRACTIONS": "DIAGONAL_FRACTIONS",
                                    "STACKED_FRACTIONS": "STACKED_FRACTIONS",
                                    "SLASHED_ZERO": "SLASHZERO",
                                    "ORDINAL": "ORDINAL" } %>

    ${helpers.gecko_bitflags_conversion(font_variant_numeric_map, 'NS_FONT_VARIANT_NUMERIC_',
                                        'VariantNumeric')}

    impl ToCss for VariantNumeric {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("normal")
            }

            let mut has_any = false;

            macro_rules! write_value {
                ($ident:ident => $str:expr) => {
                    if self.intersects($ident) {
                        if has_any {
                            try!(dest.write_str(" "));
                        }
                        has_any = true;
                        try!(dest.write_str($str));
                    }
                }
            }

            write_value!(LINING_NUMS => "lining-nums");
            write_value!(OLDSTYLE_NUMS => "oldstyle-nums");
            write_value!(PROPORTIONAL_NUMS => "proportional-nums");
            write_value!(TABULAR_NUMS => "tabular-nums");
            write_value!(DIAGONAL_FRACTIONS => "diagonal-fractions");
            write_value!(STACKED_FRACTIONS => "stacked-fractions");
            write_value!(SLASHED_ZERO => "slashed-zero");
            write_value!(ORDINAL => "ordinal");

            debug_assert!(has_any);
            Ok(())
        }
    }

    pub mod computed_value {
        pub type T = super::VariantNumeric;
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::empty()
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(VariantNumeric::empty())
    }

    /// normal |
    ///  [ <numeric-figure-values>   ||
    ///    <numeric-spacing-values>  ||
    ///    <numeric-fraction-values> ||
    ///    ordinal                   ||
    ///    slashed-zero ]
    /// <numeric-figure-values>   = [ lining-nums | oldstyle-nums ]
    /// <numeric-spacing-values>  = [ proportional-nums | tabular-nums ]
    /// <numeric-fraction-values> = [ diagonal-fractions | stacked-fractions ]
    <% numeric_figure_values = "LINING_NUMS | OLDSTYLE_NUMS" %>
    <% numeric_spacing_values = "PROPORTIONAL_NUMS | TABULAR_NUMS" %>
    <% numeric_fraction_values = "DIAGONAL_FRACTIONS | STACKED_FRACTIONS" %>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = VariantNumeric::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Value(result))
        }

        while let Ok(ident) = input.try(|input| input.expect_ident()) {
            let flag = match_ignore_ascii_case! { &ident,
                "ordinal" =>
                    exclusive_value!((result, ORDINAL) => ORDINAL),
                "slashed-zero" =>
                    exclusive_value!((result, SLASHED_ZERO) => SLASHED_ZERO),
                "lining-nums" =>
                    exclusive_value!((result, ${numeric_figure_values}) => LINING_NUMS ),
                "oldstyle-nums" =>
                    exclusive_value!((result, ${numeric_figure_values}) => OLDSTYLE_NUMS ),
                "proportional-nums" =>
                    exclusive_value!((result, ${numeric_spacing_values}) => PROPORTIONAL_NUMS ),
                "tabular-nums" =>
                    exclusive_value!((result, ${numeric_spacing_values}) => TABULAR_NUMS ),
                "diagonal-fractions" =>
                    exclusive_value!((result, ${numeric_fraction_values}) => DIAGONAL_FRACTIONS ),
                "stacked-fractions" =>
                    exclusive_value!((result, ${numeric_fraction_values}) => STACKED_FRACTIONS ),
                _ => return Err(()),
            };
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(SpecifiedValue::Value(result))
        } else {
            Err(())
        }
    }
</%helpers:longhand>

${helpers.single_keyword_system("font-variant-position",
                                "normal sub super",
                                products="gecko",
                                gecko_ffi_name="mFont.variantPosition",
                                gecko_constant_prefix="NS_FONT_VARIANT_POSITION",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-position",
                                animation_value_type="none")}

<%helpers:longhand name="font-feature-settings" products="gecko" animation_value_type="none"
                   extra_prefixes="moz" boxed="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-feature-settings">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;

    #[derive(Debug, Clone, PartialEq)]
    pub enum SpecifiedValue {
        Value(computed_value::T),
        System(SystemFont)
    }
    no_viewport_percentage!(SpecifiedValue);

    <%self:simple_system_boilerplate name="font_feature_settings"></%self:simple_system_boilerplate>

    pub mod computed_value {
        use cssparser::Parser;
        use parser::{Parse, ParserContext};
        use std::fmt;
        use style_traits::ToCss;

        #[derive(Clone, Debug, Eq, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Normal,
            Tag(Vec<FeatureTagValue>)
        }

        #[derive(Clone, Debug, Eq, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct FeatureTagValue {
            pub tag: u32,
            pub value: u32
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::Normal => dest.write_str("normal"),
                    T::Tag(ref ftvs) => {
                        let mut iter = ftvs.iter();
                        // handle head element
                        try!(iter.next().unwrap().to_css(dest));
                        // handle tail, precede each with a delimiter
                        for ftv in iter {
                            try!(dest.write_str(", "));
                            try!(ftv.to_css(dest));
                        }
                        Ok(())
                    }
                }
            }
        }

        impl Parse for T {
            /// https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings
            fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
                if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
                    return Ok(T::Normal);
                }
                input.parse_comma_separated(|i| FeatureTagValue::parse(context, i)).map(T::Tag)
            }
        }

        impl ToCss for FeatureTagValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                use std::str;
                use byteorder::{WriteBytesExt, BigEndian};
                use cssparser::serialize_string;

                let mut raw: Vec<u8> = vec!();
                raw.write_u32::<BigEndian>(self.tag).unwrap();
                serialize_string(str::from_utf8(&raw).unwrap_or_default(), dest)?;

                match self.value {
                    1 => Ok(()),
                    0 => dest.write_str(" off"),
                    x => write!(dest, " {}", x)
                }
            }
        }

        impl Parse for FeatureTagValue {
            /// https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings
            /// <string> [ on | off | <integer> ]
            fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
                use std::io::Cursor;
                use byteorder::{ReadBytesExt, BigEndian};

                let tag = try!(input.expect_string());

                // allowed strings of length 4 containing chars: <U+20, U+7E>
                if tag.len() != 4 ||
                   tag.chars().any(|c| c < ' ' || c > '~')
                {
                    return Err(())
                }

                let mut raw = Cursor::new(tag.as_bytes());
                let u_tag = raw.read_u32::<BigEndian>().unwrap();

                if let Ok(value) = input.try(|input| input.expect_integer()) {
                    // handle integer, throw if it is negative
                    if value >= 0 {
                        Ok(FeatureTagValue { tag: u_tag, value: value as u32 })
                    } else {
                        Err(())
                    }
                } else if let Ok(_) = input.try(|input| input.expect_ident_matching("on")) {
                    // on is an alias for '1'
                    Ok(FeatureTagValue { tag: u_tag, value: 1 })
                } else if let Ok(_) = input.try(|input| input.expect_ident_matching("off")) {
                    // off is an alias for '0'
                    Ok(FeatureTagValue { tag: u_tag, value: 0 })
                } else {
                    // empty value is an alias for '1'
                    Ok(FeatureTagValue { tag: u_tag, value: 1 })
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Normal
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(computed_value::T::Normal)
    }

    /// normal | <feature-tag-value>#
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        computed_value::T::parse(context, input).map(SpecifiedValue::Value)
    }
</%helpers:longhand>

<%helpers:longhand name="font-language-override" products="gecko" animation_value_type="none"
                   extra_prefixes="moz" boxed="True"
                   spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-language-override">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;
    use byteorder::{BigEndian, ByteOrder};
    no_viewport_percentage!(SpecifiedValue);

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Normal,
        Override(String),
        System(SystemFont)
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            use cssparser;
            match *self {
                SpecifiedValue::Normal => dest.write_str("normal"),
                SpecifiedValue::Override(ref lang) =>
                    cssparser::serialize_string(lang, dest),
                SpecifiedValue::System(_) => Ok(())
            }
        }
    }

    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    }

    pub mod computed_value {
        use std::{fmt, str};
        use style_traits::ToCss;
        use byteorder::{BigEndian, ByteOrder};
        use cssparser;

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.0 == 0 {
                    return dest.write_str("normal")
                }
                let mut buf = [0; 4];
                BigEndian::write_u32(&mut buf, self.0);
                // Safe because we ensure it's ASCII during computing
                let slice = if cfg!(debug_assertions) {
                    str::from_utf8(&buf).unwrap()
                } else {
                    unsafe { str::from_utf8_unchecked(&buf) }
                };
                cssparser::serialize_string(slice.trim_right(), dest)
            }
        }

        // font-language-override can only have a single three-letter
        // OpenType "language system" tag, so we should be able to compute
        // it and store it as a 32-bit integer
        // (see http://www.microsoft.com/typography/otspec/languagetags.htm).
        #[derive(PartialEq, Clone, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub u32);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(0)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Normal
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> computed_value::T {
            use std::ascii::AsciiExt;
            match *self {
                SpecifiedValue::Normal => computed_value::T(0),
                SpecifiedValue::Override(ref lang) => {
                    if lang.is_empty() || lang.len() > 4 || !lang.is_ascii() {
                        return computed_value::T(0)
                    }
                    let mut computed_lang = lang.clone();
                    while computed_lang.len() < 4 {
                        computed_lang.push(' ');
                    }
                    let bytes = computed_lang.into_bytes();
                    computed_value::T(BigEndian::read_u32(&bytes))
                }
                SpecifiedValue::System(_) => {
                    <%self:nongecko_unreachable>
                        _context.cached_system_font.as_ref().unwrap().font_language_override
                    </%self:nongecko_unreachable>
                }
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            if computed.0 == 0 {
                return SpecifiedValue::Normal
            }
            let mut buf = [0; 4];
            BigEndian::write_u32(&mut buf, computed.0);
            SpecifiedValue::Override(
                if cfg!(debug_assertions) {
                    String::from_utf8(buf.to_vec()).unwrap()
                } else {
                    unsafe { String::from_utf8_unchecked(buf.to_vec()) }
                }
            )
        }
    }

    /// normal | <string>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            Ok(SpecifiedValue::Normal)
        } else {
            input.expect_string().map(|cow| {
                SpecifiedValue::Override(cow.into_owned())
            })
        }
    }
</%helpers:longhand>

<%helpers:longhand name="-x-lang" products="gecko" animation_value_type="none" internal="True"
                   spec="Internal (not web-exposed)">
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use Atom;
        use std::fmt;
        use style_traits::ToCss;

        impl ToCss for T {
            fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
                Ok(())
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Atom);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(atom!(""))
    }

    pub fn parse(_context: &ParserContext, _input: &mut Parser) -> Result<SpecifiedValue, ()> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(())
    }
</%helpers:longhand>

// MathML properties
<%helpers:longhand name="-moz-script-size-multiplier" products="gecko" animation_value_type="none"
                   predefined_type="Number" gecko_ffi_name="mScriptSizeMultiplier"
                   spec="Internal (not web-exposed)"
                   internal="True" disable_when_testing="True">
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}

    pub mod computed_value {
        pub type T = f32;
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        ::gecko_bindings::structs::NS_MATHML_DEFAULT_SCRIPT_SIZE_MULTIPLIER as f32
    }

    pub fn parse(_context: &ParserContext, _input: &mut Parser) -> Result<SpecifiedValue, ()> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(())
    }
</%helpers:longhand>

<%helpers:longhand name="-moz-script-level" products="gecko" animation_value_type="none"
                   predefined_type="Integer" gecko_ffi_name="mScriptLevel"
                   spec="Internal (not web-exposed)"
                   internal="True" disable_when_testing="True" need_clone="True">
    use std::fmt;
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub type T = i8;
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        0
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub enum SpecifiedValue {
        Relative(i32),
        Absolute(i32),
        Auto
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Auto => dest.write_str("auto"),
                SpecifiedValue::Relative(rel) => write!(dest, "{}", rel),
                // can only be specified by pres attrs; should not
                // serialize to anything else
                SpecifiedValue::Absolute(_) => Ok(()),
            }
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, cx: &Context) -> i8 {
            use properties::longhands::_moz_math_display::SpecifiedValue as DisplayValue;
            use std::{cmp, i8};

            let int = match *self {
                SpecifiedValue::Auto => {
                    let parent = cx.inherited_style().get_font().clone__moz_script_level() as i32;
                    let display = cx.inherited_style().get_font().clone__moz_math_display();
                    if display == DisplayValue::inline {
                        parent + 1
                    } else {
                        parent
                    }
                }
                SpecifiedValue::Relative(rel) => {
                    let parent = cx.inherited_style().get_font().clone__moz_script_level();
                    parent as i32 + rel
                }
                SpecifiedValue::Absolute(abs) => abs,
            };
            cmp::min(int, i8::MAX as i32) as i8
        }
        fn from_computed_value(other: &computed_value::T) -> Self {
            SpecifiedValue::Absolute(*other as i32)
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if let Ok(i) = input.try(|i| i.expect_integer()) {
            return Ok(SpecifiedValue::Relative(i))
        }
        input.expect_ident_matching("auto")?;
        Ok(SpecifiedValue::Auto)
    }
</%helpers:longhand>

${helpers.single_keyword("-moz-math-display",
                         "inline block",
                         gecko_constant_prefix="NS_MATHML_DISPLAYSTYLE",
                         gecko_ffi_name="mMathDisplay",
                         products="gecko",
                         spec="Internal (not web-exposed)",
                         animation_value_type="none",
                         need_clone="True")}

${helpers.single_keyword("-moz-math-variant",
                         """none normal bold italic bold-italic script bold-script
                            fraktur double-struck bold-fraktur sans-serif
                            bold-sans-serif sans-serif-italic sans-serif-bold-italic
                            monospace initial tailed looped stretched""",
                         gecko_constant_prefix="NS_MATHML_MATHVARIANT",
                         gecko_ffi_name="mMathVariant",
                         products="gecko",
                         spec="Internal (not web-exposed)",
                         animation_value_type="none",
                         need_clone="True",
                         needs_conversion=True)}

<%helpers:longhand name="-moz-script-min-size" products="gecko" animation_value_type="none"
                   predefined_type="Length" gecko_ffi_name="mScriptMinSize"
                   spec="Internal (not web-exposed)"
                   internal="True" disable_when_testing="True">
    use app_units::Au;
    use gecko_bindings::structs::NS_MATHML_DEFAULT_SCRIPT_MIN_SIZE_PT;
    use std::fmt;
    use style_traits::ToCss;
    use values::specified::length::{AU_PER_PT, FontBaseSize, NoCalcLength};

    #[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
    pub struct SpecifiedValue(pub NoCalcLength);

    pub mod computed_value {
        pub type T = super::Au;
    }


    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, cx: &Context) -> Au {
            // this value is used in the computation of font-size, so
            // we use the parent size
            let base_size = FontBaseSize::InheritedStyle;
            match self.0 {
                NoCalcLength::FontRelative(value) => {
                    value.to_computed_value(cx, base_size)
                }
                NoCalcLength::ServoCharacterWidth(value) => {
                    value.to_computed_value(base_size.resolve(cx))
                }
                ref l => {
                    l.to_computed_value(cx)
                }
            }
        }
        fn from_computed_value(other: &computed_value::T) -> Self {
            SpecifiedValue(ToComputedValue::from_computed_value(other))
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        Au((NS_MATHML_DEFAULT_SCRIPT_MIN_SIZE_PT as f32 * AU_PER_PT) as i32)
    }

    pub fn parse(_context: &ParserContext, _input: &mut Parser) -> Result<SpecifiedValue, ()> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(())
    }
</%helpers:longhand>


% if product == "gecko":
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

        use app_units::Au;
        use cssparser::Parser;
        use properties::longhands;
        use std::fmt;
        use std::hash::{Hash, Hasher};
        use style_traits::ToCss;
        use values::computed::{ToComputedValue, Context};
        <%
            system_fonts = """caption icon menu message-box small-caption status-bar
                              -moz-window -moz-document -moz-workspace -moz-desktop
                              -moz-info -moz-dialog -moz-button -moz-pull-down-menu
                              -moz-list -moz-field""".split()
            kw_font_props = """font_style font_variant_caps font_stretch
                               font_kerning font_variant_position font_variant_alternates
                               font_variant_ligatures font_variant_east_asian
                               font_variant_numeric""".split()
            kw_cast = """font_style font_variant_caps font_stretch
                         font_kerning font_variant_position""".split()
        %>
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum SystemFont {
            % for font in system_fonts:
                ${to_camel_case(font)},
            % endfor
        }

        impl ToCss for SystemFont {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                dest.write_str(match *self {
                    % for font in system_fonts:
                        SystemFont::${to_camel_case(font)} => "${font}",
                    % endfor
                })
            }
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
                use gecko_bindings::bindings;
                use gecko_bindings::structs::{LookAndFeel_FontID, nsFont};
                use std::mem;

                let id = match *self {
                    % for font in system_fonts:
                        SystemFont::${to_camel_case(font)} => {
                            LookAndFeel_FontID::eFont_${to_camel_case(font.replace("-moz-", ""))}
                        }
                    % endfor
                };

                let mut system: nsFont = unsafe { mem::uninitialized() };
                unsafe {
                    bindings::Gecko_nsFont_InitSystem(&mut system, id as i32,
                                            cx.style.get_font().gecko(),
                                            &*cx.device.pres_context)
                }
                let family = system.fontlist.mFontlist.iter().map(|font| {
                    use properties::longhands::font_family::computed_value::*;
                    FontFamily::FamilyName(FamilyName {
                        name: (&*font.mName).into(),
                        quoted: true
                    })
                }).collect::<Vec<_>>();
                let weight = longhands::font_weight::computed_value::T::from_gecko_weight(system.weight);
                let ret = ComputedSystemFont {
                    font_family: longhands::font_family::computed_value::T(family),
                    font_size: Au(system.size),
                    font_weight: weight,
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
                    system_font: *self,
                };
                unsafe { bindings::Gecko_nsFont_Destroy(&mut system); }
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
            if context.cached_system_font.is_none() {
                let computed = system.to_computed_value(context);
                context.cached_system_font = Some(computed);
            }
            debug_assert!(system == context.cached_system_font.as_ref().unwrap().system_font)
        }

        #[derive(Clone, Debug)]
        pub struct ComputedSystemFont {
            % for name in SYSTEM_FONT_LONGHANDS:
                pub ${name}: longhands::${name}::computed_value::T,
            % endfor
            pub system_font: SystemFont,
        }

        impl SystemFont {
            pub fn parse(input: &mut Parser) -> Result<Self, ()> {
                Ok(match_ignore_ascii_case! { &*input.expect_ident()?,
                    % for font in system_fonts:
                        "${font}" => SystemFont::${to_camel_case(font)},
                    % endfor
                    _ => return Err(())
                })
            }
        }
    }
% else:
    pub mod system_font {
        use cssparser::Parser;

        // We don't parse system fonts, but in the interest of not littering
        // a lot of code with `if product == gecko` conditionals, we have a
        // dummy system font module that does nothing

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        /// void enum for system font, can never exist
        pub enum SystemFont {}
        impl SystemFont {
            pub fn parse(_: &mut Parser) -> Result<Self, ()> {
                Err(())
            }
        }
    }
% endif

${helpers.single_keyword("-moz-osx-font-smoothing",
                         "auto grayscale",
                         gecko_constant_prefix="NS_FONT_SMOOTHING",
                         gecko_ffi_name="mFont.smoothing",
                         products="gecko",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/font-smooth)",
                         animation_value_type="none",
                         need_clone=True)}
