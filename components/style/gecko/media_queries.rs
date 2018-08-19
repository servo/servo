/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko's media-query device and expression representation.

use app_units::AU_PER_PX;
use app_units::Au;
use cssparser::RGBA;
use euclid::Size2D;
use euclid::TypedScale;
use gecko::values::{convert_nscolor_to_rgba, convert_rgba_to_nscolor};
use gecko_bindings::bindings;
use gecko_bindings::structs;
use gecko_bindings::structs::{nsPresContext, RawGeckoPresContextOwned};
use media_queries::MediaType;
use properties::ComputedValues;
use servo_arc::Arc;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use string_cache::Atom;
use style_traits::{CSSPixel, DevicePixel};
use style_traits::viewport::ViewportConstraints;
use values::{CustomIdent, KeyframesName};
use values::computed::font::FontSize;

/// The `Device` in Gecko wraps a pres context, has a default values computed,
/// and contains all the viewport rule state.
pub struct Device {
    /// NB: The pres context lifetime is tied to the styleset, who owns the
    /// stylist, and thus the `Device`, so having a raw pres context pointer
    /// here is fine.
    pres_context: RawGeckoPresContextOwned,
    default_values: Arc<ComputedValues>,
    /// The font size of the root element
    /// This is set when computing the style of the root
    /// element, and used for rem units in other elements.
    ///
    /// When computing the style of the root element, there can't be any
    /// other style being computed at the same time, given we need the style of
    /// the parent to compute everything else. So it is correct to just use
    /// a relaxed atomic here.
    root_font_size: AtomicIsize,
    /// The body text color, stored as an `nscolor`, used for the "tables
    /// inherit from body" quirk.
    ///
    /// <https://quirks.spec.whatwg.org/#the-tables-inherit-color-from-body-quirk>
    body_text_color: AtomicUsize,
    /// Whether any styles computed in the document relied on the root font-size
    /// by using rem units.
    used_root_font_size: AtomicBool,
    /// Whether any styles computed in the document relied on the viewport size
    /// by using vw/vh/vmin/vmax units.
    used_viewport_size: AtomicBool,
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use nsstring::nsCString;

        let mut doc_uri = nsCString::new();
        unsafe {
            bindings::Gecko_nsIURI_Debug(
                (*self.document()).mDocumentURI.raw::<structs::nsIURI>(),
                &mut doc_uri,
            )
        };

        f.debug_struct("Device")
            .field("document_url", &doc_uri)
            .finish()
    }
}

unsafe impl Sync for Device {}
unsafe impl Send for Device {}

impl Device {
    /// Trivially constructs a new `Device`.
    pub fn new(pres_context: RawGeckoPresContextOwned) -> Self {
        assert!(!pres_context.is_null());
        Device {
            pres_context,
            default_values: ComputedValues::default_values(unsafe { &*pres_context }),
            // FIXME(bz): Seems dubious?
            root_font_size: AtomicIsize::new(FontSize::medium().size().0 as isize),
            body_text_color: AtomicUsize::new(unsafe { &*pres_context }.mDefaultColor as usize),
            used_root_font_size: AtomicBool::new(false),
            used_viewport_size: AtomicBool::new(false),
        }
    }

    /// Tells the device that a new viewport rule has been found, and stores the
    /// relevant viewport constraints.
    pub fn account_for_viewport_rule(&mut self, _constraints: &ViewportConstraints) {
        unreachable!("Gecko doesn't support @viewport");
    }

    /// Whether any animation name may be referenced from the style of any
    /// element.
    pub fn animation_name_may_be_referenced(&self, name: &KeyframesName) -> bool {
        unsafe {
            bindings::Gecko_AnimationNameMayBeReferencedFromStyle(
                self.pres_context(),
                name.as_atom().as_ptr(),
            )
        }
    }

    /// Returns the default computed values as a reference, in order to match
    /// Servo.
    pub fn default_computed_values(&self) -> &ComputedValues {
        &self.default_values
    }

    /// Returns the default computed values as an `Arc`.
    pub fn default_computed_values_arc(&self) -> &Arc<ComputedValues> {
        &self.default_values
    }

    /// Get the font size of the root element (for rem)
    pub fn root_font_size(&self) -> Au {
        self.used_root_font_size.store(true, Ordering::Relaxed);
        Au::new(self.root_font_size.load(Ordering::Relaxed) as i32)
    }

    /// Set the font size of the root element (for rem)
    pub fn set_root_font_size(&self, size: Au) {
        self.root_font_size.store(size.0 as isize, Ordering::Relaxed)
    }

    /// Sets the body text color for the "inherit color from body" quirk.
    ///
    /// <https://quirks.spec.whatwg.org/#the-tables-inherit-color-from-body-quirk>
    pub fn set_body_text_color(&self, color: RGBA) {
        self.body_text_color
            .store(convert_rgba_to_nscolor(&color) as usize, Ordering::Relaxed)
    }

    /// Returns the body text color.
    pub fn body_text_color(&self) -> RGBA {
        convert_nscolor_to_rgba(self.body_text_color.load(Ordering::Relaxed) as u32)
    }

    /// Gets the pres context associated with this document.
    #[inline]
    pub fn pres_context(&self) -> &nsPresContext {
        unsafe { &*self.pres_context }
    }

    /// Gets the document pointer.
    #[inline]
    pub fn document(&self) -> *mut structs::nsIDocument {
        self.pres_context().mDocument.raw::<structs::nsIDocument>()
    }

    /// Recreates the default computed values.
    pub fn reset_computed_values(&mut self) {
        self.default_values = ComputedValues::default_values(self.pres_context());
    }

    /// Rebuild all the cached data.
    pub fn rebuild_cached_data(&mut self) {
        self.reset_computed_values();
        self.used_root_font_size.store(false, Ordering::Relaxed);
        self.used_viewport_size.store(false, Ordering::Relaxed);
    }

    /// Returns whether we ever looked up the root font size of the Device.
    pub fn used_root_font_size(&self) -> bool {
        self.used_root_font_size.load(Ordering::Relaxed)
    }

    /// Recreates all the temporary state that the `Device` stores.
    ///
    /// This includes the viewport override from `@viewport` rules, and also the
    /// default computed values.
    pub fn reset(&mut self) {
        self.reset_computed_values();
    }

    /// Returns the current media type of the device.
    pub fn media_type(&self) -> MediaType {
        // Gecko allows emulating random media with mIsEmulatingMedia and
        // mMediaEmulated.
        let context = self.pres_context();
        let medium_to_use = if context.mIsEmulatingMedia() != 0 {
            context.mMediaEmulated.mRawPtr
        } else {
            context.mMedium
        };

        MediaType(CustomIdent(unsafe { Atom::from_raw(medium_to_use) }))
    }

    /// Returns the current viewport size in app units.
    pub fn au_viewport_size(&self) -> Size2D<Au> {
        let area = &self.pres_context().mVisibleArea;
        Size2D::new(Au(area.width), Au(area.height))
    }

    /// Returns the current viewport size in app units, recording that it's been
    /// used for viewport unit resolution.
    pub fn au_viewport_size_for_viewport_unit_resolution(&self) -> Size2D<Au> {
        self.used_viewport_size.store(true, Ordering::Relaxed);
        self.au_viewport_size()
    }

    /// Returns whether we ever looked up the viewport size of the Device.
    pub fn used_viewport_size(&self) -> bool {
        self.used_viewport_size.load(Ordering::Relaxed)
    }

    /// Returns the device pixel ratio.
    pub fn device_pixel_ratio(&self) -> TypedScale<f32, CSSPixel, DevicePixel> {
        let override_dppx = self.pres_context().mOverrideDPPX;
        if override_dppx > 0.0 {
            return TypedScale::new(override_dppx);
        }
        let au_per_dpx = self.pres_context().mCurAppUnitsPerDevPixel as f32;
        let au_per_px = AU_PER_PX as f32;
        TypedScale::new(au_per_px / au_per_dpx)
    }

    /// Returns whether document colors are enabled.
    pub fn use_document_colors(&self) -> bool {
        self.pres_context().mUseDocumentColors() != 0
    }

    /// Returns the default background color.
    pub fn default_background_color(&self) -> RGBA {
        convert_nscolor_to_rgba(self.pres_context().mBackgroundColor)
    }

    /// Applies text zoom to a font-size or line-height value (see nsStyleFont::ZoomText).
    pub fn zoom_text(&self, size: Au) -> Au {
        size.scale_by(self.pres_context().mEffectiveTextZoom)
    }
    /// Un-apply text zoom (see nsStyleFont::UnzoomText).
    pub fn unzoom_text(&self, size: Au) -> Au {
        size.scale_by(1. / self.pres_context().mEffectiveTextZoom)
    }
}
