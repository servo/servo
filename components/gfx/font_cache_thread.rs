/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, RangeInclusive};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{f32, fmt, mem, thread};

use app_units::Au;
use gfx_traits::WebrenderApi;
use ipc_channel::ipc::{self, IpcBytesSender, IpcReceiver, IpcSender};
use log::{debug, trace};
use net_traits::request::{Destination, Referrer, RequestBuilder};
use net_traits::{fetch_async, CoreResourceThread, FetchResponseMsg};
use serde::{Deserialize, Serialize};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use style::font_face::{
    FontFaceRuleData, FontFaceSourceFormat, FontFaceSourceFormatKeyword,
    FontStyle as FontFaceStyle, Source,
};
use style::media_queries::Device;
use style::shared_lock::SharedRwLockReadGuard;
use style::stylesheets::{Stylesheet, StylesheetInDocument};
use style::values::computed::font::{FixedPoint, FontStyleFixedPoint};
use style::values::computed::{FontStretch, FontWeight};
use style::values::specified::FontStretch as SpecifiedFontStretch;
use webrender_api::{FontInstanceKey, FontKey};

use crate::font::{FontDescriptor, FontFamilyDescriptor, FontFamilyName, FontSearchScope};
use crate::font_context::FontSource;
use crate::font_template::{
    FontTemplate, FontTemplateDescriptor, FontTemplateRef, FontTemplateRefMethods,
};
use crate::platform::font_list::{
    for_each_available_family, for_each_variation, system_default_family, LocalFontIdentifier,
    SANS_SERIF_FONT_FAMILY,
};

/// A list of font templates that make up a given font family.
#[derive(Default)]
pub struct FontTemplates {
    templates: Vec<FontTemplateRef>,
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

impl FontTemplates {
    /// Find a font in this family that matches a given descriptor.
    pub fn find_for_descriptor(
        &mut self,
        descriptor_to_match: &FontDescriptor,
    ) -> Vec<FontTemplateRef> {
        // TODO(Issue #189): optimize lookup for
        // regular/bold/italic/bolditalic with fixed offsets and a
        // static decision table for fallback between these values.
        let matching_templates: Vec<FontTemplateRef> = self
            .templates
            .iter()
            .filter(|template| template.matches_font_descriptor(descriptor_to_match))
            .cloned()
            .collect();
        if !matching_templates.is_empty() {
            return matching_templates;
        }

        // We didn't find an exact match. Do more expensive fuzzy matching.
        // TODO(#190): Do a better job.
        let mut best_templates = Vec::new();
        let mut best_distance = f32::MAX;
        for template in self.templates.iter() {
            let distance = template.descriptor_distance(descriptor_to_match);
            if distance < best_distance {
                best_templates = vec![template.clone()];
                best_distance = distance
            } else if distance == best_distance {
                best_templates.push(template.clone());
            }
        }

        if !best_templates.is_empty() {
            return best_templates;
        }

        // If a request is made for a font family that exists,
        // pick the first valid font in the family if we failed
        // to find an exact match for the descriptor.
        for template in &mut self.templates.iter() {
            return vec![template.clone()];
        }

        Vec::new()
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
    GetFontTemplates(
        FontDescriptor,
        FontFamilyDescriptor,
        IpcSender<Vec<SerializedFontTemplate>>,
    ),
    GetFontInstance(FontIdentifier, Au, IpcSender<FontInstanceKey>),
    AddWebFont(CSSFontFaceDescriptors, Vec<Source>, IpcSender<()>),
    AddDownloadedWebFont(CSSFontFaceDescriptors, ServoUrl, Vec<u8>, IpcSender<()>),
    Exit(IpcSender<()>),
    Ping,
}

/// The font cache thread itself. It maintains a list of reference counted
/// font templates that are currently in use.
struct FontCache {
    port: IpcReceiver<Command>,
    channel_to_self: IpcSender<Command>,
    generic_fonts: HashMap<FontFamilyName, LowercaseString>,
    font_data: HashMap<FontIdentifier, Arc<Vec<u8>>>,
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
                Command::GetFontTemplates(descriptor_to_match, family_descriptor, result) => {
                    let templates =
                        self.find_font_templates(&descriptor_to_match, &family_descriptor);
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
                        let _ = bytes_sender.send(&data);
                    }
                },
                Command::GetFontInstance(identifier, pt_size, result) => {
                    let _ = result.send(self.get_font_instance(identifier, pt_size));
                },
                Command::AddWebFont(css_font_face_descriptors, sources, result) => {
                    self.handle_add_web_font(css_font_face_descriptors, sources, result);
                },
                Command::AddDownloadedWebFont(css_font_face_descriptors, url, data, result) => {
                    let family_name = css_font_face_descriptors.family_name.clone();
                    let templates = &mut self.web_families.get_mut(&family_name).unwrap();
                    if let Ok(template) =
                        FontTemplate::new_web_font(url, Arc::new(data), css_font_face_descriptors)
                    {
                        templates.add_template(template);
                    }
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
        css_font_face_descriptors: CSSFontFaceDescriptors,
        mut sources: Vec<Source>,
        sender: IpcSender<()>,
    ) {
        let family_name = css_font_face_descriptors.family_name.clone();
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
                                    css_font_face_descriptors.clone(),
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
                                        css_font_face_descriptors.clone(),
                                        sources.clone(),
                                        sender.clone(),
                                    );
                                    channel_to_self.send(msg).unwrap();
                                    return;
                                },
                            };
                            let command = Command::AddDownloadedWebFont(
                                css_font_face_descriptors.clone(),
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
                    let msg =
                        Command::AddWebFont(css_font_face_descriptors.clone(), sources, sender);
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

    fn find_templates_in_local_family(
        &mut self,
        descriptor_to_match: &FontDescriptor,
        family_name: &FontFamilyName,
    ) -> Vec<FontTemplateRef> {
        // TODO(Issue #188): look up localized font family names if canonical name not found
        // look up canonical name
        // TODO(Issue #192: handle generic font families, like 'serif' and 'sans-serif'.
        // if such family exists, try to match style to a font
        let family_name = self.transform_family(family_name);
        self.local_families
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

    fn find_templates_in_web_family(
        &mut self,
        descriptor_to_match: &FontDescriptor,
        family_name: &FontFamilyName,
    ) -> Vec<FontTemplateRef> {
        let family_name = LowercaseString::from(family_name);
        self.web_families
            .get_mut(&family_name)
            .map(|templates| templates.find_for_descriptor(descriptor_to_match))
            .unwrap_or_default()
    }

    fn find_font_templates(
        &mut self,
        descriptor_to_match: &FontDescriptor,
        family_descriptor: &FontFamilyDescriptor,
    ) -> Vec<FontTemplateRef> {
        if family_descriptor.scope == FontSearchScope::Any {
            let templates =
                self.find_templates_in_web_family(descriptor_to_match, &family_descriptor.name);
            if !templates.is_empty() {
                return templates;
            }
        }

        self.find_templates_in_local_family(descriptor_to_match, &family_descriptor.name)
    }

    fn get_font_instance(&mut self, identifier: FontIdentifier, pt_size: Au) -> FontInstanceKey {
        let webrender_api = &self.webrender_api;
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
                    return webrender_api
                        .add_system_font(local_font_identifier.native_font_handle());
                }

                webrender_api.add_font(font_data, identifier.index())
            });

        *self
            .font_instances
            .entry((font_key, pt_size))
            .or_insert_with(|| webrender_api.add_font_instance(font_key, pt_size.to_f32_px()))
    }
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
    pub family_name: LowercaseString,
    pub weight: Option<(FontWeight, FontWeight)>,
    pub stretch: Option<(FontStretch, FontStretch)>,
    pub style: Option<ComputedFontStyleDescriptor>,
    pub unicode_range: Option<Vec<RangeInclusive<u32>>>,
}

impl CSSFontFaceDescriptors {
    pub fn new(family_name: &str) -> Self {
        CSSFontFaceDescriptors {
            family_name: LowercaseString::new(family_name),
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
        let style = rule_data
            .style
            .as_ref()
            .map(|style| style_to_computed(style));
        let unicode_range = rule_data
            .unicode_range
            .as_ref()
            .map(|ranges| ranges.iter().map(|range| range.start..=range.end).collect());

        CSSFontFaceDescriptors {
            family_name: LowercaseString::new(&family_name),
            weight,
            stretch,
            style,
            unicode_range,
        }
    }
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
                    font_data: HashMap::new(),
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
                .send(Command::AddWebFont(rule.into(), sources, sender))
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
    fn get_font_instance(&mut self, identifier: FontIdentifier, size: Au) -> FontInstanceKey {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.chan
            .send(Command::GetFontInstance(identifier, size, response_chan))
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
        &mut self,
        descriptor_to_match: &FontDescriptor,
        family_descriptor: FontFamilyDescriptor,
    ) -> Vec<FontTemplateRef> {
        let (response_chan, response_port) = ipc::channel().expect("failed to create IPC channel");
        self.chan
            .send(Command::GetFontTemplates(
                descriptor_to_match.clone(),
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

        reply
            .unwrap()
            .into_iter()
            .map(|serialized_font_template| {
                let font_data = serialized_font_template.bytes_receiver.recv().ok();
                Rc::new(RefCell::new(FontTemplate {
                    identifier: serialized_font_template.identifier,
                    descriptor: serialized_font_template.descriptor.clone(),
                    data: font_data.map(Arc::new),
                }))
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
