/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use log::warn;
use parking_lot::RwLock;
use style::stylesheets::DocumentStyleSheet;
use style::values::computed::{FontStyle, FontWeight};
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey};

use crate::font::FontDescriptor;
use crate::font_cache_thread::{FontIdentifier, FontSource, LowercaseFontFamilyName};
use crate::font_context::WebFontDownloadState;
use crate::font_template::{FontTemplate, FontTemplateRef, FontTemplateRefMethods, IsOblique};

#[derive(Default)]
pub struct FontStore {
    pub(crate) families: HashMap<LowercaseFontFamilyName, FontTemplates>,
    web_fonts_loading: Vec<(DocumentStyleSheet, usize)>,
}
pub(crate) type CrossThreadFontStore = Arc<RwLock<FontStore>>;

impl FontStore {
    pub(crate) fn clear(&mut self) {
        self.families.clear();
    }

    pub(crate) fn font_load_cancelled_for_stylesheet(
        &self,
        stylesheet: &DocumentStyleSheet,
    ) -> bool {
        !self
            .web_fonts_loading
            .iter()
            .any(|(loading_stylesheet, _)| loading_stylesheet == stylesheet)
    }

    pub(crate) fn handle_stylesheet_removed(&mut self, stylesheet: &DocumentStyleSheet) {
        self.web_fonts_loading
            .retain(|(loading_stylesheet, _)| loading_stylesheet != stylesheet);
    }

    pub(crate) fn handle_web_font_load_started_for_stylesheet(
        &mut self,
        stylesheet: &DocumentStyleSheet,
    ) {
        if let Some((_, count)) = self
            .web_fonts_loading
            .iter_mut()
            .find(|(loading_stylesheet, _)| loading_stylesheet == stylesheet)
        {
            *count += 1;
        } else {
            self.web_fonts_loading.push((stylesheet.clone(), 1))
        }
    }

    fn remove_one_web_font_loading_for_stylesheet(&mut self, stylesheet: &DocumentStyleSheet) {
        if let Some((_, count)) = self
            .web_fonts_loading
            .iter_mut()
            .find(|(loading_stylesheet, _)| loading_stylesheet == stylesheet)
        {
            *count -= 1;
        }
        self.web_fonts_loading.retain(|(_, count)| *count != 0);
    }

    pub(crate) fn handle_web_font_failed_to_load(&mut self, state: &WebFontDownloadState) {
        self.remove_one_web_font_loading_for_stylesheet(&state.stylesheet);
    }

    /// Handle a web font load finishing, adding the new font to the [`FontStore`]. If the web font
    /// load was canceled (for instance, if the stylesheet was removed), then do nothing and return
    /// false.
    pub(crate) fn handle_web_font_loaded(
        &mut self,
        state: &WebFontDownloadState,
        new_template: FontTemplate,
    ) -> bool {
        // Abort processing this web font if the originating stylesheet was removed.
        if self.font_load_cancelled_for_stylesheet(&state.stylesheet) {
            return false;
        }

        let family_name = state.css_font_face_descriptors.family_name.clone();
        self.families
            .entry(family_name)
            .or_default()
            .add_template(new_template);
        self.remove_one_web_font_loading_for_stylesheet(&state.stylesheet);
        true
    }

    pub(crate) fn number_of_fonts_still_loading(&self) -> usize {
        self.web_fonts_loading.iter().map(|(_, count)| count).sum()
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

    pub(crate) fn remove_all_fonts(&mut self) -> (Vec<FontKey>, Vec<FontInstanceKey>) {
        (
            self.webrender_font_key_map
                .drain()
                .map(|(_, key)| key)
                .collect(),
            self.webrender_font_instance_map
                .drain()
                .map(|(_, key)| key)
                .collect(),
        )
    }

    pub(crate) fn remove_all_fonts_for_identifiers(
        &mut self,
        identifiers: HashSet<FontIdentifier>,
    ) -> (Vec<FontKey>, Vec<FontInstanceKey>) {
        let mut removed_keys: HashSet<FontKey> = HashSet::new();
        self.webrender_font_key_map.retain(|identifier, font_key| {
            if identifiers.contains(identifier) {
                removed_keys.insert(*font_key);
                false
            } else {
                true
            }
        });

        let mut removed_instance_keys: HashSet<FontInstanceKey> = HashSet::new();
        self.webrender_font_instance_map
            .retain(|(font_key, _), instance_key| {
                if removed_keys.contains(font_key) {
                    removed_instance_keys.insert(*instance_key);
                    false
                } else {
                    true
                }
            });

        (
            removed_keys.into_iter().collect(),
            removed_instance_keys.into_iter().collect(),
        )
    }
}

/// A struct that represents the available templates in a "simple family." A simple family
/// is one that contains <= 4 available faces: regular, bold, italic, and bold italic. Having
/// this simple family abstraction makes font matching much faster for families that don't
/// have a complex set of fonts.
///
/// This optimization is taken from:
/// https://searchfox.org/mozilla-central/source/gfx/thebes/gfxFontEntry.cpp.
#[derive(Clone, Debug, Default)]
struct SimpleFamily {
    regular: Option<FontTemplateRef>,
    bold: Option<FontTemplateRef>,
    italic: Option<FontTemplateRef>,
    bold_italic: Option<FontTemplateRef>,
}

impl SimpleFamily {
    /// Find a font in this family that matches a given descriptor.
    fn find_for_descriptor(&self, descriptor_to_match: &FontDescriptor) -> Option<FontTemplateRef> {
        let want_bold = descriptor_to_match.weight >= FontWeight::BOLD_THRESHOLD;
        let want_italic = descriptor_to_match.style != FontStyle::NORMAL;

        // This represents the preference of which font to return from the [`SimpleFamily`],
        // given what kind of font we are requesting.
        let preference = match (want_bold, want_italic) {
            (true, true) => [&self.bold_italic, &self.italic, &self.bold, &self.regular],
            (true, false) => [&self.bold, &self.regular, &self.bold_italic, &self.italic],
            (false, true) => [&self.italic, &self.bold_italic, &self.regular, &self.bold],
            (false, false) => [&self.regular, &self.bold, &self.italic, &self.bold_italic],
        };
        preference
            .iter()
            .filter_map(|template| (*template).clone())
            .next()
    }

    fn remove_templates_for_stylesheet(&mut self, stylesheet: &DocumentStyleSheet) {
        let remove_if_template_matches = |template: &mut Option<FontTemplateRef>| {
            if template.as_ref().map_or(false, |template| {
                template.borrow().stylesheet.as_ref() == Some(stylesheet)
            }) {
                *template = None;
            }
        };
        remove_if_template_matches(&mut self.regular);
        remove_if_template_matches(&mut self.bold);
        remove_if_template_matches(&mut self.italic);
        remove_if_template_matches(&mut self.bold_italic);
    }

    pub(crate) fn for_all_identifiers(&self, mut callback: impl FnMut(&FontIdentifier)) {
        let mut call_if_not_none = |template: &Option<FontTemplateRef>| {
            if let Some(template) = template {
                callback(&template.identifier())
            }
        };
        call_if_not_none(&self.regular);
        call_if_not_none(&self.bold);
        call_if_not_none(&self.italic);
        call_if_not_none(&self.bold_italic);
    }
}
/// A list of font templates that make up a given font family.
#[derive(Clone, Debug)]
pub struct FontTemplates {
    pub(crate) templates: Vec<FontTemplateRef>,
    simple_family: Option<SimpleFamily>,
}

impl Default for FontTemplates {
    fn default() -> Self {
        Self {
            templates: Default::default(),
            simple_family: Some(SimpleFamily::default()),
        }
    }
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

        if self.templates.len() == 1 {
            return vec![self.templates[0].clone()];
        }

        if let Some(template) = self
            .simple_family
            .as_ref()
            .and_then(|simple_family| simple_family.find_for_descriptor(descriptor_to_match))
        {
            return vec![template];
        }

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

        let new_template = Arc::new(AtomicRefCell::new(new_template));
        self.templates.push(new_template.clone());
        self.update_simple_family(new_template);
    }

    fn update_simple_family(&mut self, added_template: FontTemplateRef) {
        // If this was detected to not be a simple family before, it cannot ever be one
        // in the future.
        let Some(simple_family) = self.simple_family.as_mut() else {
            return;
        };

        if self.templates.len() > 4 {
            self.simple_family = None;
            return;
        }

        // Variation fonts are never simple families.
        if added_template.descriptor().is_variation_font() {
            self.simple_family = None;
            return;
        }

        let Some(first) = self.templates.first() else {
            warn!("Called before adding any templates.");
            return;
        };

        // If the stretch between any of these fonts differ, it cannot be a simple family nor if this
        // font is oblique.
        let stretch = added_template.descriptor().stretch.0;
        let style = added_template.descriptor().style.0;
        if first.descriptor().stretch.0 != stretch || style.is_oblique() {
            self.simple_family = None;
            return;
        }

        let weight = added_template.descriptor().weight.0;
        let supports_bold = weight >= FontWeight::BOLD_THRESHOLD;
        let is_italic = style == FontStyle::ITALIC;
        let slot = match (supports_bold, is_italic) {
            (true, true) => &mut simple_family.bold_italic,
            (true, false) => &mut simple_family.bold,
            (false, true) => &mut simple_family.italic,
            (false, false) => &mut simple_family.regular,
        };

        // If the slot was already filled there are two fonts with the same simple properties
        // and this isn't a simple family.
        if slot.is_some() {
            self.simple_family = None;
            return;
        }

        slot.replace(added_template);
    }

    pub(crate) fn remove_templates_for_stylesheet(
        &mut self,
        stylesheet: &DocumentStyleSheet,
    ) -> bool {
        let length_before = self.templates.len();
        self.templates
            .retain(|template| template.borrow().stylesheet.as_ref() != Some(stylesheet));

        if let Some(simple_family) = self.simple_family.as_mut() {
            simple_family.remove_templates_for_stylesheet(stylesheet);
        }

        length_before != self.templates.len()
    }

    pub(crate) fn for_all_identifiers(&self, mut callback: impl FnMut(&FontIdentifier)) {
        for template in self.templates.iter() {
            callback(&template.borrow().identifier);
        }
        if let Some(ref simple_family) = self.simple_family {
            simple_family.for_all_identifiers(callback)
        }
    }
}
