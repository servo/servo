/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Font",
                         inherited=True,
                         additional_methods=[Method("compute_font_hash", is_mut=True)]) %>
<%helpers:longhand name="font-family" animatable="False">
    use self::computed_value::FontFamily;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use cssparser::ToCss;
        use std::fmt;
        use string_cache::Atom;

        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum FontFamily {
            FamilyName(Atom),
            Generic(Atom),
        }
        impl FontFamily {

            #[inline]
            pub fn atom(&self) -> &Atom {
                match *self {
                    FontFamily::FamilyName(ref name) => name,
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
                let option = match_ignore_ascii_case! { &input,
                    &atom!("serif") => Some(atom!("serif")),
                    &atom!("sans-serif") => Some(atom!("sans-serif")),
                    &atom!("cursive") => Some(atom!("cursive")),
                    &atom!("fantasy") => Some(atom!("fantasy")),
                    &atom!("monospace") => Some(atom!("monospace")),
                    _ => None
                };

                match option {
                    Some(family) => FontFamily::Generic(family),
                    None => FontFamily::FamilyName(input)
                }
            }
        }
        impl ToCss for FontFamily {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.atom().with_str(|s| dest.write_str(s))
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
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        input.parse_comma_separated(parse_one_family).map(SpecifiedValue)
    }
    pub fn parse_one_family(input: &mut Parser) -> Result<FontFamily, ()> {
        if let Ok(value) = input.try(|input| input.expect_string()) {
            return Ok(FontFamily::FamilyName(Atom::from(&*value)))
        }
        let first_ident = try!(input.expect_ident());

        // FIXME(bholley): The fast thing to do here would be to look up the
        // string (as lowercase) in the static atoms table. We don't have an
        // API to do that yet though, so we do the simple thing for now.
        match &first_ident[..] {
            s if atom!("serif").eq_str_ignore_ascii_case(s) => return Ok(FontFamily::Generic(atom!("serif"))),
            s if atom!("sans-serif").eq_str_ignore_ascii_case(s) => return Ok(FontFamily::Generic(atom!("sans-serif"))),
            s if atom!("cursive").eq_str_ignore_ascii_case(s) => return Ok(FontFamily::Generic(atom!("cursive"))),
            s if atom!("fantasy").eq_str_ignore_ascii_case(s) => return Ok(FontFamily::Generic(atom!("fantasy"))),
            s if atom!("monospace").eq_str_ignore_ascii_case(s) => return Ok(FontFamily::Generic(atom!("monospace"))),
            _ => {}
        }

        let mut value = first_ident.into_owned();
        while let Ok(ident) = input.try(|input| input.expect_ident()) {
            value.push_str(" ");
            value.push_str(&ident);
        }
        Ok(FontFamily::FamilyName(Atom::from(value)))
    }
</%helpers:longhand>


${helpers.single_keyword("font-style",
                         "normal italic oblique",
                         gecko_constant_prefix="NS_FONT_STYLE",
                         animatable=False)}
${helpers.single_keyword("font-variant",
                         "normal small-caps",
                         animatable=False)}

<%helpers:longhand name="font-weight" need_clone="True" animatable="True">
    use cssparser::ToCss;
    use std::fmt;
    use values::NoViewportPercentage;

    impl NoViewportPercentage for SpecifiedValue {}

    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
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

<%helpers:longhand name="font-size" need_clone="True" animatable="True">
    use app_units::Au;
    use cssparser::ToCss;
    use std::fmt;
    use values::FONT_MEDIUM_PX;
    use values::HasViewportPercentage;
    use values::specified::{LengthOrPercentage, Length, Percentage};

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let &SpecifiedValue(length) = self;
            return length.has_viewport_percentage()
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
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

// FIXME: This prop should be animatable
${helpers.single_keyword("font-stretch",
                         "normal ultra-condensed extra-condensed condensed \
                          semi-condensed semi-expanded expanded extra-expanded \
                          ultra-expanded",
                         animatable=False)}

${helpers.single_keyword("font-kerning",
                         "auto none normal",
                         products="gecko",
                         animatable=False)}
