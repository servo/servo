/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::{Debug, Error, Formatter};
use std::ops::RangeInclusive;
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use style::computed_values::font_stretch::T as FontStretch;
use style::computed_values::font_style::T as FontStyle;
use style::stylesheets::DocumentStyleSheet;
use style::values::computed::font::FontWeight;

use crate::font::{FontDescriptor, PlatformFontMethods};
use crate::font_cache_thread::{
    CSSFontFaceDescriptors, ComputedFontStyleDescriptor, FontIdentifier,
};
use crate::platform::font::PlatformFont;
use crate::platform::font_list::LocalFontIdentifier;

/// A reference to a [`FontTemplate`] with shared ownership and mutability.
pub type FontTemplateRef = Arc<AtomicRefCell<FontTemplate>>;

/// Describes how to select a font from a given family. This is very basic at the moment and needs
/// to be expanded or refactored when we support more of the font styling parameters.
///
/// NB: If you change this, you will need to update `style::properties::compute_font_hash()`.
#[derive(Clone, Debug, Deserialize, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct FontTemplateDescriptor {
    pub weight: (FontWeight, FontWeight),
    pub stretch: (FontStretch, FontStretch),
    pub style: (FontStyle, FontStyle),
    #[ignore_malloc_size_of = "MallocSizeOf does not yet support RangeInclusive"]
    pub unicode_range: Option<Vec<RangeInclusive<u32>>>,
}

impl Default for FontTemplateDescriptor {
    fn default() -> Self {
        Self::new(FontWeight::normal(), FontStretch::NORMAL, FontStyle::NORMAL)
    }
}

/// FontTemplateDescriptor contains floats, which are not Eq because of NaN. However,
/// we know they will never be NaN, so we can manually implement Eq.
impl Eq for FontTemplateDescriptor {}

impl FontTemplateDescriptor {
    #[inline]
    pub fn new(weight: FontWeight, stretch: FontStretch, style: FontStyle) -> Self {
        Self {
            weight: (weight, weight),
            stretch: (stretch, stretch),
            style: (style, style),
            unicode_range: None,
        }
    }

    pub fn is_variation_font(&self) -> bool {
        self.weight.0 != self.weight.1 ||
            self.stretch.0 != self.stretch.1 ||
            self.style.0 != self.style.1
    }

    /// Returns a score indicating how far apart visually the two font descriptors are. This is
    /// used for implmenting the CSS Font Matching algorithm:
    /// <https://drafts.csswg.org/css-fonts/#font-matching-algorithm>.
    ///
    /// The smaller the score, the better the fonts match. 0 indicates an exact match. This must
    /// be commutative (distance(A, B) == distance(B, A)).
    #[inline]
    fn distance_from(&self, target: &FontDescriptor) -> f32 {
        let stretch_distance = target.stretch.match_distance(&self.stretch);
        let style_distance = target.style.match_distance(&self.style);
        let weight_distance = target.weight.match_distance(&self.weight);

        // Sanity-check that the distances are within the expected range
        // (update if implementation of the distance functions is changed).
        assert!((0.0..=2000.0).contains(&stretch_distance));
        assert!((0.0..=500.0).contains(&style_distance));
        assert!((0.0..=1600.0).contains(&weight_distance));

        // Factors used to weight the distances between the available and target font
        // properties during font-matching. These ensure that we respect the CSS-fonts
        // requirement that font-stretch >> font-style >> font-weight; and in addition,
        // a mismatch between the desired and actual glyph presentation (emoji vs text)
        // will take precedence over any of the style attributes.
        //
        // Also relevant for font selection is the emoji presentation preference, but this
        // is handled later when filtering fonts based on the glyphs they contain.
        const STRETCH_FACTOR: f32 = 1.0e8;
        const STYLE_FACTOR: f32 = 1.0e4;
        const WEIGHT_FACTOR: f32 = 1.0e0;

        stretch_distance * STRETCH_FACTOR +
            style_distance * STYLE_FACTOR +
            weight_distance * WEIGHT_FACTOR
    }

    fn matches(&self, descriptor_to_match: &FontDescriptor) -> bool {
        self.weight.0 <= descriptor_to_match.weight &&
            self.weight.1 >= descriptor_to_match.weight &&
            self.style.0 <= descriptor_to_match.style &&
            self.style.1 >= descriptor_to_match.style &&
            self.stretch.0 <= descriptor_to_match.stretch &&
            self.stretch.1 >= descriptor_to_match.stretch
    }

    fn override_values_with_css_font_template_descriptors(
        &mut self,
        css_font_template_descriptors: &CSSFontFaceDescriptors,
    ) {
        if let Some(weight) = css_font_template_descriptors.weight {
            self.weight = weight;
        }
        self.style = match css_font_template_descriptors.style {
            Some(ComputedFontStyleDescriptor::Italic) => (FontStyle::ITALIC, FontStyle::ITALIC),
            Some(ComputedFontStyleDescriptor::Normal) => (FontStyle::NORMAL, FontStyle::NORMAL),
            Some(ComputedFontStyleDescriptor::Oblique(angle_1, angle_2)) => (
                FontStyle::oblique(angle_1.to_float()),
                FontStyle::oblique(angle_2.to_float()),
            ),
            None => self.style,
        };
        if let Some(stretch) = css_font_template_descriptors.stretch {
            self.stretch = stretch;
        }
        if let Some(ref unicode_range) = css_font_template_descriptors.unicode_range {
            self.unicode_range = Some(unicode_range.clone());
        }
    }
}

/// This describes all the information needed to create
/// font instance handles. It contains a unique
/// FontTemplateData structure that is platform specific.
#[derive(Clone)]
pub struct FontTemplate {
    pub identifier: FontIdentifier,
    pub descriptor: FontTemplateDescriptor,
    /// The data to use for this [`FontTemplate`]. For web fonts, this is always filled, but
    /// for local fonts, this is loaded only lazily in layout.
    pub data: Option<Arc<Vec<u8>>>,
    /// If this font is a web font, this is a reference to the stylesheet that
    /// created it. This will be used to remove this font from caches, when the
    /// stylesheet is removed.
    pub stylesheet: Option<DocumentStyleSheet>,
}

impl malloc_size_of::MallocSizeOf for FontTemplate {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        self.identifier.size_of(ops) +
            self.descriptor.size_of(ops) +
            self.data.as_ref().map_or(0, |data| (*data).size_of(ops))
    }
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
    /// Create a new [`FontTemplate`] for a system font installed locally.
    pub fn new_for_local_font(
        identifier: LocalFontIdentifier,
        descriptor: FontTemplateDescriptor,
    ) -> FontTemplate {
        FontTemplate {
            identifier: FontIdentifier::Local(identifier),
            descriptor,
            data: None,
            stylesheet: None,
        }
    }

    /// Create a new [`FontTemplate`] for a `@font-family` with a `url(...)` `src` font.
    pub fn new_for_remote_web_font(
        url: ServoUrl,
        data: Arc<Vec<u8>>,
        css_font_template_descriptors: &CSSFontFaceDescriptors,
        stylesheet: Option<DocumentStyleSheet>,
    ) -> Result<FontTemplate, &'static str> {
        let identifier = FontIdentifier::Web(url.clone());
        let Ok(handle) = PlatformFont::new_from_data(identifier, data.clone(), 0, None) else {
            return Err("Could not initialize platform font data for: {url:?}");
        };

        let mut descriptor = handle.descriptor();
        descriptor
            .override_values_with_css_font_template_descriptors(css_font_template_descriptors);
        Ok(FontTemplate {
            identifier: FontIdentifier::Web(url),
            descriptor,
            data: Some(data),
            stylesheet,
        })
    }

    /// Create a new [`FontTemplate`] for a `@font-family` with a `local(...)` `src`. This takes in
    /// the template of the local font and creates a new one that reflects the properties specified
    /// by `@font-family` in the stylesheet.
    pub fn new_for_local_web_font(
        local_template: FontTemplateRef,
        css_font_template_descriptors: &CSSFontFaceDescriptors,
        stylesheet: DocumentStyleSheet,
    ) -> Result<FontTemplate, &'static str> {
        let mut alias_template = local_template.borrow().clone();
        alias_template
            .descriptor
            .override_values_with_css_font_template_descriptors(css_font_template_descriptors);
        alias_template.stylesheet = Some(stylesheet);
        Ok(alias_template)
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
    /// Get the descriptor.
    fn descriptor(&self) -> FontTemplateDescriptor;
    /// Get the [`FontIdentifier`] for this template.
    fn identifier(&self) -> FontIdentifier;
    /// Returns true if the given descriptor matches the one in this [`FontTemplate`].
    fn matches_font_descriptor(&self, descriptor_to_match: &FontDescriptor) -> bool;
    /// Calculate the distance from this [`FontTemplate`]s descriptor and return it
    /// or None if this is not a valid [`FontTemplate`].
    fn descriptor_distance(&self, descriptor_to_match: &FontDescriptor) -> f32;
    /// Whether or not this character is in the unicode ranges specified in
    /// this temlates `@font-face` definition, if any.
    fn char_in_unicode_range(&self, character: char) -> bool;
}

impl FontTemplateRefMethods for FontTemplateRef {
    fn descriptor(&self) -> FontTemplateDescriptor {
        self.borrow().descriptor.clone()
    }

    fn identifier(&self) -> FontIdentifier {
        self.borrow().identifier.clone()
    }

    fn matches_font_descriptor(&self, descriptor_to_match: &FontDescriptor) -> bool {
        self.descriptor().matches(descriptor_to_match)
    }

    fn descriptor_distance(&self, descriptor_to_match: &FontDescriptor) -> f32 {
        self.descriptor().distance_from(descriptor_to_match)
    }

    fn data(&self) -> Arc<Vec<u8>> {
        if let Some(data) = self.borrow().data.clone() {
            return data;
        }

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

    fn char_in_unicode_range(&self, character: char) -> bool {
        let character = character as u32;
        self.borrow()
            .descriptor
            .unicode_range
            .as_ref()
            .map_or(true, |ranges| {
                ranges.iter().any(|range| range.contains(&character))
            })
    }
}

/// A trait for implementing the CSS font matching algorithm against various font features.
/// See <https://drafts.csswg.org/css-fonts/#font-matching-algorithm>.
///
/// This implementation is ported from Gecko at:
/// <https://searchfox.org/mozilla-central/rev/0529464f0d2981347ef581f7521ace8b7af7f7ac/gfx/thebes/gfxFontUtils.h#1217>.
trait FontMatchDistanceMethod: Sized {
    fn match_distance(&self, range: &(Self, Self)) -> f32;
    fn to_float(&self) -> f32;
}

impl FontMatchDistanceMethod for FontStretch {
    fn match_distance(&self, range: &(Self, Self)) -> f32 {
        // stretch distance ==> [0,2000]
        const REVERSE_DISTANCE: f32 = 1000.0;

        let min_stretch = range.0;
        let max_stretch = range.1;

        // The stretch value is a (non-negative) percentage; currently we support
        // values in the range 0 .. 1000. (If the upper limit is ever increased,
        // the kReverseDistance value used here may need to be adjusted.)
        // If aTargetStretch is >100, we prefer larger values if available;
        // if <=100, we prefer smaller values if available.
        if *self < min_stretch {
            if *self > FontStretch::NORMAL {
                return min_stretch.to_float() - self.to_float();
            }
            return (min_stretch.to_float() - self.to_float()) + REVERSE_DISTANCE;
        }

        if *self > max_stretch {
            if *self <= FontStretch::NORMAL {
                return self.to_float() - max_stretch.to_float();
            }
            return (self.to_float() - max_stretch.to_float()) + REVERSE_DISTANCE;
        }
        0.0
    }

    fn to_float(&self) -> f32 {
        self.0.to_float()
    }
}

impl FontMatchDistanceMethod for FontWeight {
    // Calculate weight distance with values in the range (0..1000). In general,
    // heavier weights match towards even heavier weights while lighter weights
    // match towards even lighter weights. Target weight values in the range
    // [400..500] are special, since they will first match up to 500, then down
    // towards 0, then up again towards 999.
    //
    // Example: with target 600 and font weight 800, distance will be 200. With
    // target 300 and font weight 600, distance will be 900, since heavier
    // weights are farther away than lighter weights. If the target is 5 and the
    // font weight 995, the distance would be 1590 for the same reason.

    fn match_distance(&self, range: &(Self, Self)) -> f32 {
        // weight distance ==> [0,1600]
        const NOT_WITHIN_CENTRAL_RANGE: f32 = 100.0;
        const REVERSE_DISTANCE: f32 = 600.0;

        let min_weight = range.0;
        let max_weight = range.1;

        if *self >= min_weight && *self <= max_weight {
            // Target is within the face's range, so it's a perfect match
            return 0.0;
        }

        if *self < FontWeight::NORMAL {
            // Requested a lighter-than-400 weight
            if max_weight < *self {
                return self.to_float() - max_weight.to_float();
            }

            // Add reverse-search penalty for bolder faces
            return (min_weight.to_float() - self.to_float()) + REVERSE_DISTANCE;
        }

        if *self > FontWeight::from_float(500.) {
            // Requested a bolder-than-500 weight
            if min_weight > *self {
                return min_weight.to_float() - self.to_float();
            }
            // Add reverse-search penalty for lighter faces
            return (self.to_float() - max_weight.to_float()) + REVERSE_DISTANCE;
        }

        // Special case for requested weight in the [400..500] range
        if min_weight > *self {
            if min_weight <= FontWeight::from_float(500.) {
                // Bolder weight up to 500 is first choice
                return min_weight.to_float() - self.to_float();
            }
            // Other bolder weights get a reverse-search penalty
            return (min_weight.to_float() - self.to_float()) + REVERSE_DISTANCE;
        }
        // Lighter weights are not as good as bolder ones within [400..500]
        (self.to_float() - max_weight.to_float()) + NOT_WITHIN_CENTRAL_RANGE
    }

    fn to_float(&self) -> f32 {
        self.value()
    }
}

impl FontMatchDistanceMethod for FontStyle {
    fn match_distance(&self, range: &(Self, Self)) -> f32 {
        // style distance ==> [0,500]
        let min_style = range.0;
        if *self == min_style {
            return 0.0; // styles match exactly ==> 0
        }

        // bias added to angle difference when searching in the non-preferred
        // direction from a target angle
        const REVERSE: f32 = 100.0;

        // bias added when we've crossed from positive to negative angles or
        // vice versa
        const NEGATE: f32 = 200.0;

        if *self == FontStyle::NORMAL {
            if min_style.is_oblique() {
                // to distinguish oblique 0deg from normal, we add 1.0 to the angle
                let min_angle = min_style.oblique_degrees();
                if min_angle >= 0.0 {
                    return 1.0 + min_angle;
                }
                let max_style = range.1;
                let max_angle = max_style.oblique_degrees();
                if max_angle >= 0.0 {
                    // [min,max] range includes 0.0, so just return our minimum
                    return 1.0;
                }
                // negative oblique is even worse than italic
                return NEGATE - max_angle;
            }
            // must be italic, which is worse than any non-negative oblique;
            // treat as a match in the wrong search direction
            assert!(min_style == FontStyle::ITALIC);
            return REVERSE;
        }

        let default_oblique_angle = FontStyle::OBLIQUE.oblique_degrees();
        if *self == FontStyle::ITALIC {
            if min_style.is_oblique() {
                let min_angle = min_style.oblique_degrees();
                if min_angle >= default_oblique_angle {
                    return 1.0 + (min_angle - default_oblique_angle);
                }
                let max_style = range.1;
                let max_angle = max_style.oblique_degrees();
                if max_angle >= default_oblique_angle {
                    return 1.0;
                }
                if max_angle > 0.0 {
                    // wrong direction but still > 0, add bias of 100
                    return REVERSE + (default_oblique_angle - max_angle);
                }
                // negative oblique angle, add bias of 300
                return REVERSE + NEGATE + (default_oblique_angle - max_angle);
            }
            // normal is worse than oblique > 0, but better than oblique <= 0
            assert!(min_style == FontStyle::NORMAL);
            return NEGATE;
        }

        // target is oblique <angle>: four different cases depending on
        // the value of the <angle>, which determines the preferred direction
        // of search
        let target_angle = self.oblique_degrees();
        if target_angle >= default_oblique_angle {
            if min_style.is_oblique() {
                let min_angle = min_style.oblique_degrees();
                if min_angle >= target_angle {
                    return min_angle - target_angle;
                }
                let max_style = range.1;
                let max_angle = max_style.oblique_degrees();
                if max_angle >= target_angle {
                    return 0.0;
                }
                if max_angle > 0.0 {
                    return REVERSE + (target_angle - max_angle);
                }
                return REVERSE + NEGATE + (target_angle - max_angle);
            }
            if min_style == FontStyle::ITALIC {
                return REVERSE + NEGATE;
            }
            return REVERSE + NEGATE + 1.0;
        }

        if target_angle <= -default_oblique_angle {
            if min_style.is_oblique() {
                let max_style = range.1;
                let max_angle = max_style.oblique_degrees();
                if max_angle <= target_angle {
                    return target_angle - max_angle;
                }
                let min_angle = min_style.oblique_degrees();
                if min_angle <= target_angle {
                    return 0.0;
                }
                if min_angle < 0.0 {
                    return REVERSE + (min_angle - target_angle);
                }
                return REVERSE + NEGATE + (min_angle - target_angle);
            }
            if min_style == FontStyle::ITALIC {
                return REVERSE + NEGATE;
            }
            return REVERSE + NEGATE + 1.0;
        }

        if target_angle >= 0.0 {
            if min_style.is_oblique() {
                let min_angle = min_style.oblique_degrees();
                if min_angle > target_angle {
                    return REVERSE + (min_angle - target_angle);
                }
                let max_style = range.1;
                let max_angle = max_style.oblique_degrees();
                if max_angle >= target_angle {
                    return 0.0;
                }
                if max_angle > 0.0 {
                    return target_angle - max_angle;
                }
                return REVERSE + NEGATE + (target_angle - max_angle);
            }
            if min_style == FontStyle::ITALIC {
                return REVERSE + NEGATE - 2.0;
            }
            return REVERSE + NEGATE - 1.0;
        }

        // last case: (targetAngle < 0.0 && targetAngle > kDefaultAngle)
        if min_style.is_oblique() {
            let max_style = range.1;
            let max_angle = max_style.oblique_degrees();
            if max_angle < target_angle {
                return REVERSE + (target_angle - max_angle);
            }
            let min_angle = min_style.oblique_degrees();
            if min_angle <= target_angle {
                return 0.0;
            }
            if min_angle < 0.0 {
                return min_angle - target_angle;
            }
            return REVERSE + NEGATE + (min_angle - target_angle);
        }
        if min_style == FontStyle::ITALIC {
            return REVERSE + NEGATE - 2.0;
        }
        REVERSE + NEGATE - 1.0
    }

    fn to_float(&self) -> f32 {
        unimplemented!("Don't know how to convert FontStyle to float.");
    }
}

pub(crate) trait IsOblique {
    fn is_oblique(&self) -> bool;
}

impl IsOblique for FontStyle {
    fn is_oblique(&self) -> bool {
        *self != FontStyle::NORMAL && *self != FontStyle::ITALIC
    }
}
