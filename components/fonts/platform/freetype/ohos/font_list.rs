/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

use base::text::{UnicodeBlock, UnicodeBlockMethod};
use log::{debug, error, warn};
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};
use style::Atom;
use unicode_script::Script;

use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontTemplate,
    FontTemplateDescriptor, LowercaseFontFamilyName,
};

lazy_static::lazy_static! {
    static ref FONT_LIST: FontList = FontList::new();
}

/// When testing the ohos font code on linux, we can pass the fonts directory of the SDK
/// via an environment variable.
#[cfg(ohos_mock)]
static OHOS_FONTS_DIR: &'static str = env!("OHOS_SDK_FONTS_DIR");

/// On OpenHarmony devices the fonts are always located here.
#[cfg(not(ohos_mock))]
static OHOS_FONTS_DIR: &'static str = "/system/fonts";

/// An identifier for a local font on OpenHarmony systems.
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    /// The path to the font.
    pub path: Atom,
}

impl LocalFontIdentifier {
    pub(crate) fn index(&self) -> u32 {
        0
    }

    pub(crate) fn read_data_from_file(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        File::open(Path::new(&*self.path))
            .expect("Couldn't open font file!")
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    }
}

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

fn enumerate_font_files() -> io::Result<Vec<PathBuf>> {
    let mut font_list = vec![];
    for elem in fs::read_dir(OHOS_FONTS_DIR)? {
        if let Ok(e) = elem {
            if e.file_type().unwrap().is_file() {
                let name = e.file_name();
                let raw_name = name.as_bytes();
                if raw_name.ends_with(b".ttf".as_ref()) || raw_name.ends_with(b".ttc".as_ref()) {
                    debug!("Found font {}", e.file_name().to_str().unwrap());
                    font_list.push(e.path())
                }
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
    if current_word.len() > 0 {
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

    let families = families
        .into_iter()
        .map(|(name, fonts)| FontFamily { name, fonts })
        .collect();
    families
}

impl FontList {
    fn new() -> FontList {
        FontList {
            families: Self::detect_installed_font_families(),
            aliases: Self::fallback_font_aliases(),
        }
    }

    /// Detect available fonts or fallback to a hardcoded list
    fn detect_installed_font_families() -> Vec<FontFamily> {
        let mut families = enumerate_font_files()
            .inspect_err(|e| error!("Failed to enumerate font files due to `{e:?}`"))
            .and_then(|font_files| Ok(parse_font_filenames(font_files)))
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
                    filepath: FontList::font_absolute_path("HMOSColorEmojiCompat.ttf".into()),
                    ..Default::default()
                }],
            },
            FontFamily {
                name: "HMOS Color Emoji Flags".to_string(),
                fonts: vec![Font {
                    filepath: FontList::font_absolute_path("HMOSColorEmojiFlags.ttf".into()),
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
        ];
        aliases
    }

    fn find_family(&self, name: &str) -> Option<&FontFamily> {
        self.families
            .iter()
            .find(|family| family.name.eq_ignore_ascii_case(name))
    }

    fn find_alias(&self, name: &str) -> Option<&FontAlias> {
        self.aliases
            .iter()
            .find(|family| family.from.eq_ignore_ascii_case(name))
    }
}

// Functions used by FontCacheThread
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
    let mut produce_font = |font: &Font| {
        let local_font_identifier = LocalFontIdentifier {
            path: Atom::from(font.filepath.clone()),
        };
        let stretch = font.width.into();
        let weight = font
            .weight
            .map(|w| StyleFontWeight::from_float(w as f32))
            .unwrap_or(StyleFontWeight::NORMAL);
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
        let descriptor = FontTemplateDescriptor::new(weight, stretch, style);
        callback(FontTemplate::new_for_local_font(
            local_font_identifier,
            descriptor,
        ));
    };

    if let Some(family) = FONT_LIST.find_family(family_name) {
        for font in &family.fonts {
            produce_font(font);
        }
        return;
    }

    if let Some(alias) = FONT_LIST.find_alias(family_name) {
        if let Some(family) = FONT_LIST.find_family(&alias.to) {
            for font in &family.fonts {
                match (alias.weight, font.weight) {
                    (None, _) => produce_font(font),
                    (Some(w1), Some(w2)) if w1 == w2 => produce_font(font),
                    _ => {},
                }
            }
        }
    }
}

// Based on fonts present in OpenHarmony.
pub fn fallback_font_families(options: FallbackFontSelectionOptions) -> Vec<&'static str> {
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
                families.push("Noto Sans KR");
            },
            UnicodeBlock::Hiragana |
            UnicodeBlock::Katakana |
            UnicodeBlock::KatakanaPhoneticExtensions => {
                families.push("Noto Sans JP");
            },
            _ => {},
        }
    }

    families.push("HarmonyOS Sans");
    families.push("Noto Sans");
    families.push("Noto Sans Symbols");
    families.push("Noto Sans Symbols 2");
    families
}

pub fn default_system_generic_font_family(generic: GenericFontFamily) -> LowercaseFontFamilyName {
    let default_font = "HarmonyOS Sans".into();
    match generic {
        GenericFontFamily::Monospace => {
            if let Some(alias) = FONT_LIST.find_alias("monospace") {
                alias.from.clone().into()
            } else {
                default_font
            }
        },
        _ => default_font,
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
