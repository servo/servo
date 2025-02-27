/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This currently only works on FreeType platforms as it requires being able to create
// local font identifiers from paths.
#[cfg(target_os = "linux")]
mod font_context {
    use std::collections::HashMap;
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;
    use std::thread;

    use app_units::Au;
    use fonts::platform::font::PlatformFont;
    use fonts::{
        fallback_font_families, FallbackFontSelectionOptions, FontContext, FontDescriptor,
        FontFamilyDescriptor, FontIdentifier, FontSearchScope, FontTemplate, FontTemplates,
        LocalFontIdentifier, PlatformFontMethods, SystemFontServiceMessage, SystemFontServiceProxy,
        SystemFontServiceProxySender,
    };
    use ipc_channel::ipc::{self, IpcReceiver};
    use net_traits::ResourceThreads;
    use parking_lot::Mutex;
    use servo_arc::Arc as ServoArc;
    use servo_atoms::Atom;
    use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
    use style::properties::style_structs::Font as FontStyleStruct;
    use style::values::computed::font::{
        FamilyName, FontFamily, FontFamilyList, FontFamilyNameSyntax, FontSize, FontStretch,
        FontStyle, FontWeight, SingleFontFamily,
    };
    use style::values::computed::{FontLanguageOverride, XLang};
    use style::values::generics::font::LineHeight;
    use style::ArcSlice;
    use webrender_api::{FontInstanceKey, FontKey, IdNamespace};
    use webrender_traits::CrossProcessCompositorApi;

    struct TestContext {
        context: FontContext,
        system_font_service: Arc<MockSystemFontService>,
        system_font_service_proxy: SystemFontServiceProxy,
    }

    impl TestContext {
        fn new() -> TestContext {
            let (system_font_service, system_font_service_proxy) = MockSystemFontService::spawn();
            let (core_sender, _) = ipc::channel().unwrap();
            let (storage_sender, _) = ipc::channel().unwrap();
            let mock_resource_threads = ResourceThreads::new(core_sender, storage_sender);
            let mock_compositor_api = CrossProcessCompositorApi::dummy();

            let proxy_clone = Arc::new(system_font_service_proxy.to_sender().to_proxy());
            Self {
                context: FontContext::new(proxy_clone, mock_compositor_api, mock_resource_threads),
                system_font_service,
                system_font_service_proxy,
            }
        }
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            self.system_font_service_proxy.exit();
        }
    }

    fn font_face_name(identifier: &FontIdentifier) -> String {
        let FontIdentifier::Local(local_identifier) = identifier else {
            unreachable!("Should never create a web font for this test.");
        };
        PathBuf::from(&*local_identifier.path)
            .file_name()
            .and_then(OsStr::to_str)
            .map(|string| string.replace(".ttf", ""))
            .expect("Could not extract font face name.")
    }

    struct MockSystemFontService {
        families: Mutex<HashMap<String, FontTemplates>>,
        find_font_count: AtomicI32,
    }

    impl MockSystemFontService {
        fn spawn() -> (Arc<MockSystemFontService>, SystemFontServiceProxy) {
            let (sender, receiver) = ipc::channel().unwrap();
            let system_font_service = Arc::new(Self::new());

            let system_font_service_clone = system_font_service.clone();
            thread::Builder::new()
                .name("MockSystemFontService".to_owned())
                .spawn(move || system_font_service_clone.run(receiver))
                .expect("Thread spawning failed");
            (
                system_font_service,
                SystemFontServiceProxySender(sender).to_proxy(),
            )
        }

        fn run(&self, receiver: IpcReceiver<SystemFontServiceMessage>) {
            loop {
                match receiver.recv().unwrap() {
                    SystemFontServiceMessage::GetFontTemplates(
                        descriptor_to_match,
                        font_family,
                        result_sender,
                    ) => {
                        self.find_font_count.fetch_add(1, Ordering::Relaxed);

                        let SingleFontFamily::FamilyName(family_name) = font_family else {
                            let _ = result_sender.send(vec![]);
                            continue;
                        };

                        let _ = result_sender.send(
                            self.families
                                .lock()
                                .get_mut(&*family_name.name)
                                .map(|family| {
                                    family.find_for_descriptor(descriptor_to_match.as_ref())
                                })
                                .unwrap()
                                .into_iter()
                                .map(|template| template.borrow().clone())
                                .collect(),
                        );
                    },
                    SystemFontServiceMessage::GetFontInstanceKey(result_sender) |
                    SystemFontServiceMessage::GetFontInstance(_, _, _, result_sender) => {
                        let _ = result_sender.send(FontInstanceKey(IdNamespace(0), 0));
                    },
                    SystemFontServiceMessage::GetFontKey(result_sender) => {
                        let _ = result_sender.send(FontKey(IdNamespace(0), 0));
                    },
                    SystemFontServiceMessage::Exit(result_sender) => {
                        let _ = result_sender.send(());
                        break;
                    },
                    SystemFontServiceMessage::Ping => {},
                }
            }
        }

        fn new() -> Self {
            let proxy = Self {
                families: Default::default(),
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

        fn add_face(&self, family: &mut FontTemplates, name: &str) {
            let mut path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "support", "CSSTest"]
                .iter()
                .collect();
            path.push(format!("{}.ttf", name));

            let local_font_identifier = LocalFontIdentifier {
                path: path.to_str().expect("Could not load test font").into(),
                variation_index: 0,
            };
            let handle =
                PlatformFont::new_from_local_font_identifier(local_font_identifier.clone(), None)
                    .expect("Could not load test font");

            family.add_template(FontTemplate::new(
                FontIdentifier::Local(local_font_identifier),
                handle.descriptor(),
                None,
            ));
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

    #[test]
    fn test_font_group_is_cached_by_style() {
        let context = TestContext::new();

        let style1 = style();

        let mut style2 = style();
        style2.set_font_style(FontStyle::ITALIC);

        assert!(
            std::ptr::eq(
                &*context
                    .context
                    .font_group(ServoArc::new(style1.clone()))
                    .read(),
                &*context
                    .context
                    .font_group(ServoArc::new(style1.clone()))
                    .read()
            ),
            "the same font group should be returned for two styles with the same hash"
        );

        assert!(
            !std::ptr::eq(
                &*context
                    .context
                    .font_group(ServoArc::new(style1.clone()))
                    .read(),
                &*context
                    .context
                    .font_group(ServoArc::new(style2.clone()))
                    .read()
            ),
            "different font groups should be returned for two styles with different hashes"
        )
    }

    #[test]
    fn test_font_group_find_by_codepoint() {
        let mut context = TestContext::new();

        let mut style = style();
        style.set_font_family(font_family(vec!["CSSTest ASCII", "CSSTest Basic"]));

        let group = context.context.font_group(ServoArc::new(style));

        let font = group
            .write()
            .find_by_codepoint(&mut context.context, 'a', None, None)
            .unwrap();
        assert_eq!(&font_face_name(&font.identifier()), "csstest-ascii");
        assert_eq!(
            context
                .system_font_service
                .find_font_count
                .fetch_add(0, Ordering::Relaxed),
            1,
            "only the first font in the list should have been loaded"
        );

        let font = group
            .write()
            .find_by_codepoint(&mut context.context, 'a', None, None)
            .unwrap();
        assert_eq!(&font_face_name(&font.identifier()), "csstest-ascii");
        assert_eq!(
            context
                .system_font_service
                .find_font_count
                .fetch_add(0, Ordering::Relaxed),
            1,
            "we shouldn't load the same font a second time"
        );

        let font = group
            .write()
            .find_by_codepoint(&mut context.context, 'á', None, None)
            .unwrap();
        assert_eq!(&font_face_name(&font.identifier()), "csstest-basic-regular");
        assert_eq!(
            context
                .system_font_service
                .find_font_count
                .fetch_add(0, Ordering::Relaxed),
            2,
            "both fonts should now have been loaded"
        );
    }

    #[test]
    fn test_font_fallback() {
        let mut context = TestContext::new();

        let mut style = style();
        style.set_font_family(font_family(vec!["CSSTest ASCII"]));

        let group = context.context.font_group(ServoArc::new(style));

        let font = group
            .write()
            .find_by_codepoint(&mut context.context, 'a', None, None)
            .unwrap();
        assert_eq!(
            &font_face_name(&font.identifier()),
            "csstest-ascii",
            "a family in the group should be used if there is a matching glyph"
        );

        let font = group
            .write()
            .find_by_codepoint(&mut context.context, 'á', None, None)
            .unwrap();
        assert_eq!(
            &font_face_name(&font.identifier()),
            "csstest-basic-regular",
            "a fallback font should be used if there is no matching glyph in the group"
        );
    }

    #[test]
    fn test_font_template_is_cached() {
        let context = TestContext::new();

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

        let font_template = context
            .context
            .matching_templates(&font_descriptor, &family_descriptor)[0]
            .clone();

        let _ = context
            .context
            .matching_templates(&font_descriptor, &family_descriptor);

        assert_eq!(
            context
                .system_font_service
                .find_font_count
                .fetch_add(0, Ordering::Relaxed),
            1,
            "we should only have requested matching templates from the font service once"
        );

        let font1 = context
            .context
            .font(font_template.clone(), &font_descriptor)
            .unwrap();

        font_descriptor.pt_size = Au(20);
        let font2 = context
            .context
            .font(font_template.clone(), &font_descriptor)
            .unwrap();

        assert_ne!(
            font1.descriptor.pt_size, font2.descriptor.pt_size,
            "the same font should not have been returned"
        );

        assert_eq!(
            context
                .system_font_service
                .find_font_count
                .fetch_add(0, Ordering::Relaxed),
            1,
            "we should only have fetched the template data from the cache thread once"
        );
    }
}
