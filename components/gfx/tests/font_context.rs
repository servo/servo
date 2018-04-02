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
use gfx::font::{fallback_font_families, FontDescriptor, FontFamilyDescriptor, FontFamilyName, FontSearchScope};
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
use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::Percentage;
use style::values::computed::font::{FamilyName, FamilyNameSyntax, FontFamily, FontFamilyList, FontSize};
use style::values::computed::font::{FontWeight, SingleFontFamily};
use style::values::generics::NonNegative;
use style::values::generics::font::FontStyle;

struct TestFontSource {
    handle: FontContextHandle,
    families: HashMap<String, FontTemplates>,
    find_font_count: Rc<Cell<isize>>,
}

impl TestFontSource {
    fn new() -> TestFontSource {
        let mut csstest_ascii = FontTemplates::new();
        Self::add_face(&mut csstest_ascii, "csstest-ascii", None);

        let mut csstest_basic = FontTemplates::new();
        Self::add_face(&mut csstest_basic, "csstest-basic-regular", None);

        let mut fallback = FontTemplates::new();
        Self::add_face(&mut fallback, "csstest-basic-regular", Some("fallback"));

        let mut families = HashMap::new();
        families.insert("CSSTest ASCII".to_owned(), csstest_ascii);
        families.insert("CSSTest Basic".to_owned(), csstest_basic);
        families.insert(fallback_font_families(None)[0].to_owned(), fallback);

        TestFontSource {
            handle: FontContextHandle::new(),
            families,
            find_font_count: Rc::new(Cell::new(0)),
        }
    }

    fn add_face(family: &mut FontTemplates, name: &str, identifier: Option<&str>) {
        let mut path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "support",
            "CSSTest",
        ].iter().collect();
        path.push(format!("{}.ttf", name));

        let file = File::open(path).unwrap();
        let identifier = Atom::from(identifier.unwrap_or(name));

        family.add_template(
            identifier,
            Some(file.bytes().map(|b| b.unwrap()).collect())
        )
    }
}

impl FontSource for TestFontSource {
    fn get_font_instance(&mut self, _key: webrender_api::FontKey, _size: Au) -> webrender_api::FontInstanceKey {
        webrender_api::FontInstanceKey(webrender_api::IdNamespace(0), 0)
    }

    fn font_template(
        &mut self,
        template_descriptor: FontTemplateDescriptor,
        family_descriptor: FontFamilyDescriptor,
    ) -> Option<FontTemplateInfo> {
        let handle = &self.handle;

        self.find_font_count.set(self.find_font_count.get() + 1);
        self.families
            .get_mut(family_descriptor.name())
            .and_then(|family| family.find_font_for_style(&template_descriptor, handle))
            .map(|template| {
                FontTemplateInfo {
                    font_template: template,
                    font_key: webrender_api::FontKey(webrender_api::IdNamespace(0), 0),
                }
            })
    }
}

fn style() -> FontStyleStruct {
    let mut style = FontStyleStruct {
        font_family: FontFamily::serif(),
        font_style: FontStyle::Normal,
        font_variant_caps: FontVariantCaps::Normal,
        font_weight: FontWeight::normal(),
        font_size: FontSize::medium(),
        font_stretch: NonNegative(Percentage(1.)),
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
    assert_eq!(&*font.borrow().identifier(), "csstest-ascii");
    assert_eq!(count.get(), 1, "only the first font in the list should have been loaded");

    let font = group.borrow_mut().find_by_codepoint(&mut context, 'a').unwrap();
    assert_eq!(&*font.borrow().identifier(), "csstest-ascii");
    assert_eq!(count.get(), 1, "we shouldn't load the same font a second time");

    let font = group.borrow_mut().find_by_codepoint(&mut context, 'á').unwrap();
    assert_eq!(&*font.borrow().identifier(), "csstest-basic-regular");
    assert_eq!(count.get(), 2, "both fonts should now have been loaded");
}

#[test]
fn test_font_fallback() {
    let source = TestFontSource::new();
    let mut context = FontContext::new(source);

    let mut style = style();
    style.set_font_family(font_family(vec!("CSSTest ASCII")));

    let group = context.font_group(Arc::new(style));

    let font = group.borrow_mut().find_by_codepoint(&mut context, 'a').unwrap();
    assert_eq!(
        &*font.borrow().identifier(), "csstest-ascii",
        "a family in the group should be used if there is a matching glyph"
    );

    let font = group.borrow_mut().find_by_codepoint(&mut context, 'á').unwrap();
    assert_eq!(
        &*font.borrow().identifier(), "fallback",
        "a fallback font should be used if there is no matching glyph in the group"
    );
}

#[test]
fn test_font_template_is_cached() {
    let source = TestFontSource::new();
    let count = source.find_font_count.clone();
    let mut context = FontContext::new(source);

    let mut font_descriptor = FontDescriptor {
        template_descriptor: FontTemplateDescriptor {
            weight: FontWeight::normal(),
            stretch: FontStretch::Normal,
            italic: false,
        },
        variant: FontVariantCaps::Normal,
        pt_size: Au(10),
    };

    let family_descriptor = FontFamilyDescriptor::new(
        FontFamilyName::from("CSSTest Basic"),
        FontSearchScope::Any,
    );

    let font1 = context.font(&font_descriptor, &family_descriptor).unwrap();

    font_descriptor.pt_size = Au(20);
    let font2 = context.font(&font_descriptor, &family_descriptor).unwrap();

    assert_ne!(
        font1.borrow().actual_pt_size,
        font2.borrow().actual_pt_size,
        "the same font should not have been returned"
    );

    assert_eq!(count.get(), 1, "we should only have fetched the template data from the cache thread once");
}
