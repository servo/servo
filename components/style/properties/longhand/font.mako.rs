/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Font",
                         inherited=True,
                         additional_methods=[Method("compute_font_hash", is_mut=True)]) %>
<%helpers:longhand name="font-family">
    use self::computed_value::FontFamily;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    const SERIF: &'static str = "serif";
    const SANS_SERIF: &'static str = "sans-serif";
    const CURSIVE: &'static str = "cursive";
    const FANTASY: &'static str = "fantasy";
    const MONOSPACE: &'static str = "monospace";

    impl ComputedValueAsSpecified for SpecifiedValue {}
    pub mod computed_value {
        use cssparser::ToCss;
        use std::fmt;
        use string_cache::Atom;

        #[derive(Debug, PartialEq, Eq, Clone, Hash, HeapSizeOf, Deserialize, Serialize)]
        pub enum FontFamily {
            FamilyName(String),
            // Generic,
            Serif,
            SansSerif,
            Cursive,
            Fantasy,
            Monospace,
        }
        impl FontFamily {
            #[inline]
            pub fn name(&self) -> &str {
                match *self {
                    FontFamily::FamilyName(ref name) => &*name,
                    FontFamily::Serif => super::SERIF,
                    FontFamily::SansSerif => super::SANS_SERIF,
                    FontFamily::Cursive => super::CURSIVE,
                    FontFamily::Fantasy => super::FANTASY,
                    FontFamily::Monospace => super::MONOSPACE
                }
            }

            pub fn from_atom(input: Atom) -> FontFamily {
                let input = input.to_string();
                let option = match_ignore_ascii_case! { &input,
                    super::SERIF => Some(FontFamily::Serif),
                    super::SANS_SERIF => Some(FontFamily::SansSerif),
                    super::CURSIVE => Some(FontFamily::Cursive),
                    super::FANTASY => Some(FontFamily::Fantasy),
                    super::MONOSPACE => Some(FontFamily::Monospace),
                    _ => None
                };

                match option {
                    Some(family) => family,
                    None => FontFamily::FamilyName(input)
                }
            }
        }
        impl ToCss for FontFamily {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                dest.write_str(self.name())
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
        #[derive(Debug, Clone, PartialEq, Eq, Hash, HeapSizeOf)]
        pub struct T(pub Vec<FontFamily>);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![FontFamily::Serif])
    }
    /// <family-name>#
    /// <family-name> = <string> | [ <ident>+ ]
    /// TODO: <generic-family>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        input.parse_comma_separated(parse_one_family).map(SpecifiedValue)
    }
    pub fn parse_one_family(input: &mut Parser) -> Result<FontFamily, ()> {
        if let Ok(value) = input.try(|input| input.expect_string()) {
            return Ok(FontFamily::FamilyName(value.into_owned()))
        }
        let first_ident = try!(input.expect_ident());

        match_ignore_ascii_case! { first_ident,
            SERIF => return Ok(FontFamily::Serif),
            SANS_SERIF => return Ok(FontFamily::SansSerif),
            CURSIVE => return Ok(FontFamily::Cursive),
            FANTASY => return Ok(FontFamily::Fantasy),
            MONOSPACE => return Ok(FontFamily::Monospace),
            _ => {}
        }
        let mut value = first_ident.into_owned();
        while let Ok(ident) = input.try(|input| input.expect_ident()) {
            value.push_str(" ");
            value.push_str(&ident);
        }
        Ok(FontFamily::FamilyName(value))
    }
</%helpers:longhand>


${helpers.single_keyword("font-style", "normal italic oblique")}
${helpers.single_keyword("font-variant", "normal small-caps")}

<%helpers:longhand name="font-weight" need_clone="True">
    use cssparser::ToCss;
    use std::fmt;

    #[derive(Debug, Clone, PartialEq, Eq, Copy, HeapSizeOf)]
    pub enum SpecifiedValue {
        Bolder,
        Lighter,
        % for weight in range(100, 901, 100):
            Weight${weight},
        % endfor
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
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
            match_ignore_ascii_case! { try!(input.expect_ident()),
                "bold" => Ok(SpecifiedValue::Weight700),
                "normal" => Ok(SpecifiedValue::Weight400),
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
    pub mod computed_value {
        use std::fmt;
        #[derive(PartialEq, Eq, Copy, Clone, Hash, Deserialize, Serialize, HeapSizeOf, Debug)]
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

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            match *self {
                % for weight in range(100, 901, 100):
                    SpecifiedValue::Weight${weight} => computed_value::T::Weight${weight},
                % endfor
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
    }
</%helpers:longhand>

<%helpers:longhand name="font-size" need_clone="True">
    use app_units::Au;
    use cssparser::ToCss;
    use std::fmt;
    use values::FONT_MEDIUM_PX;
    use values::specified::{LengthOrPercentage, Length, Percentage};

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
    pub struct SpecifiedValue(pub specified::LengthOrPercentage);
    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        Au::from_px(FONT_MEDIUM_PX)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            match self.0 {
                LengthOrPercentage::Length(Length::FontRelative(value)) => {
                    value.to_computed_value(context.inherited_style().get_font().clone_font_size(),
                                            context.style().root_font_size())
                }
                LengthOrPercentage::Length(Length::ServoCharacterWidth(value)) => {
                    value.to_computed_value(context.inherited_style().get_font().clone_font_size())
                }
                LengthOrPercentage::Length(l) => {
                    l.to_computed_value(context)
                }
                LengthOrPercentage::Percentage(Percentage(value)) => {
                    context.inherited_style().get_font().clone_font_size().scale_by(value)
                }
                LengthOrPercentage::Calc(calc) => {
                    let calc = calc.to_computed_value(context);
                    calc.length() + context.inherited_style().get_font().clone_font_size()
                                           .scale_by(calc.percentage())
                }
            }
        }
    }
    /// <length> | <percentage> | <absolute-size> | <relative-size>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use values::specified::{Length, LengthOrPercentage};

        input.try(specified::LengthOrPercentage::parse_non_negative)
        .or_else(|()| {
            let ident = try!(input.expect_ident());
            specified::Length::from_str(&ident as &str)
                .ok_or(())
                .map(specified::LengthOrPercentage::Length)
        })
        .map(SpecifiedValue)
    }
</%helpers:longhand>

${helpers.single_keyword("font-stretch",
                 "normal ultra-condensed extra-condensed condensed semi-condensed semi-expanded \
                 expanded extra-expanded ultra-expanded")}

${helpers.single_keyword("font-kerning", "auto none normal", products="gecko")}
