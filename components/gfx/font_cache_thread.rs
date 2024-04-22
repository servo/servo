/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{f32, fmt, mem, thread};

use app_units::Au;
use gfx_traits::WebrenderApi;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use log::{debug, trace};
use net_traits::request::{Destination, Referrer, RequestBuilder};
use net_traits::{fetch_async, CoreResourceThread, FetchResponseMsg};
use serde::{Deserialize, Serialize};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use style::font_face::{FontFaceSourceFormat, FontFaceSourceFormatKeyword, Source};
use style::media_queries::Device;
use style::shared_lock::SharedRwLockReadGuard;
use style::stylesheets::{Stylesheet, StylesheetInDocument};
use webrender_api::{FontInstanceKey, FontKey};

use crate::font::{FontFamilyDescriptor, FontFamilyName, FontSearchScope, PlatformFontMethods};
use crate::font_context::FontSource;
use crate::font_template::{
    FontTemplate, FontTemplateDescriptor, FontTemplateRef, FontTemplateRefMethods,
};
use crate::platform::font::PlatformFont;
use crate::platform::font_list::{
    for_each_available_family, for_each_variation, system_default_family, LocalFontIdentifier,
    SANS_SERIF_FONT_FAMILY,
};

/// A list of font templates that make up a given font family.
#[derive(Default)]
pub struct FontTemplates {
    templates: Vec<FontTemplateRef>,
}

#[derive(Clone, Debug)]
pub struct FontTemplateAndWebRenderFontKey {
    pub font_template: FontTemplateRef,
    pub font_key: FontKey,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SerializedFontTemplateAndWebRenderFontKey {
    pub serialized_font_template: SerializedFontTemplate,
    pub font_key: FontKey,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

impl SerializedFontTemplate {
    pub fn to_font_template(&self) -> FontTemplate {
        let font_data = self.bytes_receiver.recv().ok();
        FontTemplate {
            identifier: self.identifier.clone(),
            descriptor: self.descriptor,
            data: font_data.map(Arc::new),
        }
    }
}

impl FontTemplates {
    /// Find a font in this family that matches a given descriptor.
    pub fn find_font_for_style(
        &mut self,
        desc: &FontTemplateDescriptor,
    ) -> Option<FontTemplateRef> {
        // TODO(Issue #189): optimize lookup for
        // regular/bold/italic/bolditalic with fixed offsets and a
        // static decision table for fallback between these values.
        for template in &mut self.templates {
            if template.descriptor_matches(desc) {
                return Some(template.clone());
            }
        }

        // We didn't find an exact match. Do more expensive fuzzy matching.
        // TODO(#190): Do a better job.
        let (mut best_template, mut best_distance) = (None, f32::MAX);
        for template in self.templates.iter() {
            let distance = template.descriptor_distance(desc);
            if distance < best_distance {
                best_template = Some(template);
                best_distance = distance
            }
        }
        if best_template.is_some() {
            return best_template.cloned();
        }

        // If a request is made for a font family that exists,
        // pick the first valid font in the family if we failed
        // to find an exact match for the descriptor.
        for template in &mut self.templates.iter() {
            return Some(template.clone());
        }

        None
    }

    pub fn add_template(&mut self, new_template: FontTemplate) {
        for existing_template in &self.templates {
            let existing_template = existing_template.borrow();
            if *existing_template.identifier() == new_template.identifier &&
                existing_template.descriptor == new_template.descriptor
            {
                return;
            }
        }
        self.templates.push(Rc::new(RefCell::new(new_template)));
    }
}

/// Commands that the FontContext sends to the font cache thread.
#[derive(Debug, Deserialize, Serialize)]
pub enum Command {
    GetFontTemplate(
        FontTemplateDescriptor,
        FontFamilyDescriptor,
        IpcSender<Reply>,
    ),
    GetFontInstance(FontKey, Au, IpcSender<FontInstanceKey>),
    AddWebFont(LowercaseString, Vec<Source>, IpcSender<()>),
    AddDownloadedWebFont(LowercaseString, ServoUrl, Vec<u8>, IpcSender<()>),
    Exit(IpcSender<()>),
    Ping,
}

/// Reply messages sent from the font cache thread to the FontContext caller.
#[derive(Debug, Deserialize, Serialize)]
pub enum Reply {
    GetFontTemplateReply(Option<SerializedFontTemplateAndWebRenderFontKey>),
}

/// The font cache thread itself. It maintains a list of reference counted
/// font templates that are currently in use.
struct FontCache {
    port: IpcReceiver<Command>,
    channel_to_self: IpcSender<Command>,
    generic_fonts: HashMap<FontFamilyName, LowercaseString>,
    local_families: HashMap<LowercaseString, FontTemplates>,
    web_families: HashMap<LowercaseString, FontTemplates>,
    core_resource_thread: CoreResourceThread,
    webrender_api: Box<dyn WebrenderApi>,
    webrender_fonts: HashMap<FontIdentifier, FontKey>,
    font_instances: HashMap<(FontKey, Au), FontInstanceKey>,
}

fn populate_generic_fonts() -> HashMap<FontFamilyName, LowercaseString> {
    let mut generic_fonts = HashMap::with_capacity(5);

    append_map(&mut generic_fonts, "serif", "Times New Roman");
    append_map(&mut generic_fonts, "sans-serif", SANS_SERIF_FONT_FAMILY);
    append_map(&mut generic_fonts, "cursive", "Apple Chancery");
    append_map(&mut generic_fonts, "fantasy", "Papyrus");
    append_map(&mut generic_fonts, "monospace", "Menlo");

    fn append_map(
        generic_fonts: &mut HashMap<FontFamilyName, LowercaseString>,
        generic_name: &str,
        mapped_name: &str,
    ) {
        let family_name = match system_default_family(generic_name) {
            Some(system_default) => LowercaseString::new(&system_default),
            None => LowercaseString::new(mapped_name),
        };

        let generic_name = FontFamilyName::Generic(Atom::from(generic_name));

        generic_fonts.insert(generic_name, family_name);
    }

    generic_fonts
}

impl FontCache {
    fn run(&mut self) {
        loop {
            let msg = self.port.recv().unwrap();

            match msg {
                Command::GetFontTemplate(template_descriptor, family_descriptor, result) => {
                    let Some(font_template_info) =
                        self.find_font_template(&template_descriptor, &family_descriptor)
                    else {
                        let _ = result.send(Reply::GetFontTemplateReply(None));
                        continue;
                    };

                    let (bytes_sender, bytes_receiver) =
                        ipc::bytes_channel().expect("failed to create IPC channel");
                    let serialized_font_template = SerializedFontTemplate {
                        identifier: font_template_info.font_template.borrow().identifier.clone(),
                        descriptor: font_template_info.font_template.borrow().descriptor,
                        bytes_receiver,
                    };

                    let _ = result.send(Reply::GetFontTemplateReply(Some(
                        SerializedFontTemplateAndWebRenderFontKey {
                            serialized_font_template,
                            font_key: font_template_info.font_key,
                        },
                    )));

                    // NB: This will load the font into memory if it hasn't been loaded already.
                    let _ = bytes_sender.send(&font_template_info.font_template.data());
                },
                Command::GetFontInstance(font_key, size, result) => {
                    let webrender_api = &self.webrender_api;

                    let instance_key =
                        *self
                            .font_instances
                            .entry((font_key, size))
                            .or_insert_with(|| {
                                webrender_api.add_font_instance(font_key, size.to_f32_px())
                            });

                    let _ = result.send(instance_key);
                },
                Command::AddWebFont(family_name, sources, result) => {
                    self.handle_add_web_font(family_name, sources, result);
                },
                Command::AddDownloadedWebFont(family_name, url, bytes, result) => {
                    let templates = &mut self.web_families.get_mut(&family_name).unwrap();

                    let data = Arc::new(bytes);
                    let identifier = FontIdentifier::Web(url.clone());
                    let Ok(handle) = PlatformFont::new_from_data(identifier, data.clone(), 0, None)
                    else {
                        drop(result.send(()));
                        return;
                    };

                    let descriptor = FontTemplateDescriptor::new(
                        handle.boldness(),
                        handle.stretchiness(),
                        handle.style(),
                    );

                    templates.add_template(FontTemplate::new_web_font(url, descriptor, data));
                    drop(result.send(()));
                },
                Command::Ping => (),
                Command::Exit(result) => {
                    let _ = result.send(());
                    break;
                },
            }
        }
    }

    fn handle_add_web_font(
        &mut self,
        family_name: LowercaseString,
        mut sources: Vec<Source>,
        sender: IpcSender<()>,
    ) {
        let src = if let Some(src) = sources.pop() {
            src
        } else {
            sender.send(()).unwrap();
            return;
        };

        if !self.web_families.contains_key(&family_name) {
            let templates = FontTemplates::default();
            self.web_families.insert(family_name.clone(), templates);
        }

        match src {
            Source::Url(url_source) => {
                // https://drafts.csswg.org/css-fonts/#font-fetching-requirements
                let url = match url_source.url.url() {
                    Some(url) => url.clone(),
                    None => return,
                };

                // FIXME:
                // This shouldn't use NoReferrer, but the current documents url
                let request = RequestBuilder::new(url.clone().into(), Referrer::NoReferrer)
                    .destination(Destination::Font);

                let channel_to_self = self.channel_to_self.clone();
                let bytes = Mutex::new(Vec::new());
                let response_valid = Mutex::new(false);
                debug!("Loading @font-face {} from {}", family_name, url);
                fetch_async(request, &self.core_resource_thread, move |response| {
                    match response {
                        FetchResponseMsg::ProcessRequestBody |
                        FetchResponseMsg::ProcessRequestEOF => (),
                        FetchResponseMsg::ProcessResponse(meta_result) => {
                            trace!(
                                "@font-face {} metadata ok={:?}",
                                family_name,
                                meta_result.is_ok()
                            );
                            *response_valid.lock().unwrap() = meta_result.is_ok();
                        },
                        FetchResponseMsg::ProcessResponseChunk(new_bytes) => {
                            trace!("@font-face {} chunk={:?}", family_name, new_bytes);
                            if *response_valid.lock().unwrap() {
                                bytes.lock().unwrap().extend(new_bytes)
                            }
                        },
                        FetchResponseMsg::ProcessResponseEOF(response) => {
                            trace!("@font-face {} EOF={:?}", family_name, response);
                            if response.is_err() || !*response_valid.lock().unwrap() {
                                let msg = Command::AddWebFont(
                                    family_name.clone(),
                                    sources.clone(),
                                    sender.clone(),
                                );
                                channel_to_self.send(msg).unwrap();
                                return;
                            }
                            let bytes = mem::take(&mut *bytes.lock().unwrap());
                            trace!("@font-face {} data={:?}", family_name, bytes);
                            let bytes = match fontsan::process(&bytes) {
                                Ok(san) => san,
                                Err(_) => {
                                    // FIXME(servo/fontsan#1): get an error message
                                    debug!(
                                        "Sanitiser rejected web font: \
                                         family={} url={:?}",
                                        family_name, url
                                    );
                                    let msg = Command::AddWebFont(
                                        family_name.clone(),
                                        sources.clone(),
                                        sender.clone(),
                                    );
                                    channel_to_self.send(msg).unwrap();
                                    return;
                                },
                            };
                            let command = Command::AddDownloadedWebFont(
                                family_name.clone(),
                                url.clone().into(),
                                bytes,
                                sender.clone(),
                            );
                            channel_to_self.send(command).unwrap();
                        },
                    }
                });
            },
            Source::Local(ref font) => {
                let font_face_name = LowercaseString::new(&font.name);
                let templates = &mut self.web_families.get_mut(&family_name).unwrap();
                let mut found = false;
                for_each_variation(&font_face_name, |font_template| {
                    found = true;
                    templates.add_template(font_template);
                });
                if found {
                    sender.send(()).unwrap();
                } else {
                    let msg = Command::AddWebFont(family_name, sources, sender);
                    self.channel_to_self.send(msg).unwrap();
                }
            },
        }
    }

    fn refresh_local_families(&mut self) {
        self.local_families.clear();
        for_each_available_family(|family_name| {
            let family_name = LowercaseString::new(&family_name);
            self.local_families.entry(family_name).or_default();
        });
    }

    fn transform_family(&self, family_name: &FontFamilyName) -> LowercaseString {
        match self.generic_fonts.get(family_name) {
            None => LowercaseString::from(family_name),
            Some(mapped_family) => (*mapped_family).clone(),
        }
    }

    fn find_font_in_local_family(
        &mut self,
        template_descriptor: &FontTemplateDescriptor,
        family_name: &FontFamilyName,
    ) -> Option<FontTemplateRef> {
        let family_name = self.transform_family(family_name);

        // TODO(Issue #188): look up localized font family names if canonical name not found
        // look up canonical name
        if self.local_families.contains_key(&family_name) {
            debug!("FontList: Found font family with name={}", &*family_name);
            let font_templates = self.local_families.get_mut(&family_name).unwrap();

            if font_templates.templates.is_empty() {
                for_each_variation(&family_name, |font_template| {
                    font_templates.add_template(font_template);
                });
            }

            // TODO(Issue #192: handle generic font families, like 'serif' and 'sans-serif'.
            // if such family exists, try to match style to a font

            font_templates.find_font_for_style(template_descriptor)
        } else {
            debug!(
                "FontList: Couldn't find font family with name={}",
                &*family_name
            );
            None
        }
    }

    fn find_font_in_web_family(
        &mut self,
        template_descriptor: &FontTemplateDescriptor,
        family_name: &FontFamilyName,
    ) -> Option<FontTemplateRef> {
        let family_name = LowercaseString::from(family_name);

        if self.web_families.contains_key(&family_name) {
            let templates = self.web_families.get_mut(&family_name).unwrap();
            templates.find_font_for_style(template_descriptor)
        } else {
            None
        }
    }

    fn get_font_key_for_template(&mut self, template: &FontTemplateRef) -> FontKey {
        let webrender_api = &self.webrender_api;
        let webrender_fonts = &mut self.webrender_fonts;
        let identifier = template.borrow().identifier.clone();
        *webrender_fonts
            .entry(identifier.clone())
            .or_insert_with(|| {
                // CoreText cannot reliably create CoreTextFonts for system fonts stored
                // as part of TTC files, so on CoreText platforms, create a system font in
                // WebRender using the LocalFontIdentifier. This has the downside of
                // causing the font to be loaded into memory again (bummer!), so only do
                // this for those platforms.
                #[cfg(target_os = "macos")]
                if let FontIdentifier::Local(local_font_identifier) = identifier {
                    return webrender_api
                        .add_system_font(local_font_identifier.native_font_handle());
                }

                let bytes = template.data();
                webrender_api.add_font(bytes, identifier.index())
            })
    }

    fn find_font_template(
        &mut self,
        template_descriptor: &FontTemplateDescriptor,
        family_descriptor: &FontFamilyDescriptor,
    ) -> Option<FontTemplateAndWebRenderFontKey> {
        match family_descriptor.scope {
            FontSearchScope::Any => self
                .find_font_in_web_family(template_descriptor, &family_descriptor.name)
                .or_else(|| {
                    self.find_font_in_local_family(template_descriptor, &family_descriptor.name)
                }),

            FontSearchScope::Local => {
                self.find_font_in_local_family(template_descriptor, &family_descriptor.name)
            },
        }
        .map(|font_template| FontTemplateAndWebRenderFontKey {
            font_key: self.get_font_key_for_template(&font_template),
            font_template,
        })
    }
}

/// The public interface to the font cache thread, used by per-thread `FontContext` instances (via
/// the `FontSource` trait), and also by layout.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontCacheThread {
    chan: IpcSender<Command>,
}

impl FontCacheThread {
    pub fn new(
        core_resource_thread: CoreResourceThread,
        webrender_api: Box<dyn WebrenderApi + Send>,
    ) -> FontCacheThread {
        let (chan, port) = ipc::channel().unwrap();

        let channel_to_self = chan.clone();
        thread::Builder::new()
            .name("FontCache".to_owned())
            .spawn(move || {
                // TODO: Allow users to specify these.
                let generic_fonts = populate_generic_fonts();

                #[allow(clippy::default_constructed_unit_structs)]
                let mut cache = FontCache {
                    port,
                    channel_to_self,
                    generic_fonts,
                    local_families: HashMap::new(),
                    web_families: HashMap::new(),
                    core_resource_thread,
                    webrender_api,
                    webrender_fonts: HashMap::new(),
                    font_instances: HashMap::new(),
                };

                cache.refresh_local_families();
                cache.run();
            })
            .expect("Thread spawning failed");

        FontCacheThread { chan }
    }

    pub fn add_all_web_fonts_from_stylesheet(
        &self,
        stylesheet: &Stylesheet,
        guard: &SharedRwLockReadGuard,
        device: &Device,
        font_cache_sender: &IpcSender<()>,
        synchronous: bool,
    ) -> usize {
        let (sender, receiver) = if synchronous {
            let (sender, receiver) = ipc::channel().unwrap();
            (Some(sender), Some(receiver))
        } else {
            (None, None)
        };

        let mut number_loading = 0;
        stylesheet.effective_font_face_rules(device, guard, |rule| {
            let font_face = match rule.font_face() {
                Some(font_face) => font_face,
                None => return,
            };

            let sources: Vec<Source> = font_face
                .sources()
                .0
                .iter()
                .rev()
                .filter(is_supported_web_font_source)
                .cloned()
                .collect();
            if sources.is_empty() {
                return;
            }

            let sender = sender.as_ref().unwrap_or(font_cache_sender).clone();
            self.chan
                .send(Command::AddWebFont(
                    LowercaseString::new(&font_face.family().name),
                    sources,
                    sender,
                ))
                .unwrap();

            // Either increment the count of loading web fonts, or wait for a synchronous load.
            if let Some(ref receiver) = receiver {
                receiver.recv().unwrap();
            }
            number_loading += 1;
        });

        number_loading
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
    fn get_font_instance(&mut self, key: FontKey, size: Au) -> FontInstanceKey {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.chan
            .send(Command::GetFontInstance(key, size, response_chan))
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

    fn font_template(
        &mut self,
        template_descriptor: FontTemplateDescriptor,
        family_descriptor: FontFamilyDescriptor,
    ) -> Option<FontTemplateAndWebRenderFontKey> {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.chan
            .send(Command::GetFontTemplate(
                template_descriptor,
                family_descriptor,
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

        match reply.unwrap() {
            Reply::GetFontTemplateReply(maybe_serialized_font_template_info) => {
                maybe_serialized_font_template_info.map(|serialized_font_template_info| {
                    let font_template = Rc::new(RefCell::new(
                        serialized_font_template_info
                            .serialized_font_template
                            .to_font_template(),
                    ));
                    FontTemplateAndWebRenderFontKey {
                        font_template,
                        font_key: serialized_font_template_info.font_key,
                    }
                })
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LowercaseString {
    inner: String,
}

impl LowercaseString {
    pub fn new(s: &str) -> LowercaseString {
        LowercaseString {
            inner: s.to_lowercase(),
        }
    }
}

impl<'a> From<&'a FontFamilyName> for LowercaseString {
    fn from(family_name: &'a FontFamilyName) -> LowercaseString {
        LowercaseString::new(family_name.name())
    }
}

impl Deref for LowercaseString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for LowercaseString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

fn is_supported_web_font_source(source: &&Source) -> bool {
    let url_source = match source {
        Source::Url(ref url_source) => url_source,
        Source::Local(_) => return true,
    };
    let format_hint = match url_source.format_hint {
        Some(ref format_hint) => format_hint,
        None => return true,
    };

    if matches!(
        format_hint,
        FontFaceSourceFormat::Keyword(
            FontFaceSourceFormatKeyword::Truetype |
                FontFaceSourceFormatKeyword::Opentype |
                FontFaceSourceFormatKeyword::Woff |
                FontFaceSourceFormatKeyword::Woff2
        )
    ) {
        return true;
    }

    if let FontFaceSourceFormat::String(string) = format_hint {
        return string == "truetype" ||
            string == "opentype" ||
            string == "woff" ||
            string == "woff2";
    }

    false
}
