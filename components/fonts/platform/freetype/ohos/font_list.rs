/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::{HashMap, HashSet};
use std::ops::BitAnd;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{fs, io};

use base::text::{UnicodeBlock, UnicodeBlockMethod};
use log::{debug, warn};
use style::Atom;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};
use unicode_script::Script;

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
struct Font {
    // `LocalFontIdentifier` uses `Atom` for string interning and requires a String or str, so we
    // already require a String here, instead of using a PathBuf.
    filepath: String,
    weight: Option<i32>,
    style: Option<String>,
    width: FontWidth,
}

// Most font faces on OpenHarmony platform are TrueType.
// That means to properly segment font files into set of families we must read name table of .ttf or .ttc file
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
// Current servo architecture is controversial cause it doesn't allow us to read fonts here, but expects that
// we will somehow get propper separation into font families here. We will rely on OpenHarmony fontconfig.json
// but that could lead to potential problems

// TODO(ddesyatkin)
// I see the following solution:
// 1) send the request to load the fonts to webrenderer through CrossProcessCompositorApi
// 2) send the request to return filepath - font family association that WebRendered will get from parsing of font files
// 3) As soon as we got response modify FONT_LIST inside thread?

#[derive(Clone, Debug)]
struct FontFamily {
    name: String,
    fonts: Vec<Font>,
}

#[derive(Debug)]
struct FontAlias {
    from: String,
    to: String,
    weight: Option<i32>,
}

// enum unicode_block::UnicodeBlock contains 327 entries -> 9 bit min
// enum unicode_script:SCRIPT contains 255 entries -> 8 bit
// enum fonts::EmojiPresentationPreference 3 entry -> 2 bit min
// Lets reserve 8 bit for EmojiPresentationPreference
// And 16 bit for unicode_block::UnicodeBlock
// Lets create following key on top of this values:
//
// Less                                                         Most
// significant                                                  significant
// bit                                                          bit
// |     8 bit padding               16 bit padding               |
// |    |              |                  |                       |
// ↓    ↓              ↓                  ↓                       ↓
// 00000000xxxxxxxx00000000xxxxxxxx0000000000000000xxxxxxxxxxxxxxxx
//            ↑                 ↑                           ↑
//            |                 |                           |
// EmojiPresentationPreference  |                unicode_block::UnicodeBlock
//                     unicode_script:SCRIPT
//
#[derive(Clone, Eq, Hash, PartialEq)]
struct FallbackOptionsKey(u64);

struct FallbackAssociations(HashMap<FallbackOptionsKey, HashSet<String>>);

// OHOS fontconfig.json currently will use only 2 versions of
struct FontList {
    generic_families: Vec<FontFamily>,
    fallback_families: Vec<FontFamily>,
    aliases: Vec<FontAlias>,
    // Code reviewers. Please lets discuss possible colisions here.
    fallback_families_associations: FallbackAssociations,
}

pub fn enumerate_font_files(dir_path: &str) -> io::Result<Vec<PathBuf>> {
    let mut font_list = vec![];
    for elem in fs::read_dir(dir_path)?.flatten() {
        if elem.file_type().unwrap().is_file() || elem.file_type().unwrap().is_symlink() {
            let name = elem.file_name();
            let raw_name = name.as_bytes();
            if raw_name.ends_with(b".ttf".as_ref()) || raw_name.ends_with(b".ttc".as_ref()) {
                if log::log_enabled!(log::Level::Debug) {
                    debug!("Found font {}", elem.file_name().to_str().unwrap());
                }
                font_list.push(elem.path())
            }
        }
    }
    Ok(font_list)
}

impl FallbackOptionsKey {
    fn new_from_options(options: &FallbackFontSelectionOptions) -> FallbackOptionsKey {
        let mut value: u64 = 0;
        let presentation_pref = options.presentation_preference as u8;
        value |= (presentation_pref as u64) << 8;

        let script = Script::from(options.character) as u8;
        value |= (script as u64) << 24;

        if let Some(block) = options.character.block() {
            value |= (block as u64) << 48;
        }
        Self { 0: value }
    }

    // fn new_from_lang_script_str(lang_script: &str) -> FallbackOptionsKey {
    //     let mut value: u64 = 0;
    //     let mut split = lang_script.split('-');
    //     let _lang = split.next();
    //     let script = split.next();
    //     if let Some(script) = script {
    //         let sctipt = Script::inner_from_short_name(script);
    //         value |= (script as u64) << 24;
    //     }

    //     Self {
    //         0: value
    //     }
    // }

    fn new_from_script(script: Script) -> FallbackOptionsKey {
        let mut value: u64 = 0;
        value |= (script as u64) << 24;

        Self { 0: value }
    }

    fn new_from_block(block: UnicodeBlock) -> FallbackOptionsKey {
        let mut value: u64 = 0;
        value |= (block as u64) << 48;
        Self { 0: value }
    }

    fn new_from_emoji_presentations_pref(
        presentation_pref: EmojiPresentationPreference,
    ) -> FallbackOptionsKey {
        let mut value: u64 = 0;
        value |= (presentation_pref as u64) << 8;

        Self { 0: value }
    }
}

impl BitAnd<u64> for FallbackOptionsKey {
    type Output = Self;
    fn bitand(self, rhs: u64) -> Self::Output {
        Self(self.0 & rhs)
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

    fn find_by_script(
        &self,
        key: FallbackOptionsKey,
    ) -> Option<(&FallbackOptionsKey, &HashSet<String>)> {
        let script_key = key & ((u8::MAX as u64) << 24);
        self.0.get_key_value(&script_key)
    }

    fn find_by_block(
        &self,
        key: FallbackOptionsKey,
    ) -> Option<(&FallbackOptionsKey, &HashSet<String>)> {
        let block_key = key & ((u16::MAX as u64) << 48);
        self.0.get_key_value(&block_key)
    }

    fn find_by_emoji_presentation_options(
        &self,
        key: FallbackOptionsKey,
    ) -> Option<(&FallbackOptionsKey, &HashSet<String>)> {
        let emoji_key = key & ((u8::MAX as u64) << 8);
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
            let (mut generic_families, generic_families_aliases) =
                generic_font_families_from_ohos_fontconfig(&config);
            generic_families.extend(generate_hardcoded_font_families());
            // generic_families_aliases.extend(Self::generate_apple_system_font_aliases());

            let (fallback_families, _fallback_associations) =
                fallback_font_families_from_ohos_fontconfig(&mut generic_families, &config);

            if log::log_enabled!(log::Level::Debug) {
                log::warn!("Generic font families from config:");
                for test in &generic_families {
                    log::warn!("{:?}", test);
                }
                log::warn!("Generic font families aliases from config:");
                for test in &generic_families_aliases {
                    log::warn!("{:?}", test);
                }
                log::warn!("Fallback font families from config:");
                for test in &fallback_families {
                    log::warn!("{:?}", test);
                }
            }

            return FontList {
                generic_families: generic_families,
                fallback_families: fallback_families,
                aliases: generic_families_aliases,
                fallback_families_associations: generate_default_fallback_associations(),
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
                    "find_all_suitable_aliases: Not a single alias was associated with {family_name}!"
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

    pub fn fallback_font_families(&self) -> &Vec<FontFamily> {
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
    for family in FONT_LIST.fallback_font_families() {
        callback(family.name.clone());
    }
    for family in FONT_LIST.generic_font_families() {
        callback(family.name.clone());
    }
    for alias in FONT_LIST.font_aliases() {
        callback(alias.from.clone());
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(FontTemplate),
{
    let mut produce_font = |font: &Font, variation_index: &i32| {
        let local_font_identifier = LocalFontIdentifier {
            path: Atom::from(font.filepath.clone()),
            variation_index: *variation_index,
        };
        let stretch = font.width.into();
        let weight = font
            .weight
            .map(|w| StyleFontWeight::from_float(w as f32))
            .unwrap_or(StyleFontWeight::NORMAL);

        // Correct conversion code for variable font-weight.
        // But currently it is not supported.
        // let weight = match &font.weight {
        //     Some(value) => {
        //         let value = StyleFontWeight::from_float(*value as f32);
        //         (value, value)
        //     },
        //     _ => {
        //         // we parsed fontconfig and found 0, then we replaced zero to NONE
        //         // So here None means dynamic font;
        //         let min = StyleFontWeight::from_float(MIN_FONT_WEIGHT);
        //         let max = StyleFontWeight::from_float(MAX_FONT_WEIGHT);
        //         (min, max)
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
        // Not supported yet
        // let variable_font_template_example = FontTemplateDescriptor {
        //     weight,
        //     stretch: (stretch, stretch),
        //     style: (style, style),
        //     unicode_range: None,
        // };
        let descriptor = FontTemplateDescriptor::new(weight, stretch, style);
        callback(FontTemplate::new(
            FontIdentifier::Local(local_font_identifier),
            descriptor,
            None,
        ));
    };

    if let Some(family) = FONT_LIST.find_family(family_name) {
        let mut variation_index = 0;
        for font in &family.fonts {
            produce_font(font, &variation_index);
            variation_index += 1;
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
pub fn fallback_font_families(options: FallbackFontSelectionOptions) -> Vec<&'static str> {
    let mut families = vec![];
    // Construct dynamic part of the fallback;
    // It will change each time depending on FallbackFontSelectionOptions
    let fallback_families_associations = FONT_LIST.fallback_families_associations();
    let key = FallbackOptionsKey::new_from_options(&options);

    let mut final_set = HashSet::<&'static str>::new();
    let emoji_set_candidate =
        fallback_families_associations.find_by_emoji_presentation_options(key.clone());
    if let Some((_key, emoji_set)) = emoji_set_candidate {
        final_set.extend(emoji_set.iter().map(|entry| entry.as_str()));
    }
    let script_set_candidate = fallback_families_associations.find_by_script(key.clone());
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
    families.push("Noto Sans");

    // In general in this unconditional block we expect all generic system families
    // In the same order as they stated in fontconfig.json
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

    // Not in OpenHarmony generic family config. But we add them for emoji support
    families.push("Noto Sans Symbols");
    families.push("Noto Sans Symbols 2");

    if log::log_enabled!(log::Level::Debug) {
        log::warn!(
            "character: {} generated following fallback list\n{:?}",
            options.character,
            families
        );
    }
    families
}

pub fn default_system_generic_font_family(generic: GenericFontFamily) -> LowercaseFontFamilyName {
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
            // serif
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
        GenericFontFamily::None => fallback_family_name,
        _ => default_family_name,
    }
}
