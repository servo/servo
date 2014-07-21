/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::font_list::get_available_families;
use platform::font_list::get_variations_for_family;
use platform::font_list::get_last_resort_font_families;
use platform::font_context::FontContextHandle;

use std::collections::HashMap;
use sync::Arc;
use font_template::{FontTemplate, FontTemplateDescriptor};
use platform::font_template::FontTemplateData;

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

    /// Find a font in this family that matches a given desriptor.
    fn find_font_for_style<'a>(&'a mut self, desc: &FontTemplateDescriptor, fctx: &FontContextHandle)
                               -> Option<Arc<FontTemplateData>> {
        // TODO(Issue #189): optimize lookup for
        // regular/bold/italic/bolditalic with fixed offsets and a
        // static decision table for fallback between these values.

        // TODO(Issue #190): if not in the fast path above, do
        // expensive matching of weights, etc.
        for template in self.templates.mut_iter() {
            let maybe_template = template.get_if_matches(fctx, desc);
            if maybe_template.is_some() {
                return maybe_template;
            }
        }

        None
    }
}

/// Commands that the FontContext sends to the font cache task.
pub enum Command {
    GetFontTemplate(String, FontTemplateDescriptor, Sender<Reply>),
    Exit(Sender<()>),
}

/// Reply messages sent from the font cache task to the FontContext caller.
pub enum Reply {
    GetFontTemplateReply(Arc<FontTemplateData>),
}

/// The font cache task itself. It maintains a list of reference counted
/// font templates that are currently in use.
struct FontCache {
    port: Receiver<Command>,
    generic_fonts: HashMap<String, String>,
    local_families: HashMap<String, FontFamily>,
    font_context: FontContextHandle,
}

impl FontCache {
    fn run(&mut self) {
        loop {
            let msg = self.port.recv();

            match msg {
                GetFontTemplate(family, descriptor, result) => {
                    let maybe_font_template = self.get_font_template(&family, &descriptor);

                    let font_template = match maybe_font_template {
                        Some(font_template) => font_template,
                        None => self.get_last_resort_template(&descriptor),
                    };

                    result.send(GetFontTemplateReply(font_template));
                }
                Exit(result) => {
                    result.send(());
                    break;
                }
            }
        }
    }

    fn refresh_local_families(&mut self) {
        self.local_families.clear();
        get_available_families(|family_name| {
            if !self.local_families.contains_key(&family_name) {
                let family = FontFamily::new();
                self.local_families.insert(family_name, family);
            }
        });
    }

    fn transform_family(&self, family: &String) -> String {
        match self.generic_fonts.find(family) {
            None => family.to_string(),
            Some(mapped_family) => (*mapped_family).clone()
        }
    }

    fn find_font_in_family<'a>(&'a mut self,
                                   family_name: &String,
                                   desc: &FontTemplateDescriptor) -> Option<Arc<FontTemplateData>> {
        // TODO(Issue #188): look up localized font family names if canonical name not found
        // look up canonical name
        if self.local_families.contains_key(family_name) {
            debug!("FontList: Found font family with name={:s}", family_name.to_string());
            let s = self.local_families.get_mut(family_name);

            if s.templates.len() == 0 {
                get_variations_for_family(family_name.as_slice(), |path| {
                    let template = FontTemplate::new(path.as_slice());
                    s.templates.push(template);
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
            debug!("FontList: Couldn't find font family with name={:s}", family_name.to_string());
            None
        }
    }

    fn get_font_template(&mut self, family: &String, desc: &FontTemplateDescriptor) -> Option<Arc<FontTemplateData>> {
        let transformed_family_name = self.transform_family(family);
        self.find_font_in_family(&transformed_family_name, desc)
    }

    fn get_last_resort_template(&mut self, desc: &FontTemplateDescriptor) -> Arc<FontTemplateData> {
        let last_resort = get_last_resort_font_families();

        for family in last_resort.iter() {
            let maybe_font_in_family = self.find_font_in_family(family, desc);
            if maybe_font_in_family.is_some() {
                return maybe_font_in_family.unwrap();
            }
        }

        fail!("Unable to find any fonts that match (do you have fallback fonts installed?)");
    }
}

/// The public interface to the font cache task, used exclusively by
/// the per-thread/task FontContext structures.
#[deriving(Clone)]
pub struct FontCacheTask {
    chan: Sender<Command>,
}

impl FontCacheTask {
    pub fn new() -> FontCacheTask {
        let (chan, port) = channel();

        spawn(proc() {
            // TODO: Allow users to specify these.
            let mut generic_fonts = HashMap::with_capacity(5);
            generic_fonts.insert("serif".to_string(), "Times New Roman".to_string());
            generic_fonts.insert("sans-serif".to_string(), "Arial".to_string());
            generic_fonts.insert("cursive".to_string(), "Apple Chancery".to_string());
            generic_fonts.insert("fantasy".to_string(), "Papyrus".to_string());
            generic_fonts.insert("monospace".to_string(), "Menlo".to_string());

            let mut cache = FontCache {
                port: port,
                generic_fonts: generic_fonts,
                local_families: HashMap::new(),
                font_context: FontContextHandle::new(),
            };

            cache.refresh_local_families();
            cache.run();
        });

        FontCacheTask {
            chan: chan,
        }
    }

    pub fn get_font_template(&mut self, family: String, desc: FontTemplateDescriptor)
                                                -> Arc<FontTemplateData> {

        let (response_chan, response_port) = channel();
        self.chan.send(GetFontTemplate(family, desc, response_chan));

        let reply = response_port.recv();

        match reply {
            GetFontTemplateReply(data) => {
                data
            }
        }
    }

    pub fn exit(&self) {
        let (response_chan, response_port) = channel();
        self.chan.send(Exit(response_chan));
        response_port.recv();
    }
}
