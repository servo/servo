/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::font_list::get_available_families;
use platform::font_list::get_system_default_family;
use platform::font_list::get_variations_for_family;
use platform::font_list::get_last_resort_font_families;
use platform::font_context::FontContextHandle;

use collections::str::Str;
use font_template::{FontTemplate, FontTemplateDescriptor};
use platform::font_template::FontTemplateData;
use servo_net::resource_task::{ResourceTask, load_whole_resource};
use servo_net::server::{Server, SharedServerProxy};
use servo_util::task::spawn_named;
use servo_util::str::LowercaseString;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use style::Source;

/// A list of font templates that make up a given font family.
struct FontFamily {
    templates: Vec<FontTemplate>,
}

impl FontFamily {
    fn new() -> FontFamily {
        FontFamily {
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
        for template in self.templates.iter_mut() {
            let maybe_template = template.get_if_matches(fctx, desc);
            if maybe_template.is_some() {
                return maybe_template;
            }
        }

        // If a request is made for a font family that exists,
        // pick the first valid font in the family if we failed
        // to find an exact match for the descriptor.
        for template in self.templates.iter_mut() {
            let maybe_template = template.get();
            if maybe_template.is_some() {
                return maybe_template;
            }
        }

        None
    }

    fn add_template(&mut self, identifier: &str, maybe_data: Option<Vec<u8>>) {
        for template in self.templates.iter() {
            if template.identifier() == identifier {
                return;
            }
        }

        let template = FontTemplate::new(identifier, maybe_data);
        self.templates.push(template);
    }
}

/// Commands that the FontContext sends to the font cache task.
#[deriving(Decodable, Encodable)]
pub enum Command {
    GetFontTemplate(String, FontTemplateDescriptor),
    GetLastResortFontTemplate(FontTemplateDescriptor),
    AddWebFont(String, Source),
}

/// Reply messages sent from the font cache task to the FontContext caller.
#[deriving(Decodable, Encodable)]
pub enum Reply {
    GetFontTemplateReply(Option<Arc<FontTemplateData>>),
    AddWebFontReply,
}

/// The font cache task itself. It maintains a list of reference counted
/// font templates that are currently in use.
struct FontCache {
    server: Server<Command,Reply>,
    generic_fonts: HashMap<LowercaseString, LowercaseString>,
    local_families: HashMap<LowercaseString, FontFamily>,
    web_families: HashMap<LowercaseString, FontFamily>,
    font_context: FontContextHandle,
    resource_task: ResourceTask,
}

fn add_generic_font(generic_fonts: &mut HashMap<LowercaseString, LowercaseString>,
                    generic_name: &str, mapped_name: &str) {
    let opt_system_default = get_system_default_family(generic_name);
    let family_name = match opt_system_default {
        Some(system_default) => LowercaseString::new(system_default.as_slice()),
        None => LowercaseString::new(mapped_name),
    };
    generic_fonts.insert(LowercaseString::new(generic_name), family_name);
}

impl FontCache {
    fn run(&mut self) {
        while let Some(msgs) = self.server.recv() {
            for (client_id, msg) in msgs.into_iter() {
                match msg {
                    Command::GetFontTemplate(family, descriptor) => {
                        let family = LowercaseString::new(family.as_slice());
                        let maybe_font_template = self.get_font_template(&family, &descriptor);
                        self.server.send(client_id,
                                         Reply::GetFontTemplateReply(maybe_font_template));
                    }
                    Command::GetLastResortFontTemplate(descriptor) => {
                        let font_template = self.get_last_resort_font_template(&descriptor);
                        self.server.send(client_id,
                                         Reply::GetFontTemplateReply(Some(font_template)));
                    }
                    Command::AddWebFont(family_name, src) => {
                        let family_name = LowercaseString::new(family_name.as_slice());
                        if !self.web_families.contains_key(&family_name) {
                            let family = FontFamily::new();
                            self.web_families.insert(family_name.clone(), family);
                        }

                        match src {
                            Source::Url(ref url_source) => {
                                let url = &url_source.url;
                                let maybe_resource = load_whole_resource(&self.resource_task,
                                                                         url.clone());
                                match maybe_resource {
                                    Ok((_, bytes)) => {
                                        let family = &mut self.web_families[family_name];
                                        family.add_template(url.to_string().as_slice(),
                                                            Some(bytes));
                                    },
                                    Err(_) => {
                                        debug!("Failed to load web font: family={} url={}",
                                               family_name,
                                               url);
                                    }
                                }
                            }
                            Source::Local(ref local_family_name) => {
                                let family = &mut self.web_families[family_name];
                                get_variations_for_family(local_family_name.as_slice(), |path| {
                                    family.add_template(path.as_slice(), None);
                                });
                            }
                        }
                        self.server.send(client_id, Reply::AddWebFontReply);
                    }
                }
            }
        }
    }

    fn refresh_local_families(&mut self) {
        self.local_families.clear();
        get_available_families(|family_name| {
            let family_name = LowercaseString::new(family_name.as_slice());
            if !self.local_families.contains_key(&family_name) {
                let family = FontFamily::new();
                self.local_families.insert(family_name, family);
            }
        });
    }

    fn transform_family(&self, family: &LowercaseString) -> LowercaseString {
        match self.generic_fonts.get(family) {
            None => family.clone(),
            Some(mapped_family) => (*mapped_family).clone()
        }
    }

    fn find_font_in_local_family<'a>(&'a mut self,
                                     family_name: &LowercaseString,
                                     desc: &FontTemplateDescriptor)
                                     -> Option<Arc<FontTemplateData>> {
        // TODO(Issue #188): look up localized font family names if canonical name not found
        // look up canonical name
        if self.local_families.contains_key(family_name) {
            debug!("FontList: Found font family with name={}", family_name.as_slice());
            let s = &mut self.local_families[*family_name];

            if s.templates.len() == 0 {
                get_variations_for_family(family_name.as_slice(), |path| {
                    s.add_template(path.as_slice(), None);
                });
            }

            // TODO(Issue #192: handle generic font families, like 'serif' and 'sans-serif'.
            // if such family exists, try to match style to a font
            let result = s.find_font_for_style(desc, &self.font_context);
            if result.is_some() {
                return result;
            }

            None
        } else {
            debug!("FontList: Couldn't find font family with name={}", family_name.as_slice());
            None
        }
    }

    fn find_font_in_web_family<'a>(&'a mut self,
                                   family_name: &LowercaseString,
                                   desc: &FontTemplateDescriptor)
                                   -> Option<Arc<FontTemplateData>> {
        if self.web_families.contains_key(family_name) {
            let family = &mut self.web_families[*family_name];
            let maybe_font = family.find_font_for_style(desc, &self.font_context);
            maybe_font
        } else {
            None
        }
    }

    fn get_font_template(&mut self, family: &LowercaseString, desc: &FontTemplateDescriptor)
                            -> Option<Arc<FontTemplateData>> {
        let transformed_family_name = self.transform_family(family);
        let mut maybe_template = self.find_font_in_web_family(&transformed_family_name, desc);
        if maybe_template.is_none() {
            maybe_template = self.find_font_in_local_family(&transformed_family_name, desc);
        }
        maybe_template
    }

    fn get_last_resort_font_template(&mut self, desc: &FontTemplateDescriptor)
                                        -> Arc<FontTemplateData> {
        let last_resort = get_last_resort_font_families();

        for family in last_resort.iter() {
            let family = LowercaseString::new(family.as_slice());
            let maybe_font_in_family = self.find_font_in_local_family(&family, desc);
            if maybe_font_in_family.is_some() {
                return maybe_font_in_family.unwrap();
            }
        }

        panic!("Unable to find any fonts that match (do you have fallback fonts installed?)");
    }
}

/// The public interface to the font cache task, used exclusively by
/// the per-thread/task FontContext structures.
#[deriving(Clone)]
pub struct FontCacheTask {
    pub client: SharedServerProxy<Command,Reply>,
}

impl FontCacheTask {
    pub fn new(resource_task: ResourceTask) -> FontCacheTask {
        let mut server = Server::new("FontCache");
        let client = Arc::new(Mutex::new(server.create_new_client()));

        spawn_named("FontCacheTask".to_owned(), proc() {
            // TODO: Allow users to specify these.
            let mut generic_fonts = HashMap::with_capacity(5);
            add_generic_font(&mut generic_fonts, "serif", "Times New Roman");
            add_generic_font(&mut generic_fonts, "sans-serif", "Arial");
            add_generic_font(&mut generic_fonts, "cursive", "Apple Chancery");
            add_generic_font(&mut generic_fonts, "fantasy", "Papyrus");
            add_generic_font(&mut generic_fonts, "monospace", "Menlo");

            let mut cache = FontCache {
                server: server,
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
            client: client,
        }
    }

    #[inline]
    pub fn from_client(client: SharedServerProxy<Command,Reply>) -> FontCacheTask {
        FontCacheTask {
            client: client,
        }
    }

    pub fn get_font_template(&self, family: String, desc: FontTemplateDescriptor)
                                                -> Option<Arc<FontTemplateData>> {

        let response = self.client.lock().send_sync(Command::GetFontTemplate(family, desc));
        if let Reply::GetFontTemplateReply(data) = response {
            data
        } else {
            panic!("get_font_template(): unexpected server response")
        }
    }

    pub fn get_last_resort_font_template(&self, desc: FontTemplateDescriptor)
                                                -> Arc<FontTemplateData> {

        let response = self.client.lock().send_sync(Command::GetLastResortFontTemplate(desc));
        if let Reply::GetFontTemplateReply(data) = response {
            data.unwrap()
        } else {
            panic!("get_font_template(): unexpected server response")
        }
    }

    pub fn add_web_font(&self, family: String, src: Source) {
        self.client.lock().send_sync(Command::AddWebFont(family, src));
    }

    pub fn create_new_client(&self) -> FontCacheTask {
        FontCacheTask {
            client: Arc::new(Mutex::new(self.client.lock().create_new_client())),
        }
    }
}
