/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::FontHandleMethods;
use platform::font::FontHandle;
use platform::font_context::FontContextHandle;
use platform::font_template::FontTemplateData;
use servo_atoms::Atom;
use std::fmt::{Debug, Error, Formatter};
use std::io::Error as IoError;
use std::sync::{Arc, Weak};
use std::u32;
use style::computed_values::font_stretch::T as FontStretch;
use style::computed_values::font_style::T as FontStyle;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::FontWeight;

/// Describes how to select a font from a given family. This is very basic at the moment and needs
/// to be expanded or refactored when we support more of the font styling parameters.
///
/// NB: If you change this, you will need to update `style::properties::compute_font_hash()`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Serialize)]
pub struct FontTemplateDescriptor {
    pub weight: FontWeight,
    pub stretch: FontStretch,
    pub italic: bool,
}

impl FontTemplateDescriptor {
    #[inline]
    pub fn new(weight: FontWeight, stretch: FontStretch, italic: bool)
               -> FontTemplateDescriptor {
        FontTemplateDescriptor {
            weight: weight,
            stretch: stretch,
            italic: italic,
        }
    }

    /// Returns a score indicating how far apart visually the two font descriptors are. This is
    /// used for fuzzy font selection.
    ///
    /// The smaller the score, the better the fonts match. 0 indicates an exact match. This must
    /// be commutative (distance(A, B) == distance(B, A)).
    ///
    /// The policy is to care most about differences in italicness, then weight, then stretch
    #[inline]
    fn distance_from(&self, other: &FontTemplateDescriptor) -> u32 {
        let italic_part = if self.italic == other.italic { 0 } else { 1000 };
        // 0 <= weightPart <= 800
        let weight_part = ((self.weight.0 as i16) - (other.weight.0 as i16)).abs() as u32;
        // 0 <= stretchPart <= 8
        let stretch_part = (self.stretch_number() - other.stretch_number()).abs() as u32;
        italic_part + weight_part + stretch_part
    }

    /// Returns a number between 1 and 9 for the stretch property.
    /// 1 is ultra_condensed, 5 is normal, and 9 is ultra_expanded
    #[inline]
    fn stretch_number(&self) -> i32 {
        match self.stretch {
            FontStretch::UltraCondensed => 1,
            FontStretch::ExtraCondensed => 2,
            FontStretch::Condensed      => 3,
            FontStretch::SemiCondensed  => 4,
            FontStretch::Normal         => 5,
            FontStretch::SemiExpanded   => 6,
            FontStretch::Expanded       => 7,
            FontStretch::ExtraExpanded  => 8,
            FontStretch::UltraExpanded  => 9,
        }
    }
}

impl<'a> From<&'a FontStyleStruct> for FontTemplateDescriptor {
    fn from(style: &'a FontStyleStruct) -> Self {
        FontTemplateDescriptor {
            weight: style.font_weight,
            stretch: style.font_stretch,
            italic: style.font_style == FontStyle::Italic || style.font_style == FontStyle::Oblique,
        }
    }
}

impl PartialEq for FontTemplateDescriptor {
    fn eq(&self, other: &FontTemplateDescriptor) -> bool {
        self.weight == other.weight && self.stretch == other.stretch && self.italic == other.italic
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

impl Debug for FontTemplate {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        self.identifier.fmt(f)
    }
}

/// Holds all of the template information for a font that
/// is common, regardless of the number of instances of
/// this font handle per thread.
impl FontTemplate {
    pub fn new(identifier: Atom, maybe_bytes: Option<Vec<u8>>) -> Result<FontTemplate, IoError> {
        let maybe_data = match maybe_bytes {
            Some(_) => Some(FontTemplateData::new(identifier.clone(), maybe_bytes)?),
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

        Ok(FontTemplate {
            identifier: identifier,
            descriptor: None,
            weak_ref: maybe_weak_ref,
            strong_ref: maybe_strong_ref,
            is_valid: true,
        })
    }

    pub fn identifier(&self) -> &Atom {
        &self.identifier
    }

    /// Get the descriptor. Returns `None` when instantiating the data fails.
    pub fn descriptor(&mut self, font_context: &FontContextHandle) -> Option<FontTemplateDescriptor> {
        // The font template data can be unloaded when nothing is referencing
        // it (via the Weak reference to the Arc above). However, if we have
        // already loaded a font, store the style information about it separately,
        // so that we can do font matching against it again in the future
        // without having to reload the font (unless it is an actual match).

        self.descriptor.or_else(|| {
            if self.instantiate(font_context).is_err() {
                return None
            };

            Some(self.descriptor.expect("Instantiation succeeded but no descriptor?"))
        })
    }

    /// Get the data for creating a font if it matches a given descriptor.
    pub fn data_for_descriptor(&mut self,
                               fctx: &FontContextHandle,
                               requested_desc: &FontTemplateDescriptor)
                               -> Option<Arc<FontTemplateData>> {
        self.descriptor(&fctx).and_then(|descriptor| {
            if *requested_desc == descriptor {
                self.data().ok()
            } else {
                None
            }
        })
    }

    /// Returns the font data along with the distance between this font's descriptor and the given
    /// descriptor, if the font can be loaded.
    pub fn data_for_approximate_descriptor(&mut self,
                                           font_context: &FontContextHandle,
                                           requested_descriptor: &FontTemplateDescriptor)
                                           -> Option<(Arc<FontTemplateData>, u32)> {
        self.descriptor(&font_context).and_then(|descriptor| {
            self.data().ok().map(|data| {
                (data, descriptor.distance_from(requested_descriptor))
            })
        })
    }

    fn instantiate(&mut self, font_context: &FontContextHandle) -> Result<(), ()> {
        if !self.is_valid {
            return Err(())
        }

        let data = self.data().map_err(|_| ())?;
        let handle: Result<FontHandle, ()> = FontHandleMethods::new_from_template(font_context,
                                                                                  data,
                                                                                  None);
        self.is_valid = handle.is_ok();
        let handle = handle?;
        self.descriptor = Some(FontTemplateDescriptor::new(handle.boldness(),
                                                           handle.stretchiness(),
                                                           handle.is_italic()));
        Ok(())
    }

    /// Get the data for creating a font.
    pub fn get(&mut self) -> Option<Arc<FontTemplateData>> {
        if self.is_valid {
            self.data().ok()
        } else {
            None
        }
    }

    /// Get the font template data. If any strong references still
    /// exist, it will return a clone, otherwise it will load the
    /// font data and store a weak reference to it internally.
    pub fn data(&mut self) -> Result<Arc<FontTemplateData>, IoError> {
        let maybe_data = match self.weak_ref {
            Some(ref data) => data.upgrade(),
            None => None,
        };

        if let Some(data) = maybe_data {
            return Ok(data)
        }

        assert!(self.strong_ref.is_none());
        let template_data = Arc::new(FontTemplateData::new(self.identifier.clone(), None)?);
        self.weak_ref = Some(Arc::downgrade(&template_data));
        Ok(template_data)
    }
}
