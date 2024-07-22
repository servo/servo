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
use ipc_channel::ipc::{self, IpcBytesReceiver, IpcBytesSender, IpcReceiver, IpcSender};
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use servo_url::ServoUrl;
use style::font_face::{FontFaceRuleData, FontStyle as FontFaceStyle};
use style::values::computed::font::{
    FixedPoint, FontStyleFixedPoint, GenericFontFamily, SingleFontFamily,
};
use style::values::computed::{FontStretch, FontWeight};
use style::values::specified::FontStretch as SpecifiedFontStretch;
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey};
use webrender_traits::WebRenderFontApi;

use crate::font::FontDescriptor;
use crate::font_store::FontStore;
use crate::font_template::{
    FontTemplate, FontTemplateDescriptor, FontTemplateRef, FontTemplateRefMethods,
};
use crate::platform::font_list::{
    default_system_generic_font_family, for_each_available_family, for_each_variation,
    LocalFontIdentifier,
};

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
pub struct SerializedFontTemplate {
    identifier: FontIdentifier,
    descriptor: FontTemplateDescriptor,
    bytes_receiver: ipc_channel::ipc::IpcBytesReceiver,
}

/// Commands that the FontContext sends to the font cache thread.
#[derive(Debug, Deserialize, Serialize)]
pub enum Command {
    GetFontTemplates(
        Option<FontDescriptor>,
        SingleFontFamily,
        IpcSender<Vec<SerializedFontTemplate>>,
    ),
    GetFontInstance(
        FontIdentifier,
        Au,
        FontInstanceFlags,
        IpcSender<FontInstanceKey>,
    ),
    GetWebFont(IpcBytesReceiver, u32, IpcSender<FontKey>),
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

/// The font cache thread itself. It maintains a list of reference counted
/// font templates that are currently in use.
struct FontCache {
    port: IpcReceiver<Command>,
    font_data: HashMap<FontIdentifier, Arc<Vec<u8>>>,
    local_families: FontStore,
    webrender_api: Box<dyn WebRenderFontApi>,
    webrender_fonts: HashMap<FontIdentifier, FontKey>,
    font_instances: HashMap<(FontKey, Au), FontInstanceKey>,
    generic_fonts: ResolvedGenericFontFamilies,
}

impl FontCache {
    fn run(&mut self) {
        loop {
            let msg = self.port.recv().unwrap();

            match msg {
                Command::GetFontTemplates(descriptor_to_match, font_family, result) => {
                    let templates =
                        self.find_font_templates(descriptor_to_match.as_ref(), &font_family);
                    debug!("Found templates for descriptor {descriptor_to_match:?}: ");
                    debug!("  {templates:?}");

                    let (serialized_templates, senders): (
                        Vec<SerializedFontTemplate>,
                        Vec<(FontTemplateRef, IpcBytesSender)>,
                    ) = templates
                        .into_iter()
                        .map(|template| {
                            let (bytes_sender, bytes_receiver) =
                                ipc::bytes_channel().expect("failed to create IPC channel");
                            (
                                SerializedFontTemplate {
                                    identifier: template.identifier().clone(),
                                    descriptor: template.descriptor().clone(),
                                    bytes_receiver,
                                },
                                (template.clone(), bytes_sender),
                            )
                        })
                        .unzip();

                    let _ = result.send(serialized_templates);

                    // NB: This will load the font into memory if it hasn't been loaded already.
                    for (font_template, bytes_sender) in senders.iter() {
                        let identifier = font_template.identifier();
                        let data = self
                            .font_data
                            .entry(identifier)
                            .or_insert_with(|| font_template.data());
                        let _ = bytes_sender.send(data);
                    }
                },
                Command::GetFontInstance(identifier, pt_size, flags, result) => {
                    let _ = result.send(self.get_font_instance(identifier, pt_size, flags));
                },
                Command::GetWebFont(bytes_receiver, font_index, result_sender) => {
                    self.webrender_api.forward_add_font_message(
                        bytes_receiver,
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

    fn refresh_local_families(&mut self) {
        self.local_families.clear();
        for_each_available_family(|family_name| {
            self.local_families
                .families
                .entry(family_name.as_str().into())
                .or_default();
        });
    }

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
        let font_data = self
            .font_data
            .get(&identifier)
            .expect("Got unexpected FontIdentifier")
            .clone();

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

                webrender_font_api.add_font(font_data, identifier.index())
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

pub trait FontSource: Clone {
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
    fn get_web_font(&self, data: Arc<Vec<u8>>, index: u32) -> FontKey;
    fn get_web_font_instance(
        &self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey;
}

/// The public interface to the font cache thread, used by per-thread `FontContext` instances (via
/// the `FontSource` trait), and also by layout.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontCacheThread {
    chan: IpcSender<Command>,
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

impl FontCacheThread {
    pub fn new(webrender_api: Box<dyn WebRenderFontApi + Send>) -> FontCacheThread {
        let (chan, port) = ipc::channel().unwrap();

        thread::Builder::new()
            .name("FontCache".to_owned())
            .spawn(move || {
                #[allow(clippy::default_constructed_unit_structs)]
                let mut cache = FontCache {
                    port,
                    font_data: HashMap::new(),
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

        FontCacheThread { chan }
    }

    pub fn exit(&self) {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.chan
            .send(Command::Exit(response_chan))
            .expect("Couldn't send FontCacheThread exit message");
        response_port
            .recv()
            .expect("Couldn't receive FontCacheThread reply");
    }
}

impl FontSource for FontCacheThread {
    fn get_system_font_instance(
        &self,
        identifier: FontIdentifier,
        size: Au,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.chan
            .send(Command::GetFontInstance(
                identifier,
                size,
                flags,
                response_chan,
            ))
            .expect("failed to send message to font cache thread");

        let instance_key = response_port.recv();
        if instance_key.is_err() {
            let font_thread_has_closed = self.chan.send(Command::Ping).is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("Font cache thread has already exited.");
        }
        instance_key.unwrap()
    }

    fn find_matching_font_templates(
        &self,
        descriptor_to_match: Option<&FontDescriptor>,
        font_family: &SingleFontFamily,
    ) -> Vec<FontTemplateRef> {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.chan
            .send(Command::GetFontTemplates(
                descriptor_to_match.cloned(),
                font_family.clone(),
                response_chan,
            ))
            .expect("failed to send message to font cache thread");

        let reply = response_port.recv();
        if reply.is_err() {
            let font_thread_has_closed = self.chan.send(Command::Ping).is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("Font cache thread has already exited.");
        }

        reply
            .unwrap()
            .into_iter()
            .map(|serialized_font_template| {
                let font_data = serialized_font_template.bytes_receiver.recv().ok();
                Arc::new(AtomicRefCell::new(FontTemplate {
                    identifier: serialized_font_template.identifier,
                    descriptor: serialized_font_template.descriptor.clone(),
                    data: font_data.map(Arc::new),
                    stylesheet: None,
                }))
            })
            .collect()
    }

    fn get_web_font(&self, data: Arc<Vec<u8>>, index: u32) -> FontKey {
        let (result_sender, result_receiver) =
            ipc::channel().expect("failed to create IPC channel");
        let (bytes_sender, bytes_receiver) =
            ipc::bytes_channel().expect("failed to create IPC channel");
        let _ = self
            .chan
            .send(Command::GetWebFont(bytes_receiver, index, result_sender));
        let _ = bytes_sender.send(&data);
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
        let _ = self.chan.send(Command::GetWebFontInstance(
            font_key,
            font_size,
            font_flags,
            result_sender,
        ));
        result_receiver.recv().unwrap()
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
