/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::font_list::get_available_families;
use platform::font_list::get_system_default_family;
use platform::font_list::get_variations_for_family;
use platform::font_list::get_last_resort_font_families;
use platform::font_context::FontContextHandle;

use font_template::{FontTemplate, FontTemplateDescriptor};
use net_traits::{ResourceTask, load_whole_resource};
use platform::font_template::FontTemplateData;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, channel};
use string_cache::Atom;
use style::font_face::Source;
use util::str::LowercaseString;
use util::task::spawn_named;

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
        for template in &mut self.templates {
            let maybe_template = template.get_if_matches(fctx, desc);
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
pub enum Command {
    GetFontTemplate(String, FontTemplateDescriptor, Sender<Reply>),
    GetLastResortFontTemplate(FontTemplateDescriptor, Sender<Reply>),
    AddWebFont(Atom, Source, Sender<()>),
    Exit(Sender<()>),
}

unsafe impl Send for Command {}

/// Reply messages sent from the font cache task to the FontContext caller.
pub enum Reply {
    GetFontTemplateReply(Option<Arc<FontTemplateData>>),
}

unsafe impl Send for Reply {}

/// The font cache task itself. It maintains a list of reference counted
/// font templates that are currently in use.
struct FontCache {
    port: Receiver<Command>,
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
        Some(system_default) => LowercaseString::new(&system_default),
        None => LowercaseString::new(mapped_name),
    };
    generic_fonts.insert(LowercaseString::new(generic_name), family_name);
}

impl FontCache {
    fn run(&mut self) {
        loop {
            let msg = self.port.recv().unwrap();

            match msg {
                Command::GetFontTemplate(family, descriptor, result) => {
                    let family = LowercaseString::new(&family);
                    let maybe_font_template = self.get_font_template(&family, &descriptor);
                    result.send(Reply::GetFontTemplateReply(maybe_font_template)).unwrap();
                }
                Command::GetLastResortFontTemplate(descriptor, result) => {
                    let font_template = self.get_last_resort_font_template(&descriptor);
                    result.send(Reply::GetFontTemplateReply(Some(font_template))).unwrap();
                }
                Command::AddWebFont(family_name, src, result) => {
                    let family_name = LowercaseString::new(&family_name);
                    if !self.web_families.contains_key(&family_name) {
                        let family = FontFamily::new();
                        self.web_families.insert(family_name.clone(), family);
                    }

                    match src {
                        Source::Url(ref url_source) => {
                            let url = &url_source.url;
                            let maybe_resource = load_whole_resource(&self.resource_task, url.clone());
                            match maybe_resource {
                                Ok((_, bytes)) => {
                                    let family = &mut self.web_families.get_mut(&family_name).unwrap();
                                    family.add_template(Atom::from_slice(&url.to_string()), Some(bytes));
                                },
                                Err(_) => {
                                    debug!("Failed to load web font: family={:?} url={}", family_name, url);
                                }
                            }
                        }
                        Source::Local(ref local_family_name) => {
                            let family = &mut self.web_families.get_mut(&family_name).unwrap();
                            get_variations_for_family(&local_family_name, |path| {
                                family.add_template(Atom::from_slice(&path), None);
                            });
                        }
                    }
                    result.send(()).unwrap();
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
        get_available_families(|family_name| {
            let family_name = LowercaseString::new(&family_name);
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

    fn find_font_in_local_family<'a>(&'a mut self, family_name: &LowercaseString, desc: &FontTemplateDescriptor)
                                -> Option<Arc<FontTemplateData>> {
        // TODO(Issue #188): look up localized font family names if canonical name not found
        // look up canonical name
        if self.local_families.contains_key(family_name) {
            debug!("FontList: Found font family with name={}", &**family_name);
            let s = self.local_families.get_mut(family_name).unwrap();

            if s.templates.is_empty() {
                get_variations_for_family(family_name, |path| {
                    s.add_template(Atom::from_slice(&path), None);
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
            debug!("FontList: Couldn't find font family with name={}", &**family_name);
            None
        }
    }

    fn find_font_in_web_family<'a>(&'a mut self, family_name: &LowercaseString, desc: &FontTemplateDescriptor)
                                -> Option<Arc<FontTemplateData>> {
        if self.web_families.contains_key(family_name) {
            let family = self.web_families.get_mut(family_name).unwrap();
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

        for family in &last_resort {
            let family = LowercaseString::new(family);
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
#[derive(Clone)]
pub struct FontCacheTask {
    chan: Sender<Command>,
}

impl FontCacheTask {
    pub fn new(resource_task: ResourceTask) -> FontCacheTask {
        let (chan, port) = channel();

        spawn_named("FontCacheTask".to_owned(), move || {
            // TODO: Allow users to specify these.
            let mut generic_fonts = HashMap::with_capacity(5);
            add_generic_font(&mut generic_fonts, "serif", "Times New Roman");
            add_generic_font(&mut generic_fonts, "sans-serif", "Arial");
            add_generic_font(&mut generic_fonts, "cursive", "Apple Chancery");
            add_generic_font(&mut generic_fonts, "fantasy", "Papyrus");
            add_generic_font(&mut generic_fonts, "monospace", "Menlo");

            let mut cache = FontCache {
                port: port,
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

    pub fn get_font_template(&self, family: String, desc: FontTemplateDescriptor)
                                                -> Option<Arc<FontTemplateData>> {

        let (response_chan, response_port) = channel();
        self.chan.send(Command::GetFontTemplate(family, desc, response_chan)).unwrap();

        let reply = response_port.recv().unwrap();

        match reply {
            Reply::GetFontTemplateReply(data) => {
                data
            }
        }
    }

    pub fn get_last_resort_font_template(&self, desc: FontTemplateDescriptor)
                                                -> Arc<FontTemplateData> {

        let (response_chan, response_port) = channel();
        self.chan.send(Command::GetLastResortFontTemplate(desc, response_chan)).unwrap();

        let reply = response_port.recv().unwrap();

        match reply {
            Reply::GetFontTemplateReply(data) => {
                data.unwrap()
            }
        }
    }

    pub fn add_web_font(&self, family: Atom, src: Source) {
        let (response_chan, response_port) = channel();
        self.chan.send(Command::AddWebFont(family, src, response_chan)).unwrap();
        response_port.recv().unwrap();
    }

    pub fn exit(&self) {
        let (response_chan, response_port) = channel();
        self.chan.send(Command::Exit(response_chan)).unwrap();
        response_port.recv().unwrap();
    }
}
