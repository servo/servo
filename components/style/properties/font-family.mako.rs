<%page args="helpers"/>

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
            FamilyName(Atom),
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
            return Ok(FontFamily::FamilyName(Atom::from(&*value)))
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
        Ok(FontFamily::FamilyName(Atom::from(value)))
    }
</%helpers:longhand>
