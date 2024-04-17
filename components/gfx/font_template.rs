/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use style::computed_values::font_stretch::T as FontStretch;
use style::computed_values::font_style::T as FontStyle;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::FontWeight;

use crate::font_cache_thread::FontIdentifier;
use crate::platform::font_list::LocalFontIdentifier;

/// A reference to a [`FontTemplate`] with shared ownership and mutability.
pub(crate) type FontTemplateRef = Rc<RefCell<FontTemplate>>;

/// Describes how to select a font from a given family. This is very basic at the moment and needs
/// to be expanded or refactored when we support more of the font styling parameters.
///
/// NB: If you change this, you will need to update `style::properties::compute_font_hash()`.
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct FontTemplateDescriptor {
    pub weight: FontWeight,
    pub stretch: FontStretch,
    pub style: FontStyle,
}

/// FontTemplateDescriptor contains floats, which are not Eq because of NaN. However,
/// we know they will never be NaN, so we can manually implement Eq.
impl Eq for FontTemplateDescriptor {}

fn style_to_number(s: &FontStyle) -> f32 {
    match *s {
        FontStyle::NORMAL => 0.,
        FontStyle::ITALIC => FontStyle::DEFAULT_OBLIQUE_DEGREES as f32,
        _ => s.oblique_degrees(),
    }
}

impl FontTemplateDescriptor {
    #[inline]
    pub fn new(weight: FontWeight, stretch: FontStretch, style: FontStyle) -> Self {
        Self {
            weight,
            stretch,
            style,
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
    fn distance_from(&self, other: &FontTemplateDescriptor) -> f32 {
        // 0 <= style_part <= 180, since font-style obliqueness should be
        // between -90 and +90deg.
        let style_part = (style_to_number(&self.style) - style_to_number(&other.style)).abs();
        // 0 <= weightPart <= 800
        let weight_part = (self.weight.value() - other.weight.value()).abs();
        // 0 <= stretchPart <= 8
        let stretch_part = (self.stretch.to_percentage().0 - other.stretch.to_percentage().0).abs();
        style_part + weight_part + stretch_part
    }
}

impl<'a> From<&'a FontStyleStruct> for FontTemplateDescriptor {
    fn from(style: &'a FontStyleStruct) -> Self {
        FontTemplateDescriptor {
            weight: style.font_weight,
            stretch: style.font_stretch,
            style: style.font_style,
        }
    }
}

/// This describes all the information needed to create
/// font instance handles. It contains a unique
/// FontTemplateData structure that is platform specific.
pub struct FontTemplate {
    pub identifier: FontIdentifier,
    pub descriptor: FontTemplateDescriptor,
    /// The data to use for this [`FontTemplate`]. For web fonts, this is always filled, but
    /// for local fonts, this is loaded only lazily in layout.
    ///
    /// TODO: There is no mechanism for web fonts to unset their data!
    pub data: Option<Arc<Vec<u8>>>,
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
    pub fn new_local(
        identifier: LocalFontIdentifier,
        descriptor: FontTemplateDescriptor,
    ) -> FontTemplate {
        FontTemplate {
            identifier: FontIdentifier::Local(identifier),
            descriptor,
            data: None,
        }
    }

    pub fn new_web_font(
        url: ServoUrl,
        descriptor: FontTemplateDescriptor,
        data: Arc<Vec<u8>>,
    ) -> FontTemplate {
        FontTemplate {
            identifier: FontIdentifier::Web(url),
            descriptor,
            data: Some(data),
        }
    }

    pub fn identifier(&self) -> &FontIdentifier {
        &self.identifier
    }

    /// Returns a reference to the bytes in this font if they are in memory.
    /// This function never performs disk I/O.
    pub fn data_if_in_memory(&self) -> Option<Arc<Vec<u8>>> {
        self.data.clone()
    }
}

pub trait FontTemplateRefMethods {
    /// Returns a reference to the data in this font. This may be a hugely expensive
    /// operation (depending on the platform) which performs synchronous disk I/O
    /// and should never be done lightly.
    fn data(&self) -> Arc<Vec<u8>>;
    /// Get the descriptor. Returns `None` when instantiating the data fails.
    fn descriptor(&self) -> FontTemplateDescriptor;
    /// Returns true if the given descriptor matches the one in this [`FontTemplate`].
    fn descriptor_matches(&self, requested_desc: &FontTemplateDescriptor) -> bool;
    /// Calculate the distance from this [`FontTemplate`]s descriptor and return it
    /// or None if this is not a valid [`FontTemplate`].
    fn descriptor_distance(&self, requested_descriptor: &FontTemplateDescriptor) -> f32;
}

impl FontTemplateRefMethods for FontTemplateRef {
    fn descriptor(&self) -> FontTemplateDescriptor {
        self.borrow().descriptor
    }

    fn descriptor_matches(&self, requested_descriptor: &FontTemplateDescriptor) -> bool {
        self.descriptor() == *requested_descriptor
    }

    fn descriptor_distance(&self, requested_descriptor: &FontTemplateDescriptor) -> f32 {
        self.descriptor().distance_from(requested_descriptor)
    }

    fn data(&self) -> Arc<Vec<u8>> {
        let mut template = self.borrow_mut();
        let identifier = template.identifier.clone();
        template
            .data
            .get_or_insert_with(|| match identifier {
                FontIdentifier::Local(local_identifier) => {
                    Arc::new(local_identifier.read_data_from_file())
                },
                FontIdentifier::Web(_) => unreachable!("Web fonts should always have data."),
            })
            .clone()
    }
}
