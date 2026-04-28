/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fs, io};

use log::{debug, error, warn};
use read_fonts::FileRef::{Collection, Font as OHOS_Font};
use read_fonts::{FileRef, FontRef, TableProvider};
use servo_base::text::{UnicodeBlock, UnicodeBlockMethod};
use style::Atom;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};
use unicode_script::Script;

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
    for elem in fs::read_dir(OHOS_FONTS_DIR)?.flatten() {
        if elem.file_type().is_ok_and(|file_type| file_type.is_file()) {
            let name = elem.file_name();
            let raw_name = name.as_bytes();
            if raw_name.ends_with(b".ttf".as_ref()) || raw_name.ends_with(b".ttc".as_ref()) {
                debug!(
                    "Found font: {}",
                    String::from_utf8_lossy(elem.file_name().as_bytes())
                );
                font_list.push(elem.path())
            }
        }
    }
    Ok(font_list)
}

fn detect_hos_font_style(font: &FontRef, file_path: &str) -> Option<String> {
    // This implementation uses the postscript (post) table, which is one of the mandatory tables
    // according to TrueType's reference manual (https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html).
    // Therefore, raise an error if Fontations fails to read this table for some reason.

    // If angle is 0, then the font style is normal. otherwise, italic.
    if font
        .post()
        .unwrap_or_else(|_| {
            panic!("Failed to read {:?}'s postscript table!", file_path);
        })
        .italic_angle() !=
        (0 as i32).into()
    {
        Some("italic".to_string())
    } else {
        None
    }
}

fn detect_hos_font_weight_alias(font: &FontRef) -> Option<i32> {
    // According to TrueType's reference manual (https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html),
    // os2 is an optional table. Therefore, if Fontations fails to read this table, we don't treat this as an error
    // and we simply return `None`.
    match font.os2() {
        Ok(result) => Some(result.us_weight_class() as i32),
        Err(_) => None,
    }
}

fn detect_hos_font_width(font: &FontRef) -> FontWidth {
    // According to TrueType's reference manual (https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html),
    // os2 is an optional table. Therefore, if Fontations fails to read this table, we don't treat this as an error
    // and we simply return `FontWidth::Normal` as a default.
    match font.os2() {
        Ok(result) => {
            let font_width = result.us_width_class().clone();
            // According to https://learn.microsoft.com/en-us/typography/opentype/spec/os2#uswidthclass,
            // value between 1 & 4 inclusive represents condensed type.
            if font_width >= 1 && font_width <= 4 {
                FontWidth::Condensed
            } else {
                FontWidth::Normal
            }
        },
        Err(_) => FontWidth::Normal,
    }
}

/// This function generates list of `FontFamily` based on font files with the extension `.otf`, `.ttc`, or `.otf`.
/// If a font file's extension is .ttc, then all the font within it will be processed one by one.
#[servo_tracing::instrument(skip_all)]
fn get_system_font_families(font_files: Vec<PathBuf>) -> Vec<FontFamily> {
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

    let mut all_families = Vec::new();

    for font_file in font_files.iter() {
        let Ok(font_bytes) =
            File::open(font_file).and_then(|file| unsafe { memmap2::Mmap::map(&file) })
        else {
            continue;
        };
        let Ok(file_ref) = FileRef::new(&font_bytes) else {
            continue;
        };

        match file_ref {
            OHOS_Font(font) => {
                if let Some(result) = get_family_name_and_generate_font_struct(&font, &font_file) {
                    all_families.push(result);
                }
            },
            Collection(font_collection) => {
                // Process all the font files within the collection one by one.
                for f in font_collection.iter() {
                    if let Some(result) =
                        get_family_name_and_generate_font_struct(&(f.unwrap()), &font_file)
                    {
                        all_families.push(result);
                    };
                }
            },
        }
    }

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

fn get_family_name_and_generate_font_struct(
    font_ref: &FontRef,
    file_path: &PathBuf,
) -> Option<(String, Font)> {
    // Parse the file path to string. If this fails, then skip this font.
    let Some(file_path_string_slice) = file_path.to_str() else {
        return None;
    };
    let file_path_str = file_path_string_slice.to_string();

    // Obtain the font's styling
    let style = detect_hos_font_style(font_ref, file_path_string_slice);
    let weight = detect_hos_font_weight_alias(font_ref);
    let width = detect_hos_font_width(font_ref);

    // Get the family name via the name table. According to TrueType's reference manual (https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html),
    // the name table is a mandatory table. Therefore, if Fontations fails to read this table for whatever reason, return `None` to skip this font altogether.
    let Ok(font_name_table) = font_ref.name() else {
        return None;
    };
    let Some(family_name) = font_name_table
        .name_record()
        .iter()
        .filter(|record| record.name_id().to_u16() == 1) // According to the reference manual, name identifier code (nameID) `1` is the font family name.
        .find_map(|record| {
            record
                .string(font_name_table.string_data())
                .ok()
                .map(|s| s.to_string())
        })
    else {
        return None;
    };

    let font = Font {
        filepath: file_path_str,
        weight,
        style,
        width,
    };
    Some((family_name, font))
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
            .map(get_system_font_families)
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

// Functions used by SystemFontService
pub(crate) fn for_each_available_family<F>(mut callback: F)
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

pub(crate) fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(FontTemplate),
{
    let mut produce_font = |font: &Font| {
        let local_font_identifier = LocalFontIdentifier {
            path: Atom::from(font.filepath.clone()),
            face_index: 0,
            named_instance_index: 0,
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
        callback(FontTemplate::new(
            FontIdentifier::Local(local_font_identifier),
            descriptor,
            None,
            None,
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
                families.push("Noto Sans CJK KR");
                families.push("Noto Sans Mono CJK KR");
                families.push("Noto Serif CJK KR");
                families.push("Noto Sans KR");
            },
            UnicodeBlock::Hiragana |
            UnicodeBlock::Katakana |
            UnicodeBlock::KatakanaPhoneticExtensions => {
                families.push("Noto Sans CJK JP");
                families.push("Noto Sans Mono CJK JP");
                families.push("Noto Serif CJK JP");
                families.push("Noto Sans JP");
            },
            UnicodeBlock::HalfwidthandFullwidthForms => {
                families.push("HarmonyOS Sans SC");
                families.push("Noto Sans CJK SC");
                families.push("Noto Sans Mono CJK SC");
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

pub(crate) fn default_system_generic_font_family(
    generic: GenericFontFamily,
) -> LowercaseFontFamilyName {
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
    fn test_get_system_font_families() {
        use super::get_system_font_families;
        let families = get_system_font_families(vec![PathBuf::from("NotoSansCJK-Regular.ttc")]);
        assert_eq!(families.len(), 1);
        let family = families.first().unwrap();
        assert_eq!(family.name, "Noto Sans CJK".to_string());

        let families = get_system_font_families(vec![
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
    fn print_detected_families() {
        let list = super::FontList::detect_installed_font_families();
        println!("The fallback FontList is: {list:?}");
    }
}
