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

${helpers.predefined_type("font-synthesis",
                          "FontSynthesis",
                          products="gecko",
                          initial_value="specified::FontSynthesis::get_initial_value()",
                          animation_value_type="discrete",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-synthesis")}

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

${helpers.predefined_type("font-variant-alternates",
                          "FontVariantAlternates",
                          products="gecko",
                          initial_value="computed::FontVariantAlternates::get_initial_value()",
                          initial_specified_value="specified::FontVariantAlternates::get_initial_specified_value()",
                          animation_value_type="discrete",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-alternates")}

${helpers.predefined_type("font-variant-east-asian",
                          "FontVariantEastAsian",
                          products="gecko",
                          initial_value="computed::FontVariantEastAsian::empty()",
                          initial_specified_value="specified::FontVariantEastAsian::empty()",
                          animation_value_type="discrete",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-east-asian")}

${helpers.predefined_type("font-variant-ligatures",
                          "FontVariantLigatures",
                          products="gecko",
                          initial_value="computed::FontVariantLigatures::empty()",
                          initial_specified_value="specified::FontVariantLigatures::empty()",
                          animation_value_type="discrete",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-ligatures")}

${helpers.predefined_type("font-variant-numeric",
                          "FontVariantNumeric",
                          products="gecko",
                          initial_value="computed::FontVariantNumeric::empty()",
                          initial_specified_value="specified::FontVariantNumeric::empty()",
                          animation_value_type="discrete",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-numeric")}

${helpers.single_keyword_system("font-variant-position",
                                "normal sub super",
                                products="gecko",
                                gecko_ffi_name="mFont.variantPosition",
                                gecko_constant_prefix="NS_FONT_VARIANT_POSITION",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-position",
                                flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                                animation_value_type="discrete")}

${helpers.predefined_type("font-feature-settings",
                          "FontFeatureSettings",
                          products="gecko",
                          initial_value="computed::FontFeatureSettings::normal()",
                          initial_specified_value="specified::FontFeatureSettings::normal()",
                          extra_prefixes="moz",
                          boxed=True,
                          animation_value_type="discrete",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts/#propdef-font-feature-settings")}

<%
# This spec link is too long to fit elsewhere
variation_spec = """\
https://drafts.csswg.org/css-fonts-4/#low-level-font-variation-settings-control-the-font-variation-settings-property\
"""
%>

${helpers.predefined_type("font-variation-settings",
                          "FontVariantSettings",
                          products="gecko",
                          gecko_pref="layout.css.font-variations.enabled",
                          initial_value="specified::FontVariantSettings::normal()",
                          animation_value_type="ComputedValue",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="${variation_spec}")}

${helpers.predefined_type("font-language-override",
                          "FontLanguageOverride",
                          products="gecko",
                          initial_value="computed::FontLanguageOverride::zero()",
                          initial_specified_value="specified::FontLanguageOverride::normal()",
                          animation_value_type="discrete",
                          extra_prefixes="moz",
                          flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-language-override")}

<%helpers:longhand name="-x-lang" products="gecko" animation_value_type="none"
                   enabled_in=""
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
                   enabled_in="">
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
                          enabled_in="ua",
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
                          enabled_in="",
                          gecko_ffi_name="mScriptMinSize",
                          spec="Internal (not web-exposed)")}

${helpers.predefined_type("-x-text-zoom",
                          "XTextZoom",
                          "computed::XTextZoom(true)",
                          animation_value_type="none",
                          products="gecko",
                          enabled_in="",
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
                         gecko_pref="layout.css.osx-font-smoothing.enabled",
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
                          enabled_in="ua",
                          spec="None (Nonstandard internal property)")}

${helpers.predefined_type("-moz-min-font-size-ratio",
                          "Percentage",
                          "computed::Percentage::hundred()",
                          animation_value_type="none",
                          products="gecko",
                          enabled_in="ua",
                          spec="Nonstandard (Internal-only)")}
