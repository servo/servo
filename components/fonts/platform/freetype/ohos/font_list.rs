/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::ffi::OsStr;
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

#[derive(Debug, Default)]
struct Font {
    // `LocalFontIdentifier` uses `Atom` for string interning and requires a String or str, so we
    // already require a String here, instead of using a PathBuf.
    filepath: String,
    weight: Option<i32>,
    style: Option<String>,
    width: FontWidth,
}

#[derive(Debug)]
struct FontFamily {
    name: String,
    fonts: Vec<Font>,
}

struct FontAlias {
    from: String,
    to: String,
    weight: Option<i32>,
}

struct FontList {
    families: Vec<FontFamily>,
    aliases: Vec<FontAlias>,
}

pub fn enumerate_font_files(dir_path: &str) -> io::Result<Vec<PathBuf>> {
    let mut font_list = vec![];
    for elem in fs::read_dir(dir_path)?.flatten() {
        if elem.file_type().unwrap().is_file() || elem.file_type().unwrap().is_symlink() {
            let name = elem.file_name();
            let raw_name = name.as_bytes();
            if raw_name.ends_with(b".ttf".as_ref()) || raw_name.ends_with(b".ttc".as_ref()) {
                debug!("Found font {}", elem.file_name().to_str().unwrap());
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

impl FontList {
    fn new() -> FontList {
        // We can not verify correctness of ohos fontconfig without reading folders that
        // contain device fonts; So if we found them, and config was correct we return
        // them together.
        if let Some((config, _font_paths)) = json::load_and_verify_ohos_fontconfig() {
            let (mut generic_families, mut generic_families_aliases) = Self::generic_font_families_from_ohos_fontconfig(&config);
            // We don't have separate storage for fallbacks
            generic_families.extend(Self::fallback_font_families_from_ohos_fontconfig(&config));
            generic_families.extend(Self::fallback_font_families());
            generic_families.extend(Self::hardcoded_font_families());
            generic_families_aliases.extend(Self::fallback_font_aliases());
            return FontList {
                families: generic_families,
                aliases: generic_families_aliases,
            }
        }

        // Fallback strategy was not updated when fontconfig parsing was added
        // to codebase.
        FontList {
            families: Self::detect_installed_font_families(),
            aliases: Self::fallback_font_aliases(),
        }
    }

    /// Detect available fonts or fallback to a hardcoded list
    fn detect_installed_font_families() -> Vec<FontFamily> {
        let mut families = enumerate_font_files(OHOS_FONTS_DIR)
            .inspect_err(|e| error!("Failed to enumerate font files due to `{e:?}`"))
            .map(|font_files| parse_font_filenames(font_files))
            .unwrap_or_else(|_| FontList::fallback_font_families());
        families.extend(Self::hardcoded_font_families());
        families
    }

    /// A List of hardcoded fonts, added in addition to both detected and fallback font families.
    ///
    /// There are only two emoji fonts, and their filenames are stable, so we just hardcode
    /// their paths, instead of attempting to parse the family name from the filepaths.
    fn hardcoded_font_families() -> Vec<FontFamily> {
        let hardcoded_fonts = vec![
            FontFamily {
                name: "HMOS Color Emoji".to_string(),
                fonts: vec![Font {
                    filepath: FontList::font_absolute_path("HMOSColorEmojiCompat.ttf"),
                    ..Default::default()
                }],
            },
            FontFamily {
                name: "HMOS Color Emoji Flags".to_string(),
                fonts: vec![Font {
                    filepath: FontList::font_absolute_path("HMOSColorEmojiFlags.ttf"),
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

    fn get_family_path_from_ohos_fontconfig(family_name: &str, config: &FontconfigOHOS) -> Option<String> {
        let font_family_to_filepath = &config.font_file_map;
        let mut family_name_key = family_name.to_string() + " Regular";
        // TODO (ddesyatkin):
        // Need separate function to try all variants
        // Regular Bold Light Italic Digit Condensed SemiBold "UI Bold" etc.

        // Fonts in fallback notation differs from notation in font_file_map sometimes " Regular" must be added
        // Canonicalization of the name?
        if !font_family_to_filepath.contains_key(&family_name_key) {
            family_name_key = family_name.to_string();
            if !font_family_to_filepath.contains_key(&family_name.to_string()) {
                log::error!(r#"
                    Unable to find fontfile path for family in verified config!
                    Check OHOS fontconfig verification code!
                    family name: {}
                    "#, family_name);
                return None;
            }
        }
        Some(font_family_to_filepath[&family_name_key].clone())
    }

    fn get_family_weight_from_font_variations_entry(variation: &[(String, i32); 2] ) -> Option<i32> {
        // Don't forget that font-weight value is expected to be in FixedPoint format.
        // That means that i32 that should be returned actually should store weight representation as float
        if variation[0].0.contains("weight") {
            return Some(variation[0].1);
        }
        if variation[1].0.contains("weight") {
            return Some(variation[1].1);
        }
        log::error!(r#"
            Unable to get font weight from font-variations in verified config!
            Check OHOS fontconfig verification code!
        "#);
        None
    }

    fn process_generic_family_from_ohos_config(generic_font_family: &GenericFontFamilyOHOS, config: &FontconfigOHOS) -> Option<(FontFamily, Vec<FontAlias>)> {
        let family_name = &generic_font_family.family;
        let mut family_fonts = Vec::<Font>::new();

        let res = Self::get_family_path_from_ohos_fontconfig(family_name, config);
        if res.is_none() {
            return None;
        }
        let filepath = res.unwrap();

        let font_variations = &generic_font_family.font_variations;
        for variation in font_variations {
            let weight = Self::get_family_weight_from_font_variations_entry(variation);
            family_fonts.push(
                Font {
                    filepath: filepath.clone(),
                    weight,
                    ..Default::default()
                }
            );
        }
        let family = FontFamily {
            name: family_name.to_string(),
            fonts: family_fonts
        };

        let list_of_aliases_in_config = &generic_font_family.alias;
        let family_aliases = list_of_aliases_in_config
            .iter()
            .map(|(alias, weight)|{
                let effective_weight: Option<i32> = match *weight {
                    0 => None,
                    _ => Some(*weight)
                };
                FontAlias {
                    from: alias.to_string(),
                    to: family_name.to_string(),
                    weight: effective_weight
                }
            })
            .collect();


        Some((family, family_aliases))
    }

    fn generic_font_families_from_ohos_fontconfig(config: &FontconfigOHOS) -> (Vec<FontFamily>, Vec<FontAlias>) {
        let mut result_fonts = Vec::<FontFamily>::new();
        let mut result_aliases = Vec::<FontAlias>::new();
        for generic_font_family in &config.generic {
            // _fallback_name now ohos fontconfig has only one fallback strategy.
            let candidate = Self::process_generic_family_from_ohos_config(generic_font_family, config);
            if let Some((generic_family, generic_family_aliases)) = candidate {
                result_fonts.push(generic_family);
                result_aliases.extend(generic_family_aliases);
            }
        }
        (result_fonts, result_aliases)
    }

    fn process_fallback_list_from_ohos_config(fallback_list: &Vec<FallbackEntryOHOS>, config: &FontconfigOHOS) -> Vec::<FontFamily> {
        let mut result_fonts = Vec::<FontFamily>::new();
        for fallback_font in fallback_list {
            let mut family_fonts =  Vec::<Font>::new();
            // Lang script as HashMap really inconvenient change to Pair!
            let entry = fallback_font.lang_script.iter().next();
            if entry.is_none() {
                continue;
            }
            // TODO(ddesyatkin): Save all langscript value to separate global STATIC list
            // Then freserfe it as system fallback
            // Need to write function that will translate
            // "lang-script" to UnicodeBlock
            // example: "Hebr" => Some(Script::Hebrew),
            // Script:: inner_from_short_name
            let (_lang_script, family_name) = entry.unwrap();
            let font_variations = &fallback_font.font_variations;

            let res = Self::get_family_path_from_ohos_fontconfig(family_name, config);
            if res.is_none() {
                continue;
            }
            let filepath = res.unwrap();

            if font_variations.is_empty() {
                family_fonts.push(
                    Font {
                        filepath,
                        ..Default::default()
                    }
                );
                continue;
            }
            for variation in font_variations {
                let weight = Self::get_family_weight_from_font_variations_entry(variation);
                family_fonts.push(
                    Font {
                        filepath: filepath.clone(),
                        weight,
                        ..Default::default()
                    }
                );
            }


            result_fonts.push(FontFamily{
                name: family_name.to_string(),
                fonts: family_fonts
            });
        }
        result_fonts
    }

    fn fallback_font_families_from_ohos_fontconfig(config: &FontconfigOHOS) -> Vec<FontFamily> {
        let mut result = Vec::<FontFamily>::new();
        for (_fallback_name, fallback_list) in &config.fallback {
            // _fallback_name now ohos fontconfig has only one fallback strategy.
            result.extend(Self::process_fallback_list_from_ohos_config(&fallback_list, config));
        }
        result
    }

    fn fallback_font_families() -> Vec<FontFamily> {
        warn!("Falling back to hardcoded fallback font families...");
        let alternatives = [
            ("HarmonyOS Sans", "HarmonyOS_Sans.ttf"),
            ("HarmonyOS Sans SC", "HarmonyOS_Sans_SC.ttf"),
            ("serif", "NotoSerif[wdth,wght].ttf"),
        ];

        alternatives
            .iter()
            .filter(|item| Path::new(&Self::font_absolute_path(item.1)).exists())
            .map(|item| FontFamily {
                name: item.0.into(),
                fonts: vec![Font {
                    filepath: item.1.into(),
                    ..Default::default()
                }],
            })
            .collect()
    }

    fn font_absolute_path(filename: &str) -> String {
        if filename.starts_with("/") {
            String::from(filename)
        } else {
            format!("{OHOS_FONTS_DIR}/{filename}")
        }
    }

    fn fallback_font_aliases() -> Vec<FontAlias> {
        let aliases = vec![
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
            FontAlias {
                from: "microsoft yahei".to_string(),
                to: "Noto Serif".to_string(),
                weight: None,
            },
            FontAlias {
                from: "microsoft yahei".to_string(),
                to: "HarmonyOS Sans".to_string(),
                weight: None,
            },
            FontAlias {
                from: "microsoft yahei".to_string(),
                to: "Noto Sans Mono".to_string(),
                weight: Some(400),
            },
            FontAlias {
                from: "microsoft yahei".to_string(),
                to: "HarmonyOS Sans Condensed".to_string(),
                weight: None,
            },
            FontAlias {
                from: "microsoft yahei".to_string(),
                to: "HarmonyOS Sans Digit".to_string(),
                weight: None,
            },
        ];
        aliases
    }

    fn find_family(&self, name: &str) -> Option<&FontFamily> {
        self.families
            .iter()
            .find(|family| family.name.eq_ignore_ascii_case(name))
    }

    fn find_first_suitable_alias(&self, family_name: &str) -> Option<&FontAlias> {
        self.aliases
            .iter()
            .find(|alias| alias.from.eq_ignore_ascii_case(family_name))
    }

    fn find_all_suitable_aliases(&self, family_name: &str) -> Option<Vec<&FontAlias>> {
        let mut result = Vec::<&FontAlias>::new();
        result.extend(self.aliases
            .iter()
            .filter_map(|alias| {
                if alias.from.eq_ignore_ascii_case(family_name) {
                    Some(alias)
                } else {
                    None
                }
            })
        );
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

// Functions used by SystemFontService
pub fn for_each_available_family<F>(mut callback: F)
where
    F: FnMut(String),
{
    for family in &FONT_LIST.families {
        callback(family.name.clone());
    }
    for alias in &FONT_LIST.aliases {
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
        // let test = FontTemplateDescriptor {
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
        log::warn!("Was able to find alias for queried family name: {}", family_name);
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
    // We should populate some static vector and provide it here in case of config parsing
    // scenario
    let mut families = vec![];

    if options.presentation_preference == EmojiPresentationPreference::Emoji {
        families.push("HMOS Color Emoji");
        families.push("HMOS Color Emoji Flags");
    }

    if Script::from(options.character) == Script::Han {
        families.push("HarmonyOS Sans SC");
        families.push("HarmonyOS Sans TC");
    }

    if let Some(block) = options.character.block() {
        match block {
            UnicodeBlock::Hebrew => {
                families.push("Noto Sans Hebrew");
            },

            UnicodeBlock::Arabic => {
                families.push("HarmonyOS Sans Naskh Arabic");
            },

            UnicodeBlock::Devanagari => {
                families.push("Noto Sans Devanagari");
            },

            UnicodeBlock::Tamil => {
                families.push("Noto Sans Tamil");
            },

            UnicodeBlock::Thai => {
                families.push("Noto Sans Thai");
            },

            UnicodeBlock::Georgian | UnicodeBlock::GeorgianSupplement => {
                families.push("Noto Sans Georgian");
            },

            UnicodeBlock::Ethiopic | UnicodeBlock::EthiopicSupplement => {
                families.push("Noto Sans Ethiopic");
            },
            UnicodeBlock::HangulCompatibilityJamo |
            UnicodeBlock::HangulJamo |
            UnicodeBlock::HangulJamoExtendedA |
            UnicodeBlock::HangulJamoExtendedB |
            UnicodeBlock::HangulSyllables => {
                families.push("Noto Sans CJK");
                families.push("Noto Serif CJK");
                families.push("Noto Sans KR");
            },
            UnicodeBlock::Hiragana |
            UnicodeBlock::Katakana |
            UnicodeBlock::KatakanaPhoneticExtensions => {
                families.push("Noto Sans CJK");
                families.push("Noto Serif CJK");
                families.push("Noto Sans JP");
            },
            UnicodeBlock::HalfwidthandFullwidthForms => {
                families.push("HarmonyOS Sans SC");
                families.push("Noto Sans CJK");
            },
            _ => {},
        }
    }
    // All fonts that may be returned in default_system_generic_font_family
    // must be specified here
    families.push("Noto Sans Mono"); // if we parse config this is "monospace" alias
    families.push("Noto Serif");
    families.push("HarmonyOS Sans");
    families.push("HarmonyOS Sans Condensed");
    families.push("HarmonyOS Sans Digit");
    families.push("Noto Sans");
    families.push("Noto Sans Symbols");
    families.push("Noto Sans Symbols 2");
    families
}

pub fn default_system_generic_font_family(generic: GenericFontFamily) -> LowercaseFontFamilyName {
    let default_family_name = "HarmonyOS Sans".into();
    match generic {
        GenericFontFamily::Monospace => {
            // sans-serif
            if let Some(alias) = FONT_LIST.find_first_suitable_alias("monospace") {
                alias.from.clone().into()
            } else {
                default_family_name
            }
        },
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
