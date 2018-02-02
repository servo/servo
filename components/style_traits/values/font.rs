/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Some font related types moved here from the style crate, so gfx can use them
//! without depending on style.

use ::ParseError;
use Atom;
use app_units::Au;
use cssparser::{CssStringWriter, Parser, serialize_identifier};
use std::fmt;
use std::fmt::Write;
use std::slice;
#[cfg(feature = "servo")] use servo_url::ServoUrl;
use super::{CssWriter, ToCss};

/// As of CSS Fonts Module Level 3, only the following values are
/// valid: 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
///
/// However, system fonts may provide other values. Pango
/// may provide 350, 380, and 1000 (on top of the existing values), for example.
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct FontWeight(pub u16);

impl FontWeight {
    /// Value for normal
    pub fn normal() -> Self {
        FontWeight(400)
    }

    /// Value for bold
    pub fn bold() -> Self {
        FontWeight(700)
    }

    /// Convert from an integer to Weight
    pub fn from_int(n: i32) -> Result<Self, ()> {
        if n >= 100 && n <= 900 && n % 100 == 0 {
            Ok(FontWeight(n as u16))
        } else {
            Err(())
        }
    }

    /// Convert from an Gecko weight
    pub fn from_gecko_weight(weight: u16) -> Self {
        // we allow a wider range of weights than is parseable
        // because system fonts may provide custom values
        FontWeight(weight)
    }

    /// Weither this weight is bold
    pub fn is_bold(&self) -> bool {
        self.0 > 500
    }

    /// Return the bolder weight
    pub fn bolder(self) -> Self {
        if self.0 < 400 {
            FontWeight(400)
        } else if self.0 < 600 {
            FontWeight(700)
        } else {
            FontWeight(900)
        }
    }

    /// Returns the lighter weight
    pub fn lighter(self) -> Self {
        if self.0 < 600 {
            FontWeight(100)
        } else if self.0 < 800 {
            FontWeight(400)
        } else {
            FontWeight(700)
        }
    }
}

impl ToCss for FontWeight {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write {
        write!(dest, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
/// A set of faces that vary in weight, width or slope.
pub enum SingleFontFamily {
    /// The name of a font family of choice.
    FamilyName(FamilyName),
    /// Generic family name.
    Generic(Atom),
}

impl SingleFontFamily {
    #[inline]
    /// Get font family name as Atom
    pub fn atom(&self) -> &Atom {
        match *self {
            SingleFontFamily::FamilyName(ref family_name) => &family_name.name,
            SingleFontFamily::Generic(ref name) => name,
        }
    }

    #[inline]
    #[cfg(not(feature = "gecko"))] // Gecko can't borrow atoms as UTF-8.
    /// Get font family name
    pub fn name(&self) -> &str {
        self.atom()
    }

    #[cfg(not(feature = "gecko"))] // Gecko can't borrow atoms as UTF-8.
    /// Get the corresponding font-family with Atom
    pub fn from_atom(input: Atom) -> SingleFontFamily {
        match input {
            atom!("serif") |
            atom!("sans-serif") |
            atom!("cursive") |
            atom!("fantasy") |
            atom!("monospace") => {
                return SingleFontFamily::Generic(input)
            }
            _ => {}
        }
        match_ignore_ascii_case! { &input,
            "serif" => return SingleFontFamily::Generic(atom!("serif")),
            "sans-serif" => return SingleFontFamily::Generic(atom!("sans-serif")),
            "cursive" => return SingleFontFamily::Generic(atom!("cursive")),
            "fantasy" => return SingleFontFamily::Generic(atom!("fantasy")),
            "monospace" => return SingleFontFamily::Generic(atom!("monospace")),
            _ => {}
        }

        // We don't know if it's quoted or not. So we set it to
        // quoted by default.
        SingleFontFamily::FamilyName(FamilyName {
            name: input,
            syntax: FamilyNameSyntax::Quoted,
        })
    }

    /// Parse a font-family value
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(value) = input.try(|i| i.expect_string_cloned()) {
            return Ok(SingleFontFamily::FamilyName(FamilyName {
                name: Atom::from(&*value),
                syntax: FamilyNameSyntax::Quoted,
            }))
        }
        let first_ident = input.expect_ident()?.clone();

        // FIXME(bholley): The fast thing to do here would be to look up the
        // string (as lowercase) in the static atoms table. We don't have an
        // API to do that yet though, so we do the simple thing for now.
        let mut css_wide_keyword = false;
        match_ignore_ascii_case! { &first_ident,
            "serif" => return Ok(SingleFontFamily::Generic(atom!("serif"))),
            "sans-serif" => return Ok(SingleFontFamily::Generic(atom!("sans-serif"))),
            "cursive" => return Ok(SingleFontFamily::Generic(atom!("cursive"))),
            "fantasy" => return Ok(SingleFontFamily::Generic(atom!("fantasy"))),
            "monospace" => return Ok(SingleFontFamily::Generic(atom!("monospace"))),

            #[cfg(feature = "gecko")]
            "-moz-fixed" => return Ok(SingleFontFamily::Generic(atom!("-moz-fixed"))),

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

        let mut value = first_ident.as_ref().to_owned();
        let mut serialization = String::new();
        serialize_identifier(&first_ident, &mut serialization).unwrap();

        // These keywords are not allowed by themselves.
        // The only way this value can be valid with with another keyword.
        if css_wide_keyword {
            let ident = input.expect_ident()?;
            value.push(' ');
            value.push_str(&ident);
            serialization.push(' ');
            serialize_identifier(&ident, &mut serialization).unwrap();
        }
        while let Ok(ident) = input.try(|i| i.expect_ident_cloned()) {
            value.push(' ');
            value.push_str(&ident);
            serialization.push(' ');
            serialize_identifier(&ident, &mut serialization).unwrap();
        }
        Ok(SingleFontFamily::FamilyName(FamilyName {
            name: Atom::from(value),
            syntax: FamilyNameSyntax::Identifiers(serialization),
        }))
    }

    #[cfg(feature = "gecko")]
    /// Return the generic ID for a given generic font name
    pub fn generic(name: &Atom) -> (structs::FontFamilyType, u8) {
        use gecko_bindings::structs::FontFamilyType;
        if *name == atom!("serif") {
            (FontFamilyType::eFamily_serif,
             structs::kGenericFont_serif)
        } else if *name == atom!("sans-serif") {
            (FontFamilyType::eFamily_sans_serif,
             structs::kGenericFont_sans_serif)
        } else if *name == atom!("cursive") {
            (FontFamilyType::eFamily_cursive,
             structs::kGenericFont_cursive)
        } else if *name == atom!("fantasy") {
            (FontFamilyType::eFamily_fantasy,
             structs::kGenericFont_fantasy)
        } else if *name == atom!("monospace") {
            (FontFamilyType::eFamily_monospace,
             structs::kGenericFont_monospace)
        } else if *name == atom!("-moz-fixed") {
            (FontFamilyType::eFamily_moz_fixed,
             structs::kGenericFont_moz_fixed)
        } else {
            panic!("Unknown generic {}", name);
        }
    }

    #[cfg(feature = "gecko")]
    /// Get the corresponding font-family with family name
    fn from_font_family_name(family: &structs::FontFamilyName) -> SingleFontFamily {
        use gecko_bindings::structs::FontFamilyType;

        match family.mType {
            FontFamilyType::eFamily_sans_serif => SingleFontFamily::Generic(atom!("sans-serif")),
            FontFamilyType::eFamily_serif => SingleFontFamily::Generic(atom!("serif")),
            FontFamilyType::eFamily_monospace => SingleFontFamily::Generic(atom!("monospace")),
            FontFamilyType::eFamily_cursive => SingleFontFamily::Generic(atom!("cursive")),
            FontFamilyType::eFamily_fantasy => SingleFontFamily::Generic(atom!("fantasy")),
            FontFamilyType::eFamily_moz_fixed => SingleFontFamily::Generic(Atom::from("-moz-fixed")),
            FontFamilyType::eFamily_named => {
                let name = Atom::from(&*family.mName);
                let mut serialization = String::new();
                serialize_identifier(&name.to_string(), &mut serialization).unwrap();
                SingleFontFamily::FamilyName(FamilyName {
                    name: name.clone(),
                    syntax: FamilyNameSyntax::Identifiers(serialization),
                })
            },
            FontFamilyType::eFamily_named_quoted => SingleFontFamily::FamilyName(FamilyName {
                name: (&*family.mName).into(),
                syntax: FamilyNameSyntax::Quoted,
            }),
            _ => panic!("Found unexpected font FontFamilyType"),
        }
    }
}

impl ToCss for SingleFontFamily {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
        match *self {
            SingleFontFamily::FamilyName(ref name) => name.to_css(dest),

            // All generic values accepted by the parser are known to not require escaping.
            SingleFontFamily::Generic(ref name) => {
                #[cfg(feature = "gecko")] {
                    // We should treat -moz-fixed as monospace
                    if name == &atom!("-moz-fixed") {
                        return dest.write_str("monospace");
                    }
                }

                write!(dest, "{}", name)
            },
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
/// The name of a font family of choice
pub struct FamilyName {
    /// Name of the font family
    pub name: Atom,
    /// Syntax of the font family
    pub syntax: FamilyNameSyntax,
}

impl ToCss for FamilyName {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
        match self.syntax {
            FamilyNameSyntax::Quoted => {
                dest.write_char('"')?;
                write!(CssStringWriter::new(dest), "{}", self.name)?;
                dest.write_char('"')
            }
            FamilyNameSyntax::Identifiers(ref serialization) => {
                // Note that `serialization` is already escaped/
                // serialized appropriately.
                dest.write_str(&*serialization)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
/// Font family names must either be given quoted as strings,
/// or unquoted as a sequence of one or more identifiers.
pub enum FamilyNameSyntax {
    /// The family name was specified in a quoted form, e.g. "Font Name"
    /// or 'Font Name'.
    Quoted,

    /// The family name was specified in an unquoted form as a sequence of
    /// identifiers.  The `String` is the serialization of the sequence of
    /// identifiers.
    Identifiers(String),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum FontStretch {
    Normal,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum FontVariantCaps {
    Normal,
    SmallCaps,
}

/// Everything gfx needs from style::style_structs::Font
pub trait FontStyleStruct {
    fn get_size(&self) -> Au;
    fn get_hash(&self) -> u64;
    fn get_font_weight(&self) -> FontWeight;
    fn get_font_stretch(&self) -> FontStretch;
    fn get_font_variant_caps(&self) -> FontVariantCaps;
    fn get_font_families(&self) -> slice::Iter<SingleFontFamily>;
    fn is_oblique_or_italic(&self) -> bool;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "servo")]
pub enum Source {
    Url(Option<ServoUrl>),
    Local(FamilyName)
}

/// A list of effective sources that we send over through IPC to the font cache.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "servo")]
pub struct EffectiveSources(pub Vec<Source>);

#[cfg(feature = "servo")]
impl Iterator for EffectiveSources {
    type Item = Source;
    fn next(&mut self) -> Option<Source> {
        self.0.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}
