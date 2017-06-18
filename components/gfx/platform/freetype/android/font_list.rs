/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use xml5ever::Attribute;
use xml5ever::driver::parse_document;
use xml5ever::rcdom::*;
use xml5ever::rcdom::{Node, RcDom};
use xml5ever::tendril::TendrilSink;

lazy_static! {
    static ref FONT_LIST: FontList = FontList::new();
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
}

struct FontFamily {
    name: String,
    fonts: Vec<Font>,
}

struct FontAlias {
    from: String,
    to: String,
    weight: Option<i32>
}

struct FontList {
    families: Vec<FontFamily>,
    aliases: Vec<FontAlias>
}

impl FontList {
    fn new() -> FontList {
        // Possible paths containing the font mapping xml file.
        let paths = [
            "/etc/fonts.xml",
            "/system/etc/system_fonts.xml"
        ];

        // Try to load and parse paths until one of them success.
        let mut result = None;
        paths.iter().all(|path| {
            result = Self::from_path(path);
            !result.is_some()
        });

        match result {
            Some(result) => result,
            // If no xml mapping file is found fallback to some default
            // fonts expected to be on all Android devices.
            None => FontList {
                families: Self::fallback_font_families(),
                aliases: Vec::new(),
            }
        }
    }

    // Creates a new FontList from a path to the font mapping xml file.
    fn from_path(path: &str) -> Option<FontList> {
        let xml = match Self::load_file(path) {
            Ok(xml) => xml,
            _=> { return None; },
        };

        let dom: RcDom = parse_document(RcDom::default(), Default::default())
                         .one(xml);
        let doc = &dom.document;

        // find familyset root node
        let children = doc.children.borrow();
        let familyset = children.iter().find(|child| {
            match child.data {
                NodeData::Element { ref name, .. } => &*name.local == "familyset",
                _ => false,
            }
        });

        let familyset = match familyset {
            Some(node) => node,
            _ => { return None; }
        };

        // Parse familyset node
        let mut families = Vec::new();
        let mut aliases = Vec::new();

        for node in familyset.children.borrow().iter() {
            match node.data {
                NodeData::Element { ref name, ref attrs, .. } => {
                    if &*name.local == "family" {
                        Self::parse_family(&node, attrs, &mut families);
                    } else if &*name.local == "alias" {
                        // aliases come after the fonts they reference. -->
                        if !families.is_empty() {
                            Self::parse_alias(attrs, &mut aliases);
                        }
                    }
                },
                _=> {}
            }
        }

        Some(FontList {
            families: families,
            aliases: aliases
        })
    }

    // Fonts expected to exist in Android devices.
    // Only used in the unlikely case where no font xml mapping files are found.
    fn fallback_font_families() -> Vec<FontFamily> {
        let alternatives = [
            ("san-serif", "Roboto-Regular.ttf"),
            ("Droid Sans", "DroidSans.ttf"),
        ];

        alternatives.iter().filter(|item| {
            Path::new(&Self::font_absolute_path(item.1)).exists()
        }).map(|item| {
            FontFamily {
                name: item.0.into(),
                fonts: vec![Font {
                    filename: item.1.into(),
                    weight: None,
                }]
            }
        }). collect()
    }

    // All Android fonts are located in /system/fonts
    fn font_absolute_path(filename: &str) -> String {
        format!("/system/fonts/{}", filename)
    }

    fn find_family(&self, name: &str) -> Option<&FontFamily>{
        self.families.iter().find(|f| f.name == name)
    }

    fn find_alias(&self, name: &str) -> Option<&FontAlias>{
        self.aliases.iter().find(|f| f.from == name)
    }


    fn load_file(path: &str) -> Result<String, io::Error> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
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
    fn parse_family(familyset: &Node, attrs: &RefCell<Vec<Attribute>>, out:&mut Vec<FontFamily>) {
        // Fallback to old Android API v17 xml format if required
        let using_api_17 = familyset.children.borrow().iter().any(|node| {
            match node.data {
                NodeData::Element { ref name, .. } => &*name.local == "nameset",
                _=> false,
            }
        });
        if using_api_17 {
            Self::parse_family_v17(familyset, out);
            return;
        }

        // Parse family name
        let name = match Self::find_attrib("name", attrs) {
            Some(name) => name,
            _ => { return; },
        };

        let mut fonts = Vec::new();
        // Parse font variants
        for node in familyset.children.borrow().iter() {
            match node.data {
                NodeData::Element { ref name, ref attrs, .. } => {
                    if &*name.local == "font" {
                        FontList::parse_font(&node, attrs, &mut fonts);
                    }
                },
                _=> {}
            }
        }

        out.push(FontFamily {
            name: name,
            fonts: fonts
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
    fn parse_family_v17(familyset: &Node, out:&mut Vec<FontFamily>) {
        let mut nameset = Vec::new();
        let mut fileset = Vec::new();
        for node in familyset.children.borrow().iter() {
            match node.data {
                NodeData::Element { ref name, .. } => {
                    if &*name.local == "nameset" {
                        Self::collect_contents_with_tag(node, "name", &mut nameset);
                    } else if &*name.local == "fileset" {
                        Self::collect_contents_with_tag(node, "file", &mut fileset);
                    }
                },
                _=> {}
            }
        }

        // Create a families for each variation
        for name in nameset {
            let fonts: Vec<Font> = fileset.iter().map(|f| Font {
                filename: f.clone(),
                weight: None,
            }).collect();

            if !fonts.is_empty() {
                out.push(FontFamily {
                    name: name,
                    fonts: fonts
                })
            }
        }
    }

    // Example:
    // <font weight="100" style="normal">Roboto-Thin.ttf</font>
    fn parse_font(node: &Node, attrs: &RefCell<Vec<Attribute>>, out:&mut Vec<Font>) {
        // Parse font filename
        let filename = match Self::text_content(node) {
            Some(filename) => filename,
            _ => { return; }
        };

        // Parse font weight
        let weight = Self::find_attrib("weight", attrs).and_then(|w| w.parse().ok());

        out.push(Font {
            filename: filename,
            weight: weight,
        })
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
    fn parse_alias(attrs: &RefCell<Vec<Attribute>>, out:&mut Vec<FontAlias>) {
        // Parse alias name and referenced font
        let from = match Self::find_attrib("name", attrs) {
            Some(from) => from,
            _ => { return; },
        };

        // Parse referenced font
        let to = match Self::find_attrib("to", attrs) {
            Some(to) => to,
            _ => { return; },
        };

        // Parse optional weight filter
        let weight = Self::find_attrib("weight", attrs).and_then(|w| w.parse().ok());

        out.push(FontAlias {
            from: from,
            to: to,
            weight: weight,
        })
    }

    fn find_attrib(name: &str, attrs: &RefCell<Vec<Attribute>>) -> Option<String> {
        attrs.borrow().iter().find(|attr| &*attr.name.local == name).map(|s| String::from(&s.value))
    }

    fn text_content(node: &Node) -> Option<String> {
        node.children.borrow().get(0).and_then(|child| {
            match child.data {
                NodeData::Text { ref contents } => {
                    let mut result = String::new();
                    result.push_str(&contents.borrow());
                    Some(result)
                },
                _ => None
            }
        })
    }

    fn collect_contents_with_tag(node: &Node, tag: &str, out:&mut Vec<String>) {
        for child in node.children.borrow().iter() {
            match child.data {
                NodeData::Element { ref name, .. } => {
                    if &*name.local == tag {
                        if let Some(content) = Self::text_content(child) {
                            out.push(content);
                        }
                    }
                },
                _=> {}
            }
        }
    }
}

// Functions used by FontCacheThread
pub fn for_each_available_family<F>(mut callback: F) where F: FnMut(String) {
    for family in &FONT_LIST.families {
        callback(family.name.clone());
    }
    for alias in &FONT_LIST.aliases {
        callback(alias.from.clone());
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
    where F: FnMut(String)
{
    println!("Variatioooon {:?}", family_name);
    if let Some(family) = FONT_LIST.find_family(family_name) {
        for font in &family.fonts {
            callback(FontList::font_absolute_path(&font.filename));
        }
        return;
    }

    if let Some(alias) = FONT_LIST.find_alias(family_name) {
        if let Some(family) = FONT_LIST.find_family(&alias.to) {
            for font in &family.fonts {
                match (alias.weight, font.weight) {
                    (None, _) => callback(FontList::font_absolute_path(&font.filename)),
                    (Some(w1), Some(w2)) => {
                        if w1 == w2 {
                            callback(FontList::font_absolute_path(&font.filename))
                        }
                    },
                    _ => {}
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
        //  First font defined in the fonts.xml is the default on Android.
        FONT_LIST.families.get(0).map(|family| family.name.clone())
    }
}

pub fn last_resort_font_families() -> Vec<String> {
    vec!(
        "sans-serif".to_owned(),
        "Droid Sans".to_owned(),
        "serif".to_owned(),
    )
}

pub static SANS_SERIF_FONT_FAMILY: &'static str = "sans-serif";
