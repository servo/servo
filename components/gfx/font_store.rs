/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use parking_lot::RwLock;
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey};

use crate::font::FontDescriptor;
use crate::font_cache_thread::{FontIdentifier, FontSource, LowercaseString};
use crate::font_template::{FontTemplate, FontTemplateRef, FontTemplateRefMethods};

#[derive(Default)]
pub struct FontStore {
    pub(crate) families: HashMap<LowercaseString, FontTemplates>,
}
pub(crate) type CrossThreadFontStore = Arc<RwLock<FontStore>>;

impl FontStore {
    pub(crate) fn clear(&mut self) {
        self.families.clear();
    }
}

#[derive(Default)]
pub struct WebRenderFontStore {
    pub(crate) webrender_font_key_map: HashMap<FontIdentifier, FontKey>,
    pub(crate) webrender_font_instance_map: HashMap<(FontKey, Au), FontInstanceKey>,
}
pub(crate) type CrossThreadWebRenderFontStore = Arc<RwLock<WebRenderFontStore>>;

impl WebRenderFontStore {
    pub(crate) fn get_font_instance<FCT: FontSource>(
        &mut self,
        font_cache_thread: &FCT,
        font_template: FontTemplateRef,
        pt_size: Au,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        let webrender_font_key_map = &mut self.webrender_font_key_map;
        let identifier = font_template.identifier().clone();
        let font_key = *webrender_font_key_map
            .entry(identifier.clone())
            .or_insert_with(|| {
                font_cache_thread.get_web_font(font_template.data(), identifier.index())
            });

        *self
            .webrender_font_instance_map
            .entry((font_key, pt_size))
            .or_insert_with(|| {
                font_cache_thread.get_web_font_instance(font_key, pt_size.to_f32_px(), flags)
            })
    }
}

/// A list of font templates that make up a given font family.
#[derive(Clone, Debug, Default)]
pub struct FontTemplates {
    pub(crate) templates: Vec<FontTemplateRef>,
}

impl FontTemplates {
    /// Find a font in this family that matches a given descriptor.
    pub fn find_for_descriptor(
        &self,
        descriptor_to_match: Option<&FontDescriptor>,
    ) -> Vec<FontTemplateRef> {
        let Some(descriptor_to_match) = descriptor_to_match else {
            return self.templates.clone();
        };

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
        if let Some(template) = self.templates.first() {
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
        self.templates
            .push(Arc::new(AtomicRefCell::new(new_template)));
    }
}
