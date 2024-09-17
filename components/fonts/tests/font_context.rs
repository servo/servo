/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use app_units::Au;
use fonts::platform::font::PlatformFont;
use fonts::{
    fallback_font_families, FallbackFontSelectionOptions, FontContext, FontData, FontDescriptor,
    FontFamilyDescriptor, FontIdentifier, FontSearchScope, FontTemplate, FontTemplateRef,
    FontTemplates, PlatformFontMethods, SystemFontServiceProxyTrait,
};
use ipc_channel::ipc;
use net_traits::ResourceThreads;
use parking_lot::Mutex;
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::{
    FamilyName, FontFamily, FontFamilyList, FontFamilyNameSyntax, FontSize, FontStretch, FontStyle,
    FontWeight, SingleFontFamily,
};
use style::values::computed::{FontLanguageOverride, XLang};
use style::values::generics::font::LineHeight;
use style::ArcSlice;
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey, IdNamespace};

struct MockFontCacheThread {
    families: Mutex<HashMap<String, FontTemplates>>,
    data: Mutex<HashMap<FontIdentifier, Arc<FontData>>>,
    find_font_count: AtomicI32,
}

impl MockFontCacheThread {
    fn new() -> Self {
        let proxy = Self {
            families: Default::default(),
            data: Default::default(),
            find_font_count: AtomicI32::new(0),
        };

        let mut csstest_ascii = FontTemplates::default();
        proxy.add_face(&mut csstest_ascii, "csstest-ascii");

        let mut csstest_basic = FontTemplates::default();
        proxy.add_face(&mut csstest_basic, "csstest-basic-regular");

        let mut fallback = FontTemplates::default();
        proxy.add_face(&mut fallback, "csstest-basic-regular");

        {
            let mut families = proxy.families.lock();
            families.insert("CSSTest ASCII".to_owned(), csstest_ascii);
            families.insert("CSSTest Basic".to_owned(), csstest_basic);
            families.insert(
                fallback_font_families(FallbackFontSelectionOptions::default())[0].to_owned(),
                fallback,
            );
        }

        proxy
    }

    fn identifier_for_font_name(name: &str) -> FontIdentifier {
        FontIdentifier::Web(Self::url_for_font_name(name))
    }

    fn url_for_font_name(name: &str) -> ServoUrl {
        let mut path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "support", "CSSTest"]
            .iter()
            .collect();
        path.push(format!("{}.ttf", name));
        ServoUrl::from_file_path(path).unwrap()
    }

    fn add_face(&self, family: &mut FontTemplates, name: &str) {
        let mut path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "support", "CSSTest"]
            .iter()
            .collect();
        path.push(format!("{}.ttf", name));

        let file = File::open(path).unwrap();
        let data = Arc::new(FontData::from_bytes(
            file.bytes().map(|b| b.unwrap()).collect(),
        ));

        let url = Self::url_for_font_name(name);
        let identifier = FontIdentifier::Web(url.clone());
        let handle =
            PlatformFont::new_from_data(identifier.clone(), data.as_arc().clone(), 0, None)
                .expect("Could not load test font");
        let template =
            FontTemplate::new_for_remote_web_font(url, handle.descriptor(), None).unwrap();
        family.add_template(template);

        self.data.lock().insert(identifier, data);
    }
}

impl SystemFontServiceProxyTrait for MockFontCacheThread {
    fn find_matching_font_templates(
        &self,
        descriptor_to_match: Option<&FontDescriptor>,
        font_family: &SingleFontFamily,
    ) -> Vec<FontTemplateRef> {
        self.find_font_count.fetch_add(1, Ordering::Relaxed);

        let SingleFontFamily::FamilyName(family_name) = font_family else {
            return Vec::new();
        };

        self.families
            .lock()
            .get_mut(&*family_name.name)
            .map(|family| family.find_for_descriptor(descriptor_to_match))
            .unwrap_or_default()
    }

    fn get_system_font_instance(
        &self,
        _font_identifier: FontIdentifier,
        _size: Au,
        _flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        FontInstanceKey(IdNamespace(0), 0)
    }

    fn get_web_font(&self, _data: Arc<fonts::FontData>, _index: u32) -> FontKey {
        FontKey(IdNamespace(0), 0)
    }

    fn get_web_font_instance(
        &self,
        _font_key: FontKey,
        _size: f32,
        _flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        FontInstanceKey(IdNamespace(0), 0)
    }

    fn get_font_data(&self, identifier: &FontIdentifier) -> Option<Arc<fonts::FontData>> {
        self.data.lock().get(identifier).cloned()
    }
}

fn style() -> FontStyleStruct {
    let mut style = FontStyleStruct {
        font_family: FontFamily::serif(),
        font_style: FontStyle::NORMAL,
        font_variant_caps: FontVariantCaps::Normal,
        font_weight: FontWeight::normal(),
        font_size: FontSize::medium(),
        font_stretch: FontStretch::hundred(),
        hash: 0,
        font_language_override: FontLanguageOverride::normal(),
        line_height: LineHeight::Normal,
        _x_lang: XLang::get_initial_value(),
    };
    style.compute_font_hash();
    style
}

fn font_family(names: Vec<&str>) -> FontFamily {
    let names = names.into_iter().map(|name| {
        SingleFontFamily::FamilyName(FamilyName {
            name: Atom::from(name),
            syntax: FontFamilyNameSyntax::Quoted,
        })
    });

    FontFamily {
        families: FontFamilyList {
            list: ArcSlice::from_iter(names),
        },
        is_system_font: false,
        is_initial: false,
    }
}

fn mock_resource_threads() -> ResourceThreads {
    let (core_sender, _) = ipc::channel().unwrap();
    let (storage_sender, _) = ipc::channel().unwrap();
    ResourceThreads::new(core_sender, storage_sender)
}

#[test]
fn test_font_group_is_cached_by_style() {
    let source = Arc::new(MockFontCacheThread::new());
    let context = FontContext::new(source, mock_resource_threads());

    let style1 = style();

    let mut style2 = style();
    style2.set_font_style(FontStyle::ITALIC);

    assert!(
        std::ptr::eq(
            &*context.font_group(ServoArc::new(style1.clone())).read(),
            &*context.font_group(ServoArc::new(style1.clone())).read()
        ),
        "the same font group should be returned for two styles with the same hash"
    );

    assert!(
        !std::ptr::eq(
            &*context.font_group(ServoArc::new(style1.clone())).read(),
            &*context.font_group(ServoArc::new(style2.clone())).read()
        ),
        "different font groups should be returned for two styles with different hashes"
    )
}

#[test]
fn test_font_group_find_by_codepoint() {
    let source = Arc::new(MockFontCacheThread::new());
    let mut context = FontContext::new(source.clone(), mock_resource_threads());

    let mut style = style();
    style.set_font_family(font_family(vec!["CSSTest ASCII", "CSSTest Basic"]));

    let group = context.font_group(ServoArc::new(style));

    let font = group
        .write()
        .find_by_codepoint(&mut context, 'a', None)
        .unwrap();
    assert_eq!(
        font.identifier(),
        MockFontCacheThread::identifier_for_font_name("csstest-ascii")
    );
    assert_eq!(
        source.find_font_count.fetch_add(0, Ordering::Relaxed),
        1,
        "only the first font in the list should have been loaded"
    );

    let font = group
        .write()
        .find_by_codepoint(&mut context, 'a', None)
        .unwrap();
    assert_eq!(
        font.identifier(),
        MockFontCacheThread::identifier_for_font_name("csstest-ascii")
    );
    assert_eq!(
        source.find_font_count.fetch_add(0, Ordering::Relaxed),
        1,
        "we shouldn't load the same font a second time"
    );

    let font = group
        .write()
        .find_by_codepoint(&mut context, 'á', None)
        .unwrap();
    assert_eq!(
        font.identifier(),
        MockFontCacheThread::identifier_for_font_name("csstest-basic-regular")
    );
    assert_eq!(
        source.find_font_count.fetch_add(0, Ordering::Relaxed),
        2,
        "both fonts should now have been loaded"
    );
}

#[test]
fn test_font_fallback() {
    let source = Arc::new(MockFontCacheThread::new());
    let mut context = FontContext::new(source, mock_resource_threads());

    let mut style = style();
    style.set_font_family(font_family(vec!["CSSTest ASCII"]));

    let group = context.font_group(ServoArc::new(style));

    let font = group
        .write()
        .find_by_codepoint(&mut context, 'a', None)
        .unwrap();
    assert_eq!(
        font.identifier(),
        MockFontCacheThread::identifier_for_font_name("csstest-ascii"),
        "a family in the group should be used if there is a matching glyph"
    );

    let font = group
        .write()
        .find_by_codepoint(&mut context, 'á', None)
        .unwrap();
    assert_eq!(
        font.identifier(),
        MockFontCacheThread::identifier_for_font_name("csstest-basic-regular"),
        "a fallback font should be used if there is no matching glyph in the group"
    );
}

#[test]
fn test_font_template_is_cached() {
    let source = Arc::new(MockFontCacheThread::new());
    let context = FontContext::new(source.clone(), mock_resource_threads());

    let mut font_descriptor = FontDescriptor {
        weight: FontWeight::normal(),
        stretch: FontStretch::hundred(),
        style: FontStyle::normal(),
        variant: FontVariantCaps::Normal,
        pt_size: Au(10),
    };

    let family = SingleFontFamily::FamilyName(FamilyName {
        name: "CSSTest Basic".into(),
        syntax: FontFamilyNameSyntax::Quoted,
    });
    let family_descriptor = FontFamilyDescriptor::new(family, FontSearchScope::Any);

    let font_template = context.matching_templates(&font_descriptor, &family_descriptor)[0].clone();

    let font1 = context
        .font(font_template.clone(), &font_descriptor)
        .unwrap();

    font_descriptor.pt_size = Au(20);
    let font2 = context
        .font(font_template.clone(), &font_descriptor)
        .unwrap();

    assert_ne!(
        font1.descriptor.pt_size, font2.descriptor.pt_size,
        "the same font should not have been returned"
    );

    assert_eq!(
        source.find_font_count.fetch_add(0, Ordering::Relaxed),
        1,
        "we should only have fetched the template data from the cache thread once"
    );
}
