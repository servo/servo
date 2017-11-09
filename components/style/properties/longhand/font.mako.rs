/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method, to_camel_case, to_rust_ident, to_camel_case_lower, SYSTEM_FONT_LONGHANDS %>

<% data.new_style_struct("Font", inherited=True) %>

<%def name="nongecko_unreachable()">
    %if product == "gecko":
        ${caller.body()}
    %else:
        unreachable!()
    %endif
</%def>

#[cfg(feature = "gecko")]
macro_rules! impl_gecko_keyword_conversions {
    ($name: ident, $utype: ty) => {
        impl From<$utype> for $name {
            fn from(bits: $utype) -> $name {
                $name::from_gecko_keyword(bits)
            }
        }

        impl From<$name> for $utype {
            fn from(v: $name) -> $utype {
                v.to_gecko_keyword()
            }
        }
    };
}

// Define ToComputedValue, ToCss, and other boilerplate for a specified value
// which is of the form `enum SpecifiedValue {Value(..), System(SystemFont)}`
<%def name="simple_system_boilerplate(name)">
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

<%helpers:longhand name="font-family" animation_value_type="discrete"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-family">
    #[cfg(feature = "gecko")] use gecko_bindings::bindings;
    #[cfg(feature = "gecko")] use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
    use properties::longhands::system_font::SystemFont;
    use self::computed_value::{FontFamily, FontFamilyList, FamilyName};
    use std::fmt;
    use style_traits::ToCss;


    pub mod computed_value {
        use cssparser::{CssStringWriter, Parser, serialize_identifier};
        #[cfg(feature = "gecko")] use gecko_bindings::{bindings, structs};
        #[cfg(feature = "gecko")] use gecko_bindings::sugar::refptr::RefPtr;
        #[cfg(feature = "gecko")] use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
        use std::fmt::{self, Write};
        #[cfg(feature = "gecko")] use std::hash::{Hash, Hasher};
        #[cfg(feature = "servo")] use std::slice;
        use style_traits::{ToCss, ParseError};
        use Atom;
        pub use self::FontFamily as SingleComputedValue;

        #[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
        #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
        pub enum FontFamily {
            FamilyName(FamilyName),
            Generic(Atom),
        }

        #[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
        #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
        pub struct FamilyName {
            pub name: Atom,
            pub syntax: FamilyNameSyntax,
        }

        #[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
        #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
        pub enum FamilyNameSyntax {
            /// The family name was specified in a quoted form, e.g. "Font Name"
            /// or 'Font Name'.
            Quoted,

            /// The family name was specified in an unquoted form as a sequence of
            /// identifiers.  The `String` is the serialization of the sequence of
            /// identifiers.
            Identifiers(String),
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
                    syntax: FamilyNameSyntax::Quoted,
                })
            }

            /// Parse a font-family value
            pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
                if let Ok(value) = input.try(|i| i.expect_string_cloned()) {
                    return Ok(FontFamily::FamilyName(FamilyName {
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
                Ok(FontFamily::FamilyName(FamilyName {
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
            fn from_font_family_name(family: &structs::FontFamilyName) -> FontFamily {
                use gecko_bindings::structs::FontFamilyType;

                match family.mType {
                    FontFamilyType::eFamily_serif => FontFamily::Generic(atom!("serif")),
                    FontFamilyType::eFamily_sans_serif => FontFamily::Generic(atom!("sans-serif")),
                    FontFamilyType::eFamily_monospace => FontFamily::Generic(atom!("monospace")),
                    FontFamilyType::eFamily_cursive => FontFamily::Generic(atom!("cursive")),
                    FontFamilyType::eFamily_fantasy => FontFamily::Generic(atom!("fantasy")),
                    FontFamilyType::eFamily_moz_fixed => FontFamily::Generic(Atom::from("-moz-fixed")),
                    FontFamilyType::eFamily_named => {
                        let name = Atom::from(&*family.mName);
                        let mut serialization = String::new();
                        serialize_identifier(&name.to_string(), &mut serialization).unwrap();
                        FontFamily::FamilyName(FamilyName {
                            name: name.clone(),
                            syntax: FamilyNameSyntax::Identifiers(serialization),
                        })
                    },
                    FontFamilyType::eFamily_named_quoted => FontFamily::FamilyName(FamilyName {
                        name: (&*family.mName).into(),
                        syntax: FamilyNameSyntax::Quoted,
                    }),
                    x => panic!("Found unexpected font FontFamilyType: {:?}", x),
                }
            }
        }

        impl ToCss for FamilyName {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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

        impl ToCss for FontFamily {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    FontFamily::FamilyName(ref name) => name.to_css(dest),

                    // All generic values accepted by the parser are known to not require escaping.
                    FontFamily::Generic(ref name) => {
                        % if product == "gecko":
                            // We should treat -moz-fixed as monospace
                            if name == &atom!("-moz-fixed") {
                                return dest.write_str("monospace");
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
                iter.next().unwrap().to_css(dest)?;
                for family in iter {
                    dest.write_str(", ")?;
                    family.to_css(dest)?;
                }
                Ok(())
            }
        }

        #[cfg(feature = "servo")]
        #[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
        pub struct FontFamilyList(Vec<FontFamily>);

        #[cfg(feature = "gecko")]
        #[derive(Clone, Debug)]
        pub struct FontFamilyList(pub RefPtr<structs::SharedFontList>);

        #[cfg(feature = "gecko")]
        impl Hash for FontFamilyList {
            fn hash<H>(&self, state: &mut H) where H: Hasher {
                for name in self.0.mNames.iter() {
                    name.mType.hash(state);
                    name.mName.hash(state);
                }
            }
        }

        #[cfg(feature = "gecko")]
        impl PartialEq for FontFamilyList {
            fn eq(&self, other: &FontFamilyList) -> bool {
                if self.0.mNames.len() != other.0.mNames.len() {
                    return false;
                }
                for (a, b) in self.0.mNames.iter().zip(other.0.mNames.iter()) {
                    if a.mType != b.mType || &*a.mName != &*b.mName {
                        return false;
                    }
                }
                true
            }
        }

        #[cfg(feature = "gecko")]
        impl Eq for FontFamilyList {}

        impl FontFamilyList {
            #[cfg(feature = "servo")]
            pub fn new(families: Vec<FontFamily>) -> FontFamilyList {
                FontFamilyList(families)
            }

            #[cfg(feature = "gecko")]
            pub fn new(families: Vec<FontFamily>) -> FontFamilyList {
                let fontlist;
                let names;
                unsafe {
                    fontlist = bindings::Gecko_SharedFontList_Create();
                    names = &mut (*fontlist).mNames;
                    names.ensure_capacity(families.len());
                };

                for family in families {
                    match family {
                        FontFamily::FamilyName(ref f) => {
                            let quoted = matches!(f.syntax, FamilyNameSyntax::Quoted);
                            unsafe {
                                bindings::Gecko_nsTArray_FontFamilyName_AppendNamed(
                                    names,
                                    f.name.as_ptr(),
                                    quoted
                                );
                            }
                        }
                        FontFamily::Generic(ref name) => {
                            let (family_type, _generic) = FontFamily::generic(name);
                            unsafe {
                                bindings::Gecko_nsTArray_FontFamilyName_AppendGeneric(
                                    names,
                                    family_type
                                );
                            }
                        }
                    }
                }

                FontFamilyList(unsafe { RefPtr::from_addrefed(fontlist) })
            }

            #[cfg(feature = "servo")]
            pub fn iter(&self) -> slice::Iter<FontFamily> {
                self.0.iter()
            }

            #[cfg(feature = "gecko")]
            pub fn iter(&self) -> FontFamilyNameIter {
                FontFamilyNameIter {
                    names: &self.0.mNames,
                    cur: 0,
                }
            }

            #[cfg(feature = "gecko")]
            /// Return the generic ID if it is a single generic font
            pub fn single_generic(&self) -> Option<u8> {
                let mut iter = self.iter();
                if let Some(FontFamily::Generic(ref name)) = iter.next() {
                    if iter.next().is_none() {
                        return Some(FontFamily::generic(name).1);
                    }
                }
                None
            }
        }

        #[cfg(feature = "gecko")]
        pub struct FontFamilyNameIter<'a> {
            names: &'a structs::nsTArray<structs::FontFamilyName>,
            cur: usize,
        }

        #[cfg(feature = "gecko")]
        impl<'a> Iterator for FontFamilyNameIter<'a> {
            type Item = FontFamily;

            fn next(&mut self) -> Option<Self::Item> {
                if self.cur < self.names.len() {
                    let item = FontFamily::from_font_family_name(&self.names[self.cur]);
                    self.cur += 1;
                    Some(item)
                } else {
                    None
                }
            }
        }

        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        #[cfg_attr(feature = "servo", derive(MallocSizeOf))]
        pub struct T(pub FontFamilyList);

        #[cfg(feature = "gecko")]
        impl MallocSizeOf for T {
            fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
                // SharedFontList objects are generally shared from the pointer
                // stored in the specified value. So only count this if the
                // SharedFontList is unshared.
                unsafe {
                    bindings::Gecko_SharedFontList_SizeOfIncludingThisIfUnshared(
                        (self.0).0.get()
                    )
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(
            FontFamilyList::new(vec![FontFamily::Generic(atom!("serif"))])
        )
    }

    /// <family-name>#
    /// <family-name> = <string> | [ <ident>+ ]
    /// TODO: <generic-family>
    pub fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        SpecifiedValue::parse(input)
    }

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub enum SpecifiedValue {
        Values(FontFamilyList),
        System(SystemFont),
    }

    #[cfg(feature = "gecko")]
    impl SpecifiedValue {
        /// Return the generic ID if it is a single generic font
        pub fn single_generic(&self) -> Option<u8> {
            match *self {
                SpecifiedValue::Values(ref values) => values.single_generic(),
                _ => None,
            }
        }
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

    #[cfg(feature = "gecko")]
    impl MallocSizeOf for SpecifiedValue {
        fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
            match *self {
                SpecifiedValue::Values(ref v) => {
                    // Although a SharedFontList object is refcounted, we always
                    // attribute its size to the specified value.
                    unsafe {
                        bindings::Gecko_SharedFontList_SizeOfIncludingThis(
                            v.0.get()
                        )
                    }
                }
                SpecifiedValue::System(_) => 0,
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

        pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
            input.parse_comma_separated(|input| FontFamily::parse(input)).map(|v| {
                SpecifiedValue::Values(FontFamilyList::new(v))
            })
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
                SpecifiedValue::System(sys) => sys.to_css(dest),
            }
        }
    }

    /// `FamilyName::parse` is based on `FontFamily::parse` and not the other way around
    /// because we want the former to exclude generic family keywords.
    impl Parse for FamilyName {
        fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
            match FontFamily::parse(input) {
                Ok(FontFamily::FamilyName(name)) => Ok(name),
                Ok(FontFamily::Generic(_)) => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
                Err(e) => Err(e)
            }
        }
    }
</%helpers:longhand>

${helpers.single_keyword_system("font-style",
                                "normal italic oblique",
                                gecko_constant_prefix="NS_FONT_STYLE",
                                gecko_ffi_name="mFont.style",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-style",
                                flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                                animation_value_type="discrete")}


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
                                flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                                animation_value_type="discrete")}

${helpers.predefined_type("font-weight",
                          "FontWeight",
                          initial_value="computed::FontWeight::normal()",
                          initial_specified_value="specified::FontWeight::Normal",
                          animation_value_type="ComputedValue",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-weight")}

${helpers.predefined_type("font-size",
                          "FontSize",
                          initial_value="computed::FontSize::medium()",
                          initial_specified_value="specified::FontSize::medium()",
                          animation_value_type="NonNegativeLength",
                          allow_quirks=True,
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-size")}

${helpers.predefined_type("font-size-adjust",
                          "FontSizeAdjust",
                          products="gecko",
                          initial_value="computed::FontSizeAdjust::none()",
                          initial_specified_value="specified::FontSizeAdjust::none()",
                          animation_value_type="ComputedValue",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-size-adjust")}

<%helpers:longhand products="gecko" name="font-synthesis" animation_value_type="discrete"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-synthesis">
    use std::fmt;
    use style_traits::ToCss;

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    #[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
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

    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let mut result = SpecifiedValue { weight: false, style: false };
        // FIXME: remove clone() when lifetimes are non-lexical
        try_match_ident_ignore_ascii_case! { input,
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
        }
    }

    #[cfg(feature = "gecko")]
    impl From<u8> for SpecifiedValue {
        fn from(bits: u8) -> SpecifiedValue {
            use gecko_bindings::structs;

            SpecifiedValue {
                weight: bits & structs::NS_FONT_SYNTHESIS_WEIGHT as u8 != 0,
                style: bits & structs::NS_FONT_SYNTHESIS_STYLE as u8 != 0
            }
        }
    }

    #[cfg(feature = "gecko")]
    impl From<SpecifiedValue> for u8 {
        fn from(v: SpecifiedValue) -> u8 {
            use gecko_bindings::structs;

            let mut bits: u8 = 0;
            if v.weight {
                bits |= structs::NS_FONT_SYNTHESIS_WEIGHT as u8;
            }
            if v.style {
                bits |= structs::NS_FONT_SYNTHESIS_STYLE as u8;
            }
            bits
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
                                flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                                animation_value_type="ComputedValue")}

${helpers.single_keyword_system("font-kerning",
                                "auto none normal",
                                products="gecko",
                                gecko_ffi_name="mFont.kerning",
                                gecko_constant_prefix="NS_FONT_KERNING",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-kerning",
                                flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                                animation_value_type="discrete")}

<%helpers:longhand name="font-variant-alternates" products="gecko" animation_value_type="discrete"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-alternates">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;
    use values::CustomIdent;


    #[derive(Clone, Debug, MallocSizeOf, PartialEq)]
    pub enum VariantAlternates {
        Stylistic(CustomIdent),
        Styleset(Box<[CustomIdent]>),
        CharacterVariant(Box<[CustomIdent]>),
        Swash(CustomIdent),
        Ornaments(CustomIdent),
        Annotation(CustomIdent),
        HistoricalForms,
    }

    #[derive(Clone, Debug, MallocSizeOf, PartialEq)]
    pub struct VariantAlternatesList(pub Box<[VariantAlternates]>);

    #[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
    pub enum SpecifiedValue {
        Value(VariantAlternatesList),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_variant_alternates"></%self:simple_system_boilerplate>

    impl ToCss for VariantAlternates {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                % for value in "swash stylistic ornaments annotation".split():
                VariantAlternates::${to_camel_case(value)}(ref atom) => {
                    dest.write_str("${value}")?;
                    dest.write_str("(")?;
                    atom.to_css(dest)?;
                    dest.write_str(")")
                },
                % endfor
                % for value in "styleset character-variant".split():
                VariantAlternates::${to_camel_case(value)}(ref vec) => {
                    dest.write_str("${value}")?;
                    dest.write_str("(")?;
                    let mut iter = vec.iter();
                    iter.next().unwrap().to_css(dest)?;
                    for c in iter {
                        dest.write_str(", ")?;
                        c.to_css(dest)?;
                    }
                    dest.write_str(")")
                },
                % endfor
                VariantAlternates::HistoricalForms => {
                    dest.write_str("historical-forms")
                },
            }
        }
    }

    impl ToCss for VariantAlternatesList {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.0.is_empty() {
                return dest.write_str("normal");
            }

            let mut iter = self.0.iter();
            iter.next().unwrap().to_css(dest)?;
            for alternate in iter {
                dest.write_str(" ")?;
                alternate.to_css(dest)?;
            }
            Ok(())
        }
    }

    impl VariantAlternatesList {
        /// Returns the length of all variant alternates.
        pub fn len(&self) -> usize {
            self.0.iter().fold(0, |acc, alternate| {
                match *alternate {
                    % for value in "Swash Stylistic Ornaments Annotation".split():
                        VariantAlternates::${value}(_) => {
                            acc + 1
                        },
                    % endfor
                    % for value in "Styleset CharacterVariant".split():
                        VariantAlternates::${value}(ref slice) => {
                            acc + slice.len()
                        }
                    % endfor
                    _ => acc,
                }
            })
        }
    }

    pub mod computed_value {
        pub type T = super::VariantAlternatesList;
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        VariantAlternatesList(vec![].into_boxed_slice())
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(VariantAlternatesList(vec![].into_boxed_slice()))
    }

    bitflags! {
        #[cfg_attr(feature = "servo", derive(MallocSizeOf))]
        pub struct ParsingFlags: u8 {
            const NORMAL = 0;
            const HISTORICAL_FORMS = 0x01;
            const STYLISTIC = 0x02;
            const STYLESET = 0x04;
            const CHARACTER_VARIANT = 0x08;
            const SWASH = 0x10;
            const ORNAMENTS = 0x20;
            const ANNOTATION = 0x40;
        }
    }

    /// normal |
    ///  [ stylistic(<feature-value-name>)           ||
    ///    historical-forms                          ||
    ///    styleset(<feature-value-name> #)          ||
    ///    character-variant(<feature-value-name> #) ||
    ///    swash(<feature-value-name>)               ||
    ///    ornaments(<feature-value-name>)           ||
    ///    annotation(<feature-value-name>) ]
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let mut alternates = Vec::new();
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Value(VariantAlternatesList(alternates.into_boxed_slice())));
        }

        let mut parsed_alternates = ParsingFlags::empty();
        macro_rules! check_if_parsed(
            ($input:expr, $flag:path) => (
                if parsed_alternates.contains($flag) {
                    return Err($input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                }
                parsed_alternates |= $flag;
            )
        );
        while let Ok(_) = input.try(|input| {
            // FIXME: remove clone() when lifetimes are non-lexical
            match input.next()?.clone() {
                Token::Ident(ref ident) => {
                    if *ident == "historical-forms" {
                        check_if_parsed!(input, ParsingFlags::HISTORICAL_FORMS);
                        alternates.push(VariantAlternates::HistoricalForms);
                        Ok(())
                    } else {
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }
                },
                Token::Function(ref name) => {
                    input.parse_nested_block(|i| {
                        match_ignore_ascii_case! { &name,
                            % for value in "swash stylistic ornaments annotation".split():
                            "${value}" => {
                                check_if_parsed!(i, ParsingFlags::${value.upper()});
                                let location = i.current_source_location();
                                let ident = CustomIdent::from_ident(location, i.expect_ident()?, &[])?;
                                alternates.push(VariantAlternates::${to_camel_case(value)}(ident));
                                Ok(())
                            },
                            % endfor
                            % for value in "styleset character-variant".split():
                            "${value}" => {
                                check_if_parsed!(i, ParsingFlags:: ${to_rust_ident(value).upper()});
                                let idents = i.parse_comma_separated(|i| {
                                    let location = i.current_source_location();
                                    CustomIdent::from_ident(location, i.expect_ident()?, &[])
                                })?;
                                alternates.push(VariantAlternates::${to_camel_case(value)}(idents.into_boxed_slice()));
                                Ok(())
                            },
                            % endfor
                            _ => return Err(i.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
                        }
                    })
                },
                _ => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
            }
        }) { }

        if parsed_alternates.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(SpecifiedValue::Value(VariantAlternatesList(alternates.into_boxed_slice())))
    }
</%helpers:longhand>

#[cfg(feature = "gecko")]
macro_rules! exclusive_value {
    (($value:ident, $set:expr) => $ident:path) => {
        if $value.intersects($set) {
            return Err(())
        } else {
            $ident
        }
    }
}

<%helpers:longhand name="font-variant-east-asian" products="gecko" animation_value_type="discrete"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-east-asian">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;


    bitflags! {
        #[derive(MallocSizeOf)]
        pub struct VariantEastAsian: u16 {
            const NORMAL = 0;
            const JIS78 = 0x01;
            const JIS83 = 0x02;
            const JIS90 = 0x04;
            const JIS04 = 0x08;
            const SIMPLIFIED = 0x10;
            const TRADITIONAL = 0x20;
            const FULL_WIDTH = 0x40;
            const PROPORTIONAL_WIDTH = 0x80;
            const RUBY = 0x100;
        }
    }

    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(Clone, Debug, PartialEq, ToCss)]
    pub enum SpecifiedValue {
        Value(VariantEastAsian),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_variant_east_asian"></%self:simple_system_boilerplate>

    //                                 servo_bit: gecko_bit
    <% font_variant_east_asian_map = { "VariantEastAsian::JIS78": "JIS78",
                                       "VariantEastAsian::JIS83": "JIS83",
                                       "VariantEastAsian::JIS90": "JIS90",
                                       "VariantEastAsian::JIS04": "JIS04",
                                       "VariantEastAsian::SIMPLIFIED": "SIMPLIFIED",
                                       "VariantEastAsian::TRADITIONAL": "TRADITIONAL",
                                       "VariantEastAsian::FULL_WIDTH": "FULL_WIDTH",
                                       "VariantEastAsian::PROPORTIONAL_WIDTH": "PROP_WIDTH",
                                       "VariantEastAsian::RUBY": "RUBY" } %>

    ${helpers.gecko_bitflags_conversion(font_variant_east_asian_map, 'NS_FONT_VARIANT_EAST_ASIAN_',
                                        'VariantEastAsian', kw_type='u16')}


    impl ToCss for VariantEastAsian {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("normal")
            }

            let mut has_any = false;

            macro_rules! write_value {
                ($ident:path => $str:expr) => {
                    if self.intersects($ident) {
                        if has_any {
                            dest.write_str(" ")?;
                        }
                        has_any = true;
                        dest.write_str($str)?;
                    }
                }
            }

            write_value!(VariantEastAsian::JIS78 => "jis78");
            write_value!(VariantEastAsian::JIS83 => "jis83");
            write_value!(VariantEastAsian::JIS90 => "jis90");
            write_value!(VariantEastAsian::JIS04 => "jis04");
            write_value!(VariantEastAsian::SIMPLIFIED => "simplified");
            write_value!(VariantEastAsian::TRADITIONAL => "traditional");
            write_value!(VariantEastAsian::FULL_WIDTH => "full-width");
            write_value!(VariantEastAsian::PROPORTIONAL_WIDTH => "proportional-width");
            write_value!(VariantEastAsian::RUBY => "ruby");

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
    <% east_asian_variant_values = """VariantEastAsian::JIS78 | VariantEastAsian::JIS83 |
                                      VariantEastAsian::JIS90 | VariantEastAsian::JIS04 |
                                      VariantEastAsian::SIMPLIFIED | VariantEastAsian::TRADITIONAL""" %>
    <% east_asian_width_values = "VariantEastAsian::FULL_WIDTH | VariantEastAsian::PROPORTIONAL_WIDTH" %>
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let mut result = VariantEastAsian::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Value(result))
        }

        while let Ok(flag) = input.try(|input| {
            Ok(match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                "jis78" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => VariantEastAsian::JIS78),
                "jis83" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => VariantEastAsian::JIS83),
                "jis90" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => VariantEastAsian::JIS90),
                "jis04" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => VariantEastAsian::JIS04),
                "simplified" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => VariantEastAsian::SIMPLIFIED),
                "traditional" =>
                    exclusive_value!((result, ${east_asian_variant_values}) => VariantEastAsian::TRADITIONAL),
                "full-width" =>
                    exclusive_value!((result, ${east_asian_width_values}) => VariantEastAsian::FULL_WIDTH),
                "proportional-width" =>
                    exclusive_value!((result, ${east_asian_width_values}) => VariantEastAsian::PROPORTIONAL_WIDTH),
                "ruby" =>
                    exclusive_value!((result, VariantEastAsian::RUBY) => VariantEastAsian::RUBY),
                _ => return Err(()),
            })
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(SpecifiedValue::Value(result))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }

    #[cfg(feature = "gecko")]
    impl_gecko_keyword_conversions!(VariantEastAsian, u16);
</%helpers:longhand>

<%helpers:longhand name="font-variant-ligatures" products="gecko" animation_value_type="discrete"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-ligatures">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;


    bitflags! {
      #[derive(MallocSizeOf)]
        pub struct VariantLigatures: u16 {
            const NORMAL = 0;
            const NONE = 0x01;
            const COMMON_LIGATURES = 0x02;
            const NO_COMMON_LIGATURES = 0x04;
            const DISCRETIONARY_LIGATURES = 0x08;
            const NO_DISCRETIONARY_LIGATURES = 0x10;
            const HISTORICAL_LIGATURES = 0x20;
            const NO_HISTORICAL_LIGATURES = 0x40;
            const CONTEXTUAL = 0x80;
            const NO_CONTEXTUAL = 0x100;
        }
    }

    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(Clone, Debug, PartialEq, ToCss)]
    pub enum SpecifiedValue {
        Value(VariantLigatures),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_variant_ligatures"></%self:simple_system_boilerplate>

    //                                 servo_bit: gecko_bit
    <% font_variant_ligatures_map = { "VariantLigatures::NONE": "NONE",
                                      "VariantLigatures::COMMON_LIGATURES": "COMMON",
                                      "VariantLigatures::NO_COMMON_LIGATURES": "NO_COMMON",
                                      "VariantLigatures::DISCRETIONARY_LIGATURES": "DISCRETIONARY",
                                      "VariantLigatures::NO_DISCRETIONARY_LIGATURES": "NO_DISCRETIONARY",
                                      "VariantLigatures::HISTORICAL_LIGATURES": "HISTORICAL",
                                      "VariantLigatures::NO_HISTORICAL_LIGATURES": "NO_HISTORICAL",
                                      "VariantLigatures::CONTEXTUAL": "CONTEXTUAL",
                                      "VariantLigatures::NO_CONTEXTUAL": "NO_CONTEXTUAL" } %>

    ${helpers.gecko_bitflags_conversion(font_variant_ligatures_map, 'NS_FONT_VARIANT_LIGATURES_',
                                        'VariantLigatures', kw_type='u16')}

    impl ToCss for VariantLigatures {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("normal")
            }
            if self.contains(VariantLigatures::NONE) {
                return dest.write_str("none")
            }

            let mut has_any = false;

            macro_rules! write_value {
                ($ident:path => $str:expr) => {
                    if self.intersects($ident) {
                        if has_any {
                            dest.write_str(" ")?;
                        }
                        has_any = true;
                        dest.write_str($str)?;
                    }
                }
            }

            write_value!(VariantLigatures::COMMON_LIGATURES => "common-ligatures");
            write_value!(VariantLigatures::NO_COMMON_LIGATURES => "no-common-ligatures");
            write_value!(VariantLigatures::DISCRETIONARY_LIGATURES => "discretionary-ligatures");
            write_value!(VariantLigatures::NO_DISCRETIONARY_LIGATURES => "no-discretionary-ligatures");
            write_value!(VariantLigatures::HISTORICAL_LIGATURES => "historical-ligatures");
            write_value!(VariantLigatures::NO_HISTORICAL_LIGATURES => "no-historical-ligatures");
            write_value!(VariantLigatures::CONTEXTUAL => "contextual");
            write_value!(VariantLigatures::NO_CONTEXTUAL => "no-contextual");

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
    #[inline]
    pub fn get_none_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(VariantLigatures::NONE)
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
    <% common_lig_values = "VariantLigatures::COMMON_LIGATURES | VariantLigatures::NO_COMMON_LIGATURES" %>
    <% discretionary_lig_values = """VariantLigatures::DISCRETIONARY_LIGATURES |
                                     VariantLigatures::NO_DISCRETIONARY_LIGATURES""" %>
    <% historical_lig_values = "VariantLigatures::HISTORICAL_LIGATURES | VariantLigatures::NO_HISTORICAL_LIGATURES" %>
    <% contextual_alt_values = "VariantLigatures::CONTEXTUAL | VariantLigatures::NO_CONTEXTUAL" %>
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let mut result = VariantLigatures::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Value(result))
        }
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::Value(VariantLigatures::NONE))
        }

        while let Ok(flag) = input.try(|input| {
            Ok(match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                "common-ligatures" =>
                    exclusive_value!((result, ${common_lig_values}) => VariantLigatures::COMMON_LIGATURES),
                "no-common-ligatures" =>
                    exclusive_value!((result, ${common_lig_values}) => VariantLigatures::NO_COMMON_LIGATURES),
                "discretionary-ligatures" =>
                    exclusive_value!((result, ${discretionary_lig_values}) =>
                        VariantLigatures::DISCRETIONARY_LIGATURES),
                "no-discretionary-ligatures" =>
                    exclusive_value!((result, ${discretionary_lig_values}) =>
                        VariantLigatures::NO_DISCRETIONARY_LIGATURES),
                "historical-ligatures" =>
                    exclusive_value!((result, ${historical_lig_values}) => VariantLigatures::HISTORICAL_LIGATURES),
                "no-historical-ligatures" =>
                    exclusive_value!((result, ${historical_lig_values}) => VariantLigatures::NO_HISTORICAL_LIGATURES),
                "contextual" =>
                    exclusive_value!((result, ${contextual_alt_values}) => VariantLigatures::CONTEXTUAL),
                "no-contextual" =>
                    exclusive_value!((result, ${contextual_alt_values}) => VariantLigatures::NO_CONTEXTUAL),
                _ => return Err(()),
            })
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(SpecifiedValue::Value(result))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }

    #[cfg(feature = "gecko")]
    impl_gecko_keyword_conversions!(VariantLigatures, u16);
</%helpers:longhand>

<%helpers:longhand name="font-variant-numeric" products="gecko" animation_value_type="discrete"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-numeric">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;


    bitflags! {
        #[derive(MallocSizeOf)]
        pub struct VariantNumeric: u8 {
            const NORMAL = 0;
            const LINING_NUMS = 0x01;
            const OLDSTYLE_NUMS = 0x02;
            const PROPORTIONAL_NUMS = 0x04;
            const TABULAR_NUMS = 0x08;
            const DIAGONAL_FRACTIONS = 0x10;
            const STACKED_FRACTIONS = 0x20;
            const SLASHED_ZERO = 0x40;
            const ORDINAL = 0x80;
        }
    }

    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(Clone, Debug, PartialEq, ToCss)]
    pub enum SpecifiedValue {
        Value(VariantNumeric),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_variant_numeric"></%self:simple_system_boilerplate>


    //                              servo_bit: gecko_bit
    <% font_variant_numeric_map = { "VariantNumeric::LINING_NUMS": "LINING",
                                    "VariantNumeric::OLDSTYLE_NUMS": "OLDSTYLE",
                                    "VariantNumeric::PROPORTIONAL_NUMS": "PROPORTIONAL",
                                    "VariantNumeric::TABULAR_NUMS": "TABULAR",
                                    "VariantNumeric::DIAGONAL_FRACTIONS": "DIAGONAL_FRACTIONS",
                                    "VariantNumeric::STACKED_FRACTIONS": "STACKED_FRACTIONS",
                                    "VariantNumeric::SLASHED_ZERO": "SLASHZERO",
                                    "VariantNumeric::ORDINAL": "ORDINAL" } %>

    ${helpers.gecko_bitflags_conversion(font_variant_numeric_map, 'NS_FONT_VARIANT_NUMERIC_',
                                        'VariantNumeric')}

    impl ToCss for VariantNumeric {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("normal")
            }

            let mut has_any = false;

            macro_rules! write_value {
                ($ident:path => $str:expr) => {
                    if self.intersects($ident) {
                        if has_any {
                            dest.write_str(" ")?;
                        }
                        has_any = true;
                        dest.write_str($str)?;
                    }
                }
            }

            write_value!(VariantNumeric::LINING_NUMS => "lining-nums");
            write_value!(VariantNumeric::OLDSTYLE_NUMS => "oldstyle-nums");
            write_value!(VariantNumeric::PROPORTIONAL_NUMS => "proportional-nums");
            write_value!(VariantNumeric::TABULAR_NUMS => "tabular-nums");
            write_value!(VariantNumeric::DIAGONAL_FRACTIONS => "diagonal-fractions");
            write_value!(VariantNumeric::STACKED_FRACTIONS => "stacked-fractions");
            write_value!(VariantNumeric::SLASHED_ZERO => "slashed-zero");
            write_value!(VariantNumeric::ORDINAL => "ordinal");

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
    <% numeric_figure_values = "VariantNumeric::LINING_NUMS | VariantNumeric::OLDSTYLE_NUMS" %>
    <% numeric_spacing_values = "VariantNumeric::PROPORTIONAL_NUMS | VariantNumeric::TABULAR_NUMS" %>
    <% numeric_fraction_values = "VariantNumeric::DIAGONAL_FRACTIONS | VariantNumeric::STACKED_FRACTIONS" %>
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let mut result = VariantNumeric::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(SpecifiedValue::Value(result))
        }

        while let Ok(flag) = input.try(|input| {
            Ok(match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                "ordinal" =>
                    exclusive_value!((result, VariantNumeric::ORDINAL) => VariantNumeric::ORDINAL),
                "slashed-zero" =>
                    exclusive_value!((result, VariantNumeric::SLASHED_ZERO) => VariantNumeric::SLASHED_ZERO),
                "lining-nums" =>
                    exclusive_value!((result, ${numeric_figure_values}) => VariantNumeric::LINING_NUMS),
                "oldstyle-nums" =>
                    exclusive_value!((result, ${numeric_figure_values}) => VariantNumeric::OLDSTYLE_NUMS),
                "proportional-nums" =>
                    exclusive_value!((result, ${numeric_spacing_values}) => VariantNumeric::PROPORTIONAL_NUMS),
                "tabular-nums" =>
                    exclusive_value!((result, ${numeric_spacing_values}) => VariantNumeric::TABULAR_NUMS),
                "diagonal-fractions" =>
                    exclusive_value!((result, ${numeric_fraction_values}) => VariantNumeric::DIAGONAL_FRACTIONS),
                "stacked-fractions" =>
                    exclusive_value!((result, ${numeric_fraction_values}) => VariantNumeric::STACKED_FRACTIONS),
                _ => return Err(()),
            })
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(SpecifiedValue::Value(result))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }

    #[cfg(feature = "gecko")]
    impl_gecko_keyword_conversions!(VariantNumeric, u8);
</%helpers:longhand>

${helpers.single_keyword_system("font-variant-position",
                                "normal sub super",
                                products="gecko",
                                gecko_ffi_name="mFont.variantPosition",
                                gecko_constant_prefix="NS_FONT_VARIANT_POSITION",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-position",
                                flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                                animation_value_type="discrete")}

<%helpers:longhand name="font-feature-settings" products="gecko" animation_value_type="discrete"
                   extra_prefixes="moz" boxed="True"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-feature-settings">
    use properties::longhands::system_font::SystemFont;
    use values::generics::FontSettings;

    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(Clone, Debug, PartialEq, ToCss)]
    pub enum SpecifiedValue {
        Value(computed_value::T),
        System(SystemFont)
    }

    <%self:simple_system_boilerplate name="font_feature_settings"></%self:simple_system_boilerplate>

    pub mod computed_value {
        use values::generics::{FontSettings, FontSettingTagInt};
        pub type T = FontSettings<FontSettingTagInt>;
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        FontSettings::Normal
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(FontSettings::Normal)
    }

    /// normal | <feature-tag-value>#
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        computed_value::T::parse(context, input).map(SpecifiedValue::Value)
    }
</%helpers:longhand>

<%
# This spec link is too long to fit elsewhere
variation_spec = """\
https://drafts.csswg.org/css-fonts-4/#low-level-font-variation-settings-control-the-font-variation-settings-property\
"""
%>
<%helpers:longhand name="font-variation-settings" products="gecko"
                   animation_value_type="ComputedValue"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="${variation_spec}">
    use values::generics::FontSettings;

    pub type SpecifiedValue = computed_value::T;

    pub mod computed_value {
        use values::generics::{FontSettings, FontSettingTagFloat};
        pub type T = FontSettings<FontSettingTagFloat>;
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        FontSettings::Normal
    }

    /// normal | <feature-tag-value>#
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        computed_value::T::parse(context, input)
    }
</%helpers:longhand>

<%helpers:longhand name="font-language-override" products="gecko" animation_value_type="discrete"
                   extra_prefixes="moz" boxed="True"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-language-override">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;
    use byteorder::{BigEndian, ByteOrder};

    #[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq)]
    pub enum SpecifiedValue {
        Normal,
        Override(String),
        System(SystemFont)
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Normal => dest.write_str("normal"),
                SpecifiedValue::Override(ref lang) => lang.to_css(dest),
                SpecifiedValue::System(sys) => sys.to_css(dest),
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
                slice.trim_right().to_css(dest)
            }
        }

        // font-language-override can only have a single three-letter
        // OpenType "language system" tag, so we should be able to compute
        // it and store it as a 32-bit integer
        // (see http://www.microsoft.com/typography/otspec/languagetags.htm).
        #[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq)]
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
            #[allow(unused_imports)] use std::ascii::AsciiExt;
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
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            Ok(SpecifiedValue::Normal)
        } else {
            input.expect_string().map(|s| {
                SpecifiedValue::Override(s.as_ref().to_owned())
            }).map_err(|e| e.into())
        }
    }

    /// Used in @font-face.
    impl Parse for SpecifiedValue {
        fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<Self, ParseError<'i>> {
            parse(context, input)
        }
    }

    #[cfg(feature = "gecko")]
    impl From<u32> for computed_value::T {
        fn from(bits: u32) -> computed_value::T {
            computed_value::T(bits)
        }
    }

    #[cfg(feature = "gecko")]
    impl From<computed_value::T> for u32 {
        fn from(v: computed_value::T) -> u32 {
            v.0
        }
    }
</%helpers:longhand>

<%helpers:longhand name="-x-lang" products="gecko" animation_value_type="none" internal="True"
                   spec="Internal (not web-exposed)">
    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        use Atom;
        use std::fmt;
        use style_traits::ToCss;

        impl ToCss for T {
            fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
                Ok(())
            }
        }

        #[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
        pub struct T(pub Atom);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(atom!(""))
    }

    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
</%helpers:longhand>

// MathML properties
<%helpers:longhand name="-moz-script-size-multiplier" products="gecko" animation_value_type="none"
                   predefined_type="Number" gecko_ffi_name="mScriptSizeMultiplier"
                   spec="Internal (not web-exposed)"
                   internal="True">
    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        pub type T = f32;
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        ::gecko_bindings::structs::NS_MATHML_DEFAULT_SCRIPT_SIZE_MULTIPLIER as f32
    }

    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
</%helpers:longhand>

${helpers.predefined_type("-moz-script-level",
                          "MozScriptLevel",
                          0,
                          animation_value_type="none",
                          products="gecko",
                          internal=True,
                          gecko_ffi_name="mScriptLevel",
                          spec="Internal (not web-exposed)")}

${helpers.single_keyword("-moz-math-display",
                         "inline block",
                         gecko_constant_prefix="NS_MATHML_DISPLAYSTYLE",
                         gecko_ffi_name="mMathDisplay",
                         products="gecko",
                         spec="Internal (not web-exposed)",
                         animation_value_type="none")}

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
                         needs_conversion=True)}

${helpers.predefined_type("-moz-script-min-size",
                          "MozScriptMinSize",
                          "specified::MozScriptMinSize::get_initial_value()",
                          animation_value_type="none",
                          products="gecko",
                          internal=True,
                          gecko_ffi_name="mScriptMinSize",
                          spec="Internal (not web-exposed)")}

${helpers.predefined_type("-x-text-zoom",
                          "XTextZoom",
                          "computed::XTextZoom(true)",
                          animation_value_type="none",
                          products="gecko",
                          internal=True,
                          spec="Internal (not web-exposed)")}

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
        use cssparser::{Parser, ToCss};
        use gecko_bindings::structs::FontFamilyType;
        use properties::longhands;
        use std::fmt;
        use std::hash::{Hash, Hasher};
        use style_traits::ParseError;
        use values::computed::{ToComputedValue, Context};

        <%
            system_fonts = """caption icon menu message-box small-caption status-bar
                              -moz-window -moz-document -moz-workspace -moz-desktop
                              -moz-info -moz-dialog -moz-button -moz-pull-down-menu
                              -moz-list -moz-field""".split()
            kw_font_props = """font_style font_variant_caps font_stretch
                               font_kerning font_variant_position font_variant_ligatures
                               font_variant_east_asian font_variant_numeric""".split()
            kw_cast = """font_style font_variant_caps font_stretch
                         font_kerning font_variant_position""".split()
        %>
        #[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToCss)]
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
                use gecko_bindings::bindings;
                use gecko_bindings::structs::{LookAndFeel_FontID, nsFont};
                use std::mem;
                use values::computed::font::FontSize;

                let id = match *self {
                    % for font in system_fonts:
                        SystemFont::${to_camel_case(font)} => {
                            LookAndFeel_FontID::eFont_${to_camel_case(font.replace("-moz-", ""))}
                        }
                    % endfor
                };

                let mut system: nsFont = unsafe { mem::uninitialized() };
                unsafe {
                    bindings::Gecko_nsFont_InitSystem(
                        &mut system,
                        id as i32,
                        cx.style().get_font().gecko(),
                        cx.device().pres_context()
                    )
                }
                let weight = longhands::font_weight::computed_value::T::from_gecko_weight(system.weight);
                let ret = ComputedSystemFont {
                    font_family: longhands::font_family::computed_value::T(
                        longhands::font_family::computed_value::FontFamilyList(
                            unsafe { system.fontlist.mFontlist.mBasePtr.to_safe() }
                        )
                    ),
                    font_size: FontSize {
                            size: Au(system.size).into(),
                            keyword_info: None
                    },
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
                    font_variant_alternates: longhands::font_variant_alternates::get_initial_value(),
                    system_font: *self,
                    default_font_type: system.fontlist.mDefaultFontType,
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
            pub default_font_type: FontFamilyType,
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
        // a lot of code with `if product == gecko` conditionals, we have a
        // dummy system font module that does nothing

        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ToCss)]
        #[cfg_attr(feature = "servo", derive(MallocSizeOf))]
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
                         flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                         animation_value_type="discrete")}

${helpers.predefined_type("-moz-font-smoothing-background-color",
                          "RGBAColor",
                          "RGBA::transparent()",
                          animation_value_type="AnimatedRGBA",
                          products="gecko",
                          gecko_ffi_name="mFont.fontSmoothingBackgroundColor",
                          internal=True,
                          spec="None (Nonstandard internal property)")}

${helpers.predefined_type("-moz-min-font-size-ratio",
                          "Percentage",
                          "computed::Percentage::hundred()",
                          animation_value_type="none",
                          products="gecko",
                          internal=True,
                          spec="Nonstandard (Internal-only)")}
