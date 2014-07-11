/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::computed_values::font_weight;
use platform::font_context::FontContextHandle;
use platform::font::FontHandle;
use platform::font_template::FontTemplateData;

use sync::{Arc, Weak};
use font::FontHandleMethods;

/// Describes how to select a font from a given family.
/// This is very basic at the moment and needs to be
/// expanded or refactored when we support more of the
/// font styling parameters.
#[deriving(Clone)]
pub struct FontTemplateDescriptor {
    pub weight: font_weight::T,
    pub italic: bool,
}

impl FontTemplateDescriptor {
    pub fn new(weight: font_weight::T, italic: bool) -> FontTemplateDescriptor {
        FontTemplateDescriptor {
            weight: weight,
            italic: italic,
        }
    }
}

impl PartialEq for FontTemplateDescriptor {
    fn eq(&self, other: &FontTemplateDescriptor) -> bool {
        self.weight.is_bold() == other.weight.is_bold() &&
        self.italic == other.italic
    }
}

/// This describes all the information needed to create
/// font instance handles. It contains a unique
/// FontTemplateData structure that is platform specific.
pub struct FontTemplate {
    identifier: String,
    descriptor: Option<FontTemplateDescriptor>,
    data: Option<Weak<FontTemplateData>>,
}

/// Holds all of the template information for a font that
/// is common, regardless of the number of instances of
/// this font handle per thread.
impl FontTemplate {
    pub fn new(identifier: &str) -> FontTemplate {
        FontTemplate {
            identifier: identifier.to_string(),
            descriptor: None,
            data: None,
        }
    }

    /// Get the data for creating a font if it matches a given descriptor.
    pub fn get_if_matches(&mut self, fctx: &FontContextHandle,
                            requested_desc: &FontTemplateDescriptor) -> Option<Arc<FontTemplateData>> {
        // The font template data can be unloaded when nothing is referencing
        // it (via the Weak reference to the Arc above). However, if we have
        // already loaded a font, store the style information about it separately,
        // so that we can do font matching against it again in the future
        // without having to reload the font (unless it is an actual match).
        match self.descriptor {
            Some(actual_desc) => {
                if *requested_desc == actual_desc {
                    Some(self.get_data())
                } else {
                    None
                }
            },
            None => {
                let data = self.get_data();
                let handle = FontHandleMethods::new_from_template(fctx, data.clone(), None);
                let handle: FontHandle = match handle {
                    Ok(handle) => handle,
                    Err(()) => fail!("TODO - Handle failure to create a font from template."),
                };
                let actual_desc = FontTemplateDescriptor::new(handle.boldness(),
                                    handle.is_italic());
                let desc_match = actual_desc == *requested_desc;

                self.descriptor = Some(actual_desc);
                if desc_match {
                    Some(data)
                } else {
                    None
                }
            }
        }
    }

    /// Get the font template data. If any strong references still
    /// exist, it will return a clone, otherwise it will load the
    /// font data and store a weak reference to it internally.
    pub fn get_data(&mut self) -> Arc<FontTemplateData> {
        let maybe_data = match self.data {
            Some(ref data) => data.upgrade(),
            None => None,
        };

        match maybe_data {
            Some(data) => data,
            None => {
                let template_data = Arc::new(FontTemplateData::new(self.identifier.as_slice()));
                self.data = Some(template_data.downgrade());
                template_data
            }
        }
    }
}
