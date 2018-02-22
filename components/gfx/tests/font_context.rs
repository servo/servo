/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate app_units;
extern crate gfx;
extern crate servo_arc;
extern crate servo_atoms;
extern crate style;
extern crate webrender_api;

use app_units::Au;
use gfx::font::FontHandleMethods;
use gfx::font_cache_thread::{FontTemplates, FontTemplateInfo};
use gfx::font_context::{FontContext, FontContextHandle, FontSource};
use gfx::font_template::FontTemplateDescriptor;
use servo_arc::Arc;
use servo_atoms::Atom;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;
use style::properties::longhands::font_stretch::computed_value::T as FontStretch;
use style::properties::longhands::font_style::computed_value::T as FontStyle;
use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::{FamilyName, FamilyNameSyntax, FontFamily, FontFamilyList, FontSize};
use style::values::computed::font::{FontWeight, SingleFontFamily};

struct TestFontSource {
    handle: FontContextHandle,
    families: HashMap<Atom, FontTemplates>,
    find_font_count: Rc<Cell<isize>>,
}

impl TestFontSource {
    fn new() -> TestFontSource {
        let mut csstest_ascii = FontTemplates::new();
        Self::add_face(&mut csstest_ascii, "csstest-ascii");

        let mut csstest_basic = FontTemplates::new();
        Self::add_face(&mut csstest_basic, "csstest-basic-regular");

        let mut families = HashMap::new();
        families.insert(Atom::from("CSSTest ASCII"), csstest_ascii);
        families.insert(Atom::from("CSSTest Basic"), csstest_basic);

        TestFontSource {
            handle: FontContextHandle::new(),
            families,
            find_font_count: Rc::new(Cell::new(0)),
        }
    }

    fn add_face(family: &mut FontTemplates, name: &str) {
        let mut path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "support",
            "CSSTest",
        ].iter().collect();
        path.push(format!("{}.ttf", name));

        let file = File::open(path).unwrap();

        family.add_template(
            Atom::from(name),
            Some(file.bytes().map(|b| b.unwrap()).collect())
        )
    }
}

impl FontSource for TestFontSource {
    fn get_font_instance(&mut self, _key: webrender_api::FontKey, _size: Au) -> webrender_api::FontInstanceKey {
        webrender_api::FontInstanceKey(webrender_api::IdNamespace(0), 0)
    }

    fn find_font_template(
        &mut self,
        family: SingleFontFamily,
        desc: FontTemplateDescriptor
    ) -> Option<FontTemplateInfo> {
        let handle = &self.handle;

        self.find_font_count.set(self.find_font_count.get() + 1);
        self.families
            .get_mut(family.atom())
            .and_then(|family| family.find_font_for_style(&desc, handle))
            .map(|template| {
                FontTemplateInfo {
                    font_template: template,
                    font_key: webrender_api::FontKey(webrender_api::IdNamespace(0), 0),
                }
            })
    }

    fn last_resort_font_template(&mut self, _desc: FontTemplateDescriptor) -> FontTemplateInfo {
        unimplemented!();
    }
}

fn style() -> FontStyleStruct {
    let mut style = FontStyleStruct {
        font_family: FontFamily::serif(),
        font_style: FontStyle::Normal,
        font_variant_caps: FontVariantCaps::Normal,
        font_weight: FontWeight::normal(),
        font_size: FontSize::medium(),
        font_stretch: FontStretch::Normal,
        hash: 0,
    };
    style.compute_font_hash();
    style
}

fn font_family(names: Vec<&str>) -> FontFamily {
    let names: Vec<SingleFontFamily> = names.into_iter().map(|name|
        SingleFontFamily::FamilyName(FamilyName {
            name: Atom::from(name),
            syntax: FamilyNameSyntax::Quoted,
        })
    ).collect();

    FontFamily(FontFamilyList::new(names.into_boxed_slice()))
}

#[test]
fn test_font_group_is_cached_by_style() {
    let source = TestFontSource::new();
    let mut context = FontContext::new(source);

    let style1 = style();

    let mut style2 = style();
    style2.set_font_style(FontStyle::Italic);

    assert_eq!(
        context.font_group(Arc::new(style1.clone())).as_ptr(),
        context.font_group(Arc::new(style1.clone())).as_ptr(),
        "the same font group should be returned for two styles with the same hash"
    );

    assert_ne!(
        context.font_group(Arc::new(style1.clone())).as_ptr(),
        context.font_group(Arc::new(style2.clone())).as_ptr(),
        "different font groups should be returned for two styles with different hashes"
    )
}

#[test]
fn test_font_group_find_by_codepoint() {
    let source = TestFontSource::new();
    let count = source.find_font_count.clone();
    let mut context = FontContext::new(source);

    let mut style = style();
    style.set_font_family(font_family(vec!("CSSTest ASCII", "CSSTest Basic")));

    let group = context.font_group(Arc::new(style));

    let font = group.borrow_mut().find_by_codepoint(&mut context, 'a').unwrap();
    assert_eq!(font.borrow().handle.family_name(), "CSSTest ASCII");
    assert_eq!(count.get(), 1, "only the first font in the list should have been loaded");

    let font = group.borrow_mut().find_by_codepoint(&mut context, 'a').unwrap();
    assert_eq!(font.borrow().handle.family_name(), "CSSTest ASCII");
    assert_eq!(count.get(), 1, "we shouldn't load the same font a second time");

    let font = group.borrow_mut().find_by_codepoint(&mut context, 'á').unwrap();
    assert_eq!(font.borrow().handle.family_name(), "CSSTest Basic");
    assert_eq!(count.get(), 2, "both fonts should now have been loaded");
}
