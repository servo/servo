/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use base::text::UnicodeBlock;
// Proper locale handling
use icu_locid::subtags::script;
use icu_locid::LanguageIdentifier;
use log::{error, warn};

use crate::EmojiPresentationPreference;
use crate::platform::font_list::{
    FallbackAssociations, FallbackOptionsKey, FontAlias, FontFamily, FontWidth, OHOS_FONTS_DIR,
    OpenHarmonyFontDescriptor, enumerate_font_files,
};

/* Functions bellow is used when we failed to parse fontconfig.json of
 * OpenHarmony by any reason. This is just hardcoded families. With
 * hardcoded font paths. In the future we want to delete this file
 * as soon as config parsing will become reliable enough
 */

// TODO(ddesyatkin): After finalizing config version rewrite fallback so they produce simmilar behaviour
// on API Level 12

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

fn generate_default_font_absolute_path(filename: &str) -> String {
    if filename.starts_with("/") {
        String::from(filename)
    } else {
        format!("{OHOS_FONTS_DIR}/{filename}")
    }
}

/// Parse the font file names to determine the available FontFamilies
///
/// Note: For OH 5.0+ this function is intended to only be a fallback path, if parsing the
/// `fontconfig.json` fails for some reason. Beta 1 of OH 5.0 still has a bug in the fontconfig.json
/// though, so the "normal path" is currently unimplemented.
pub fn parse_font_filenames(font_files: Vec<PathBuf>) -> Vec<FontFamily> {
    let harmonyos_prefix = "HarmonyOS_Sans";

    let weight_aliases = ["Light", "Regular", "Medium", "Bold"];
    let style_modifiers = ["Italic"];
    let width_modifiers = ["Condensed"];

    let mut families: HashMap<String, Vec<OpenHarmonyFontDescriptor>> = HashMap::new();

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

    let harmony_os_fonts = font_files.iter().flat_map(|file_path| {
        let mut res = Vec::<(String, OpenHarmonyFontDescriptor)>::new();
        let stem = match file_path.file_stem() {
            Some(data) => data,
            _ => return res,
        };

        let stem = match stem.to_str() {
            Some(data) => data,
            _ => return res,
        };

        let stem_no_prefix = match stem.strip_prefix(harmonyos_prefix) {
            Some(data) => data,
            _ => return res,
        };

        let name_components: Vec<&str> = stem_no_prefix.split('_').collect();
        let style = detect_hos_font_style(&name_components);
        let _weight = detect_hos_font_weight_alias(&name_components);
        let width = detect_hos_font_width(&name_components);

        let mut name_components = name_components;
        // If we remove all the modifiers, we are left with the family name
        name_components.retain(|component| {
            !weight_aliases.contains(component) &&
                !style_modifiers.contains(component) &&
                !width_modifiers.contains(component) &&
                !component.is_empty()
        });
        let weights = [100, 200, 300, 400, 500, 600, 700, 800, 900];
        name_components.insert(0, "HarmonyOS Sans");
        let family_name = name_components.join(" ");
        let filepath = match file_path.to_str() {
            Some(data) => data,
            _ => return res,
        };
        for weight in weights {
            let font = OpenHarmonyFontDescriptor {
                filepath: filepath.to_string(),
                weight: Some(weight),
                style: style.clone(),
                width: width.clone(),
                language: LanguageIdentifier::default(),
                unicode_range: None,
            };
            res.push((family_name.clone(), font));
        }
        res
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
        let font = OpenHarmonyFontDescriptor {
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

/// Detect available fonts or fallback to a hardcoded list
pub fn detect_installed_font_families() -> Vec<FontFamily> {
    let mut families = enumerate_font_files(OHOS_FONTS_DIR)
        .inspect_err(|e| error!("Failed to enumerate font files due to `{e:?}`"))
        .map(|font_files| parse_font_filenames(font_files))
        .unwrap_or_else(|_| Vec::<FontFamily>::new());
    // In case we was unable to parse any fonts, extend with Hardcoded option.
    families.extend(generate_hardcoded_font_families());
    families
}

pub fn generate_default_fallback_font_families() -> Vec<FontFamily> {
    warn!("Falling back to hardcoded fallback font families...");
    // All families here are dynamic;
    let weights = [100, 200, 300, 400, 500, 600, 700, 800, 900];

    let alternatives = [
        ("HarmonyOS Sans", "HarmonyOS_Sans.ttf"),
        ("HarmonyOS Sans SC", "HarmonyOS_Sans_SC.ttf"),
        ("serif", "NotoSerif[wdth,wght].ttf"),
    ];

    alternatives
        .iter()
        .filter(|item| Path::new(&generate_default_font_absolute_path(item.1)).exists())
        .map(|item| {
            let mut fonts = Vec::<OpenHarmonyFontDescriptor>::new();
            for weight in weights {
                fonts.push(OpenHarmonyFontDescriptor {
                    filepath: item.1.into(),
                    weight: Some(weight),
                    ..Default::default()
                });
            }
            FontFamily {
                name: item.0.into(),
                fonts: fonts,
            }
        })
        .collect()
}

pub fn generate_default_fallback_font_aliases() -> Vec<FontAlias> {
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
    ];
    aliases
}

pub fn generate_default_fallback_associations() -> FallbackAssociations {
    let mut result_associations = FallbackAssociations::new();

    let key =
        FallbackOptionsKey::new_from_emoji_presentations_pref(EmojiPresentationPreference::Emoji);
    result_associations.add_value_to_set_on_key(key.clone(), "HMOS Color Emoji".to_string());
    result_associations.add_value_to_set_on_key(key.clone(), "HMOS Color Emoji Flags".to_string());

    // Block for Chinese
    let key = FallbackOptionsKey::new_from_script(script!("Hans").into());
    result_associations.add_value_to_set_on_key(key.clone(), "HarmonyOS Sans SC".to_string());
    let key = FallbackOptionsKey::new_from_script(script!("Hant").into());
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

/// A List of hardcoded fonts, added in addition to both detected and fallback font families.
///
/// There are only two emoji fonts, and their filenames are stable, so we just hardcode
/// their paths, instead of attempting to parse the family name from the filepaths.
pub fn generate_hardcoded_font_families() -> Vec<FontFamily> {
    let hardcoded_fonts = vec![
        FontFamily {
            name: "HMOS Color Emoji".to_string(),
            fonts: vec![OpenHarmonyFontDescriptor {
                filepath: generate_default_font_absolute_path("HMOSColorEmojiCompat.ttf"),
                ..Default::default()
            }],
        },
        FontFamily {
            name: "HMOS Color Emoji Flags".to_string(),
            fonts: vec![OpenHarmonyFontDescriptor {
                filepath: generate_default_font_absolute_path("HMOSColorEmojiFlags.ttf"),
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
                        "Hardcoded Emoji OpenHarmonyFontDescriptor {} was not found at `{}`",
                        family.name, font.filepath
                    )
                }
            }
        }
    }
    hardcoded_fonts
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    #[test]
    pub fn split_noto_font_name_test() {
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
    pub fn test_parse_font_filenames() {
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
