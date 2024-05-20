/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use log::warn;
use serde::{Deserialize, Serialize};
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};
use style::Atom;
use ucd::{Codepoint, UnicodeBlock};
use webrender_api::NativeFontHandle;

use crate::font_template::{FontTemplate, FontTemplateDescriptor};
use crate::text::util::is_cjk;

lazy_static::lazy_static! {
    static ref FONT_LIST: FontList = FontList::new();
}

/// An identifier for a local font on OpenHarmony systems.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
        // We don't support parsing `/system/etc/fontconfig.json` yet.
        FontList {
            families: Self::fallback_font_families(),
            aliases: Vec::new(),
        }
    }

    // Fonts expected to exist in OpenHarmony devices.
    // Used until parsing of the fontconfig.json file is implemented.
    fn fallback_font_families() -> Vec<FontFamily> {
        let alternatives = [
            ("HarmonyOS Sans", "HarmonyOS_Sans_SC_Regular.ttf"),
            ("sans-serif", "HarmonyOS_Sans_SC_Regular.ttf"),
        ];

        alternatives
            .iter()
            .filter(|item| Path::new(&Self::font_absolute_path(item.1)).exists())
            .map(|item| FontFamily {
                name: item.0.into(),
                fonts: vec![Font {
                    filename: item.1.into(),
                    weight: None,
                    style: None,
                }],
            })
            .collect()
    }

    // OHOS fonts are located in /system/fonts
    fn font_absolute_path(filename: &str) -> String {
        if filename.starts_with("/") {
            String::from(filename)
        } else {
            format!("/system/fonts/{}", filename)
        }
    }

    fn find_family(&self, name: &str) -> Option<&FontFamily> {
        self.families.iter().find(|f| f.name == name)
    }

    fn find_alias(&self, name: &str) -> Option<&FontAlias> {
        self.aliases.iter().find(|f| f.from == name)
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
            path: Atom::from(FontList::font_absolute_path(&font.filename)),
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
        let descriptor = FontTemplateDescriptor {
            weight,
            stretch,
            style,
        };
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

pub fn system_default_family(generic_name: &str) -> Option<String> {
    if let Some(family) = FONT_LIST.find_family(&generic_name) {
        Some(family.name.clone())
    } else if let Some(alias) = FONT_LIST.find_alias(&generic_name) {
        Some(alias.from.clone())
    } else {
        FONT_LIST.families.get(0).map(|family| family.name.clone())
    }
}

// Based on fonts present in OpenHarmony.
pub fn fallback_font_families(codepoint: Option<char>) -> Vec<&'static str> {
    let mut families = vec![];

    if let Some(block) = codepoint.and_then(|c| c.block()) {
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

            _ => {
                if is_cjk(codepoint.unwrap()) {
                    families.push("Noto Sans JP");
                    families.push("Noto Sans KR");
                }
            },
        }
    }

    families.push("HarmonyOS Sans");
    families
}

pub static SANS_SERIF_FONT_FAMILY: &'static str = "HarmonyOS Sans";
