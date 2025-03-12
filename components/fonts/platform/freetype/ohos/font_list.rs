/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::ops::BitAnd;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fs, io};

use base::text::{UnicodeBlock, UnicodeBlockMethod};
use log::{debug, error, warn};
use style::Atom;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};
use unicode_script::Script;

use super::json::{self, FallbackEntryOHOS, FontconfigOHOS, GenericFontFamilyOHOS};
use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontIdentifier, FontTemplate,
    FontTemplateDescriptor, LocalFontIdentifier, LowercaseFontFamilyName,
};

static FONT_LIST: LazyLock<FontList> = LazyLock::new(FontList::new);
// static FALLBACK_ASSOCIATIONS: LazyLock<FallbackAssociations> = LazyLock::new(generate_default_fallback_associations);

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

fn detect_hos_font_style(font_modifiers: &[&str]) -> Option<String> {
    if font_modifiers.contains(&"Italic") {
        Some("italic".to_string())
    } else {
        None
    }
}

// Note: The weights here are taken from the `alias` section of the fontconfig.json
fn detect_hos_font_weight_alias(font_modifiers: &[&str]) -> Option<i32> {
    if font_modifiers.contains(&"Light") {
        Some(100)
    } else if font_modifiers.contains(&"Regular") {
        Some(400)
    } else if font_modifiers.contains(&"Medium") {
        Some(700)
    } else if font_modifiers.contains(&"Bold") {
        Some(900)
    } else {
        None
    }
}

fn noto_weight_alias(alias: &str) -> Option<i32> {
    match alias.to_ascii_lowercase().as_str() {
        "thin" => Some(100),
        "extralight" => Some(200),
        "light" => Some(300),
        "regular" => Some(400),
        "medium" => Some(500),
        "semibold" => Some(600),
        "bold" => Some(700),
        "extrabold" => Some(800),
        "black" => Some(900),
        _unknown_alias => {
            warn!("Unknown weight alias `{alias}` encountered.");
            None
        },
    }
}

fn detect_hos_font_width(font_modifiers: &[&str]) -> FontWidth {
    if font_modifiers.contains(&"Condensed") {
        FontWidth::Condensed
    } else {
        FontWidth::Normal
    }
}

/// Split a Noto font filename into the family name with spaces
///
/// E.g. `NotoSansTeluguUI` -> `Noto Sans Telugu UI`
/// Or for older OH 4.1 fonts: `NotoSans_JP_Bold` -> `Noto Sans JP Bold`
fn split_noto_font_name(name: &str) -> Vec<String> {
    let mut name_components = vec![];
    let mut current_word = String::new();
    let mut chars = name.chars();
    // To not split acronyms like `UI` or `CJK`, we only start a new word if the previous
    // char was not uppercase.
    let mut previous_char_was_uppercase = true;
    if let Some(first) = chars.next() {
        current_word.push(first);
        for c in chars {
            if c.is_uppercase() {
                if !previous_char_was_uppercase {
                    name_components.push(current_word.clone());
                    current_word = String::new();
                }
                previous_char_was_uppercase = true;
                current_word.push(c)
            } else if c == '_' {
                name_components.push(current_word.clone());
                current_word = String::new();
                previous_char_was_uppercase = true;
                // Skip the underscore itself
            } else {
                previous_char_was_uppercase = false;
                current_word.push(c)
            }
        }
    }
    if !current_word.is_empty() {
        name_components.push(current_word);
    }
    name_components
}

/// Parse the font file names to determine the available FontFamilies
///
/// Note: For OH 5.0+ this function is intended to only be a fallback path, if parsing the
/// `fontconfig.json` fails for some reason. Beta 1 of OH 5.0 still has a bug in the fontconfig.json
/// though, so the "normal path" is currently unimplemented.
fn parse_font_filenames(font_files: Vec<PathBuf>) -> Vec<FontFamily> {
    let harmonyos_prefix = "HarmonyOS_Sans";

    let weight_aliases = ["Light", "Regular", "Medium", "Bold"];
    let style_modifiers = ["Italic"];
    let width_modifiers = ["Condensed"];

    let mut families: HashMap<String, Vec<Font>> = HashMap::new();

    let font_files: Vec<PathBuf> = font_files
        .into_iter()
        .filter(|file_path| {
            if let Some(extension) = file_path.extension() {
                // whitelist of extensions we expect to be fonts
                let valid_font_extensions =
                    [OsStr::new("ttf"), OsStr::new("ttc"), OsStr::new("otf")];
                if valid_font_extensions.contains(&extension) {
                    return true;
                }
            }
            false
        })
        .collect();

    let harmony_os_fonts = font_files.iter().filter_map(|file_path| {
        let stem = file_path.file_stem()?.to_str()?;
        let stem_no_prefix = stem.strip_prefix(harmonyos_prefix)?;
        let name_components: Vec<&str> = stem_no_prefix.split('_').collect();
        let style = detect_hos_font_style(&name_components);
        let weight = detect_hos_font_weight_alias(&name_components);
        let width = detect_hos_font_width(&name_components);

        let mut name_components = name_components;
        // If we remove all the modifiers, we are left with the family name
        name_components.retain(|component| {
            !weight_aliases.contains(component) &&
                !style_modifiers.contains(component) &&
                !width_modifiers.contains(component) &&
                !component.is_empty()
        });
        name_components.insert(0, "HarmonyOS Sans");
        let family_name = name_components.join(" ");
        let font = Font {
            filepath: file_path.to_str()?.to_string(),
            weight,
            style,
            width,
        };
        Some((family_name, font))
    });

    let noto_fonts = font_files.iter().filter_map(|file_path| {
        let stem = file_path.file_stem()?.to_str()?;
        // Filter out non-noto fonts
        if !stem.starts_with("Noto") {
            return None;
        }
        // Strip the weight alias from the filename, e.g. `-Regular` or `_Regular`.
        // We use `rsplit_once()`, since there is e.g. `NotoSansPhags-Pa-Regular.ttf`, where the
        // Pa is part of the font family name and not a modifier.
        // There seem to be no more than one modifier at once per font filename.
        let (base, weight) = if let Some((stripped_base, weight_suffix)) =
            stem.rsplit_once("-").or_else(|| stem.rsplit_once("_"))
        {
            (stripped_base, noto_weight_alias(weight_suffix))
        } else {
            (stem, None)
        };
        // Do some special post-processing for `NotoSansPhags-Pa-Regular.ttf` and any friends.
        let base = if base.contains("-") {
            if !base.ends_with("-Pa") {
                warn!("Unknown `-` pattern in Noto font filename: {base}");
            }
            // Note: We assume here that the following character is uppercase, so that
            // the word splitting later functions correctly.
            base.replace("-", "")
        } else {
            base.to_string()
        };
        // Remove suffixes `[wght]` or `[wdth,wght]`. These suffixes seem to be mutually exclusive
        // with the weight alias suffixes from before.
        let base_name = base
            .strip_suffix("[wght]")
            .or_else(|| base.strip_suffix("[wdth,wght]"))
            .unwrap_or(base.as_str());
        let family_name = split_noto_font_name(base_name).join(" ");
        let font = Font {
            filepath: file_path.to_str()?.to_string(),
            weight,
            ..Default::default()
        };
        Some((family_name, font))
    });

    let all_families = harmony_os_fonts.chain(noto_fonts);

    for (family_name, font) in all_families {
        if let Some(font_list) = families.get_mut(&family_name) {
            font_list.push(font);
        } else {
            families.insert(family_name, vec![font]);
        }
    }

    families
        .into_iter()
        .map(|(name, fonts)| FontFamily { name, fonts })
        .collect()
}

impl FallbackOptionsKey {
    fn new() -> FallbackOptionsKey {
        Self { 0: 0 }
    }

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

    fn contains(&self, key: &FallbackOptionsKey) -> bool {
        self.0.contains_key(&key)
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
    // Here we just tell Rust to use the types we're delegating to.
    // This is just (&'h String, &'h String)
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
            let (mut generic_families, mut generic_families_aliases) =
                Self::generic_font_families_from_ohos_fontconfig(&config);
            generic_families.extend(Self::generate_hardcoded_font_families());
            // generic_families_aliases.extend(Self::generate_apple_system_font_aliases());

            let (fallback_families, _fallback_associations) =
                Self::fallback_font_families_from_ohos_fontconfig(&mut generic_families, &config);

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
                fallback_families_associations: Self::generate_default_fallback_associations(),
            };
        }

        FontList {
            generic_families: Self::detect_installed_font_families(),
            fallback_families: Self::generate_default_fallback_font_families(),
            aliases: Self::generate_default_fallback_font_aliases(),
            fallback_families_associations: Self::generate_default_fallback_associations(),
        }
    }

    fn get_generic_family_font_file_path_from_ohos_fontconfig<'a>(
        family_name: &'a str,
        config: &'a FontconfigOHOS,
    ) -> Option<&'a str> {
        let font_full_name_to_filepath = &config.font_file_map;
        let mut family_name_key = family_name.to_string();

        // Awfull performance. Rewrite this.
        if let Some(res) = font_full_name_to_filepath
            .iter()
            .find(|entry| family_name_key == entry.0)
        {
            return Some(&res.1);
        } else {
            log::warn!("Was unable to find font file with canonicalized naming");
            log::warn!("Will try regular variant");
            family_name_key = family_name.to_string() + " Regular";
            if let Some(res) = font_full_name_to_filepath
                .iter()
                .find(|entry| family_name_key == entry.0)
            {
                return Some(&res.1);
            } else {
                if log::log_enabled!(log::Level::Error) {
                    log::error!(
                        r#"
                        Unable to find fontfile path for family in verified config!
                        Check OHOS fontconfig verification code!
                        family name: {}
                        "#,
                        family_name
                    );
                }
            }
        }
        None
    }

    fn get_all_family_font_file_paths_from_ohos_fontconfig<'a>(
        family_name: &'a str,
        config: &'a FontconfigOHOS,
    ) -> Vec<&'a str> {
        let mut result = Vec::<&'a str>::new();
        let font_full_name_to_filepath = &config.font_file_map;
        for (font_full_name, font_file_path) in font_full_name_to_filepath.iter() {
            if font_full_name.contains(family_name) {
                result.push(font_full_name)
            }
        }
        result
    }

    fn get_family_weight_from_font_variations_entry(variation: &[(String, i32); 2]) -> Option<i32> {
        // Don't forget that font-weight value is expected to be in FixedPoint format.
        // That means that i32 that should be returned actually should store weight representation as float
        if variation[0].0.contains("weight") {
            return Some(variation[0].1);
        }
        if variation[1].0.contains("weight") {
            return Some(variation[1].1);
        }
        if log::log_enabled!(log::Level::Error) {
            log::error!(
                r#"
                Unable to get font weight from font-variations in verified config!
                Check OHOS fontconfig verification code!
            "#
            );
        }
        None
    }

    fn process_generic_family_from_ohos_config(
        generic_font_family: &GenericFontFamilyOHOS,
        config: &FontconfigOHOS,
    ) -> Option<(FontFamily, Vec<FontAlias>)> {
        let family_name = &generic_font_family.family;
        let mut family_fonts = Vec::<Font>::new();

        let res = Self::get_generic_family_font_file_path_from_ohos_fontconfig(family_name, config);
        if res.is_none() {
            return None;
        }
        let filepath = res.unwrap().to_string();

        let font_variations = &generic_font_family.font_variations;
        for variation in font_variations {
            let weight = Self::get_family_weight_from_font_variations_entry(variation);
            family_fonts.push(Font {
                filepath: filepath.clone(),
                weight,
                ..Default::default()
            });
        }
        if font_variations.is_empty() {
            family_fonts.push(Font {
                filepath: filepath.clone(),
                ..Default::default()
            });
        }
        // need to write proper function for font name canonicalization
        // replace to_ascii_lowercase
        let family = FontFamily {
            name: family_name.to_string(),
            fonts: family_fonts,
        };

        let list_of_aliases_in_config = &generic_font_family.alias;
        let family_aliases = list_of_aliases_in_config
            .iter()
            .map(|(alias, weight)| {
                let effective_weight: Option<i32> = match *weight {
                    0 => None,
                    _ => Some(*weight),
                };
                FontAlias {
                    from: alias.to_string(),
                    to: family_name.to_string(),
                    weight: effective_weight,
                }
            })
            .collect();

        Some((family, family_aliases))
    }

    fn generic_font_families_from_ohos_fontconfig(
        config: &FontconfigOHOS,
    ) -> (Vec<FontFamily>, Vec<FontAlias>) {
        let mut result_fonts = Vec::<FontFamily>::new();
        let mut result_aliases = Vec::<FontAlias>::new();
        for generic_font_family in &config.generic {
            // _fallback_name now ohos fontconfig has only one fallback strategy.
            let candidate =
                Self::process_generic_family_from_ohos_config(generic_font_family, config);
            if let Some((generic_family, generic_family_aliases)) = candidate {
                result_fonts.push(generic_family);
                result_aliases.extend(generic_family_aliases);
            }
        }
        (result_fonts, result_aliases)
    }

    fn find_full_name_to_generic_family_name_association<'a>(
        full_name: &'a str,
        generic_families: &'a mut Vec<FontFamily>,
    ) -> Option<&'a mut FontFamily> {
        let mut candidate: Option<&mut FontFamily> = None;
        for font_family in generic_families {
            let family_name = &font_family.name;
            if full_name.contains(family_name) {
                // Process first ever found candidate
                if candidate.is_none() {
                    candidate = Some(font_family);
                    continue;
                }
                // We will return longest candidate
                // Decide between 2 candidates
                if let Some(ref cur_candidate) = candidate {
                    if family_name.len() > cur_candidate.name.len() {
                        candidate = Some(font_family);
                    }
                }
            }
        }
        candidate
    }

    fn process_fallback_list_from_ohos_config(
        fallback_list: &[FallbackEntryOHOS],
        generic_families: &mut Vec<FontFamily>,
        config: &FontconfigOHOS,
    ) -> (Vec<FontFamily>, FallbackAssociations) {
        let mut result_fonts = Vec::<FontFamily>::new();
        let result_fallback_associations = FallbackAssociations::new();
        let mut processed_filepaths = HashSet::<String>::new();

        for fallback_font in fallback_list {
            let mut family_fonts = Vec::<Font>::new();

            let [lang_script, font_family_with_script] = &fallback_font.lang_script_to_family;
            // TODO(ddesyatkin): Save all langscript value to separate global STATIC list
            // then reserve it as system fallback. Need to write function that will translate
            // "lang-script" to UnicodeBlock
            // example: "Hebr" => Some(Script::Hebrew),
            // Script:: inner_from_short_name
            let font_variations = &fallback_font.font_variations;

            // Try to find generic system family that will match with current font_family_with_script
            let mut generic_family_candidate =
                Self::find_full_name_to_generic_family_name_association(
                    font_family_with_script,
                    generic_families,
                );

            if let Some(ref generic_family) = generic_family_candidate {
                generic_family.fonts.iter().for_each(|font| {
                    processed_filepaths.insert(font.filepath.to_string());
                });
            }

            // Get all posible candidates for new system font file paths;
            let res = Self::get_all_family_font_file_paths_from_ohos_fontconfig(
                font_family_with_script,
                config,
            );
            if res.is_empty() {
                continue;
            }

            // Filter paths from all generic (system) families that we was able to process before
            let filepaths: Vec<&str> = res
                .into_iter()
                .filter_map(|filepath| {
                    if processed_filepaths.contains(filepath) {
                        return None;
                    }
                    Some(filepath)
                })
                .collect();
            // let key = FallbackOptionsKey::new_from_lang_script_str(&lang_script);

            for filepath in filepaths {
                if font_variations.is_empty() {
                    family_fonts.push(Font {
                        filepath: filepath.to_string(),
                        ..Default::default()
                    });
                }

                for variation in font_variations {
                    let weight = Self::get_family_weight_from_font_variations_entry(variation);
                    family_fonts.push(Font {
                        filepath: filepath.to_string(),
                        weight,
                        ..Default::default()
                    });
                }
            }

            // Add fallback fonts that corresponds to generic font family into
            // existing font family.
            if let Some(ref mut generic_family) = generic_family_candidate {
                generic_family.fonts.extend(family_fonts);
                continue;
            }

            // If we met some family that doesn't have clear lang_script instructions
            // that family should become default fallback family if we was unable to match against any style that
            // user asked (GenericFontFamily::None)

            // So we should add it to generic system families cause only they are visible through
            // default_system_generic_font_family function
            if lang_script.is_empty() {
                generic_families.push(FontFamily {
                    name: font_family_with_script.to_string(),
                    fonts: family_fonts.clone(),
                });
            }

            // If we was unable to find family in generic families, create new (currently unused)
            // fallback font family.
            result_fonts.push(FontFamily {
                name: font_family_with_script.to_string(),
                fonts: family_fonts,
            });
        }
        (result_fonts, result_fallback_associations)
    }

    fn fallback_font_families_from_ohos_fontconfig(
        generic_families: &mut Vec<FontFamily>,
        config: &FontconfigOHOS,
    ) -> (Vec<FontFamily>, FallbackAssociations) {
        let mut result = Vec::<FontFamily>::new();
        let mut result_associations = FallbackAssociations::new();
        for (_fallback_name, fallback_list) in &config.fallback {
            // _fallback_name now ohos fontconfig has only one fallback strategy.
            let (strategy_families_vec, strategy_families_associations) =
                Self::process_fallback_list_from_ohos_config(
                    &fallback_list,
                    generic_families,
                    config,
                );
            result.extend(strategy_families_vec);
            result_associations.extend(strategy_families_associations)
        }
        (result, result_associations)
    }

    /// Detect available fonts or fallback to a hardcoded list
    fn detect_installed_font_families() -> Vec<FontFamily> {
        let mut families = enumerate_font_files(OHOS_FONTS_DIR)
            .inspect_err(|e| error!("Failed to enumerate font files due to `{e:?}`"))
            .map(|font_files| parse_font_filenames(font_files))
            .unwrap_or_else(|_| Vec::<FontFamily>::new());
        // In case we was unable to parse any fonts, extend with Hardcoded option.
        families.extend(Self::generate_hardcoded_font_families());
        families
    }

    /// A List of hardcoded fonts, added in addition to both detected and fallback font families.
    ///
    /// There are only two emoji fonts, and their filenames are stable, so we just hardcode
    /// their paths, instead of attempting to parse the family name from the filepaths.
    fn generate_hardcoded_font_families() -> Vec<FontFamily> {
        let hardcoded_fonts = vec![
            FontFamily {
                name: "HMOS Color Emoji".to_string(),
                fonts: vec![Font {
                    filepath: FontList::generate_default_font_absolute_path(
                        "HMOSColorEmojiCompat.ttf",
                    ),
                    ..Default::default()
                }],
            },
            FontFamily {
                name: "HMOS Color Emoji Flags".to_string(),
                fonts: vec![Font {
                    filepath: FontList::generate_default_font_absolute_path(
                        "HMOSColorEmojiFlags.ttf",
                    ),
                    ..Default::default()
                }],
            },
        ];
        if log::log_enabled!(log::Level::Warn) {
            for family in hardcoded_fonts.iter() {
                for font in &family.fonts {
                    let path = Path::new(&font.filepath);
                    if !path.exists() {
                        warn!(
                            "Hardcoded Emoji Font {} was not found at `{}`",
                            family.name, font.filepath
                        )
                    }
                }
            }
        }
        hardcoded_fonts
    }

    fn generate_default_fallback_font_families() -> Vec<FontFamily> {
        warn!("Falling back to hardcoded fallback font families...");
        let alternatives = [
            ("HarmonyOS Sans", "HarmonyOS_Sans.ttf"),
            ("HarmonyOS Sans SC", "HarmonyOS_Sans_SC.ttf"),
            ("serif", "NotoSerif[wdth,wght].ttf"),
        ];

        alternatives
            .iter()
            .filter(|item| Path::new(&Self::generate_default_font_absolute_path(item.1)).exists())
            .map(|item| FontFamily {
                name: item.0.into(),
                fonts: vec![Font {
                    filepath: item.1.into(),
                    ..Default::default()
                }],
            })
            .collect()
    }

    fn generate_default_font_absolute_path(filename: &str) -> String {
        if filename.starts_with("/") {
            String::from(filename)
        } else {
            format!("{OHOS_FONTS_DIR}/{filename}")
        }
    }

    fn generate_apple_system_font_aliases() -> Vec<FontAlias> {
        let aliases = vec![
            // Add fallback for -apple-system-font families.
            // For now we will replace them with the fonts that are native to OpenHarmony
            // Should it be here or should we load web-fonts from somewhere? -apple-system-font
            // autogenerate alliases from full list of generic fonts
            // Generic
            FontAlias {
                from: "-apple-system-font".to_string(),
                to: "HarmonyOS Sans".to_string(),
                weight: None,
            },
            FontAlias {
                from: "-apple-system-font".to_string(),
                to: "HarmonyOS Sans Condensed".to_string(),
                weight: None,
            },
            FontAlias {
                from: "-apple-system-font".to_string(),
                to: "HarmonyOS Sans Digit".to_string(),
                weight: None,
            },
            FontAlias {
                from: "-apple-system-font".to_string(),
                to: "Noto Serif".to_string(),
                weight: None,
            },
            FontAlias {
                from: "-apple-system-font".to_string(),
                to: "Noto Sans Mono".to_string(),
                weight: Some(400),
            },
        ];
        aliases
    }

    fn generate_default_fallback_font_aliases() -> Vec<FontAlias> {
        let mut aliases = vec![
            // Note: ideally the aliases should be read from fontconfig.json
            FontAlias {
                from: "serif".to_string(),
                to: "Noto Serif".to_string(),
                weight: None,
            },
            FontAlias {
                from: "sans-serif".to_string(),
                to: "HarmonyOS Sans".to_string(),
                weight: None,
            },
            FontAlias {
                from: "monospace".to_string(),
                to: "Noto Sans Mono".to_string(),
                weight: Some(400),
            },
            FontAlias {
                from: "HarmonyOS-Sans-Condensed".to_string(),
                to: "HarmonyOS Sans Condensed".to_string(),
                weight: None,
            },
            FontAlias {
                from: "HarmonyOS-Sans-Digit".to_string(),
                to: "HarmonyOS Sans Digit".to_string(),
                weight: None,
            },
        ];
        // Add fallback for -apple-system-font families.
        // For now we will replace them with the fonts that are native to OpenHarmony
        // Should it be here or should we load web-fonts from somewhere? -apple-system-font
        // aliases.extend(Self::generate_apple_system_font_aliases());
        aliases
    }

    fn generate_default_fallback_associations() -> FallbackAssociations {
        let mut result_associations = FallbackAssociations::new();

        let key = FallbackOptionsKey::new_from_emoji_presentations_pref(
            EmojiPresentationPreference::Emoji,
        );
        result_associations.add_value_to_set_on_key(key.clone(), "HMOS Color Emoji".to_string());
        result_associations
            .add_value_to_set_on_key(key.clone(), "HMOS Color Emoji Flags".to_string());

        // Block for Chinese
        let key = FallbackOptionsKey::new_from_script(Script::Han);
        result_associations.add_value_to_set_on_key(key.clone(), "HarmonyOS Sans SC".to_string());
        result_associations.add_value_to_set_on_key(key.clone(), "HarmonyOS Sans TC".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Hebrew);
        result_associations.add_value_to_set_on_key(key, "Noto Sans Hebrew".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Arabic);
        result_associations.add_value_to_set_on_key(key, "HarmonyOS Sans Naskh Arabic".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Devanagari);
        result_associations.add_value_to_set_on_key(key, "Noto Sans Devanagari".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Tamil);
        result_associations.add_value_to_set_on_key(key, "Noto Sans Tamil".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Thai);
        result_associations.add_value_to_set_on_key(key, "Noto Sans Thai".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Georgian);
        result_associations.add_value_to_set_on_key(key, "Noto Sans Georgian".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::GeorgianSupplement);
        result_associations.add_value_to_set_on_key(key, "Noto Sans Georgian".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Ethiopic);
        result_associations.add_value_to_set_on_key(key, "Noto Sans Ethiopic".to_string());

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::EthiopicSupplement);
        result_associations.add_value_to_set_on_key(key, "Noto Sans Ethiopic".to_string());

        // TODO(recode structure): FallbackAssociations.
        // If we have several keys. Everything becomes really inconvenient.
        // Should it be Vector with (FallbackOptionsKey, HashSet<String>)?
        // In that case we can use FallbackOptionsKey as a mask nad write custom algorithm
        // This vector is not supposed to be too big also. O(log 327+255+3) <- in worst case
        // If we will sort it before placing it to FONT_LIST storage

        // Block for Hangul
        let hangul_font_set = HashSet::<String>::from([
            "Noto Sans CJK".to_string(),
            "Noto Serif CJK".to_string(),
            "Noto Sans KR".to_string(),
        ]);
        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::HangulCompatibilityJamo);
        result_associations.insert((key, hangul_font_set.clone()));

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::HangulJamo);
        result_associations.insert((key, hangul_font_set.clone()));

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::HangulJamoExtendedA);
        result_associations.insert((key, hangul_font_set.clone()));

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::HangulJamoExtendedB);
        result_associations.insert((key, hangul_font_set.clone()));

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::HangulSyllables);
        result_associations.insert((key, hangul_font_set));

        // Block for Japanese
        let japan_font_set = HashSet::<String>::from([
            "Noto Sans CJK".to_string(),
            "Noto Serif CJK".to_string(),
            "Noto Sans JP".to_string(),
        ]);
        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Hiragana);
        result_associations.insert((key, japan_font_set.clone()));

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::Katakana);
        result_associations.insert((key, japan_font_set.clone()));

        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::KatakanaPhoneticExtensions);
        result_associations.insert((key, japan_font_set));

        // Block for HalfwidthandFullwidthForms
        let key = FallbackOptionsKey::new_from_block(UnicodeBlock::HalfwidthandFullwidthForms);
        result_associations.add_value_to_set_on_key(key.clone(), "HarmonyOS Sans SC".to_string());
        result_associations.add_value_to_set_on_key(key, "Noto Sans CJK".to_string());

        result_associations
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
                log::error!("find_all_suitable_aliases: Not a single alias was associated with {family_name}!");
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

        // Correct template for variable font-weight.
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

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    #[test]
    fn split_noto_font_name_test() {
        use super::split_noto_font_name;
        assert_eq!(
            split_noto_font_name("NotoSansSinhala"),
            vec!["Noto", "Sans", "Sinhala"]
        );
        assert_eq!(
            split_noto_font_name("NotoSansTamilUI"),
            vec!["Noto", "Sans", "Tamil", "UI"]
        );
        assert_eq!(
            split_noto_font_name("NotoSerifCJK"),
            vec!["Noto", "Serif", "CJK"]
        );
    }

    #[test]
    fn test_parse_font_filenames() {
        use super::parse_font_filenames;
        let families = parse_font_filenames(vec![PathBuf::from("NotoSansCJK-Regular.ttc")]);
        assert_eq!(families.len(), 1);
        let family = families.first().unwrap();
        assert_eq!(family.name, "Noto Sans CJK".to_string());

        let families = parse_font_filenames(vec![
            PathBuf::from("NotoSerifGeorgian[wdth,wght].ttf"),
            PathBuf::from("HarmonyOS_Sans_Naskh_Arabic_UI.ttf"),
            PathBuf::from("HarmonyOS_Sans_Condensed.ttf"),
            PathBuf::from("HarmonyOS_Sans_Condensed_Italic.ttf"),
            PathBuf::from("NotoSansDevanagariUI-Bold.ttf"),
            PathBuf::from("NotoSansDevanagariUI-Medium.ttf"),
            PathBuf::from("NotoSansDevanagariUI-Regular.ttf"),
            PathBuf::from("NotoSansDevanagariUI-SemiBold.ttf"),
        ]);
        assert_eq!(families.len(), 4);
    }

    #[test]
    fn test_parse_noto_sans_phags_pa() {
        use super::parse_font_filenames;

        let families = parse_font_filenames(vec![PathBuf::from("NotoSansPhags-Pa-Regular.ttf")]);
        let family = families.first().unwrap();
        assert_eq!(family.name, "Noto Sans Phags Pa");
    }

    #[test]
    fn test_old_noto_sans() {
        use super::parse_font_filenames;

        let families = parse_font_filenames(vec![
            PathBuf::from("NotoSans_JP_Regular.otf"),
            PathBuf::from("NotoSans_KR_Regular.otf"),
            PathBuf::from("NotoSans_JP_Bold.otf"),
        ]);
        assert_eq!(families.len(), 2, "actual families: {families:?}");
        let first_family = families.first().unwrap();
        let second_family = families.last().unwrap();
        // We don't have a requirement on the order of the family names,
        // we just want to test existence.
        let names = [first_family.name.as_str(), second_family.name.as_str()];
        assert!(names.contains(&"Noto Sans JP"));
        assert!(names.contains(&"Noto Sans KR"));
    }

    #[test]
    fn print_detected_families() {
        let list = super::FontList::detect_installed_font_families();
        println!("The fallback FontList is: {list:?}");
    }
}
