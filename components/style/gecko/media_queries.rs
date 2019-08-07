/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gecko's media-query device and expression representation.

use crate::custom_properties::CssEnvironment;
use crate::gecko::values::{convert_nscolor_to_rgba, convert_rgba_to_nscolor};
use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs;
use crate::media_queries::MediaType;
use crate::properties::ComputedValues;
use crate::string_cache::Atom;
use crate::values::computed::font::FontSize;
use crate::values::{CustomIdent, KeyframesName};
use app_units::Au;
use app_units::AU_PER_PX;
use cssparser::RGBA;
use euclid::default::Size2D;
use euclid::Scale;
use servo_arc::Arc;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use style_traits::viewport::ViewportConstraints;
use style_traits::{CSSPixel, DevicePixel};

/// The `Device` in Gecko wraps a pres context, has a default values computed,
/// and contains all the viewport rule state.
pub struct Device {
    /// NB: The document owns the styleset, who owns the stylist, and thus the
    /// `Device`, so having a raw document pointer here is fine.
    document: *const structs::Document,
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
    /// The CssEnvironment object responsible of getting CSS environment
    /// variables.
    environment: CssEnvironment,
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
    pub fn new(document: *const structs::Document) -> Self {
        assert!(!document.is_null());
        let doc = unsafe { &*document };
        let prefs = unsafe { &*bindings::Gecko_GetPrefSheetPrefs(doc) };
        Device {
            document,
            default_values: ComputedValues::default_values(doc),
            // FIXME(bz): Seems dubious?
            root_font_size: AtomicIsize::new(FontSize::medium().size().0 as isize),
            body_text_color: AtomicUsize::new(prefs.mDefaultColor as usize),
            used_root_font_size: AtomicBool::new(false),
            used_viewport_size: AtomicBool::new(false),
            environment: CssEnvironment,
        }
    }

    /// Get the relevant environment to resolve `env()` functions.
    #[inline]
    pub fn environment(&self) -> &CssEnvironment {
        &self.environment
    }

    /// Tells the device that a new viewport rule has been found, and stores the
    /// relevant viewport constraints.
    pub fn account_for_viewport_rule(&mut self, _constraints: &ViewportConstraints) {
        unreachable!("Gecko doesn't support @viewport");
    }

    /// Whether any animation name may be referenced from the style of any
    /// element.
    pub fn animation_name_may_be_referenced(&self, name: &KeyframesName) -> bool {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return false,
        };

        unsafe {
            bindings::Gecko_AnimationNameMayBeReferencedFromStyle(pc, name.as_atom().as_ptr())
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
        self.root_font_size
            .store(size.0 as isize, Ordering::Relaxed)
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

    /// Gets the document pointer.
    #[inline]
    pub fn document(&self) -> &structs::Document {
        unsafe { &*self.document }
    }

    /// Gets the pres context associated with this document.
    #[inline]
    pub fn pres_context(&self) -> Option<&structs::nsPresContext> {
        unsafe {
            self.document()
                .mPresShell
                .as_ref()?
                .mPresContext
                .mRawPtr
                .as_ref()
        }
    }

    /// Gets the preference stylesheet prefs for our document.
    #[inline]
    pub fn pref_sheet_prefs(&self) -> &structs::PreferenceSheet_Prefs {
        unsafe { &*bindings::Gecko_GetPrefSheetPrefs(self.document()) }
    }

    /// Recreates the default computed values.
    pub fn reset_computed_values(&mut self) {
        self.default_values = ComputedValues::default_values(self.document());
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
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return MediaType::screen(),
        };

        // Gecko allows emulating random media with mMediaEmulationData.mMedium.
        let medium_to_use = if !pc.mMediaEmulationData.mMedium.mRawPtr.is_null() {
            pc.mMediaEmulationData.mMedium.mRawPtr
        } else {
            pc.mMedium as *const structs::nsAtom as *mut _
        };

        MediaType(CustomIdent(unsafe { Atom::from_raw(medium_to_use) }))
    }

    /// Returns the current viewport size in app units.
    pub fn au_viewport_size(&self) -> Size2D<Au> {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return Size2D::new(Au(0), Au(0)),
        };
        let area = &pc.mVisibleArea;
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
    pub fn device_pixel_ratio(&self) -> Scale<f32, CSSPixel, DevicePixel> {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return Scale::new(1.),
        };

        if pc.mMediaEmulationData.mDPPX > 0.0 {
            return Scale::new(pc.mMediaEmulationData.mDPPX);
        }

        let au_per_dpx = pc.mCurAppUnitsPerDevPixel as f32;
        let au_per_px = AU_PER_PX as f32;
        Scale::new(au_per_px / au_per_dpx)
    }

    /// Returns whether document colors are enabled.
    #[inline]
    pub fn use_document_colors(&self) -> bool {
        let doc = self.document();
        if doc.mIsBeingUsedAsImage() {
            return true;
        }
        let document_color_use = static_prefs::pref!("browser.display.document_color_use");
        let prefs = self.pref_sheet_prefs();
        match document_color_use {
            1 => true,
            2 => prefs.mIsChrome,
            _ => !prefs.mUseAccessibilityTheme,
        }
    }

    /// Returns the default background color.
    pub fn default_background_color(&self) -> RGBA {
        convert_nscolor_to_rgba(self.pref_sheet_prefs().mDefaultBackgroundColor)
    }

    /// Returns the current effective text zoom.
    #[inline]
    fn effective_text_zoom(&self) -> f32 {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return 1.,
        };
        pc.mEffectiveTextZoom
    }

    /// Applies text zoom to a font-size or line-height value (see nsStyleFont::ZoomText).
    #[inline]
    pub fn zoom_text(&self, size: Au) -> Au {
        size.scale_by(self.effective_text_zoom())
    }

    /// Un-apply text zoom.
    #[inline]
    pub fn unzoom_text(&self, size: Au) -> Au {
        size.scale_by(1. / self.effective_text_zoom())
    }
}
