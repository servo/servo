/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font_template::{FontTemplate, FontTemplateDescriptor};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use mime::{TopLevel, SubLevel};
use net_traits::{AsyncResponseTarget, LoadContext, PendingAsyncLoad, CoreResourceThread, ResponseAction, RequestSource};
use platform::font_context::FontContextHandle;
use platform::font_list::SANS_SERIF_FONT_FAMILY;
use platform::font_list::for_each_available_family;
use platform::font_list::for_each_variation;
use platform::font_list::last_resort_font_families;
use platform::font_list::system_default_family;
use platform::font_template::FontTemplateData;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::mem;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::u32;
use string_cache::Atom;
use style::font_face::{EffectiveSources, Source};
use style::properties::longhands::font_family::computed_value::FontFamily;
use url::Url;
use util::prefs::PREFS;
use util::thread::spawn_named;
use webrender_traits;

/// A list of font templates that make up a given font family.
struct FontTemplates {
    templates: Vec<FontTemplate>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FontTemplateInfo {
    pub font_template: Arc<FontTemplateData>,
    pub font_key: Option<webrender_traits::FontKey>,
}

impl FontTemplates {
    fn new() -> FontTemplates {
        FontTemplates {
            templates: vec!(),
        }
    }

    /// Find a font in this family that matches a given descriptor.
    fn find_font_for_style(&mut self, desc: &FontTemplateDescriptor, fctx: &FontContextHandle)
                               -> Option<Arc<FontTemplateData>> {
        // TODO(Issue #189): optimize lookup for
        // regular/bold/italic/bolditalic with fixed offsets and a
        // static decision table for fallback between these values.
        for template in &mut self.templates {
            let maybe_template = template.data_for_descriptor(fctx, desc);
            if maybe_template.is_some() {
                return maybe_template;
            }
        }

        // We didn't find an exact match. Do more expensive fuzzy matching.
        // TODO(#190): Do a better job.
        let (mut best_template_data, mut best_distance) = (None, u32::MAX);
        for template in &mut self.templates {
            if let Some((template_data, distance)) =
                    template.data_for_approximate_descriptor(fctx, desc) {
                if distance < best_distance {
                    best_template_data = Some(template_data);
                    best_distance = distance
                }
            }
        }
        if best_template_data.is_some() {
            return best_template_data
        }

        // If a request is made for a font family that exists,
        // pick the first valid font in the family if we failed
        // to find an exact match for the descriptor.
        for template in &mut self.templates {
            let maybe_template = template.get();
            if maybe_template.is_some() {
                return maybe_template;
            }
        }

        None
    }

    fn add_template(&mut self, identifier: Atom, maybe_data: Option<Vec<u8>>) {
        for template in &self.templates {
            if *template.identifier() == identifier {
                return;
            }
        }

        let template = FontTemplate::new(identifier, maybe_data);
        self.templates.push(template);
    }
}

/// Commands that the FontContext sends to the font cache thread.
#[derive(Deserialize, Serialize, Debug)]
pub enum Command {
    GetFontTemplate(FontFamily, FontTemplateDescriptor, IpcSender<Reply>),
    GetLastResortFontTemplate(FontTemplateDescriptor, IpcSender<Reply>),
    AddWebFont(LowercaseString, EffectiveSources, IpcSender<()>),
    AddDownloadedWebFont(LowercaseString, Url, Vec<u8>, IpcSender<()>),
    Exit(IpcSender<()>),
}

/// Reply messages sent from the font cache thread to the FontContext caller.
#[derive(Deserialize, Serialize, Debug)]
pub enum Reply {
    GetFontTemplateReply(Option<FontTemplateInfo>),
}

/// The font cache thread itself. It maintains a list of reference counted
/// font templates that are currently in use.
struct FontCache {
    port: IpcReceiver<Command>,
    channel_to_self: IpcSender<Command>,
    generic_fonts: HashMap<FontFamily, LowercaseString>,
    local_families: HashMap<LowercaseString, FontTemplates>,
    web_families: HashMap<LowercaseString, FontTemplates>,
    font_context: FontContextHandle,
    core_resource_thread: CoreResourceThread,
    webrender_api: Option<webrender_traits::RenderApi>,
    webrender_fonts: HashMap<Atom, webrender_traits::FontKey>,
}

fn populate_generic_fonts() -> HashMap<FontFamily, LowercaseString> {
    let mut generic_fonts = HashMap::with_capacity(5);

    append_map(&mut generic_fonts, FontFamily::Generic(atom!("serif")), "Times New Roman");
    append_map(&mut generic_fonts, FontFamily::Generic(atom!("sans-serif")), SANS_SERIF_FONT_FAMILY);
    append_map(&mut generic_fonts, FontFamily::Generic(atom!("cursive")), "Apple Chancery");
    append_map(&mut generic_fonts, FontFamily::Generic(atom!("fantasy")), "Papyrus");
    append_map(&mut generic_fonts, FontFamily::Generic(atom!("monospace")), "Menlo");

    fn append_map(generic_fonts: &mut HashMap<FontFamily, LowercaseString>,
                  font_family: FontFamily,
                  mapped_name: &str) {
        let family_name = {
            let opt_system_default = system_default_family(font_family.name());
            match opt_system_default {
                Some(system_default) => LowercaseString::new(&system_default),
                None => LowercaseString::new(mapped_name)
            }
        };

        generic_fonts.insert(font_family, family_name);
    }


    generic_fonts
}

impl FontCache {
    fn run(&mut self) {
        loop {
            let msg = self.port.recv().unwrap();

            match msg {
                Command::GetFontTemplate(family, descriptor, result) => {
                    let maybe_font_template = self.find_font_template(&family, &descriptor);
                    let _ = result.send(Reply::GetFontTemplateReply(maybe_font_template));
                }
                Command::GetLastResortFontTemplate(descriptor, result) => {
                    let font_template = self.last_resort_font_template(&descriptor);
                    let _ = result.send(Reply::GetFontTemplateReply(Some(font_template)));
                }
                Command::AddWebFont(family_name, sources, result) => {
                    self.handle_add_web_font(family_name, sources, result);
                }
                Command::AddDownloadedWebFont(family_name, url, bytes, result) => {
                    let templates = &mut self.web_families.get_mut(&family_name).unwrap();
                    templates.add_template(Atom::from(url.to_string()), Some(bytes));
                    drop(result.send(()));
                }
                Command::Exit(result) => {
                    let _ = result.send(());
                    break;
                }
            }
        }
    }

    fn handle_add_web_font(&mut self,
                           family_name: LowercaseString,
                           mut sources: EffectiveSources,
                           sender: IpcSender<()>) {
        let src = if let Some(src) = sources.next() {
            src
        } else {
            sender.send(()).unwrap();
            return;
        };

        if !self.web_families.contains_key(&family_name) {
            let templates = FontTemplates::new();
            self.web_families.insert(family_name.clone(), templates);
        }

        match src {
            Source::Url(ref url_source) => {
                let url = &url_source.url;
                let load = PendingAsyncLoad::new(LoadContext::Font,
                                                 self.core_resource_thread.clone(),
                                                 url.clone(),
                                                 None,
                                                 None,
                                                 None,
                                                 RequestSource::None);
                let (data_sender, data_receiver) = ipc::channel().unwrap();
                let data_target = AsyncResponseTarget {
                    sender: data_sender,
                };
                load.load_async(data_target);
                let channel_to_self = self.channel_to_self.clone();
                let url = (*url).clone();
                let bytes = Mutex::new(Vec::new());
                let response_valid = Mutex::new(false);
                ROUTER.add_route(data_receiver.to_opaque(), box move |message| {
                    let response: ResponseAction = message.to().unwrap();
                    match response {
                        ResponseAction::HeadersAvailable(meta_result) => {
                            let is_response_valid = match meta_result {
                                Ok(ref metadata) => {
                                    metadata.content_type.as_ref().map_or(false, |content_type| {
                                        let mime = &content_type.0;
                                        is_supported_font_type(&mime.0, &mime.1)
                                    })
                                }
                                Err(_) => false,
                            };

                            info!("{} font with MIME type {}",
                                  if is_response_valid { "Loading" } else { "Ignoring" },
                                  meta_result.map(|ref meta| format!("{:?}", meta.content_type))
                                             .unwrap_or(format!("<Network Error>")));
                            *response_valid.lock().unwrap() = is_response_valid;
                        }
                        ResponseAction::DataAvailable(new_bytes) => {
                            if *response_valid.lock().unwrap() {
                                bytes.lock().unwrap().extend(new_bytes.into_iter())
                            }
                        }
                        ResponseAction::ResponseComplete(response) => {
                            if response.is_err() || !*response_valid.lock().unwrap() {
                                let msg = Command::AddWebFont(family_name.clone(), sources.clone(), sender.clone());
                                channel_to_self.send(msg).unwrap();
                                return;
                            }
                            let mut bytes = bytes.lock().unwrap();
                            let bytes = mem::replace(&mut *bytes, Vec::new());
                            let command =
                                Command::AddDownloadedWebFont(family_name.clone(),
                                                              url.clone(),
                                                              bytes,
                                                              sender.clone());
                            channel_to_self.send(command).unwrap();
                        }
                    }
                });
            }
            Source::Local(ref font) => {
                let font_face_name = LowercaseString::new(font.name());
                let templates = &mut self.web_families.get_mut(&family_name).unwrap();
                let mut found = false;
                for_each_variation(&font_face_name, |path| {
                    found = true;
                    templates.add_template(Atom::from(&*path), None);
                });
                if found {
                    sender.send(()).unwrap();
                } else {
                    let msg = Command::AddWebFont(family_name, sources, sender);
                    self.channel_to_self.send(msg).unwrap();
                }
            }
        }
    }

    fn refresh_local_families(&mut self) {
        self.local_families.clear();
        for_each_available_family(|family_name| {
            let family_name = LowercaseString::new(&family_name);
            if !self.local_families.contains_key(&family_name) {
                let templates = FontTemplates::new();
                self.local_families.insert(family_name, templates);
            }
        });
    }

    fn transform_family(&self, family: &FontFamily) -> LowercaseString {
        match self.generic_fonts.get(family) {
            None => LowercaseString::new(family.name()),
            Some(mapped_family) => (*mapped_family).clone()
        }
    }

    fn find_font_in_local_family(&mut self, family_name: &LowercaseString, desc: &FontTemplateDescriptor)
                                -> Option<Arc<FontTemplateData>> {
        // TODO(Issue #188): look up localized font family names if canonical name not found
        // look up canonical name
        if self.local_families.contains_key(family_name) {
            debug!("FontList: Found font family with name={}", &**family_name);
            let s = self.local_families.get_mut(family_name).unwrap();

            if s.templates.is_empty() {
                for_each_variation(family_name, |path| {
                    s.add_template(Atom::from(&*path), None);
                });
            }

            // TODO(Issue #192: handle generic font families, like 'serif' and 'sans-serif'.
            // if such family exists, try to match style to a font

            s.find_font_for_style(desc, &self.font_context)
        } else {
            debug!("FontList: Couldn't find font family with name={}", &**family_name);
            None
        }
    }

    fn find_font_in_web_family(&mut self, family: &FontFamily, desc: &FontTemplateDescriptor)
                                -> Option<Arc<FontTemplateData>> {
        let family_name = LowercaseString::new(family.name());

        if self.web_families.contains_key(&family_name) {
            let templates = self.web_families.get_mut(&family_name).unwrap();
            templates.find_font_for_style(desc, &self.font_context)
        } else {
            None
        }
    }

    fn get_font_template_info(&mut self, template: Arc<FontTemplateData>) -> FontTemplateInfo {
        let webrender_fonts = &mut self.webrender_fonts;
        let font_key = self.webrender_api.as_ref().map(|webrender_api| {
            *webrender_fonts.entry(template.identifier.clone()).or_insert_with(|| {
                match (template.bytes_if_in_memory(), template.native_font()) {
                    (Some(bytes), _) => webrender_api.add_raw_font(bytes),
                    (None, Some(native_font)) => webrender_api.add_native_font(native_font),
                    (None, None) => webrender_api.add_raw_font(template.bytes().clone()),
                }
            })
        });

        FontTemplateInfo {
            font_template: template,
            font_key: font_key,
        }
    }

    fn find_font_template(&mut self, family: &FontFamily, desc: &FontTemplateDescriptor)
                            -> Option<FontTemplateInfo> {
        let template = self.find_font_in_web_family(family, desc)
            .or_else(|| {
                let transformed_family = self.transform_family(family);
                self.find_font_in_local_family(&transformed_family, desc)
            });

        template.map(|template| {
            self.get_font_template_info(template)
        })
    }

    fn last_resort_font_template(&mut self, desc: &FontTemplateDescriptor)
                                        -> FontTemplateInfo {
        let last_resort = last_resort_font_families();

        for family in &last_resort {
            let family = LowercaseString::new(family);
            let maybe_font_in_family = self.find_font_in_local_family(&family, desc);
            if let Some(family) = maybe_font_in_family {
                return self.get_font_template_info(family)
            }
        }

        panic!("Unable to find any fonts that match (do you have fallback fonts installed?)");
    }
}

/// The public interface to the font cache thread, used exclusively by
/// the per-thread/thread FontContext structures.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct FontCacheThread {
    chan: IpcSender<Command>,
}

impl FontCacheThread {
    pub fn new(core_resource_thread: CoreResourceThread,
               webrender_api: Option<webrender_traits::RenderApi>) -> FontCacheThread {
        let (chan, port) = ipc::channel().unwrap();

        let channel_to_self = chan.clone();
        spawn_named("FontCacheThread".to_owned(), move || {
            // TODO: Allow users to specify these.
            let generic_fonts = populate_generic_fonts();

            let mut cache = FontCache {
                port: port,
                channel_to_self: channel_to_self,
                generic_fonts: generic_fonts,
                local_families: HashMap::new(),
                web_families: HashMap::new(),
                font_context: FontContextHandle::new(),
                core_resource_thread: core_resource_thread,
                webrender_api: webrender_api,
                webrender_fonts: HashMap::new(),
            };

            cache.refresh_local_families();
            cache.run();
        });

        FontCacheThread {
            chan: chan,
        }
    }

    pub fn find_font_template(&self, family: FontFamily, desc: FontTemplateDescriptor)
                                                -> Option<FontTemplateInfo> {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.chan.send(Command::GetFontTemplate(family, desc, response_chan)).unwrap();

        let reply = response_port.recv().unwrap();

        match reply {
            Reply::GetFontTemplateReply(data) => {
                data
            }
        }
    }

    pub fn last_resort_font_template(&self, desc: FontTemplateDescriptor)
                                                -> FontTemplateInfo {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.chan.send(Command::GetLastResortFontTemplate(desc, response_chan)).unwrap();

        let reply = response_port.recv().unwrap();

        match reply {
            Reply::GetFontTemplateReply(data) => {
                data.unwrap()
            }
        }
    }

    pub fn add_web_font(&self, family: FontFamily, sources: EffectiveSources, sender: IpcSender<()>) {
        self.chan.send(Command::AddWebFont(LowercaseString::new(family.name()), sources, sender)).unwrap();
    }

    pub fn exit(&self) {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.chan.send(Command::Exit(response_chan)).expect("Couldn't send FontCacheThread exit message");
        response_port.recv().expect("Couldn't receive FontCacheThread reply");
    }
}

// derived from http://stackoverflow.com/a/10864297/3830
fn is_supported_font_type(toplevel: &TopLevel, sublevel: &SubLevel) -> bool {
    if !PREFS.get("network.mime.sniff").as_boolean().unwrap_or(false) {
        return true;
    }

    match (toplevel, sublevel) {
        (&TopLevel::Application, &SubLevel::Ext(ref ext)) => {
            match &ext[..] {
                //FIXME: once sniffing is enabled by default, we shouldn't need nonstandard
                //       MIME types here.
                "font-sfnt" | "x-font-ttf" | "x-font-truetype" | "x-font-opentype" => true,
                _ => false,
            }
        }
        _ => false,
    }
}


#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize, Serialize)]
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

impl Deref for LowercaseString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &*self.inner
    }
}
