/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font_template::{FontTemplate, FontTemplateDescriptor};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use net_traits::{AsyncResponseTarget, PendingAsyncLoad, ResourceTask, ResponseAction};
use platform::font_context::FontContextHandle;
use platform::font_list::for_each_available_family;
use platform::font_list::for_each_variation;
use platform::font_list::last_resort_font_families;
use platform::font_list::system_default_family;
use platform::font_template::FontTemplateData;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::mem;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use style::font_face::Source;
use style::properties::longhands::font_family::computed_value::FontFamily;
use url::Url;
use util::str::LowercaseString;
use util::task::spawn_named;

/// A list of font templates that make up a given font family.
struct FontTemplates {
    templates: Vec<FontTemplate>,
}

impl FontTemplates {
    fn new() -> FontTemplates {
        FontTemplates {
            templates: vec!(),
        }
    }

    /// Find a font in this family that matches a given descriptor.
    fn find_font_for_style<'a>(&'a mut self, desc: &FontTemplateDescriptor, fctx: &FontContextHandle)
                               -> Option<Arc<FontTemplateData>> {
        // TODO(Issue #189): optimize lookup for
        // regular/bold/italic/bolditalic with fixed offsets and a
        // static decision table for fallback between these values.

        // TODO(Issue #190): if not in the fast path above, do
        // expensive matching of weights, etc.
        for template in &mut self.templates {
            let maybe_template = template.data_for_descriptor(fctx, desc);
            if maybe_template.is_some() {
                return maybe_template;
            }
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

/// Commands that the FontContext sends to the font cache task.
#[derive(Deserialize, Serialize)]
pub enum Command {
    GetFontTemplate(FontFamily, FontTemplateDescriptor, IpcSender<Reply>),
    GetLastResortFontTemplate(FontTemplateDescriptor, IpcSender<Reply>),
    AddWebFont(FontFamily, Source, IpcSender<()>),
    AddDownloadedWebFont(FontFamily, Url, Vec<u8>, IpcSender<()>),
    Exit(IpcSender<()>),
}

/// Reply messages sent from the font cache task to the FontContext caller.
#[derive(Deserialize, Serialize)]
pub enum Reply {
    GetFontTemplateReply(Option<Arc<FontTemplateData>>),
}

/// The font cache task itself. It maintains a list of reference counted
/// font templates that are currently in use.
struct FontCache {
    port: IpcReceiver<Command>,
    channel_to_self: IpcSender<Command>,
    generic_fonts: HashMap<FontFamily, LowercaseString>,
    local_families: HashMap<LowercaseString, FontTemplates>,
    web_families: HashMap<FontFamily, FontTemplates>,
    font_context: FontContextHandle,
    resource_task: ResourceTask,
}

fn populate_generic_fonts() -> HashMap<FontFamily, LowercaseString> {
    let mut generic_fonts = HashMap::with_capacity(5);

    append_map(&mut generic_fonts, FontFamily::Serif, "Times New Roman");
    append_map(&mut generic_fonts, FontFamily::SansSerif, "Arial");
    append_map(&mut generic_fonts, FontFamily::Cursive, "Apple Chancery");
    append_map(&mut generic_fonts, FontFamily::Fantasy, "Papyrus");
    append_map(&mut generic_fonts, FontFamily::Monospace, "Menlo");

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
                    result.send(Reply::GetFontTemplateReply(maybe_font_template)).unwrap();
                }
                Command::GetLastResortFontTemplate(descriptor, result) => {
                    let font_template = self.last_resort_font_template(&descriptor);
                    result.send(Reply::GetFontTemplateReply(Some(font_template))).unwrap();
                }
                Command::AddWebFont(family, src, result) => {
                    if !self.web_families.contains_key(&family) {
                        let templates = FontTemplates::new();
                        self.web_families.insert(family.clone(), templates);
                    }

                    match src {
                        Source::Url(ref url_source) => {
                            let url = &url_source.url;
                            let load = PendingAsyncLoad::new(self.resource_task.clone(),
                                                             url.clone(),
                                                             None);
                            let (data_sender, data_receiver) = ipc::channel().unwrap();
                            let data_target = AsyncResponseTarget {
                                sender: data_sender,
                            };
                            load.load_async(data_target);
                            let channel_to_self = self.channel_to_self.clone();
                            let url = (*url).clone();
                            let bytes = Mutex::new(Vec::new());
                            ROUTER.add_route(data_receiver.to_opaque(), box move |message| {
                                let response: ResponseAction = message.to().unwrap();
                                match response {
                                    ResponseAction::HeadersAvailable(_) |
                                    ResponseAction::ResponseComplete(Err(_)) => {}
                                    ResponseAction::DataAvailable(new_bytes) => {
                                        bytes.lock().unwrap().extend(new_bytes.into_iter())
                                    }
                                    ResponseAction::ResponseComplete(Ok(_)) => {
                                        let mut bytes = bytes.lock().unwrap();
                                        let bytes = mem::replace(&mut *bytes, Vec::new());
                                        let command =
                                            Command::AddDownloadedWebFont(family.clone(),
                                                                          url.clone(),
                                                                          bytes,
                                                                          result.clone());
                                        channel_to_self.send(command).unwrap();
                                    }
                                }
                            });
                        }
                        Source::Local(ref family) => {
                            let family_name = self.transform_family(&family);
                            let templates = &mut self.web_families.get_mut(&family).unwrap();
                            for_each_variation(&family_name, |path| {
                                templates.add_template(Atom::from(&*path), None);
                            });
                            result.send(()).unwrap();
                        }
                    }
                }
                Command::AddDownloadedWebFont(family, url, bytes, result) => {
                    let templates = &mut self.web_families.get_mut(&family).unwrap();
                    templates.add_template(Atom::from(&*url.to_string()), Some(bytes));
                    drop(result.send(()));
                }
                Command::Exit(result) => {
                    result.send(()).unwrap();
                    break;
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

    fn find_font_in_local_family<'a>(&'a mut self, family_name: &LowercaseString, desc: &FontTemplateDescriptor)
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

    fn find_font_in_web_family<'a>(&'a mut self, family: &FontFamily, desc: &FontTemplateDescriptor)
                                -> Option<Arc<FontTemplateData>> {
        if self.web_families.contains_key(family) {
            let templates = self.web_families.get_mut(family).unwrap();
            let maybe_font = templates.find_font_for_style(desc, &self.font_context);
            maybe_font
        } else {
            None
        }
    }

    fn find_font_template(&mut self, family: &FontFamily, desc: &FontTemplateDescriptor)
                            -> Option<Arc<FontTemplateData>> {
        self.find_font_in_web_family(family, desc)
            .or_else(|| {
                let transformed_family = self.transform_family(family);
                self.find_font_in_local_family(&transformed_family, desc)
            })
    }

    fn last_resort_font_template(&mut self, desc: &FontTemplateDescriptor)
                                        -> Arc<FontTemplateData> {
        let last_resort = last_resort_font_families();

        for family in &last_resort {
            let family = LowercaseString::new(family);
            let maybe_font_in_family = self.find_font_in_local_family(&family, desc);
            if let Some(family) = maybe_font_in_family {
                return family;
            }
        }

        panic!("Unable to find any fonts that match (do you have fallback fonts installed?)");
    }
}

/// The public interface to the font cache task, used exclusively by
/// the per-thread/task FontContext structures.
#[derive(Clone, Deserialize, Serialize)]
pub struct FontCacheTask {
    chan: IpcSender<Command>,
}

impl FontCacheTask {
    pub fn new(resource_task: ResourceTask) -> FontCacheTask {
        let (chan, port) = ipc::channel().unwrap();

        let channel_to_self = chan.clone();
        spawn_named("FontCacheTask".to_owned(), move || {
            // TODO: Allow users to specify these.
            let generic_fonts = populate_generic_fonts();

            let mut cache = FontCache {
                port: port,
                channel_to_self: channel_to_self,
                generic_fonts: generic_fonts,
                local_families: HashMap::new(),
                web_families: HashMap::new(),
                font_context: FontContextHandle::new(),
                resource_task: resource_task,
            };

            cache.refresh_local_families();
            cache.run();
        });

        FontCacheTask {
            chan: chan,
        }
    }

    pub fn find_font_template(&self, family: FontFamily, desc: FontTemplateDescriptor)
                                                -> Option<Arc<FontTemplateData>> {

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
                                                -> Arc<FontTemplateData> {

        let (response_chan, response_port) = ipc::channel().unwrap();
        self.chan.send(Command::GetLastResortFontTemplate(desc, response_chan)).unwrap();

        let reply = response_port.recv().unwrap();

        match reply {
            Reply::GetFontTemplateReply(data) => {
                data.unwrap()
            }
        }
    }

    pub fn add_web_font(&self, family: FontFamily, src: Source, sender: IpcSender<()>) {
        self.chan.send(Command::AddWebFont(family, src, sender)).unwrap();
    }

    pub fn exit(&self) {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.chan.send(Command::Exit(response_chan)).unwrap();
        response_port.recv().unwrap();
    }
}
