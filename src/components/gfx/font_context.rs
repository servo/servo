/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::{Font, FontDescriptor, FontGroup, FontHandleMethods, FontStyle,
           SelectorPlatformIdentifier};
use font::{SpecifiedFontStyle, UsedFontStyle};
use font_list::FontList;
use servo_util::cache::{Cache, LRUCache};
use servo_util::time::ProfilerChan;

use platform::font::FontHandle;
use platform::font_context::FontContextHandle;

use azure::azure_hl::BackendType;
use std::hashmap::HashMap;

// TODO(Rust #3934): creating lots of new dummy styles is a workaround
// for not being able to store symbolic enums in top-level constants.
pub fn dummy_style() -> FontStyle {
    use font::FontWeight300;
    return FontStyle {
        pt_size: 20f64,
        weight: FontWeight300,
        italic: false,
        oblique: false,
        families: ~"serif, sans-serif",
    }
}

pub trait FontContextHandleMethods {
    fn clone(&self) -> FontContextHandle;
    fn create_font_from_identifier(&self, ~str, UsedFontStyle) -> Result<FontHandle, ()>;
}

pub struct FontContext {
    instance_cache: LRUCache<FontDescriptor, @mut Font>,
    font_list: Option<FontList>, // only needed by layout
    group_cache: LRUCache<SpecifiedFontStyle, @FontGroup>,
    handle: FontContextHandle,
    backend: BackendType,
    generic_fonts: HashMap<~str,~str>,
    profiler_chan: ProfilerChan,
}

impl<'self> FontContext {
    pub fn new(backend: BackendType,
           needs_font_list: bool,
           profiler_chan: ProfilerChan)
           -> FontContext {
        let handle = FontContextHandle::new();
        let font_list = if needs_font_list { 
                            Some(FontList::new(&handle, profiler_chan.clone())) }
                        else { None };

        // TODO: Allow users to specify these.
        let mut generic_fonts = HashMap::with_capacity(5);
        generic_fonts.insert(~"serif", ~"Times New Roman");
        generic_fonts.insert(~"sans-serif", ~"Arial");
        generic_fonts.insert(~"cursive", ~"Apple Chancery");
        generic_fonts.insert(~"fantasy", ~"Papyrus");
        generic_fonts.insert(~"monospace", ~"Menlo");

        FontContext { 
            instance_cache: LRUCache::new(10),
            font_list: font_list,
            group_cache: LRUCache::new(10),
            handle: handle,
            backend: backend,
            generic_fonts: generic_fonts,
            profiler_chan: profiler_chan,
        }
    }

    fn get_font_list(&'self self) -> &'self FontList {
        self.font_list.get_ref()
    }

    pub fn get_resolved_font_for_style(&mut self, style: &SpecifiedFontStyle) -> @FontGroup {
        match self.group_cache.find(style) {
            Some(fg) => {
                debug!("font group cache hit");
                fg
            },
            None => {
                debug!("font group cache miss");
                let fg = self.create_font_group(style);
                self.group_cache.insert(style.clone(), fg);
                fg
            }
        }
    }

    pub fn get_font_by_descriptor(&mut self, desc: &FontDescriptor) -> Result<@mut Font, ()> {
        match self.instance_cache.find(desc) {
            Some(f) => {
                debug!("font cache hit");
                Ok(f)
            },
            None => { 
                debug!("font cache miss");
                let result = self.create_font_instance(desc);
                match result {
                    Ok(font) => {
                        self.instance_cache.insert(desc.clone(), font);
                    }, _ => {}
                };
                result
            }
        }
    }

    fn transform_family(&self, family: &str) -> ~str {
        // FIXME: Need a find_like() in HashMap.
        let family = family.to_str();
        debug!("(transform family) searching for `{:s}`", family);
        match self.generic_fonts.find(&family) {
            None => family,
            Some(mapped_family) => (*mapped_family).clone()
        }
    }

    fn create_font_group(&mut self, style: &SpecifiedFontStyle) -> @FontGroup {
        let mut fonts = ~[];

        debug!("(create font group) --- starting ---");

        // TODO(Issue #193): make iteration over 'font-family' more robust.
        for family in style.families.split_iter(',') {
            let family_name = family.trim();
            let transformed_family_name = self.transform_family(family_name);
            debug!("(create font group) transformed family is `{:s}`", transformed_family_name);
            let mut found = false;

            let result = match self.font_list {
                Some(ref mut fl) => {
                    let font_in_family = fl.find_font_in_family(&transformed_family_name, style);
                    if font_in_family.is_some() {
                        let font_entry = font_in_family.unwrap();
                        let font_id =
                            SelectorPlatformIdentifier(font_entry.handle.face_identifier());
                            let font_desc = FontDescriptor::new((*style).clone(), font_id);
                            Some(font_desc)
                    } else {
                        None
                    }
                }
                None => None,
            };

            if result.is_some() {
                found = true;
                let instance = self.get_font_by_descriptor(&result.unwrap());

                for font in instance.iter() { fonts.push(*font); }
            }


            if !found {
                debug!("(create font group) didn't find `{:s}`", transformed_family_name);
            }
        }

        if fonts.len() == 0 {
            let last_resort = FontList::get_last_resort_font_families();
            for family in last_resort.iter() {
                let result = match self.font_list {
                    Some(ref fl) => fl.find_font_in_family(*family, style),
                        None => None,
                };

                for font_entry in result.iter() {
                    let font_id =
                        SelectorPlatformIdentifier(font_entry.handle.face_identifier());
                    let font_desc = FontDescriptor::new((*style).clone(), font_id);
                    let instance = self.get_font_by_descriptor(&font_desc);

                    for font in instance.iter() {
                        fonts.push(*font);
                    }
                }
            }
        }

        assert!(fonts.len() > 0);
        // TODO(Issue #179): Split FontStyle into specified and used styles
        let used_style = (*style).clone();

        debug!("(create font group) --- finished ---");

        FontGroup::new(style.families.to_owned(), &used_style, fonts)
    }

    fn create_font_instance(&self, desc: &FontDescriptor) -> Result<@mut Font, ()> {
        return match &desc.selector {
            // TODO(Issue #174): implement by-platform-name font selectors.
            &SelectorPlatformIdentifier(ref identifier) => { 
                let result_handle = self.handle.create_font_from_identifier((*identifier).clone(),
                                                                            desc.style.clone());
                do result_handle.and_then |handle| {
                    Ok(Font::new_from_adopted_handle(self,
                                                     handle,
                                                     &desc.style,
                                                     self.backend,
                                                     self.profiler_chan.clone()))
                }
            }
        };
    }
}
