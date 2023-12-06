/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gecko's media-query device and expression representation.

use crate::color::AbsoluteColor;
use crate::context::QuirksMode;
use crate::custom_properties::CssEnvironment;
use crate::font_metrics::FontMetrics;
use crate::gecko::values::{convert_absolute_color_to_nscolor, convert_nscolor_to_absolute_color};
use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs;
use crate::media_queries::MediaType;
use crate::properties::ComputedValues;
use crate::string_cache::Atom;
use crate::values::computed::font::GenericFontFamily;
use crate::values::computed::{ColorScheme, Length, NonNegativeLength};
use crate::values::specified::color::SystemColor;
use crate::values::specified::font::FONT_MEDIUM_PX;
use crate::values::specified::ViewportVariant;
use crate::values::{CustomIdent, KeyframesName};
use app_units::{Au, AU_PER_PX};
use euclid::default::Size2D;
use euclid::{Scale, SideOffsets2D};
use servo_arc::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::{cmp, fmt};
use style_traits::{CSSPixel, DevicePixel};

/// The `Device` in Gecko wraps a pres context, has a default values computed,
/// and contains all the viewport rule state.
pub struct Device {
    /// NB: The document owns the styleset, who owns the stylist, and thus the
    /// `Device`, so having a raw document pointer here is fine.
    document: *const structs::Document,
    default_values: Arc<ComputedValues>,
    /// The font size of the root element.
    ///
    /// This is set when computing the style of the root element, and used for
    /// rem units in other elements.
    ///
    /// When computing the style of the root element, there can't be any other
    /// style being computed at the same time, given we need the style of the
    /// parent to compute everything else. So it is correct to just use a
    /// relaxed atomic here.
    root_font_size: AtomicU32,
    /// The body text color, stored as an `nscolor`, used for the "tables
    /// inherit from body" quirk.
    ///
    /// <https://quirks.spec.whatwg.org/#the-tables-inherit-color-from-body-quirk>
    body_text_color: AtomicUsize,
    /// Whether any styles computed in the document relied on the root font-size
    /// by using rem units.
    used_root_font_size: AtomicBool,
    /// Whether any styles computed in the document relied on font metrics.
    used_font_metrics: AtomicBool,
    /// Whether any styles computed in the document relied on the viewport size
    /// by using vw/vh/vmin/vmax units.
    used_viewport_size: AtomicBool,
    /// Whether any styles computed in the document relied on the viewport size
    /// by using dvw/dvh/dvmin/dvmax units.
    used_dynamic_viewport_size: AtomicBool,
    /// The CssEnvironment object responsible of getting CSS environment
    /// variables.
    environment: CssEnvironment,
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use nsstring::nsCString;

        let mut doc_uri = nsCString::new();
        unsafe {
            bindings::Gecko_nsIURI_Debug((*self.document()).mDocumentURI.raw(), &mut doc_uri)
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
            root_font_size: AtomicU32::new(FONT_MEDIUM_PX.to_bits()),
            // This gets updated when we see the <body>, so it doesn't really
            // matter which color-scheme we look at here.
            body_text_color: AtomicUsize::new(prefs.mLightColors.mDefault as usize),
            used_root_font_size: AtomicBool::new(false),
            used_font_metrics: AtomicBool::new(false),
            used_viewport_size: AtomicBool::new(false),
            used_dynamic_viewport_size: AtomicBool::new(false),
            environment: CssEnvironment,
        }
    }

    /// Get the relevant environment to resolve `env()` functions.
    #[inline]
    pub fn environment(&self) -> &CssEnvironment {
        &self.environment
    }

    /// Returns the computed line-height for the font in a given computed values instance.
    ///
    /// If you pass down an element, then the used line-height is returned.
    pub fn calc_line_height(
        &self,
        line_height: &crate::values::computed::LineHeight,
        vertical: bool,
        font: &crate::properties::style_structs::Font,
        element: Option<super::wrapper::GeckoElement>,
    ) -> NonNegativeLength {
        let pres_context = self.pres_context();
        let au = Au(unsafe {
            bindings::Gecko_CalcLineHeight(
                line_height,
                pres_context.map_or(std::ptr::null(), |pc| pc),
                vertical,
                &**font,
                element.map_or(std::ptr::null(), |e| e.0),
            )
        });
        NonNegativeLength::new(au.to_f32_px())
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
    pub fn root_font_size(&self) -> Length {
        self.used_root_font_size.store(true, Ordering::Relaxed);
        Length::new(f32::from_bits(self.root_font_size.load(Ordering::Relaxed)))
    }

    /// Set the font size of the root element (for rem)
    pub fn set_root_font_size(&self, size: Length) {
        self.root_font_size
            .store(size.px().to_bits(), Ordering::Relaxed)
    }

    /// The quirks mode of the document.
    pub fn quirks_mode(&self) -> QuirksMode {
        self.document().mCompatMode.into()
    }

    /// Sets the body text color for the "inherit color from body" quirk.
    ///
    /// <https://quirks.spec.whatwg.org/#the-tables-inherit-color-from-body-quirk>
    pub fn set_body_text_color(&self, color: AbsoluteColor) {
        self.body_text_color.store(
            convert_absolute_color_to_nscolor(&color) as usize,
            Ordering::Relaxed,
        )
    }

    /// Gets the base size given a generic font family and a language.
    pub fn base_size_for_generic(&self, language: &Atom, generic: GenericFontFamily) -> Length {
        unsafe { bindings::Gecko_GetBaseSize(self.document(), language.as_ptr(), generic) }
    }

    /// Gets the size of the scrollbar in CSS pixels.
    pub fn scrollbar_inline_size(&self) -> Length {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            // XXX: we could have a more reasonable default perhaps.
            None => return Length::new(0.0),
        };
        Length::new(unsafe { bindings::Gecko_GetScrollbarInlineSize(pc) })
    }

    /// Queries font metrics
    pub fn query_font_metrics(
        &self,
        vertical: bool,
        font: &crate::properties::style_structs::Font,
        base_size: Length,
        in_media_query: bool,
        retrieve_math_scales: bool,
    ) -> FontMetrics {
        self.used_font_metrics.store(true, Ordering::Relaxed);
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return Default::default(),
        };
        let gecko_metrics = unsafe {
            bindings::Gecko_GetFontMetrics(
                pc,
                vertical,
                &**font,
                base_size,
                // we don't use the user font set in a media query
                !in_media_query,
                retrieve_math_scales,
            )
        };
        FontMetrics {
            x_height: Some(gecko_metrics.mXSize),
            zero_advance_measure: if gecko_metrics.mChSize.px() >= 0. {
                Some(gecko_metrics.mChSize)
            } else {
                None
            },
            cap_height: if gecko_metrics.mCapHeight.px() >= 0. {
                Some(gecko_metrics.mCapHeight)
            } else {
                None
            },
            ic_width: if gecko_metrics.mIcWidth.px() >= 0. {
                Some(gecko_metrics.mIcWidth)
            } else {
                None
            },
            ascent: gecko_metrics.mAscent,
            script_percent_scale_down: if gecko_metrics.mScriptPercentScaleDown > 0. {
                Some(gecko_metrics.mScriptPercentScaleDown)
            } else {
                None
            },
            script_script_percent_scale_down: if gecko_metrics.mScriptScriptPercentScaleDown > 0. {
                Some(gecko_metrics.mScriptScriptPercentScaleDown)
            } else {
                None
            },
        }
    }

    /// Returns the body text color.
    pub fn body_text_color(&self) -> AbsoluteColor {
        convert_nscolor_to_absolute_color(self.body_text_color.load(Ordering::Relaxed) as u32)
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
        self.used_font_metrics.store(false, Ordering::Relaxed);
        self.used_viewport_size.store(false, Ordering::Relaxed);
        self.used_dynamic_viewport_size
            .store(false, Ordering::Relaxed);
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

    /// Returns whether this document is in print preview.
    pub fn is_print_preview(&self) -> bool {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return false,
        };
        pc.mType == structs::nsPresContext_nsPresContextType_eContext_PrintPreview
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

    // It may make sense to account for @page rule margins here somehow, however
    // it's not clear how that'd work, see:
    // https://github.com/w3c/csswg-drafts/issues/5437
    fn page_size_minus_default_margin(&self, pc: &structs::nsPresContext) -> Size2D<Au> {
        debug_assert!(pc.mIsRootPaginatedDocument() != 0);
        let area = &pc.mPageSize;
        let margin = &pc.mDefaultPageMargin;
        let width = area.width - margin.left - margin.right;
        let height = area.height - margin.top - margin.bottom;
        Size2D::new(Au(cmp::max(width, 0)), Au(cmp::max(height, 0)))
    }

    /// Returns the current viewport size in app units.
    pub fn au_viewport_size(&self) -> Size2D<Au> {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return Size2D::new(Au(0), Au(0)),
        };

        if pc.mIsRootPaginatedDocument() != 0 {
            return self.page_size_minus_default_margin(pc);
        }

        let area = &pc.mVisibleArea;
        Size2D::new(Au(area.width), Au(area.height))
    }

    /// Returns the current viewport size in app units, recording that it's been
    /// used for viewport unit resolution.
    pub fn au_viewport_size_for_viewport_unit_resolution(
        &self,
        variant: ViewportVariant,
    ) -> Size2D<Au> {
        self.used_viewport_size.store(true, Ordering::Relaxed);
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return Size2D::new(Au(0), Au(0)),
        };

        if pc.mIsRootPaginatedDocument() != 0 {
            return self.page_size_minus_default_margin(pc);
        }

        match variant {
            ViewportVariant::UADefault => {
                let size = &pc.mSizeForViewportUnits;
                Size2D::new(Au(size.width), Au(size.height))
            },
            ViewportVariant::Small => {
                let size = &pc.mVisibleArea;
                Size2D::new(Au(size.width), Au(size.height))
            },
            ViewportVariant::Large => {
                let size = &pc.mVisibleArea;
                // Looks like IntCoordTyped is treated as if it's u32 in Rust.
                debug_assert!(
                    /* pc.mDynamicToolbarMaxHeight >=0 && */
                    pc.mDynamicToolbarMaxHeight < i32::MAX as u32
                );
                Size2D::new(
                    Au(size.width),
                    Au(size.height +
                        pc.mDynamicToolbarMaxHeight as i32 * pc.mCurAppUnitsPerDevPixel),
                )
            },
            ViewportVariant::Dynamic => {
                self.used_dynamic_viewport_size
                    .store(true, Ordering::Relaxed);
                let size = &pc.mVisibleArea;
                // Looks like IntCoordTyped is treated as if it's u32 in Rust.
                debug_assert!(
                    /* pc.mDynamicToolbarHeight >=0 && */
                    pc.mDynamicToolbarHeight < i32::MAX as u32
                );
                Size2D::new(
                    Au(size.width),
                    Au(size.height +
                        (pc.mDynamicToolbarMaxHeight - pc.mDynamicToolbarHeight) as i32 *
                            pc.mCurAppUnitsPerDevPixel),
                )
            },
        }
    }

    /// Returns whether we ever looked up the viewport size of the Device.
    pub fn used_viewport_size(&self) -> bool {
        self.used_viewport_size.load(Ordering::Relaxed)
    }

    /// Returns whether we ever looked up the dynamic viewport size of the Device.
    pub fn used_dynamic_viewport_size(&self) -> bool {
        self.used_dynamic_viewport_size.load(Ordering::Relaxed)
    }

    /// Returns whether font metrics have been queried.
    pub fn used_font_metrics(&self) -> bool {
        self.used_font_metrics.load(Ordering::Relaxed)
    }

    /// Returns whether visited styles are enabled.
    pub fn visited_styles_enabled(&self) -> bool {
        unsafe { bindings::Gecko_VisitedStylesEnabled(self.document()) }
    }

    /// Returns the number of app units per device pixel we're using currently.
    pub fn app_units_per_device_pixel(&self) -> i32 {
        match self.pres_context() {
            Some(pc) => pc.mCurAppUnitsPerDevPixel,
            None => AU_PER_PX,
        }
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
        self.pref_sheet_prefs().mUseDocumentColors
    }

    /// Computes a system color and returns it as an nscolor.
    pub(crate) fn system_nscolor(
        &self,
        system_color: SystemColor,
        color_scheme: &ColorScheme,
    ) -> u32 {
        unsafe { bindings::Gecko_ComputeSystemColor(system_color, self.document(), color_scheme) }
    }

    /// Returns the default background color.
    ///
    /// This is only for forced-colors/high-contrast, so looking at light colors
    /// is ok.
    pub fn default_background_color(&self) -> AbsoluteColor {
        let normal = ColorScheme::normal();
        convert_nscolor_to_absolute_color(self.system_nscolor(SystemColor::Canvas, &normal))
    }

    /// Returns the default foreground color.
    ///
    /// See above for looking at light colors only.
    pub fn default_color(&self) -> AbsoluteColor {
        let normal = ColorScheme::normal();
        convert_nscolor_to_absolute_color(self.system_nscolor(SystemColor::Canvastext, &normal))
    }

    /// Returns the current effective text zoom.
    #[inline]
    fn text_zoom(&self) -> f32 {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return 1.,
        };
        pc.mTextZoom
    }

    /// Applies text zoom to a font-size or line-height value (see nsStyleFont::ZoomText).
    #[inline]
    pub fn zoom_text(&self, size: Length) -> Length {
        size.scale_by(self.text_zoom())
    }

    /// Un-apply text zoom.
    #[inline]
    pub fn unzoom_text(&self, size: Length) -> Length {
        size.scale_by(1. / self.text_zoom())
    }

    /// Returns safe area insets
    pub fn safe_area_insets(&self) -> SideOffsets2D<f32, CSSPixel> {
        let pc = match self.pres_context() {
            Some(pc) => pc,
            None => return SideOffsets2D::zero(),
        };
        let mut top = 0.0;
        let mut right = 0.0;
        let mut bottom = 0.0;
        let mut left = 0.0;
        unsafe {
            bindings::Gecko_GetSafeAreaInsets(pc, &mut top, &mut right, &mut bottom, &mut left)
        };
        SideOffsets2D::new(top, right, bottom, left)
    }

    /// Returns true if the given MIME type is supported
    pub fn is_supported_mime_type(&self, mime_type: &str) -> bool {
        unsafe {
            bindings::Gecko_IsSupportedImageMimeType(mime_type.as_ptr(), mime_type.len() as u32)
        }
    }

    /// Return whether the document is a chrome document.
    ///
    /// This check is consistent with how we enable chrome rules for chrome:// and resource://
    /// stylesheets (and thus chrome:// documents).
    #[inline]
    pub fn chrome_rules_enabled_for_document(&self) -> bool {
        self.document().mChromeRulesEnabled()
    }
}
