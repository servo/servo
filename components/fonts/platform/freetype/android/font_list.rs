/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Read;
use std::path::Path;

use base::text::{is_cjk, UnicodeBlock, UnicodeBlockMethod};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};
use style::Atom;

use super::xml::{Attribute, Node};
use crate::{
    FallbackFontSelectionOptions, FontTemplate, FontTemplateDescriptor, LowercaseFontFamilyName,
};

lazy_static::lazy_static! {
    static ref FONT_LIST: FontList = FontList::new();
}

/// An identifier for a local font on Android systems.
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

// Android doesn't provide an API to query system fonts until Android O:
// https://developer.android.com/reference/android/text/FontConfig.html
// System font configuration files must be parsed until Android O version is set as the minimum target.
// Android uses XML files to handle font mapping configurations.
// On Android API 21+ font mappings are loaded from /etc/fonts.xml.
// Each entry consists of a family with various font names, or a font alias.
// Example:
//   <familyset>
//       <!-- first font is default -->
//       <family name="sans-serif">
//           <font weight="100" style="normal">Roboto-Thin.ttf</font>
//           <font weight="100" style="italic">Roboto-ThinItalic.ttf</font>
//           <font weight="300" style="normal">Roboto-Light.ttf</font>
//           <font weight="300" style="italic">Roboto-LightItalic.ttf</font>
//           <font weight="400" style="normal">Roboto-Regular.ttf</font>
//           <font weight="400" style="italic">Roboto-Italic.ttf</font>
//           <font weight="500" style="normal">Roboto-Medium.ttf</font>
//           <font weight="500" style="italic">Roboto-MediumItalic.ttf</font>
//           <font weight="900" style="normal">Roboto-Black.ttf</font>
//           <font weight="900" style="italic">Roboto-BlackItalic.ttf</font>
//           <font weight="700" style="normal">Roboto-Bold.ttf</font>
//           <font weight="700" style="italic">Roboto-BoldItalic.ttf</font>
//       </family>//

//       <!-- Note that aliases must come after the fonts they reference. -->
//       <alias name="sans-serif-thin" to="sans-serif" weight="100" />
//       <alias name="sans-serif-light" to="sans-serif" weight="300" />
//       <alias name="sans-serif-medium" to="sans-serif" weight="500" />
//       <alias name="sans-serif-black" to="sans-serif" weight="900" />
//       <alias name="arial" to="sans-serif" />
//       <alias name="helvetica" to="sans-serif" />
//       <alias name="tahoma" to="sans-serif" />
//       <alias name="verdana" to="sans-serif" />

//       <family name="sans-serif-condensed">
//           <font weight="300" style="normal">RobotoCondensed-Light.ttf</font>
//           <font weight="300" style="italic">RobotoCondensed-LightItalic.ttf</font>
//           <font weight="400" style="normal">RobotoCondensed-Regular.ttf</font>
//           <font weight="400" style="italic">RobotoCondensed-Italic.ttf</font>
//           <font weight="700" style="normal">RobotoCondensed-Bold.ttf</font>
//           <font weight="700" style="italic">RobotoCondensed-BoldItalic.ttf</font>
//       </family>
//       <alias name="sans-serif-condensed-light" to="sans-serif-condensed" weight="300" />
//   </familyset>
// On Android API 17-20 font mappings are loaded from /system/etc/system_fonts.xml
// Each entry consists of a family with a nameset and a fileset.
// Example:
//  <familyset>
//      <family>
//          <nameset>
//              <name>sans-serif</name>
//              <name>arial</name>
//              <name>helvetica</name>
//              <name>tahoma</name>
//              <name>verdana</name>
//          </nameset>
//          <fileset>
//              <file>Roboto-Regular.ttf</file>
//              <file>Roboto-Bold.ttf</file>
//              <file>Roboto-Italic.ttf</file>
//              <file>Roboto-BoldItalic.ttf</file>
//          </fileset>
//      </family>//

//      <family>
//          <nameset>
//              <name>sans-serif-light</name>
//          </nameset>
//          <fileset>
//              <file>Roboto-Light.ttf</file>
//              <file>Roboto-LightItalic.ttf</file>
//          </fileset>
//      </family>//

//      <family>
//          <nameset>
//              <name>sans-serif-thin</name>
//          </nameset>
//          <fileset>
//              <file>Roboto-Thin.ttf</file>
//              <file>Roboto-ThinItalic.ttf</file>
//          </fileset>
//      </family>
//  </familyset>

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
        // Possible paths containing the font mapping xml file.
        let paths = [
            "/etc/fonts.xml",
            "/system/etc/system_fonts.xml",
            "/package/etc/fonts.xml",
        ];

        // Try to load and parse paths until one of them success.
        let mut result = None;
        paths.iter().all(|path| {
            result = Self::from_path(path);
            !result.is_some()
        });

        if result.is_none() {
            warn!("Couldn't find font list");
        }

        match result {
            Some(result) => result,
            // If no xml mapping file is found fallback to some default
            // fonts expected to be on all Android devices.
            None => FontList {
                families: Self::fallback_font_families(),
                aliases: Vec::new(),
            },
        }
    }

    // Creates a new FontList from a path to the font mapping xml file.
    fn from_path(path: &str) -> Option<FontList> {
        let bytes = std::fs::read(path).ok()?;
        let nodes = super::xml::parse(&bytes).ok()?;

        // find familyset root node
        let familyset = nodes.iter().find_map(|e| match e {
            Node::Element { name, children, .. } if name.local_name == "familyset" => {
                Some(children)
            },
            _ => None,
        })?;

        // Parse familyset node
        let mut families = Vec::new();
        let mut aliases = Vec::new();

        for node in familyset {
            if let Node::Element {
                name,
                attributes,
                children,
            } = node
            {
                if name.local_name == "family" {
                    Self::parse_family(children, attributes, &mut families);
                } else if name.local_name == "alias" {
                    // aliases come after the fonts they reference. -->
                    if !families.is_empty() {
                        Self::parse_alias(attributes, &mut aliases);
                    }
                }
            }
        }

        Some(FontList {
            families: families,
            aliases: aliases,
        })
    }

    // Fonts expected to exist in Android devices.
    // Only used in the unlikely case where no font xml mapping files are found.
    fn fallback_font_families() -> Vec<FontFamily> {
        let alternatives = [
            ("sans-serif", "Roboto-Regular.ttf"),
            ("Droid Sans", "DroidSans.ttf"),
            (
                "Lomino",
                "/system/etc/ml/kali/Fonts/Lomino/Medium/LominoUI_Md.ttf",
            ),
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

    // Parse family and font file names
    // Example:
    // <family name="sans-serif">
    //     <font weight="100" style="normal">Roboto-Thin.ttf</font>
    //     <font weight="100" style="italic">Roboto-ThinItalic.ttf</font>
    //     <font weight="300" style="normal">Roboto-Light.ttf</font>
    //     <font weight="300" style="italic">Roboto-LightItalic.ttf</font>
    //     <font weight="400" style="normal">Roboto-Regular.ttf</font>
    // </family>
    fn parse_family(familyset: &[Node], attrs: &[Attribute], out: &mut Vec<FontFamily>) {
        // Fallback to old Android API v17 xml format if required
        let using_api_17 = familyset.iter().any(|node| match node {
            Node::Element { name, .. } => name.local_name == "nameset",
            _ => false,
        });
        if using_api_17 {
            Self::parse_family_v17(familyset, out);
            return;
        }

        // Parse family name
        let name = if let Some(name) = Self::find_attrib("name", attrs) {
            name
        } else {
            return;
        };

        let mut fonts = Vec::new();
        // Parse font variants
        for node in familyset {
            match node {
                Node::Element {
                    name,
                    attributes,
                    children,
                } => {
                    if name.local_name == "font" {
                        FontList::parse_font(&children, attributes, &mut fonts);
                    }
                },
                _ => {},
            }
        }

        out.push(FontFamily {
            name: name,
            fonts: fonts,
        });
    }

    // Parse family and font file names for Androi API < 21
    // Example:
    // <family>
    //     <nameset>
    //         <name>sans-serif</name>
    //         <name>arial</name>
    //         <name>helvetica</name>
    //         <name>tahoma</name>
    //         <name>verdana</name>
    //     </nameset>
    //     <fileset>
    //         <file>Roboto-Regular.ttf</file>
    //         <file>Roboto-Bold.ttf</file>
    //         <file>Roboto-Italic.ttf</file>
    //         <file>Roboto-BoldItalic.ttf</file>
    //     </fileset>
    // </family>
    fn parse_family_v17(familyset: &[Node], out: &mut Vec<FontFamily>) {
        let mut nameset = Vec::new();
        let mut fileset = Vec::new();
        for node in familyset {
            if let Node::Element { name, children, .. } = node {
                if name.local_name == "nameset" {
                    Self::collect_contents_with_tag(children, "name", &mut nameset);
                } else if name.local_name == "fileset" {
                    Self::collect_contents_with_tag(children, "file", &mut fileset);
                }
            }
        }

        // Create a families for each variation
        for name in nameset {
            let fonts: Vec<Font> = fileset
                .iter()
                .map(|f| Font {
                    filename: f.clone(),
                    weight: None,
                    style: None,
                })
                .collect();

            if !fonts.is_empty() {
                out.push(FontFamily {
                    name: name,
                    fonts: fonts,
                })
            }
        }
    }

    // Example:
    // <font weight="100" style="normal">Roboto-Thin.ttf</font>
    fn parse_font(nodes: &[Node], attrs: &[Attribute], out: &mut Vec<Font>) {
        // Parse font filename
        if let Some(filename) = Self::text_content(nodes) {
            // Parse font weight
            let weight = Self::find_attrib("weight", attrs).and_then(|w| w.parse().ok());
            let style = Self::find_attrib("style", attrs);

            out.push(Font {
                filename,
                weight,
                style,
            })
        }
    }

    // Example:
    // <alias name="sans-serif-thin" to="sans-serif" weight="100" />
    // <alias name="sans-serif-light" to="sans-serif" weight="300" />
    // <alias name="sans-serif-medium" to="sans-serif" weight="500" />
    // <alias name="sans-serif-black" to="sans-serif" weight="900" />
    // <alias name="arial" to="sans-serif" />
    // <alias name="helvetica" to="sans-serif" />
    // <alias name="tahoma" to="sans-serif" />
    // <alias name="verdana" to="sans-serif" />
    fn parse_alias(attrs: &[Attribute], out: &mut Vec<FontAlias>) {
        // Parse alias name and referenced font
        let from = match Self::find_attrib("name", attrs) {
            Some(from) => from,
            _ => {
                return;
            },
        };

        // Parse referenced font
        let to = match Self::find_attrib("to", attrs) {
            Some(to) => to,
            _ => {
                return;
            },
        };

        // Parse optional weight filter
        let weight = Self::find_attrib("weight", attrs).and_then(|w| w.parse().ok());

        out.push(FontAlias {
            from: from,
            to: to,
            weight: weight,
        })
    }

    fn find_attrib(name: &str, attrs: &[Attribute]) -> Option<String> {
        attrs
            .iter()
            .find(|attr| attr.name.local_name == name)
            .map(|attr| attr.value.clone())
    }

    fn text_content(nodes: &[Node]) -> Option<String> {
        nodes.get(0).and_then(|child| match child {
            Node::Text(contents) => Some(contents.trim().into()),
            Node::Element { .. } => None,
        })
    }

    fn collect_contents_with_tag(nodes: &[Node], tag: &str, out: &mut Vec<String>) {
        for node in nodes {
            if let Node::Element { name, children, .. } = node {
                if name.local_name == tag {
                    if let Some(content) = Self::text_content(children) {
                        out.push(content);
                    }
                }
            }
        }
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

pub fn default_system_generic_font_family(generic: GenericFontFamily) -> LowercaseFontFamilyName {
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
