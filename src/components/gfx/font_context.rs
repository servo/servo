/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::{Font, FontDescriptor, FontGroup, FontHandleMethods,
           SelectorPlatformIdentifier};
use font::{SpecifiedFontStyle, UsedFontStyle};
use font_list::FontList;
use servo_util::cache::{Cache, LRUCache};
use servo_util::time::ProfilerChan;

use platform::font::FontHandle;
use platform::font_context::FontContextHandle;

use azure::azure_hl::BackendType;
use std::hashmap::HashMap;

use std::rc::Rc;
use std::cell::RefCell;


pub trait FontContextHandleMethods {
    fn create_font_from_identifier(&self, ~str, UsedFontStyle) -> Result<FontHandle, ()>;
}

pub struct FontContext {
    instance_cache: LRUCache<FontDescriptor, Rc<RefCell<Font>>>,
    font_list: Option<FontList>, // only needed by layout
    group_cache: LRUCache<SpecifiedFontStyle, Rc<RefCell<FontGroup>>>,
    handle: FontContextHandle,
    backend: BackendType,
    generic_fonts: HashMap<~str,~str>,
    profiler_chan: ProfilerChan,
}

impl FontContext {
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

    pub fn get_resolved_font_for_style(&mut self, style: &SpecifiedFontStyle)
                                       -> Rc<RefCell<FontGroup>> {
        match self.group_cache.find(style) {
            Some(fg) => {
                debug!("font group cache hit");
                fg
            },
            None => {
                debug!("font group cache miss");
                let fg = self.create_font_group(style);
                self.group_cache.insert(style.clone(), fg.clone());
                fg
            }
        }
    }

    pub fn get_font_by_descriptor(&mut self, desc: &FontDescriptor)
                                  -> Result<Rc<RefCell<Font>>, ()> {
        match self.instance_cache.find(desc) {
            Some(f) => {
                debug!("font cache hit");
                Ok(f)
            },
            None => {
                debug!("font cache miss");
                let result = self.create_font_instance(desc);
                match result.clone() {
                    Ok(ref font) => {
                        self.instance_cache.insert(desc.clone(), font.clone());
                    }, _ => {}
                };
                result
            }
        }
    }

    fn transform_family(&self, family: &~str) -> ~str {
        debug!("(transform family) searching for `{:s}`", family.as_slice());
        match self.generic_fonts.find(family) {
            None => family.to_owned(),
            Some(mapped_family) => (*mapped_family).clone()
        }
    }

    fn create_font_group(&mut self, style: &SpecifiedFontStyle) -> Rc<RefCell<FontGroup>> {
        let mut fonts = ~[];

        debug!("(create font group) --- starting ---");

        // TODO(Issue #193): make iteration over 'font-family' more robust.
        for family in style.families.iter() {
            let transformed_family_name = self.transform_family(family);
            debug!("(create font group) transformed family is `{:s}`", transformed_family_name);
            let mut found = false;

            let result = match self.font_list {
                Some(ref mut fl) => {
                    let font_in_family = fl.find_font_in_family(&transformed_family_name, style);
                    match font_in_family {
                        Some(font_entry) => {
                            let font_id =
                                SelectorPlatformIdentifier(font_entry.handle.face_identifier());
                            let font_desc = FontDescriptor::new((*style).clone(), font_id);
                            Some(font_desc)
                        },
                        None => {
                            None
                        }
                    }
                }
                None => None,
            };

            match result {
                Some(ref result) => {
                    found = true;
                    let instance = self.get_font_by_descriptor(result);
                    instance.map(|font| fonts.push(font.clone()));
                },
                _ => {}
            }

            if !found {
                debug!("(create font group) didn't find `{:s}`", transformed_family_name);
            }
        }

        if fonts.len() == 0 {
            let last_resort = FontList::get_last_resort_font_families();
            for family in last_resort.iter() {
                let font_desc = match self.font_list {
                    Some(ref mut font_list) => {
                        let font_desc = {
                            let font_entry = font_list.find_font_in_family(family, style);
                            match font_entry {
                                Some(v) => {
                                    let font_id =
                                        SelectorPlatformIdentifier(v.handle.face_identifier());
                                    Some(FontDescriptor::new((*style).clone(), font_id))
                                },
                                None => {
                                    None
                                }
                            }
                        };
                        font_desc
                    },
                    None => {
                        None
                    }
                };

                match font_desc {
                    Some(ref fd) => {
                        let instance = self.get_font_by_descriptor(fd);
                        instance.map(|font| fonts.push(font.clone()));
                    },
                    None => { }
                };
            }
        }
        assert!(fonts.len() > 0);
        // TODO(Issue #179): Split FontStyle into specified and used styles
        let used_style = (*style).clone();

        debug!("(create font group) --- finished ---");

        unsafe {
            Rc::new_unchecked(
                RefCell::new(
                    FontGroup::new(style.families.to_owned(), &used_style, fonts)))
        }
    }

    fn create_font_instance(&self, desc: &FontDescriptor) -> Result<Rc<RefCell<Font>>, ()> {
        return match &desc.selector {
            // TODO(Issue #174): implement by-platform-name font selectors.
            &SelectorPlatformIdentifier(ref identifier) => {
                let result_handle = self.handle.create_font_from_identifier((*identifier).clone(),
                                                                            desc.style.clone());
                result_handle.and_then(|handle| {
                    Ok(
                        Rc::from_mut(
                            RefCell::new(
                                Font::new_from_adopted_handle(self,
                                                              handle,
                                                              &desc.style,
                                                              self.backend))))
                })
            }
        };
    }
}
