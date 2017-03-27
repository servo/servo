/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Font",
                         inherited=True) %>
<%helpers:longhand name="font-family" animatable="False" need_index="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-family">
    use self::computed_value::{FontFamily, FamilyName};
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use cssparser::{CssStringWriter, Parser};
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
        pub struct FamilyName(pub Atom);

        impl FontFamily {
            #[inline]
            pub fn atom(&self) -> &Atom {
                match *self {
                    FontFamily::FamilyName(ref name) => &name.0,
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
                FontFamily::FamilyName(FamilyName(input))
            }

            /// Parse a font-family value
            pub fn parse(input: &mut Parser) -> Result<Self, ()> {
                if let Ok(value) = input.try(|input| input.expect_string()) {
                    return Ok(FontFamily::FamilyName(FamilyName(Atom::from(&*value))))
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
                Ok(FontFamily::FamilyName(FamilyName(Atom::from(value))))
            }
        }

        impl ToCss for FamilyName {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                dest.write_char('"')?;
                write!(CssStringWriter::new(dest), "{}", self.0)?;
                dest.write_char('"')
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

    impl SpecifiedValue {
        pub fn parse(input: &mut Parser) -> Result<Self, ()> {
            input.parse_comma_separated(|input| FontFamily::parse(input)).map(SpecifiedValue)
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


${helpers.single_keyword("font-style",
                         "normal italic oblique",
                         gecko_constant_prefix="NS_FONT_STYLE",
                         gecko_ffi_name="mFont.style",
                         spec="https://drafts.csswg.org/css-fonts/#propdef-font-style",
                         animatable=False)}

${helpers.single_keyword("font-variant",
                         "normal small-caps",
                         spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant",
                         animatable=False)}


<% font_variant_caps_custom_consts= { "small-caps": "SMALLCAPS",
                                      "all-small": "ALLSMALL",
                                      "petite-caps": "PETITECAPS",
                                      "all-petite": "ALLPETITE",
                                      "titling-caps": "TITLING" } %>

${helpers.single_keyword("font-variant-caps",
                         "normal small-caps all-small petite-caps unicase titling-caps",
                         gecko_constant_prefix="NS_FONT_VARIANT_CAPS",
                         gecko_ffi_name="mFont.variantCaps",
                         products="gecko",
                         spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-caps",
                         custom_consts=font_variant_caps_custom_consts,
                         animatable=False)}

<%helpers:longhand name="font-weight" need_clone="True" animatable="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-weight">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

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
            match try!(input.expect_integer()) {
                100 => Ok(SpecifiedValue::Weight100),
                200 => Ok(SpecifiedValue::Weight200),
                300 => Ok(SpecifiedValue::Weight300),
                400 => Ok(SpecifiedValue::Weight400),
                500 => Ok(SpecifiedValue::Weight500),
                600 => Ok(SpecifiedValue::Weight600),
                700 => Ok(SpecifiedValue::Weight700),
                800 => Ok(SpecifiedValue::Weight800),
                900 => Ok(SpecifiedValue::Weight900),
                _ => Err(())
            }
        })
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
                SpecifiedValue::Lighter => Err(())
            }
        }
    }

    pub mod computed_value {
        use std::fmt;
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

<%helpers:longhand name="font-size" need_clone="True" animatable="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-size">
    use app_units::Au;
    use std::fmt;
    use style_traits::ToCss;
    use values::{FONT_MEDIUM_PX, HasViewportPercentage};
    use values::specified::{LengthOrPercentage, Length, NoCalcLength, Percentage};

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Length(ref lop) => lop.to_css(dest),
                SpecifiedValue::Keyword(kw) => kw.to_css(dest),
                SpecifiedValue::Smaller => dest.write_str("smaller"),
                SpecifiedValue::Larger => dest.write_str("larger"),
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
        Keyword(KeywordSize),
        Smaller,
        Larger,
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
                use gecko_bindings::bindings::Gecko_nsStyleFont_GetBaseSize;
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

                let base_size = unsafe {
                    Gecko_nsStyleFont_GetBaseSize(cx.style().get_font().gecko(),
                                                  &*cx.device.pres_context)
                };
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
            })
        }
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_value() -> computed_value::T {
        Au::from_px(FONT_MEDIUM_PX)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Keyword(Medium)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            use values::specified::length::FontRelativeLength;
            match *self {
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        NoCalcLength::FontRelative(value))) => {
                    value.to_computed_value(context, /* use inherited */ true)
                }
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        NoCalcLength::ServoCharacterWidth(value))) => {
                    value.to_computed_value(context.inherited_style().get_font().clone_font_size())
                }
                SpecifiedValue::Length(LengthOrPercentage::Length(ref l)) => {
                    l.to_computed_value(context)
                }
                SpecifiedValue::Length(LengthOrPercentage::Percentage(Percentage(value))) => {
                    context.inherited_style().get_font().clone_font_size().scale_by(value)
                }
                SpecifiedValue::Length(LengthOrPercentage::Calc(ref calc)) => {
                    let calc = calc.to_computed_value(context);
                    calc.length() + context.inherited_style().get_font().clone_font_size()
                                           .scale_by(calc.percentage())
                }
                SpecifiedValue::Keyword(ref key) => {
                    key.to_computed_value(context)
                }
                SpecifiedValue::Smaller => {
                    FontRelativeLength::Em(0.85).to_computed_value(context,
                                                                   /* use_inherited */ true)
                }
                SpecifiedValue::Larger => {
                    FontRelativeLength::Em(1.2).to_computed_value(context,
                                                                   /* use_inherited */ true)
                }
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        ToComputedValue::from_computed_value(computed)
                ))
        }
    }
    /// <length> | <percentage> | <absolute-size> | <relative-size>
    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if let Ok(lop) = input.try(specified::LengthOrPercentage::parse_non_negative) {
            Ok(SpecifiedValue::Length(lop))
        } else if let Ok(kw) = input.try(KeywordSize::parse) {
            Ok(SpecifiedValue::Keyword(kw))
        } else {
            match_ignore_ascii_case! {&*input.expect_ident()?,
                "smaller" => Ok(SpecifiedValue::Smaller),
                "larger" => Ok(SpecifiedValue::Larger),
                _ => Err(())
            }
        }
    }
</%helpers:longhand>

<%helpers:longhand products="gecko" name="font-size-adjust" animatable="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-size-adjust">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    no_viewport_percentage!(SpecifiedValue);

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        None,
        Number(specified::Number),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result
            where W: fmt::Write,
        {
            match *self {
                SpecifiedValue::None => dest.write_str("none"),
                SpecifiedValue::Number(number) => number.to_css(dest),
            }
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
            match *self {
                SpecifiedValue::None => computed_value::T::None,
                SpecifiedValue::Number(ref n) => computed_value::T::Number(n.to_computed_value(context)),
            }
        }

        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::None => SpecifiedValue::None,
                computed_value::T::Number(ref v) => SpecifiedValue::Number(specified::Number::from_computed_value(v)),
            }
        }
    }

    pub mod computed_value {
        use properties::animated_properties::Interpolate;
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

        impl Interpolate for T {
            fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
                match (*self, *other) {
                    (T::Number(ref number), T::Number(ref other)) =>
                        Ok(T::Number(try!(number.interpolate(other, time)))),
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
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use values::specified::Number;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::None);
        }

        Ok(SpecifiedValue::Number(try!(Number::parse_non_negative(input))))
    }
</%helpers:longhand>

<%helpers:longhand products="gecko" name="font-synthesis" animatable="False"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-synthesis">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
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

// FIXME: This prop should be animatable
${helpers.single_keyword("font-stretch",
                         "normal ultra-condensed extra-condensed condensed \
                          semi-condensed semi-expanded expanded extra-expanded \
                          ultra-expanded",
                         gecko_ffi_name="mFont.stretch",
                         gecko_constant_prefix="NS_FONT_STRETCH",
                         cast_type='i16',
                         spec="https://drafts.csswg.org/css-fonts/#propdef-font-stretch",
                         animatable=False)}

${helpers.single_keyword("font-kerning",
                         "auto none normal",
                         products="gecko",
                         gecko_ffi_name="mFont.kerning",
                         gecko_constant_prefix="NS_FONT_KERNING",
                         spec="https://drafts.csswg.org/css-fonts/#propdef-font-stretch",
                         animatable=False)}

${helpers.single_keyword("font-variant-position",
                         "normal sub super",
                         products="gecko",
                         gecko_ffi_name="mFont.variantPosition",
                         gecko_constant_prefix="NS_FONT_VARIANT_POSITION",
                         spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-position",
                         animatable=False)}

<%helpers:longhand name="font-feature-settings" products="none" animatable="False" extra_prefixes="moz"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-feature-settings">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use cssparser::Parser;
        use parser::{Parse, ParserContext};
        use std::fmt;
        use style_traits::ToCss;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Normal,
            Tag(Vec<FeatureTagValue>)
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct FeatureTagValue {
            pub tag: String,
            pub value: i32
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

        impl ToCss for FeatureTagValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.value {
                    1 => write!(dest, "\"{}\"", self.tag),
                    0 => write!(dest, "\"{}\" off", self.tag),
                    x => write!(dest, "\"{}\" {}", self.tag, x)
                }
            }
        }

        impl Parse for FeatureTagValue {
            /// https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings
            /// <string> [ on | off | <integer> ]
            fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
                let tag = try!(input.expect_string());

                // allowed strings of length 4 containing chars: <U+20, U+7E>
                if tag.len() != 4 ||
                   tag.chars().any(|c| c < ' ' || c > '~')
                {
                    return Err(())
                }

                if let Ok(value) = input.try(|input| input.expect_integer()) {
                    // handle integer, throw if it is negative
                    if value >= 0 {
                        Ok(FeatureTagValue { tag: tag.into_owned(), value: value })
                    } else {
                        Err(())
                    }
                } else if let Ok(_) = input.try(|input| input.expect_ident_matching("on")) {
                    // on is an alias for '1'
                    Ok(FeatureTagValue { tag: tag.into_owned(), value: 1 })
                } else if let Ok(_) = input.try(|input| input.expect_ident_matching("off")) {
                    // off is an alias for '0'
                    Ok(FeatureTagValue { tag: tag.into_owned(), value: 0 })
                } else {
                    // empty value is an alias for '1'
                    Ok(FeatureTagValue { tag:tag.into_owned(), value: 1 })
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Normal
    }

    /// normal | <feature-tag-value>#
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            Ok(computed_value::T::Normal)
        } else {
            input.parse_comma_separated(|i| computed_value::FeatureTagValue::parse(context, i))
                 .map(computed_value::T::Tag)
        }
    }
</%helpers:longhand>

// https://www.w3.org/TR/css-fonts-3/#propdef-font-language-override
<%helpers:longhand name="font-language-override" products="none" animatable="False" extra_prefixes="moz"
                   spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-language-override">
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::Normal => dest.write_str("normal"),
                    T::Override(ref lang) => write!(dest, "\"{}\"", lang),
                }
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Normal,
            Override(String),
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Normal
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Normal
    }

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

<%helpers:longhand name="-x-lang" products="gecko" animatable="False" internal="True"
                   spec="Internal (not web-exposed)"
                   internal="True">
    use values::HasViewportPercentage;
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
