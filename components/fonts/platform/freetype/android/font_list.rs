/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fs::File;
use std::sync::LazyLock;

use base::text::{UnicodeBlock, UnicodeBlockMethod, is_cjk};
use log::warn;
use ndk::font::SystemFontIterator;
use read_fonts::{FileRef, FontRef, TableProvider};
use style::Atom;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};

use crate::{
    FallbackFontSelectionOptions, FontIdentifier, FontTemplate, FontTemplateDescriptor,
    LocalFontIdentifier, LowercaseFontFamilyName,
};

static FONT_LIST: LazyLock<FontList> = LazyLock::new(FontList::new);

struct Font {
    filename: String,
    weight: Option<i32>,
    style: Option<String>,
}

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

impl FontList {
    fn new() -> FontList {
        let mut font_family_hashmap: HashMap<String, Vec<Font>> = HashMap::new();
        let system_font_iterator =
            SystemFontIterator::new().expect("Failed to create SystemFontIterator");
        let mut font_families_vector = Vec::new();

        for system_font in system_font_iterator {
            // Obtain the font file
            let font_bytes = File::open(system_font.path())
                .and_then(|file| unsafe { memmap2::Mmap::map(&file) })
                .unwrap();

            // Read the font file
            let font = FileRef::new(&font_bytes).expect("Font file cannot be read.");
            match font {
                FileRef::Font(f) => {
                    // Case 1: This means it's a .ttf or .otf file.
                    // Get the name table
                    let name_table = f
                        .name()
                        .expect("Font file is corrupted as it has no name table!");
                    let family_name = name_table
                        .name_record()
                        .iter()
                        .filter(|record| record.name_id().to_u16() == 1)
                        .find_map(|record| {
                            record
                                .string(name_table.string_data())
                                .ok()
                                .map(|s| s.to_string())
                        });

                    let filepath = system_font
                        .path()
                        .to_str()
                        .expect("Failed to convert path to string!")
                        .to_string();
                    let filename = filepath.split("/").last().unwrap_or("").to_string();

                    let mut style = "normal";
                    if system_font.is_italic() {
                        style = "italic";
                    }

                    // Create Font entry
                    let font_entry = Font {
                        filename,
                        weight: Some(system_font.weight().to_u16() as i32),
                        style: Some(style.to_string()),
                    };

                    // Insert into hashmap
                    font_family_hashmap
                        .entry(family_name.expect("Font has no family name!"))
                        .or_insert(Vec::new())
                        .push(font_entry);
                },
                FileRef::Collection(_) => {
                    // Case 2: This means it's a .ttc file.
                    let mut traversable = true;
                    let mut index = 0;

                    while traversable {
                        let ttc_font = FontRef::from_index(&font_bytes, index);
                        match ttc_font {
                            Ok(ttc_f) => {
                                // Get the name table
                                let name_table = ttc_f
                                    .name()
                                    .expect("Font file is corrupted as it has no name table!");
                                let family_name = name_table
                                    .name_record()
                                    .iter()
                                    .filter(|record| record.name_id().to_u16() == 1)
                                    .find_map(|record| {
                                        record
                                            .string(name_table.string_data())
                                            .ok()
                                            .map(|s| s.to_string())
                                    });

                                let filepath = system_font
                                    .path()
                                    .to_str()
                                    .expect("Failed to convert path to string!")
                                    .to_string();
                                let filename = filepath.split("/").last().unwrap_or("").to_string();

                                let mut style = "normal";
                                if system_font.is_italic() {
                                    style = "italic";
                                }

                                // Create Font entry
                                let font_entry = Font {
                                    filename,
                                    weight: Some(system_font.weight().to_u16() as i32),
                                    style: Some(style.to_string()),
                                };

                                // Insert into hashmap
                                font_family_hashmap
                                    .entry(family_name.expect("Font has no family name!"))
                                    .or_insert(Vec::new())
                                    .push(font_entry);
                            },
                            Err(_) => {
                                // No more fonts in the .ttc file
                                traversable = false;
                            },
                        }
                        index += 1;
                    }
                },
            }
        }

        // unpack hashmap
        for (key, values) in &font_family_hashmap {
            let mut fonts = Vec::new();
            for font in values {
                fonts.push(Font {
                    filename: font.filename.clone(),
                    weight: font.weight,
                    style: font.style.clone(),
                });
            }

            let font_family_entry = FontFamily {
                name: key.to_string(),
                fonts,
            };
            font_families_vector.push(font_family_entry);
        }

        // return FontList
        FontList {
            families: font_families_vector,
            aliases: Vec::new(),
        }
    }

    // All Android fonts are located in /system/fonts
    fn font_absolute_path(filename: &str) -> String {
        if filename.starts_with("/") {
            String::from(filename)
        } else {
            format!("/system/fonts/{}", filename)
        }
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

// Functions used by SystemFontSerivce
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
            path: Atom::from(FontList::font_absolute_path(&font.filename)),
            face_index: 0,
            named_instance_index: 0,
        };
        let stretch = StyleFontStretch::NORMAL;
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
                    font.filename
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

// Based on gfxAndroidPlatform::GetCommonFallbackFonts() in Gecko
pub fn fallback_font_families(options: FallbackFontSelectionOptions) -> Vec<&'static str> {
    let mut families = vec![];

    if let Some(block) = options.character.block() {
        match block {
            UnicodeBlock::Armenian => {
                families.push("Droid Sans Armenian");
            },

            UnicodeBlock::Hebrew => {
                families.push("Droid Sans Hebrew");
            },

            UnicodeBlock::Arabic => {
                families.push("Droid Sans Arabic");
            },

            UnicodeBlock::Devanagari => {
                families.push("Noto Sans Devanagari");
                families.push("Droid Sans Devanagari");
            },

            UnicodeBlock::Tamil => {
                families.push("Noto Sans Tamil");
                families.push("Droid Sans Tamil");
            },

            UnicodeBlock::Thai => {
                families.push("Noto Sans Thai");
                families.push("Droid Sans Thai");
            },

            UnicodeBlock::Georgian | UnicodeBlock::GeorgianSupplement => {
                families.push("Droid Sans Georgian");
            },

            UnicodeBlock::Ethiopic | UnicodeBlock::EthiopicSupplement => {
                families.push("Droid Sans Ethiopic");
            },

            UnicodeBlock::Bengali => {
                families.push("Noto Sans Bengali");
            },

            UnicodeBlock::Gujarati => {
                families.push("Noto Sans Gujarati");
            },

            UnicodeBlock::Gurmukhi => {
                families.push("Noto Sans Gurmukhi");
            },

            UnicodeBlock::Oriya => {
                families.push("Noto Sans Oriya");
            },

            UnicodeBlock::Kannada => {
                families.push("Noto Sans Kannada");
            },

            UnicodeBlock::Telugu => {
                families.push("Noto Sans Telugu");
            },

            UnicodeBlock::Malayalam => {
                families.push("Noto Sans Malayalam");
            },

            UnicodeBlock::Sinhala => {
                families.push("Noto Sans Sinhala");
            },

            UnicodeBlock::Lao => {
                families.push("Noto Sans Lao");
            },

            UnicodeBlock::Tibetan => {
                families.push("Noto Sans Tibetan");
            },

            _ => {
                if is_cjk(options.character) {
                    families.push("MotoyaLMaru");
                    families.push("Noto Sans CJK JP");
                    families.push("Droid Sans Japanese");
                }
            },
        }
    }

    families.push("Droid Sans Fallback");
    families
}

pub(crate) fn default_system_generic_font_family(
    generic: GenericFontFamily,
) -> LowercaseFontFamilyName {
    match generic {
        GenericFontFamily::None | GenericFontFamily::Serif => "serif",
        GenericFontFamily::SansSerif => "sans-serif",
        GenericFontFamily::Monospace => "monospace",
        GenericFontFamily::Cursive => "cursive",
        GenericFontFamily::Fantasy => "serif",
        GenericFontFamily::SystemUi => "Droid Sans",
    }
    .into()
}
