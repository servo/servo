/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::FontHandleMethods;
use platform::font::FontHandle;
use platform::font_context::FontContextHandle;
use platform::font_template::FontTemplateData;
use string_cache::Atom;

use std::sync::{Arc, Weak};
use style::computed_values::{font_stretch, font_weight};

/// Describes how to select a font from a given family. This is very basic at the moment and needs
/// to be expanded or refactored when we support more of the font styling parameters.
///
/// NB: If you change this, you will need to update `style::properties::compute_font_hash()`.
#[derive(Clone, Copy, Eq, Hash)]
pub struct FontTemplateDescriptor {
    pub weight: font_weight::T,
    pub stretch: font_stretch::T,
    pub italic: bool,
}

impl FontTemplateDescriptor {
    #[inline]
    pub fn new(weight: font_weight::T, stretch: font_stretch::T, italic: bool)
               -> FontTemplateDescriptor {
        FontTemplateDescriptor {
            weight: weight,
            stretch: stretch,
            italic: italic,
        }
    }
}

impl PartialEq for FontTemplateDescriptor {
    fn eq(&self, other: &FontTemplateDescriptor) -> bool {
        self.weight.is_bold() == other.weight.is_bold() &&
            self.stretch == other.stretch &&
            self.italic == other.italic
    }
}

/// This describes all the information needed to create
/// font instance handles. It contains a unique
/// FontTemplateData structure that is platform specific.
pub struct FontTemplate {
    identifier: Atom,
    descriptor: Option<FontTemplateDescriptor>,
    weak_ref: Option<Weak<FontTemplateData>>,
    // GWTODO: Add code path to unset the strong_ref for web fonts!
    strong_ref: Option<Arc<FontTemplateData>>,
    is_valid: bool,
}

/// Holds all of the template information for a font that
/// is common, regardless of the number of instances of
/// this font handle per thread.
impl FontTemplate {
    pub fn new(identifier: Atom, maybe_bytes: Option<Vec<u8>>) -> FontTemplate {
        let maybe_data = match maybe_bytes {
            Some(_) => Some(FontTemplateData::new(identifier.clone(), maybe_bytes)),
            None => None,
        };

        let maybe_strong_ref = match maybe_data {
            Some(data) => Some(Arc::new(data)),
            None => None,
        };

        let maybe_weak_ref = match maybe_strong_ref {
            Some(ref strong_ref) => Some(Arc::downgrade(strong_ref)),
            None => None,
        };

        FontTemplate {
            identifier: identifier,
            descriptor: None,
            weak_ref: maybe_weak_ref,
            strong_ref: maybe_strong_ref,
            is_valid: true,
        }
    }

    pub fn identifier(&self) -> &Atom {
        &self.identifier
    }

    /// Get the data for creating a font if it matches a given descriptor.
    pub fn data_for_descriptor(&mut self,
                          fctx: &FontContextHandle,
                          requested_desc: &FontTemplateDescriptor)
                          -> Option<Arc<FontTemplateData>> {
        // The font template data can be unloaded when nothing is referencing
        // it (via the Weak reference to the Arc above). However, if we have
        // already loaded a font, store the style information about it separately,
        // so that we can do font matching against it again in the future
        // without having to reload the font (unless it is an actual match).
        match self.descriptor {
            Some(actual_desc) => {
                if *requested_desc == actual_desc {
                    Some(self.data())
                } else {
                    None
                }
            },
            None if self.is_valid => {
                let data = self.data();
                let handle: Result<FontHandle, ()> =
                    FontHandleMethods::new_from_template(fctx, data.clone(), None);
                match handle {
                    Ok(handle) => {
                        let actual_desc = FontTemplateDescriptor::new(handle.boldness(),
                                                                      handle.stretchiness(),
                                                                      handle.is_italic());
                        let desc_match = actual_desc == *requested_desc;

                        self.descriptor = Some(actual_desc);
                        self.is_valid = true;
                        if desc_match {
                            Some(data)
                        } else {
                            None
                        }
                    }
                    Err(()) => {
                        self.is_valid = false;
                        debug!("Unable to create a font from template {}", self.identifier);
                        None
                    }
                }
            }
            None => None,
        }
    }

    /// Get the data for creating a font.
    pub fn get(&mut self) -> Option<Arc<FontTemplateData>> {
        if self.is_valid {
            Some(self.data())
        } else {
            None
        }
    }

    /// Get the font template data. If any strong references still
    /// exist, it will return a clone, otherwise it will load the
    /// font data and store a weak reference to it internally.
    pub fn data(&mut self) -> Arc<FontTemplateData> {
        let maybe_data = match self.weak_ref {
            Some(ref data) => data.upgrade(),
            None => None,
        };

        if let Some(data) = maybe_data {
            return data
        }

        assert!(self.strong_ref.is_none());
        let template_data = Arc::new(FontTemplateData::new(self.identifier.clone(), None));
        self.weak_ref = Some(Arc::downgrade(&template_data));
        template_data
    }
}
