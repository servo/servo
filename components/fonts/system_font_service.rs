/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::thread;

use app_units::Au;
use compositing_traits::CrossProcessCompositorApi;
use fonts_traits::{
    FontDescriptor, FontIdentifier, FontTemplate, FontTemplateRef, LowercaseFontFamilyName,
};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use log::debug;
use malloc_size_of::MallocSizeOf as MallocSizeOfTrait;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::{Mutex, RwLock};
use profile_traits::mem::{
    ProcessReports, ProfilerChan, Report, ReportKind, ReportsChan, perform_memory_report,
};
use profile_traits::path;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use style::values::computed::font::{GenericFontFamily, SingleFontFamily};
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey, FontVariation};

use crate::font_store::FontStore;
use crate::platform::font_list::{
    default_system_generic_font_family, for_each_available_family, for_each_variation,
};

/// Commands that the `FontContext` sends to the `SystemFontService`.
#[derive(Debug, Deserialize, Serialize)]
pub enum SystemFontServiceMessage {
    GetFontTemplates(
        Option<FontDescriptor>,
        SingleFontFamily,
        IpcSender<Vec<FontTemplate>>,
    ),
    GetFontInstance(
        FontIdentifier,
        Au,
        FontInstanceFlags,
        Vec<FontVariation>,
        IpcSender<FontInstanceKey>,
    ),
    GetFontKey(IpcSender<FontKey>),
    GetFontInstanceKey(IpcSender<FontInstanceKey>),
    CollectMemoryReport(ReportsChan),
    Exit(IpcSender<()>),
    Ping,
}

#[derive(Default, MallocSizeOf)]
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
#[derive(MallocSizeOf)]
pub struct SystemFontService {
    port: IpcReceiver<SystemFontServiceMessage>,
    local_families: FontStore,
    compositor_api: CrossProcessCompositorApi,
    webrender_fonts: HashMap<FontIdentifier, FontKey>,
    font_instances: HashMap<(FontKey, Au, Vec<FontVariation>), FontInstanceKey>,
    generic_fonts: ResolvedGenericFontFamilies,

    /// This is an optimization that allows the [`SystemFontService`] to send font data to
    /// the compositor asynchronously for creating WebRender fonts, while immediately
    /// returning a font key for that data. Once the free keys are exhausted, the
    /// [`SystemFontService`] will fetch a new batch.
    free_font_keys: Vec<FontKey>,

    /// This is an optimization that allows the [`SystemFontService`] to create WebRender font
    /// instances in the compositor asynchronously, while immediately returning a font
    /// instance key for the instance. Once the free keys are exhausted, the
    /// [`SystemFontService`] will fetch a new batch.
    free_font_instance_keys: Vec<FontInstanceKey>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SystemFontServiceProxySender(pub IpcSender<SystemFontServiceMessage>);

impl SystemFontServiceProxySender {
    pub fn to_proxy(&self) -> SystemFontServiceProxy {
        SystemFontServiceProxy {
            sender: Mutex::new(self.0.clone()),
            templates: Default::default(),
        }
    }
}

impl SystemFontService {
    pub fn spawn(
        compositor_api: CrossProcessCompositorApi,
        memory_profiler_sender: ProfilerChan,
    ) -> SystemFontServiceProxySender {
        let (sender, receiver) = ipc::channel().unwrap();
        let memory_reporter_sender = sender.clone();

        thread::Builder::new()
            .name("SystemFontService".to_owned())
            .spawn(move || {
                #[allow(clippy::default_constructed_unit_structs)]
                let mut cache = SystemFontService {
                    port: receiver,
                    local_families: Default::default(),
                    compositor_api,
                    webrender_fonts: HashMap::new(),
                    font_instances: HashMap::new(),
                    generic_fonts: Default::default(),
                    free_font_keys: Default::default(),
                    free_font_instance_keys: Default::default(),
                };

                cache.fetch_new_keys();
                cache.refresh_local_families();

                memory_profiler_sender.run_with_memory_reporting(
                    || cache.run(),
                    "system-fonts".to_owned(),
                    memory_reporter_sender,
                    SystemFontServiceMessage::CollectMemoryReport,
                );
            })
            .expect("Thread spawning failed");

        SystemFontServiceProxySender(sender)
    }

    fn run(&mut self) {
        loop {
            let msg = self.port.recv().unwrap();

            #[cfg(feature = "tracing")]
            let _span =
                tracing::trace_span!("SystemFontServiceMessage", servo_profiling = true).entered();
            match msg {
                SystemFontServiceMessage::GetFontTemplates(
                    font_descriptor,
                    font_family,
                    result_sender,
                ) => {
                    let _ =
                        result_sender.send(self.get_font_templates(font_descriptor, font_family));
                },
                SystemFontServiceMessage::GetFontInstance(
                    identifier,
                    pt_size,
                    flags,
                    variations,
                    result,
                ) => {
                    let _ =
                        result.send(self.get_font_instance(identifier, pt_size, flags, variations));
                },
                SystemFontServiceMessage::GetFontKey(result_sender) => {
                    self.fetch_new_keys();
                    let _ = result_sender.send(self.free_font_keys.pop().unwrap());
                },
                SystemFontServiceMessage::GetFontInstanceKey(result_sender) => {
                    self.fetch_new_keys();
                    let _ = result_sender.send(self.free_font_instance_keys.pop().unwrap());
                },
                SystemFontServiceMessage::CollectMemoryReport(report_sender) => {
                    self.collect_memory_report(report_sender);
                },
                SystemFontServiceMessage::Ping => (),
                SystemFontServiceMessage::Exit(result) => {
                    let _ = result.send(());
                    break;
                },
            }
        }
    }

    fn collect_memory_report(&self, report_sender: ReportsChan) {
        perform_memory_report(|ops| {
            let reports = vec![Report {
                path: path!["system-fonts"],
                kind: ReportKind::ExplicitSystemHeapSize,
                size: self.size_of(ops),
            }];
            report_sender.send(ProcessReports::new(reports));
        });
    }

    #[servo_tracing::instrument(skip_all)]
    fn fetch_new_keys(&mut self) {
        if !self.free_font_keys.is_empty() && !self.free_font_instance_keys.is_empty() {
            return;
        }

        const FREE_FONT_KEYS_BATCH_SIZE: usize = 40;
        const FREE_FONT_INSTANCE_KEYS_BATCH_SIZE: usize = 40;
        let (mut new_font_keys, mut new_font_instance_keys) = self.compositor_api.fetch_font_keys(
            FREE_FONT_KEYS_BATCH_SIZE - self.free_font_keys.len(),
            FREE_FONT_INSTANCE_KEYS_BATCH_SIZE - self.free_font_instance_keys.len(),
        );
        self.free_font_keys.append(&mut new_font_keys);
        self.free_font_instance_keys
            .append(&mut new_font_instance_keys);
    }

    #[servo_tracing::instrument(skip_all)]
    fn get_font_templates(
        &mut self,
        font_descriptor: Option<FontDescriptor>,
        font_family: SingleFontFamily,
    ) -> Vec<FontTemplate> {
        let templates = self.find_font_templates(font_descriptor.as_ref(), &font_family);
        templates
            .into_iter()
            .map(|template| template.borrow().clone())
            .collect()
    }

    #[servo_tracing::instrument(skip_all)]
    fn refresh_local_families(&mut self) {
        self.local_families.clear();
        for_each_available_family(|family_name| {
            self.local_families
                .families
                .entry(family_name.as_str().into())
                .or_default();
        });
    }

    #[servo_tracing::instrument(skip_all)]
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

    #[servo_tracing::instrument(skip_all)]
    fn get_font_instance(
        &mut self,
        identifier: FontIdentifier,
        pt_size: Au,
        flags: FontInstanceFlags,
        variations: Vec<FontVariation>,
    ) -> FontInstanceKey {
        self.fetch_new_keys();

        let compositor_api = &self.compositor_api;
        let webrender_fonts = &mut self.webrender_fonts;

        let font_key = *webrender_fonts
            .entry(identifier.clone())
            .or_insert_with(|| {
                let font_key = self.free_font_keys.pop().unwrap();
                let FontIdentifier::Local(local_font_identifier) = identifier else {
                    unreachable!("Should never have a web font in the system font service");
                };
                compositor_api
                    .add_system_font(font_key, local_font_identifier.native_font_handle());
                font_key
            });

        *self
            .font_instances
            .entry((font_key, pt_size, variations.clone()))
            .or_insert_with(|| {
                let font_instance_key = self.free_font_instance_keys.pop().unwrap();
                compositor_api.add_font_instance(
                    font_instance_key,
                    font_key,
                    pt_size.to_f32_px(),
                    flags,
                    variations,
                );
                font_instance_key
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
                    GenericFontFamily::None => pref!(fonts_default),
                    GenericFontFamily::Serif => pref!(fonts_serif),
                    GenericFontFamily::SansSerif => pref!(fonts_sans_serif),
                    GenericFontFamily::Monospace => pref!(fonts_monospace),
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

#[derive(Debug, Eq, Hash, MallocSizeOf, PartialEq)]
struct FontTemplateCacheKey {
    font_descriptor: Option<FontDescriptor>,
    family_descriptor: SingleFontFamily,
}

/// The public interface to the [`SystemFontService`], used by per-Document
/// `FontContext` instances.
#[derive(Debug, MallocSizeOf)]
pub struct SystemFontServiceProxy {
    sender: Mutex<IpcSender<SystemFontServiceMessage>>,
    templates: RwLock<HashMap<FontTemplateCacheKey, Vec<FontTemplateRef>>>,
}

impl SystemFontServiceProxy {
    pub fn exit(&self) {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.sender
            .lock()
            .send(SystemFontServiceMessage::Exit(response_chan))
            .expect("Couldn't send SystemFontService exit message");
        response_port
            .recv()
            .expect("Couldn't receive SystemFontService reply");
    }

    pub fn to_sender(&self) -> SystemFontServiceProxySender {
        SystemFontServiceProxySender(self.sender.lock().clone())
    }

    pub(crate) fn get_system_font_instance(
        &self,
        identifier: FontIdentifier,
        size: Au,
        flags: FontInstanceFlags,
        variations: Vec<FontVariation>,
    ) -> FontInstanceKey {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.sender
            .lock()
            .send(SystemFontServiceMessage::GetFontInstance(
                identifier,
                size,
                flags,
                variations,
                response_chan,
            ))
            .expect("failed to send message to system font service");

        let instance_key = response_port.recv();
        if instance_key.is_err() {
            let font_thread_has_closed = self
                .sender
                .lock()
                .send(SystemFontServiceMessage::Ping)
                .is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("SystemFontService has already exited.");
        }
        instance_key.unwrap()
    }

    pub(crate) fn find_matching_font_templates(
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
            .send(SystemFontServiceMessage::GetFontTemplates(
                descriptor_to_match.cloned(),
                family_descriptor.clone(),
                response_chan,
            ))
            .expect("failed to send message to system font service");

        let Ok(templates) = response_port.recv() else {
            let font_thread_has_closed = self
                .sender
                .lock()
                .send(SystemFontServiceMessage::Ping)
                .is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("SystemFontService has already exited.");
        };

        let templates: Vec<_> = templates.into_iter().map(FontTemplateRef::new).collect();
        self.templates.write().insert(cache_key, templates.clone());

        templates
    }

    pub(crate) fn generate_font_key(&self) -> FontKey {
        let (result_sender, result_receiver) =
            ipc::channel().expect("failed to create IPC channel");
        self.sender
            .lock()
            .send(SystemFontServiceMessage::GetFontKey(result_sender))
            .expect("failed to send message to system font service");
        result_receiver
            .recv()
            .expect("Failed to communicate with system font service.")
    }

    pub(crate) fn generate_font_instance_key(&self) -> FontInstanceKey {
        let (result_sender, result_receiver) =
            ipc::channel().expect("failed to create IPC channel");
        self.sender
            .lock()
            .send(SystemFontServiceMessage::GetFontInstanceKey(result_sender))
            .expect("failed to send message to system font service");
        result_receiver
            .recv()
            .expect("Failed to communicate with system font service.")
    }
}
