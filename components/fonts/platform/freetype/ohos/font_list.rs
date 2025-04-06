/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::{HashMap, HashSet};
use std::convert::From;
use std::hash::{Hash, Hasher};
use std::ops::{BitAnd, RangeInclusive};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{fmt, fs, io, thread, time};

use base::text::{UnicodeBlock, UnicodeBlockMethod};
use icu_locid::LanguageIdentifier;
// Proper locale handling
use icu_locid::subtags::{Language, Script};
use log::{debug, warn};
use style::Atom;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};

mod config_local_font_resolution;
mod fallback_local_font_resolution;

use super::json;
use crate::platform::font_list::config_local_font_resolution::{
    fallback_font_families_from_ohos_fontconfig, generic_font_families_from_ohos_fontconfig,
};
use crate::platform::font_list::fallback_local_font_resolution::{
    detect_installed_font_families, generate_default_fallback_associations,
    generate_default_fallback_font_aliases, generate_default_fallback_font_families,
    generate_hardcoded_font_families,
};
use crate::platform::freetype::ohos::iso_values_converter::{convert_language, convert_script};
use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontIdentifier, FontTemplate,
    FontTemplateDescriptor, LocalFontIdentifier, LowercaseFontFamilyName,
};

static FONT_LIST: LazyLock<FontList> = LazyLock::new(FontList::new);

/// When testing the ohos font code on linux, we can pass the fonts directory of the SDK
/// via an environment variable.
#[cfg(ohos_mock)]
static OHOS_FONTS_DIR: &str = env!("OHOS_SDK_FONTS_DIR");

/// On OpenHarmony devices the fonts are always located here.
#[cfg(not(ohos_mock))]
static OHOS_FONTS_DIR: &str = "/system/fonts";

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default)]
// HarmonyOS only comes in Condensed and Normal variants
enum FontWidth {
    Condensed,
    #[default]
    Normal,
}

impl From<FontWidth> for StyleFontStretch {
    fn from(value: FontWidth) -> Self {
        match value {
            FontWidth::Condensed => Self::CONDENSED,
            FontWidth::Normal => Self::NORMAL,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct OpenHarmonyFontDescriptor {
    // `LocalFontIdentifier` uses `Atom` for string interning and requires a String or str, so we
    // already require a String here, instead of using a PathBuf.
    filepath: String,
    weight: Option<i32>,
    style: Option<String>,
    width: FontWidth,
    #[allow(unused)]
    unicode_range: Option<Vec<RangeInclusive<u32>>>,
    language: LanguageIdentifier,
}

// Most font faces on OpenHarmony platform are TrueType.
// That means to properly segment font files into set of families we must read name table of .ttf or .ttc file
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
// Current servo architecture is controversial cause it doesn't allow us to read fonts here, but expects that
// we will somehow get propper separation into font families here. We will rely on OpenHarmony fontconfig.json
// but that could lead to potential problems

// TODO(ddesyatkin) I see the following solution:
// 1) send the request to load the fonts to webrenderer through CrossProcessCompositorApi
// 2) send the request to return filepath - font family association that WebRendered will get from parsing of font files
// 3) As soon as we got response modify FONT_LIST inside thread?

#[derive(Clone, Debug)]
struct FontFamily {
    name: String,
    fonts: Vec<OpenHarmonyFontDescriptor>,
}

#[derive(Debug)]
struct FontAlias {
    from: String,
    to: String,
    weight: Option<i32>,
}

// enum unicode_block::UnicodeBlock contains 327 entries -> 9 bit min
// enum fonts::EmojiPresentationPreference 3 entries -> 2 bit min
//
// https://www.loc.gov/standards/iso639-2/php/code_list.php
// https://www.loc.gov/standards/iso639-2/langcodes.html
// ISO 639.2 contains 3 letter codes. We can represent them in ascii in that case they will occupy 24 bits.
//
// https://unicode.org/iso15924/iso15924-codes.html
// ISO 15924 contains 4 letter codes. They also have numerical representation with upper value 999. 10 bit
//
// Total requred number of bits = 2+9+24+10 = 45
//
// Bsed on information above, lets create the following key
// Lets reserve:
// 8 bit for EmojiPresentationPreference
// 16 bit for ISO 15924 Script code
// 24 bit for ISO 639.2 Lang code
// 16 bit for unicode_block::UnicodeBlock
// Lets create following key on top of this values:
//  Less                                                              Most
//  significant                                                       significant
//  bit                                                               bit
//  |     ISO 15924 Script code       ISO 639.2 Lang code             |
//  |          |                             |                        |
//  ↓----------↓-----------------------------↓------------------------↓
// |xxxxxxxx|xxxxxxxxxxxxxxxx|xxxxxxxxxxxxxxxxxxxxxxxx|xxxxxxxxxxxxxxxx|
//  -↑---------------------------------------------------------↑-------
//   |                                                         |
//  EmojiPresentationPreference                       unicode_block::UnicodeBlock
//

// TODO(ddesyatkin): Should I express it in bitfields? What is the idiomatic way in rust?
// https://github.com/dzamlo/rust-bitfield/blob/master/examples/ipv4.rs
// I feel like expressing it in single value is more beneficial so leave as is for now
#[derive(Clone, Eq, PartialEq)]
struct FallbackOptionsKey(u64);

struct FallbackAssociations(HashMap<FallbackOptionsKey, HashSet<String>>);

/// All fonts in this FontList should follow CSS font families definitions!;
/// <https://www.w3.org/TR/css-fonts-4>
struct FontList {
    /// Array of font descriptors (Generic families for OpenHarmony, not for servo);
    /// each FontFamily contains pair of family name set of font-faces available within
    /// family in form of descriptors
    /// Servo should get following values from here:
    /// (ui-serif, ui-sans-serif, ui-monospace, ui-rounded)
    generic_families: Vec<FontFamily>,

    /// Set of font families that will participate in `installed font fallback`;
    /// they are mentioned in:
    /// <https://www.w3.org/TR/css-fonts-4/#font-style-matching>
    /// point 7
    fallback_families: Vec<FontFamily>,
    /// Set of font aliases; they are mentioned in:
    /// <https://www.w3.org/TR/css-fonts-4/#font-style-matching>
    /// point 3
    aliases: Vec<FontAlias>,

    /// Helper structure that contains set of font fallback association
    /// that is required for script specific font matching.
    fallback_families_associations: FallbackAssociations,
}

pub fn enumerate_font_files(dir_path: &str) -> io::Result<Vec<PathBuf>> {
    let mut font_list = vec![];
    for elem in fs::read_dir(dir_path)?.flatten() {
        if elem.file_type().unwrap().is_file() || elem.file_type().unwrap().is_symlink() {
            let name = elem.file_name();
            let raw_name = name.as_bytes();
            if raw_name.ends_with(b".ttf".as_ref()) || raw_name.ends_with(b".ttc".as_ref()) {
                let Ok(canonical_path) = fs::canonicalize(elem.path()) else {
                    warn!(
                        "We was not able to canonicalize path for: {} file",
                        elem.file_name().to_str().unwrap()
                    );
                    continue;
                };

                if log::log_enabled!(log::Level::Debug) {
                    debug!("Found font {}", elem.file_name().to_str().unwrap());
                }
                font_list.push(canonical_path)
            }
        }
    }
    Ok(font_list)
}

impl FallbackOptionsKey {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::new_full_properties(0, 0, 0, 0)
    }

    pub fn new_from_options(options: &FallbackFontSelectionOptions) -> FallbackOptionsKey {
        let mut block_u64: u64 = 0;
        let language_u64 = convert_language(options.language.language) as u64;
        let script_u64 = convert_script(options.language.script) as u64;
        let presenation_pref_u64 = options.presentation_preference as u64;
        if let Some(block) = options.character.block() {
            block_u64 = block as u64;
        }
        Self::new_full_properties(block_u64, language_u64, script_u64, presenation_pref_u64)
    }

    pub fn new_from_block(block: UnicodeBlock) -> FallbackOptionsKey {
        let block_u64 = block as u64;
        Self::new_full_properties(block_u64, 0, 0, 0)
    }

    #[allow(unused)]
    pub fn new_from_language(subtag_language: Language) -> FallbackOptionsKey {
        let language_u64 = convert_language(subtag_language) as u64;
        Self::new_full_properties(0, language_u64, 0, 0)
    }

    pub fn new_from_script(subtag_script: Option<Script>) -> FallbackOptionsKey {
        let script_u64 = convert_script(subtag_script) as u64;
        Self::new_full_properties(0, 0, script_u64, 0)
    }

    pub fn new_from_lang_id(language_id: LanguageIdentifier) -> FallbackOptionsKey {
        let language_u64 = convert_language(language_id.language) as u64;
        let script_u64 = convert_script(language_id.script) as u64;
        Self::new_full_properties(0, language_u64, script_u64, 0)
    }

    pub fn new_from_emoji_presentations_pref(
        presentation_pref: EmojiPresentationPreference,
    ) -> FallbackOptionsKey {
        let presentation_pref_u64 = presentation_pref as u64;
        Self::new_full_properties(0, 0, 0, presentation_pref_u64)
    }

    // Do not forget to update masks with constructor
    pub fn unicode_block_mask() -> u64 {
        const UNICODE_BLOCK_MASK: u64 = (u16::MAX as u64) << 48;
        return UNICODE_BLOCK_MASK;
    }

    pub fn language_mask() -> u64 {
        // We must take only 24 bits here that was shifted on 24 bits
        // everything is correct;
        const LANG_MASK: u64 = (2_u64.pow(24) - 1) << 24;
        return LANG_MASK;
    }

    pub fn script_mask() -> u64 {
        const SCRIPT_MASK: u64 = (u16::MAX as u64) << 8;
        return SCRIPT_MASK;
    }

    pub fn emoji_presentation_pref_mask() -> u64 {
        const EMOJI_PRESENTATION_PREF_MASK: u64 = u8::MAX as u64;
        return EMOJI_PRESENTATION_PREF_MASK;
    }

    // combined mask
    pub fn language_script_mask() -> u64 {
        Self::language_mask() & Self::script_mask()
    }

    fn new_full_properties(block: u64, language: u64, script: u64, presentation_pref: u64) -> Self {
        // Function to state all bitshifts only once to ease constructor maintanence
        let mut value: u64 = 0;
        value |= presentation_pref as u64;
        value |= (script as u64) << 8;
        value |= (language as u64) << 24;
        value |= (block as u64) << 48;
        Self { 0: value }
    }
}

impl fmt::Display for FallbackOptionsKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#066b}", self.0)
    }
}

impl fmt::Debug for FallbackOptionsKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#066b}", self.0)
    }
}

impl BitAnd<u64> for FallbackOptionsKey {
    type Output = Self;
    fn bitand(self, rhs: u64) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl Hash for FallbackOptionsKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl FallbackAssociations {
    fn new() -> FallbackAssociations {
        Self {
            0: HashMap::<FallbackOptionsKey, HashSet<String>>::new(),
        }
    }

    fn insert(&mut self, entry: (FallbackOptionsKey, HashSet<String>)) {
        self.0.insert(entry.0, entry.1);
    }

    fn add_value_to_set_on_key(&mut self, key: FallbackOptionsKey, value: String) {
        if self.0.contains_key(&key) {
            let family_set = self.0.get_mut(&key).unwrap();
            family_set.insert(value);
        } else {
            let mut new_family_set = HashSet::<String>::new();
            new_family_set.insert(value);
            self.0.insert(key, new_family_set);
        }
    }

    fn extend(&mut self, other: FallbackAssociations) {
        other.into_iter().for_each(|other_entry| {
            let (other_key, other_family_set) = other_entry;
            if self.0.contains_key(&other_key) {
                let self_family_set = self.0.get_mut(&other_key).unwrap();
                self_family_set.extend(other_family_set);
            } else {
                self.0.insert(other_key, other_family_set);
            }
        });
    }

    fn find_by_block(
        &self,
        key: FallbackOptionsKey,
    ) -> Option<(&FallbackOptionsKey, &HashSet<String>)> {
        let block_mask = FallbackOptionsKey::unicode_block_mask();
        let block_key = key & block_mask;
        self.0.get_key_value(&block_key)
    }

    #[allow(unused)]
    fn find_by_language(
        &self,
        key: FallbackOptionsKey,
    ) -> Option<(&FallbackOptionsKey, &HashSet<String>)> {
        let lang_mask = FallbackOptionsKey::language_mask();
        let lang_key = key & lang_mask;
        self.0.get_key_value(&lang_key)
    }

    #[allow(unused)]
    fn find_by_script(
        &self,
        key: FallbackOptionsKey,
    ) -> Option<(&FallbackOptionsKey, &HashSet<String>)> {
        let script_mask = FallbackOptionsKey::script_mask();
        let script_key = key & script_mask;
        self.0.get_key_value(&script_key)
    }

    fn find_by_language_script(
        &self,
        key: FallbackOptionsKey,
    ) -> Option<(&FallbackOptionsKey, &HashSet<String>)> {
        let lang_script_mask = FallbackOptionsKey::language_script_mask();
        let lang_script_key = key & lang_script_mask;
        self.0.get_key_value(&lang_script_key)
    }

    fn find_by_emoji_presentation_options(
        &self,
        key: FallbackOptionsKey,
    ) -> Option<(&FallbackOptionsKey, &HashSet<String>)> {
        let emoji_pres_pref_mask = FallbackOptionsKey::emoji_presentation_pref_mask();
        let emoji_key = key & emoji_pres_pref_mask;
        self.0.get_key_value(&emoji_key)
    }
}

impl IntoIterator for FallbackAssociations {
    type Item = <HashMap<FallbackOptionsKey, HashSet<String>> as IntoIterator>::Item;
    type IntoIter = <HashMap<FallbackOptionsKey, HashSet<String>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FontList {
    fn new() -> FontList {
        // We can not verify correctness of ohos fontconfig without reading folders that
        // contain device fonts; So if we found them, and config was correct we return
        // them together.
        if let Some((config, _font_paths)) = json::load_and_verify_ohos_fontconfig() {
            // Process OS config generic families entry
            let (mut generic_families, generic_families_aliases) =
                generic_font_families_from_ohos_fontconfig(&config);
            // Process OS fallback families; We may modify families_from_generic here!
            // Do not try to use families_from_generic before
            let (fallback_families, fallback_associations) =
                fallback_font_families_from_ohos_fontconfig(&mut generic_families, &config);
            generic_families.extend(generate_hardcoded_font_families());
            if log::log_enabled!(log::Level::Debug) {
                thread::sleep(time::Duration::from_millis(1000));
                log::warn!("Generic font families from config:");
                for test in &generic_families {
                    thread::sleep(time::Duration::from_millis(1));
                    log::warn!("{:#?}", test);
                }
                thread::sleep(time::Duration::from_millis(1));
                log::warn!("Generic font families aliases from config:");
                for test in &generic_families_aliases {
                    thread::sleep(time::Duration::from_millis(1));
                    log::warn!("{:#?}", test);
                }
                thread::sleep(time::Duration::from_millis(1));
                log::warn!("Fallback font families from config:");
                for test in &fallback_families {
                    thread::sleep(time::Duration::from_millis(1));
                    log::warn!("{:#?}", test);
                }
                thread::sleep(time::Duration::from_millis(1));
                log::warn!("Fallback font families associations from config:");
                for test in fallback_associations.0.iter() {
                    thread::sleep(time::Duration::from_millis(1));
                    log::warn!("{:#?}", test);
                }
            }

            return FontList {
                generic_families: generic_families,
                fallback_families: fallback_families,
                aliases: generic_families_aliases,
                fallback_families_associations: fallback_associations,
            };
        }

        FontList {
            generic_families: detect_installed_font_families(),
            fallback_families: generate_default_fallback_font_families(),
            aliases: generate_default_fallback_font_aliases(),
            fallback_families_associations: generate_default_fallback_associations(),
        }
    }

    pub fn find_family(&self, name: &str) -> Option<&FontFamily> {
        let generic = self
            .generic_families
            .iter()
            .find(|family| family.name.eq_ignore_ascii_case(name));
        if generic.is_some() {
            return generic;
        }
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("find_family: looking in fallback families");
        }
        let fallback = self
            .fallback_families
            .iter()
            .find(|family| family.name.eq_ignore_ascii_case(name));
        if fallback.is_some() {
            return fallback;
        }
        if log::log_enabled!(log::Level::Error) {
            log::error!(
                "find_family: {} was not found in generic families nor in fallback families!",
                name
            );
        }
        None
    }

    pub fn find_first_suitable_alias(&self, family_name: &str) -> Option<&FontAlias> {
        let first_alias = self
            .aliases
            .iter()
            .find(|alias| alias.from.eq_ignore_ascii_case(family_name));
        if first_alias.is_some() {
            return first_alias;
        }
        if log::log_enabled!(log::Level::Error) {
            log::error!(
                "find_first_suitable_alias: Not a single alias was associated with {family_name}!"
            );
        }
        None
    }

    pub fn find_all_suitable_aliases(&self, family_name: &str) -> Option<Vec<&FontAlias>> {
        let mut result = Vec::<&FontAlias>::new();
        result.extend(self.aliases.iter().filter_map(|alias| {
            if alias.from.eq_ignore_ascii_case(family_name) {
                Some(alias)
            } else {
                None
            }
        }));
        if result.is_empty() {
            if log::log_enabled!(log::Level::Error) {
                log::error!(
                    "find_all_suitable_aliases: Not a single alias was associated with {}!",
                    family_name
                );
            }
            None
        } else {
            Some(result)
        }
    }

    pub fn generic_font_families(&self) -> &Vec<FontFamily> {
        &self.generic_families
    }

    pub fn fallback_families(&self) -> &Vec<FontFamily> {
        &self.fallback_families
    }

    pub fn font_aliases(&self) -> &Vec<FontAlias> {
        &self.aliases
    }

    pub fn fallback_families_associations(&self) -> &FallbackAssociations {
        &self.fallback_families_associations
    }
}

// Functions used by SystemFontService
pub fn for_each_available_family<F>(mut callback: F)
where
    F: FnMut(String),
{
    for family in FONT_LIST.generic_font_families() {
        callback(family.name.clone());
    }
    for family in FONT_LIST.fallback_families() {
        callback(family.name.clone());
    }
    // Probably aliases was created before, cause only
    // generic family can add them in ohos fontconfig. But lets,
    // traverse them nontheless
    for alias in FONT_LIST.font_aliases() {
        callback(alias.to.clone());
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(FontTemplate),
{
    let mut produce_font = |font: &OpenHarmonyFontDescriptor, variation_index: &i32| {
        let local_font_identifier = LocalFontIdentifier {
            path: Atom::from(font.filepath.clone()),
            variation_index: *variation_index,
        };
        let stretch = font.width.into();
        let weight = font
            .weight
            .map(|w| StyleFontWeight::from_float(w as f32))
            .unwrap_or(StyleFontWeight::NORMAL);

        // After variable fonts will be supported uncomment this
        // Correct conversion code for variable font-weight.
        // But currently it is not supported.
        // let weight_pair = match &font.weight {
        //     Some(value) => {
        //         if *value == 0 {
        //             let min_weight = StyleFontWeight::from_float(MIN_FONT_WEIGHT);
        //             let max_weight = StyleFontWeight::from_float(MAX_FONT_WEIGHT);
        //             (min_weight, max_weight)
        //         } else {
        //             let weight_value = StyleFontWeight::from_float(*value as f32);
        //             (weight_value, weight_value)
        //         }
        //     },
        //     _ => {
        //         (StyleFontWeight::NORMAL, StyleFontWeight::NORMAL)
        //     }
        // };

        let style = match font.style.as_deref() {
            Some("italic") => StyleFontStyle::ITALIC,
            Some("normal") => StyleFontStyle::NORMAL,
            Some(value) => {
                warn!(
                    "unknown value \"{value}\" for \"style\" attribute in the font {}",
                    font.filepath
                );
                StyleFontStyle::NORMAL
            },
            None => StyleFontStyle::NORMAL,
        };

        // Example of template for variable font
        let variable_font_template_descriptor = FontTemplateDescriptor {
            weight: (weight, weight),
            stretch: (stretch, stretch),
            style: (style, style),
            languages: Some(vec![font.language.clone()]),
            unicode_range: None,
        };
        callback(FontTemplate::new(
            FontIdentifier::Local(local_font_identifier),
            variable_font_template_descriptor,
            None,
        ));
    };

    if let Some(family) = FONT_LIST.find_family(family_name) {
        let variation_index = 0;
        for font in &family.fonts {
            produce_font(font, &variation_index);
        }
        return;
    }

    if let Some(aliases) = FONT_LIST.find_all_suitable_aliases(family_name) {
        if log::log_enabled!(log::Level::Warn) {
            log::warn!(
                "Was able to find alias for queried family name: {}",
                family_name
            );
        }
        // TODO(ddesyatkin): Too many levels. Separate to different functions
        for alias in aliases {
            if let Some(family) = FONT_LIST.find_family(&alias.to) {
                let mut variation_index = 0;
                for font in &family.fonts {
                    match (alias.weight, font.weight) {
                        (None, _) => {
                            produce_font(font, &variation_index);
                            variation_index += 1;
                        },
                        (Some(w1), Some(w2)) => {
                            if w1 == w2 {
                                produce_font(font, &variation_index);
                                variation_index += 1;
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
    }
}

// Based on fonts present in OpenHarmony.
pub fn os_fallback_families(options: FallbackFontSelectionOptions) -> Vec<&'static str> {
    let mut families = vec![];
    // Construct dynamic part of the fallback;
    // It will change each time depending on FallbackFontSelectionOptions

    // I added script to FontTemplateDescriptor, so now is should be obsolete!
    let fallback_families_associations = FONT_LIST.fallback_families_associations();
    let key = FallbackOptionsKey::new_from_options(&options);

    let mut final_set = HashSet::<&'static str>::new();
    let emoji_set_candidate =
        fallback_families_associations.find_by_emoji_presentation_options(key.clone());
    if let Some((_key, emoji_set)) = emoji_set_candidate {
        final_set.extend(emoji_set.iter().map(|entry| entry.as_str()));
    }
    let script_set_candidate = fallback_families_associations.find_by_language_script(key.clone());
    if let Some((_key, script_set)) = script_set_candidate {
        final_set.extend(script_set.iter().map(|entry| entry.as_str()));
    }
    let block_set_candidate = fallback_families_associations.find_by_block(key.clone());
    if let Some((_key, block_set)) = block_set_candidate {
        final_set.extend(block_set.iter().map(|entry| entry.as_str()));
    }
    families.extend(final_set.iter());

    // Construct static part of the fallback
    // "fallback": [
    //     { "": [
    //             ...,
    //             {
    //             "": "Noto Sans"
    //             },
    //             ...,
    //         ]
    //     }
    // ]
    // Currently we have one unconditional fallback in ohos fontconfig.json
    // I interpret this as default family in case we have not matched against any
    // system font.

    // Currently if FONT_LIST was generated by fontconfig we have generic families in dynamic part
    // In general in this unconditional block we expect all generic system families
    // "generic": [
    // {
    //     "family": "HarmonyOS Sans",
    //      ...
    // }
    // {
    //     "family": "HarmonyOS Sans Condensed",
    //      ...
    // }
    // {
    //     "family": "HarmonyOS Sans Digit",
    //     ...
    // }
    // {
    //     "family": "Noto Serif",
    //     ...
    // }
    // {
    //     "family": "Noto Sans Mono",
    //     ...
    // }
    families.extend(
        FONT_LIST
            .generic_font_families()
            .iter()
            .map(|entry| entry.name.as_str()),
    );

    // if log::log_enabled!(log::Level::Debug) {
    //     log::warn!(
    //         "character: {} generated following fallback list\n{:?}",
    //         options.character,
    //         families
    //     );
    // }
    families
}

pub fn default_system_generic_font_family(generic: GenericFontFamily) -> LowercaseFontFamilyName {
    // It is weird that OpenHarmony fontconfig.json provides generic families as alliases
    // but we will use it as is for now. However it is definately break
    // https://www.w3.org/TR/css-fonts-4/#font-style-matching

    // Currently it is hardcoded think how we can properly extract this from fontconfig.json
    let default_family_name = "HarmonyOS Sans".into();
    let fallback_family_name = "Noto Sans".into();
    match generic {
        GenericFontFamily::Serif => {
            // serif
            if let Some(alias) = FONT_LIST.find_first_suitable_alias("serif") {
                alias.from.clone().into()
            } else {
                default_family_name
            }
        },
        GenericFontFamily::SansSerif => {
            // sans-serif
            if let Some(alias) = FONT_LIST.find_first_suitable_alias("sans-serif") {
                alias.from.clone().into()
            } else {
                default_family_name
            }
        },
        GenericFontFamily::Monospace => {
            // monospace
            if let Some(alias) = FONT_LIST.find_first_suitable_alias("monospace") {
                alias.from.clone().into()
            } else {
                default_family_name
            }
        },
        GenericFontFamily::SystemUi => {
            // system-ui
            if let Some(alias) = FONT_LIST.find_first_suitable_alias("system-ui") {
                alias.from.clone().into()
            } else {
                default_family_name
            }
        },
        GenericFontFamily::None => fallback_family_name,
        _ => default_family_name,
    }
}
