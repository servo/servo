/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Font",
                         inherited=True,
                         additional_methods=[Method("compute_font_hash", is_mut=True)]) %>
<%helpers:longhand name="font-family" animatable="False" need_index="True">
    use self::computed_value::FontFamily;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use std::fmt;
        use Atom;
        use style_traits::ToCss;
        pub use self::FontFamily as SingleComputedValue;

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
                match_ignore_ascii_case! { input,
                    "serif" => return FontFamily::Generic(atom!("serif")),
                    "sans-serif" => return FontFamily::Generic(atom!("sans-serif")),
                    "cursive" => return FontFamily::Generic(atom!("cursive")),
                    "fantasy" => return FontFamily::Generic(atom!("fantasy")),
                    "monospace" => return FontFamily::Generic(atom!("monospace")),
                    _ => {}
                }
                FontFamily::FamilyName(input)
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
        match_ignore_ascii_case! { first_ident,
            "serif" => return Ok(FontFamily::Generic(atom!("serif"))),
            "sans-serif" => return Ok(FontFamily::Generic(atom!("sans-serif"))),
            "cursive" => return Ok(FontFamily::Generic(atom!("cursive"))),
            "fantasy" => return Ok(FontFamily::Generic(atom!("fantasy"))),
            "monospace" => return Ok(FontFamily::Generic(atom!("monospace"))),
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
                         gecko_ffi_name="mFont.style",
                         animatable=False)}

${helpers.single_keyword("font-variant",
                         "normal small-caps",
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
                         custom_consts=font_variant_caps_custom_consts,
                         animatable=False)}

<%helpers:longhand name="font-weight" need_clone="True" animatable="True">
    use std::fmt;
    use style_traits::ToCss;
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

<%helpers:longhand name="font-size" need_clone="True" animatable="True">
    use app_units::Au;
    use std::fmt;
    use style_traits::ToCss;
    use values::{FONT_MEDIUM_PX, HasViewportPercentage};
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

    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_value() -> computed_value::T {
        Au::from_px(FONT_MEDIUM_PX)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match self.0 {
                LengthOrPercentage::Length(Length::FontRelative(value)) => {
                    value.to_computed_value(context, /* use inherited */ true)
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

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
                SpecifiedValue(LengthOrPercentage::Length(
                        ToComputedValue::from_computed_value(computed)
                ))
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

// https://www.w3.org/TR/css-fonts-3/#font-size-adjust-prop
<%helpers:longhand products="gecko" name="font-size-adjust" animatable="True">
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    use values::specified::Number;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        None,
        Number(Number),
    }

    pub mod computed_value {
        use style_traits::ToCss;
        use std::fmt;
        use properties::animated_properties::Interpolate;
        use values::specified::Number;

        pub use super::SpecifiedValue as T;

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
                        Ok(T::Number(Number(try!(number.0.interpolate(&other.0, time))))),
                    _ => Err(()),
                }
            }
        }
    }

    #[inline] pub fn get_initial_value() -> computed_value::T {
        computed_value::T::None
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

<%helpers:longhand products="gecko" name="font-synthesis" animatable="False">
    use std::fmt;
    use style_traits::ToCss;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

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
        match_ignore_ascii_case! {try!(input.expect_ident()),
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
                         animatable=False)}

${helpers.single_keyword("font-kerning",
                         "auto none normal",
                         products="gecko",
                         gecko_ffi_name="mFont.kerning",
                         gecko_constant_prefix="NS_FONT_KERNING",
                         animatable=False)}

${helpers.single_keyword("font-variant-position",
                         "normal sub super",
                         products="gecko",
                         gecko_ffi_name="mFont.variantPosition",
                         gecko_constant_prefix="NS_FONT_VARIANT_POSITION",
                         animatable=False)}

<%helpers:longhand name="font-feature-settings" products="none" animatable="False">
    use std::fmt;
    use style_traits::ToCss;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

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
<%helpers:longhand name="font-language-override" products="none" animatable="False">
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

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
