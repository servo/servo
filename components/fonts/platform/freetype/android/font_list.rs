/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::sync::LazyLock;

use ndk::font::{Font as NDK_Font, SystemFontIterator};
use read_fonts::tables::name::{Name, NameRecord};
use read_fonts::{FileRef, FontData, TableProvider};
use servo_base::text::{UnicodeBlock, UnicodeBlockMethod, is_cjk};
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

#[derive(Clone)]
enum FontStyle {
    Normal,
    Italic,
}

struct Font {
    filename: String,
    weight: Option<i32>,
    style: FontStyle,
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
        let system_font_iterator =
            SystemFontIterator::new().expect("Failed to create SystemFontIterator");
        let mut families = Vec::new();

        for system_font in system_font_iterator {
            // Obtain the font file
            // TODO: This operation is slow and might need optimizations.
            let font_bytes = File::open(system_font.path())
                .and_then(|file| unsafe { memmap2::Mmap::map(&file) })
                .unwrap();

            // Read the font file
            let Ok(font_file) = FileRef::new(&font_bytes) else {
                continue;
            };

            match font_file {
                FileRef::Font(font) => {
                    let Ok(name_table) = font.name() else {
                        continue;
                    };
                    Self::create_font_family_entry(&system_font, name_table, &mut families);
                },
                FileRef::Collection(font_collection) => {
                    for font in font_collection.iter() {
                        let Ok(font_binding) = font else {
                            continue;
                        };
                        let Ok(name_table) = font_binding.name() else {
                            continue;
                        };
                        Self::create_font_family_entry(&system_font, name_table, &mut families);
                    }
                },
            }
        }

        FontList {
            families,
            // TODO: Also get localized names for these fonts.
            aliases: Vec::new(),
        }
    }

    fn create_font_family_entry(
        system_font: &NDK_Font,
        name_table: Name,
        families: &mut Vec<FontFamily>,
    ) {
        for name_record in name_table.name_record() {
            let Some((family_name, font_entry)) =
                Self::create_single_font_entry(system_font, *name_record, name_table.string_data())
            else {
                continue;
            };
            families.push(FontFamily {
                name: family_name,
                fonts: vec![Font {
                    filename: font_entry.filename.clone(),
                    weight: font_entry.weight,
                    style: font_entry.style.clone(),
                }],
            });
        }
    }

    fn create_single_font_entry(
        system_font: &NDK_Font,
        name_record: NameRecord,
        name_table_data: FontData,
    ) -> Option<(String, Font)> {
        // According to TrueType's specifications, https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
        // Name table's name id 1 corresponds to the family name. therefore, if the current record's name id is not 1, return None.
        if name_record.name_id().to_u16() != 1 {
            return None;
        }

        let family_name = name_record.string(name_table_data).ok()?.to_string();
        let filename = system_font.path().file_name()?.to_str()?;

        let style = if system_font.is_italic() {
            FontStyle::Normal
        } else {
            FontStyle::Italic
        };

        let font_entry = Font {
            filename: filename.to_string(),
            weight: Some(system_font.weight().to_u16() as i32),
            style: style,
        };

        Some((family_name, font_entry))
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
        let style = match font.style {
            FontStyle::Italic => StyleFontStyle::ITALIC,
            FontStyle::Normal => StyleFontStyle::NORMAL,
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
