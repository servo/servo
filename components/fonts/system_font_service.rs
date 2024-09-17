/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::ops::{Deref, RangeInclusive};
use std::sync::Arc;
use std::{fmt, thread};

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::{ReentrantMutex, RwLock};
use serde::{Deserialize, Serialize};
use servo_config::pref;
use servo_url::ServoUrl;
use style::font_face::{FontFaceRuleData, FontStyle as FontFaceStyle};
use style::values::computed::font::{
    FixedPoint, FontStyleFixedPoint, GenericFontFamily, SingleFontFamily,
};
use style::values::computed::{FontStretch, FontWeight};
use style::values::specified::FontStretch as SpecifiedFontStretch;
use tracing::{span, Level};
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey};
use webrender_traits::WebRenderFontApi;

use crate::font::FontDescriptor;
use crate::font_store::FontStore;
use crate::font_template::{FontTemplate, FontTemplateRef};
use crate::platform::font_list::{
    default_system_generic_font_family, for_each_available_family, for_each_variation,
    LocalFontIdentifier,
};
use crate::FontData;

#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum FontIdentifier {
    Local(LocalFontIdentifier),
    Web(ServoUrl),
}

impl FontIdentifier {
    pub fn index(&self) -> u32 {
        match *self {
            Self::Local(ref local_font_identifier) => local_font_identifier.index(),
            Self::Web(_) => 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FontTemplateRequestResult {
    templates: Vec<FontTemplate>,
    template_data: Vec<(FontIdentifier, Arc<FontData>)>,
}

/// Commands that the `FontContext` sends to the `SystemFontService`.
#[derive(Debug, Deserialize, Serialize)]
pub enum Command {
    GetFontTemplates(
        Option<FontDescriptor>,
        SingleFontFamily,
        IpcSender<FontTemplateRequestResult>,
    ),
    GetFontInstance(
        FontIdentifier,
        Au,
        FontInstanceFlags,
        IpcSender<FontInstanceKey>,
    ),
    GetWebFont(Arc<FontData>, u32, IpcSender<FontKey>),
    GetWebFontInstance(FontKey, f32, FontInstanceFlags, IpcSender<FontInstanceKey>),
    Exit(IpcSender<()>),
    Ping,
}

#[derive(Default)]
struct ResolvedGenericFontFamilies {
    default: OnceCell<LowercaseFontFamilyName>,
    serif: OnceCell<LowercaseFontFamilyName>,
    sans_serif: OnceCell<LowercaseFontFamilyName>,
    monospace: OnceCell<LowercaseFontFamilyName>,
    fantasy: OnceCell<LowercaseFontFamilyName>,
    cursive: OnceCell<LowercaseFontFamilyName>,
    system_ui: OnceCell<LowercaseFontFamilyName>,
}

/// The system font service. There is one of these for every Servo instance. This is a thread,
/// responsible for reading the list of system fonts, handling requests to match against
/// them, and ensuring that only one copy of system font data is loaded at a time.
pub struct SystemFontService {
    port: IpcReceiver<Command>,
    local_families: FontStore,
    webrender_api: Box<dyn WebRenderFontApi>,
    webrender_fonts: HashMap<FontIdentifier, FontKey>,
    font_instances: HashMap<(FontKey, Au), FontInstanceKey>,
    generic_fonts: ResolvedGenericFontFamilies,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SystemFontServiceProxySender(IpcSender<Command>);

impl SystemFontServiceProxySender {
    pub fn to_proxy(&self) -> SystemFontServiceProxy {
        SystemFontServiceProxy {
            sender: ReentrantMutex::new(self.0.clone()),
            templates: Default::default(),
            data_cache: Default::default(),
        }
    }
}

impl SystemFontService {
    pub fn spawn(webrender_api: Box<dyn WebRenderFontApi + Send>) -> SystemFontServiceProxySender {
        let (sender, receiver) = ipc::channel().unwrap();

        thread::Builder::new()
            .name("SystemFontService".to_owned())
            .spawn(move || {
                #[allow(clippy::default_constructed_unit_structs)]
                let mut cache = SystemFontService {
                    port: receiver,
                    local_families: Default::default(),
                    webrender_api,
                    webrender_fonts: HashMap::new(),
                    font_instances: HashMap::new(),
                    generic_fonts: Default::default(),
                };

                cache.refresh_local_families();
                cache.run();
            })
            .expect("Thread spawning failed");

        SystemFontServiceProxySender(sender)
    }

    #[tracing::instrument(skip(self), fields(servo_profiling = true))]
    fn run(&mut self) {
        loop {
            let msg = self.port.recv().unwrap();

            match msg {
                Command::GetFontTemplates(font_descriptor, font_family, result_sender) => {
                    let span = span!(Level::TRACE, "GetFontTemplates", servo_profiling = true);
                    let _span = span.enter();
                    let _ =
                        result_sender.send(self.get_font_templates(font_descriptor, font_family));
                },
                Command::GetFontInstance(identifier, pt_size, flags, result) => {
                    let _ = result.send(self.get_font_instance(identifier, pt_size, flags));
                },
                Command::GetWebFont(data, font_index, result_sender) => {
                    self.webrender_api.forward_add_font_message(
                        data.as_ipc_shared_memory(),
                        font_index,
                        result_sender,
                    );
                },
                Command::GetWebFontInstance(
                    font_key,
                    font_size,
                    font_instance_flags,
                    result_sender,
                ) => {
                    self.webrender_api.forward_add_font_instance_message(
                        font_key,
                        font_size,
                        font_instance_flags,
                        result_sender,
                    );
                },
                Command::Ping => (),
                Command::Exit(result) => {
                    let _ = result.send(());
                    break;
                },
            }
        }
    }

    fn get_font_templates(
        &mut self,
        font_descriptor: Option<FontDescriptor>,
        font_family: SingleFontFamily,
    ) -> FontTemplateRequestResult {
        let templates = self.find_font_templates(font_descriptor.as_ref(), &font_family);
        let templates: Vec<_> = templates
            .into_iter()
            .map(|template| template.borrow().clone())
            .collect();

        // The `FontData` for all templates is also sent along with the `FontTemplate`s. This is to ensure that
        // the data is not read from disk in each content process. The data is loaded once here in the system
        // font service and each process gets another handle to the `IpcSharedMemory` view of that data.
        let template_data = templates
            .iter()
            .map(|template| {
                let identifier = template.identifier.clone();
                let data = self
                    .local_families
                    .get_or_initialize_font_data(&identifier)
                    .clone();
                (identifier, data)
            })
            .collect();

        FontTemplateRequestResult {
            templates,
            template_data,
        }
    }

    fn refresh_local_families(&mut self) {
        self.local_families.clear();
        for_each_available_family(|family_name| {
            self.local_families
                .families
                .entry(family_name.as_str().into())
                .or_default();
        });
    }

    #[tracing::instrument(skip(self), fields(servo_profiling = true))]
    fn find_font_templates(
        &mut self,
        descriptor_to_match: Option<&FontDescriptor>,
        family: &SingleFontFamily,
    ) -> Vec<FontTemplateRef> {
        // TODO(Issue #188): look up localized font family names if canonical name not found
        // look up canonical name
        let family_name = self.family_name_for_single_font_family(family);
        self.local_families
            .families
            .get_mut(&family_name)
            .map(|font_templates| {
                if font_templates.templates.is_empty() {
                    for_each_variation(&family_name, |font_template| {
                        font_templates.add_template(font_template);
                    });
                }

                font_templates.find_for_descriptor(descriptor_to_match)
            })
            .unwrap_or_default()
    }

    fn get_font_instance(
        &mut self,
        identifier: FontIdentifier,
        pt_size: Au,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        let webrender_font_api = &self.webrender_api;
        let webrender_fonts = &mut self.webrender_fonts;
        let font_data = self.local_families.get_or_initialize_font_data(&identifier);

        let font_key = *webrender_fonts
            .entry(identifier.clone())
            .or_insert_with(|| {
                // CoreText cannot reliably create CoreTextFonts for system fonts stored
                // as part of TTC files, so on CoreText platforms, create a system font in
                // WebRender using the LocalFontIdentifier. This has the downside of
                // causing the font to be loaded into memory again (bummer!), so only do
                // this for those platforms.
                #[cfg(target_os = "macos")]
                if let FontIdentifier::Local(local_font_identifier) = identifier {
                    return webrender_font_api
                        .add_system_font(local_font_identifier.native_font_handle());
                }

                webrender_font_api.add_font(font_data.as_ipc_shared_memory(), identifier.index())
            });

        *self
            .font_instances
            .entry((font_key, pt_size))
            .or_insert_with(|| {
                webrender_font_api.add_font_instance(font_key, pt_size.to_f32_px(), flags)
            })
    }

    pub(crate) fn family_name_for_single_font_family(
        &mut self,
        family: &SingleFontFamily,
    ) -> LowercaseFontFamilyName {
        let generic = match family {
            SingleFontFamily::FamilyName(family_name) => return family_name.name.clone().into(),
            SingleFontFamily::Generic(generic) => generic,
        };

        let resolved_font = match generic {
            GenericFontFamily::None => &self.generic_fonts.default,
            GenericFontFamily::Serif => &self.generic_fonts.serif,
            GenericFontFamily::SansSerif => &self.generic_fonts.sans_serif,
            GenericFontFamily::Monospace => &self.generic_fonts.monospace,
            GenericFontFamily::Cursive => &self.generic_fonts.cursive,
            GenericFontFamily::Fantasy => &self.generic_fonts.fantasy,
            GenericFontFamily::SystemUi => &self.generic_fonts.system_ui,
        };

        resolved_font
            .get_or_init(|| {
                // First check whether the font is set in the preferences.
                let family_name = match generic {
                    GenericFontFamily::None => pref!(fonts.default),
                    GenericFontFamily::Serif => pref!(fonts.serif),
                    GenericFontFamily::SansSerif => pref!(fonts.sans_serif),
                    GenericFontFamily::Monospace => pref!(fonts.monospace),
                    _ => String::new(),
                };

                if !family_name.is_empty() {
                    return family_name.into();
                }

                // Otherwise ask the platform for the default family for the generic font.
                default_system_generic_font_family(*generic)
            })
            .clone()
    }
}

/// A trait for accessing the [`SystemFontServiceProxy`] necessary for unit testing.
pub trait SystemFontServiceProxyTrait: Send + Sync {
    fn find_matching_font_templates(
        &self,
        descriptor_to_match: Option<&FontDescriptor>,
        font_family_name: &SingleFontFamily,
    ) -> Vec<FontTemplateRef>;
    fn get_system_font_instance(
        &self,
        font_identifier: FontIdentifier,
        size: Au,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey;
    fn get_web_font(&self, data: Arc<FontData>, index: u32) -> FontKey;
    fn get_web_font_instance(
        &self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey;
    fn get_font_data(&self, identifier: &FontIdentifier) -> Option<Arc<FontData>>;
}

#[derive(Debug, Eq, Hash, MallocSizeOf, PartialEq)]
struct FontTemplateCacheKey {
    font_descriptor: Option<FontDescriptor>,
    family_descriptor: SingleFontFamily,
}

/// The public interface to the [`SystemFontService`], used by per-Document `FontContext`
/// instances (via [`SystemFontServiceProxyTrait`]).
#[derive(Debug)]
pub struct SystemFontServiceProxy {
    sender: ReentrantMutex<IpcSender<Command>>,
    templates: RwLock<HashMap<FontTemplateCacheKey, Vec<FontTemplateRef>>>,
    data_cache: RwLock<HashMap<FontIdentifier, Arc<FontData>>>,
}

/// A version of `FontStyle` from Stylo that is serializable. Normally this is not
/// because the specified version of `FontStyle` contains floats.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ComputedFontStyleDescriptor {
    Normal,
    Italic,
    Oblique(FontStyleFixedPoint, FontStyleFixedPoint),
}

/// This data structure represents the various optional descriptors that can be
/// applied to a `@font-face` rule in CSS. These are used to create a [`FontTemplate`]
/// from the given font data used as the source of the `@font-face` rule. If values
/// like weight, stretch, and style are not specified they are initialized based
/// on the contents of the font itself.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CSSFontFaceDescriptors {
    pub family_name: LowercaseFontFamilyName,
    pub weight: Option<(FontWeight, FontWeight)>,
    pub stretch: Option<(FontStretch, FontStretch)>,
    pub style: Option<ComputedFontStyleDescriptor>,
    pub unicode_range: Option<Vec<RangeInclusive<u32>>>,
}

impl CSSFontFaceDescriptors {
    pub fn new(family_name: &str) -> Self {
        CSSFontFaceDescriptors {
            family_name: family_name.into(),
            ..Default::default()
        }
    }
}

impl From<&FontFaceRuleData> for CSSFontFaceDescriptors {
    fn from(rule_data: &FontFaceRuleData) -> Self {
        let family_name = rule_data
            .family
            .as_ref()
            .expect("Expected rule to contain a font family.")
            .name
            .clone();
        let weight = rule_data
            .weight
            .as_ref()
            .map(|weight_range| (weight_range.0.compute(), weight_range.1.compute()));

        let stretch_to_computed = |specified: SpecifiedFontStretch| match specified {
            SpecifiedFontStretch::Stretch(percentage) => {
                FontStretch::from_percentage(percentage.compute().0)
            },
            SpecifiedFontStretch::Keyword(keyword) => keyword.compute(),
            SpecifiedFontStretch::System(_) => FontStretch::NORMAL,
        };
        let stretch = rule_data.stretch.as_ref().map(|stretch_range| {
            (
                stretch_to_computed(stretch_range.0),
                stretch_to_computed(stretch_range.1),
            )
        });

        fn style_to_computed(specified: &FontFaceStyle) -> ComputedFontStyleDescriptor {
            match specified {
                FontFaceStyle::Normal => ComputedFontStyleDescriptor::Normal,
                FontFaceStyle::Italic => ComputedFontStyleDescriptor::Italic,
                FontFaceStyle::Oblique(angle_a, angle_b) => ComputedFontStyleDescriptor::Oblique(
                    FixedPoint::from_float(angle_a.degrees()),
                    FixedPoint::from_float(angle_b.degrees()),
                ),
            }
        }
        let style = rule_data.style.as_ref().map(style_to_computed);
        let unicode_range = rule_data
            .unicode_range
            .as_ref()
            .map(|ranges| ranges.iter().map(|range| range.start..=range.end).collect());

        CSSFontFaceDescriptors {
            family_name: family_name.into(),
            weight,
            stretch,
            style,
            unicode_range,
        }
    }
}

impl SystemFontServiceProxy {
    pub fn exit(&self) {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.sender
            .lock()
            .send(Command::Exit(response_chan))
            .expect("Couldn't send SystemFontService exit message");
        response_port
            .recv()
            .expect("Couldn't receive SystemFontService reply");
    }

    pub fn to_sender(&self) -> SystemFontServiceProxySender {
        SystemFontServiceProxySender(self.sender.lock().clone())
    }
}

impl SystemFontServiceProxyTrait for SystemFontServiceProxy {
    fn get_system_font_instance(
        &self,
        identifier: FontIdentifier,
        size: Au,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.sender
            .lock()
            .send(Command::GetFontInstance(
                identifier,
                size,
                flags,
                response_chan,
            ))
            .expect("failed to send message to system font service");

        let instance_key = response_port.recv();
        if instance_key.is_err() {
            let font_thread_has_closed = self.sender.lock().send(Command::Ping).is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("SystemFontService has already exited.");
        }
        instance_key.unwrap()
    }

    fn find_matching_font_templates(
        &self,
        descriptor_to_match: Option<&FontDescriptor>,
        family_descriptor: &SingleFontFamily,
    ) -> Vec<FontTemplateRef> {
        let cache_key = FontTemplateCacheKey {
            font_descriptor: descriptor_to_match.cloned(),
            family_descriptor: family_descriptor.clone(),
        };
        if let Some(templates) = self.templates.read().get(&cache_key).cloned() {
            return templates;
        }

        debug!(
            "SystemFontServiceProxy: cache miss for template_descriptor={:?} family_descriptor={:?}",
            descriptor_to_match, family_descriptor
        );

        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.sender
            .lock()
            .send(Command::GetFontTemplates(
                descriptor_to_match.cloned(),
                family_descriptor.clone(),
                response_chan,
            ))
            .expect("failed to send message to system font service");

        let reply = response_port.recv();
        let Ok(reply) = reply else {
            let font_thread_has_closed = self.sender.lock().send(Command::Ping).is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("SystemFontService has already exited.");
        };

        let templates: Vec<_> = reply
            .templates
            .into_iter()
            .map(AtomicRefCell::new)
            .map(Arc::new)
            .collect();
        self.data_cache.write().extend(reply.template_data);

        templates
    }

    fn get_web_font(&self, data: Arc<FontData>, index: u32) -> FontKey {
        let (result_sender, result_receiver) =
            ipc::channel().expect("failed to create IPC channel");
        let _ = self
            .sender
            .lock()
            .send(Command::GetWebFont(data, index, result_sender));
        result_receiver.recv().unwrap()
    }

    fn get_web_font_instance(
        &self,
        font_key: FontKey,
        font_size: f32,
        font_flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        let (result_sender, result_receiver) =
            ipc::channel().expect("failed to create IPC channel");
        let _ = self.sender.lock().send(Command::GetWebFontInstance(
            font_key,
            font_size,
            font_flags,
            result_sender,
        ));
        result_receiver.recv().unwrap()
    }

    fn get_font_data(&self, identifier: &FontIdentifier) -> Option<Arc<FontData>> {
        self.data_cache.read().get(identifier).cloned()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LowercaseFontFamilyName {
    inner: String,
}

impl<T: AsRef<str>> From<T> for LowercaseFontFamilyName {
    fn from(value: T) -> Self {
        LowercaseFontFamilyName {
            inner: value.as_ref().to_lowercase(),
        }
    }
}

impl Deref for LowercaseFontFamilyName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for LowercaseFontFamilyName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}
