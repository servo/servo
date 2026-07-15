/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::iter;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use app_units::Au;
use content_security_policy::Violation;
use fonts_traits::{
    CSSFontFaceDescriptors, FontDescriptor, FontFaceRuleWithOrigin, FontIdentifier, FontTemplate,
    FontTemplateRef, FontTemplateRefMethods, StylesheetWebFontLoadFinishedCallback,
    WebFontSetDifference,
};
use log::{debug, trace};
use malloc_size_of::MallocSizeOf;
use malloc_size_of_derive::MallocSizeOf;
use net_traits::blob_url_store::UrlWithBlobClaim;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CredentialsMode, Destination, Referrer, RequestBuilder, RequestClient, RequestMode,
    ServiceWorkersMode,
};
use net_traits::{
    CoreResourceThread, FetchResponseMsg, ResourceFetchTiming, ResourceThreads, fetch_async,
};
use paint_api::CrossProcessPaintApi;
use parking_lot::{Mutex, RwLock};
use rustc_hash::FxHashSet;
use servo_arc::Arc as ServoArc;
use servo_base::id::{PainterId, WebViewId};
use servo_config::pref;
use servo_url::ServoUrl;
use style::Atom;
use style::computed_values::font_variant_caps::T as FontVariantCaps;
use style::font_face::{
    FontFaceSourceFormat, FontFaceSourceFormatKeyword, Source, SourceList, UrlSource,
};
use style::properties::generated::font_face::Descriptors as FontFaceRuleDescriptors;
use style::properties::style_structs::Font as FontStyleStruct;
use style::shared_lock::StylesheetGuards;
use style::stylesheets::FontFaceRule;
use style::stylist::Stylist;
use style::values::computed::font::{FamilyName, FontFamilyNameSyntax, SingleFontFamily};
use url::Url;
use uuid::Uuid;
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey, FontVariation};

use crate::font::{Font, FontFamilyDescriptor, FontGroup, FontRef, FontSearchScope};
use crate::font_store::{CrossThreadFontStore, FontStore};
use crate::platform::font::PlatformFont;
use crate::{FontData, LowercaseFontFamilyName, PlatformFontMethods, SystemFontServiceProxy};

static SMALL_CAPS_SCALE_FACTOR: f32 = 0.8; // Matches FireFox (see gfxFont.h)

#[derive(Eq, Hash, MallocSizeOf, PartialEq)]
pub(crate) struct FontParameters {
    pub(crate) font_key: FontKey,
    pub(crate) pt_size: Au,
    pub(crate) variations: Vec<FontVariation>,
    pub(crate) flags: FontInstanceFlags,
}

pub type FontGroupRef = Arc<FontGroup>;

/// The FontContext represents the per-thread/thread state necessary for
/// working with fonts. It is the public API used by the layout and
/// paint code. It talks directly to the system font service where
/// required.
#[derive(MallocSizeOf)]
pub struct FontContext {
    #[conditional_malloc_size_of]
    system_font_service_proxy: Arc<SystemFontServiceProxy>,

    resource_threads: Mutex<CoreResourceThread>,

    /// A sender that can send messages and receive replies from `Paint`.
    paint_api: Mutex<CrossProcessPaintApi>,

    /// The actual instances of fonts ie a [`FontTemplate`] combined with a size and
    /// other font properties, along with the font data and a platform font instance.
    fonts: RwLock<HashMap<FontCacheKey, Option<FontRef>>>,

    /// A caching map between the specification of a font in CSS style and
    /// resolved [`FontGroup`] which contains information about all fonts that
    /// can be selected with that style.
    #[conditional_malloc_size_of]
    resolved_font_groups: RwLock<HashMap<FontGroupCacheKey, FontGroupRef>>,

    web_fonts: CrossThreadFontStore,

    /// A collection of WebRender [`FontKey`]s generated for the web fonts that this
    /// [`FontContext`] controls.
    webrender_font_keys: RwLock<HashMap<FontIdentifier, FontKey>>,

    /// A collection of WebRender [`FontInstanceKey`]s generated for the web fonts that
    /// this [`FontContext`] controls.
    webrender_font_instance_keys: RwLock<HashMap<FontParameters, FontInstanceKey>>,

    /// The data for each web font [`FontIdentifier`]. This data might be used by more than one
    /// [`FontTemplate`] as each identifier refers to a URL.
    font_data: RwLock<HashMap<FontIdentifier, FontData>>,

    have_removed_web_fonts: AtomicBool,

    /// Maps from a URL to all the `@font-face` rules that are currently waiting for the load to
    /// finish.
    currently_downloading_fonts: Mutex<HashMap<ServoUrl, Vec<WebFontDownloadState>>>,

    /// The set of `@font-face` rules that are currently present in the CSS cascade. This is not necessarily
    /// equivalent to the rules that actually apply to the page, because rules that are invalid or not
    /// yet downloaded are also included.
    known_font_face_rules: Mutex<KnownFontFaceRules>,
}

/// A callback that will be invoked on the Fetch thread if a web font download
/// results in CSP violations. This handler will be cloned each time a new
/// web font download is initiated.
pub trait CspViolationHandler: Send + std::fmt::Debug + MallocSizeOf {
    fn process_violations(&self, violations: Vec<Violation>);
    fn clone(&self) -> Box<dyn CspViolationHandler>;
}

/// A callback that will be invoked on the Fetch thread when a web font
/// download succeeds, providing timing information about the request.
pub trait NetworkTimingHandler: Send + std::fmt::Debug + MallocSizeOf {
    fn submit_timing(&self, url: ServoUrl, response: ResourceFetchTiming);
    fn clone(&self) -> Box<dyn NetworkTimingHandler>;
}

/// Document-specific data required to fetch a web font.
#[derive(Debug, MallocSizeOf)]
pub struct WebFontDocumentContext {
    pub policy_container: PolicyContainer,
    pub request_client: RequestClient,
    pub document_url: ServoUrl,
    pub csp_handler: Box<dyn CspViolationHandler>,
    pub network_timing_handler: Box<dyn NetworkTimingHandler>,
}

impl Clone for WebFontDocumentContext {
    fn clone(&self) -> WebFontDocumentContext {
        Self {
            policy_container: self.policy_container.clone(),
            request_client: self.request_client.clone(),
            document_url: self.document_url.clone(),
            csp_handler: self.csp_handler.clone(),
            network_timing_handler: self.network_timing_handler.clone(),
        }
    }
}

impl FontContext {
    pub fn new(
        system_font_service_proxy: Arc<SystemFontServiceProxy>,
        paint_api: CrossProcessPaintApi,
        resource_threads: ResourceThreads,
    ) -> Self {
        Self {
            system_font_service_proxy,
            resource_threads: Mutex::new(resource_threads.core_thread),
            paint_api: Mutex::new(paint_api),
            fonts: Default::default(),
            resolved_font_groups: Default::default(),
            web_fonts: Default::default(),
            webrender_font_keys: RwLock::default(),
            webrender_font_instance_keys: RwLock::default(),
            have_removed_web_fonts: AtomicBool::new(false),
            font_data: RwLock::default(),
            currently_downloading_fonts: Default::default(),
            known_font_face_rules: Default::default(),
        }
    }

    pub fn web_fonts_still_loading(&self) -> usize {
        self.currently_downloading_fonts.lock().len()
    }

    fn get_font_data(&self, identifier: &FontIdentifier) -> Option<FontData> {
        match identifier {
            FontIdentifier::Web(_) | FontIdentifier::ArrayBuffer(_) => {
                self.font_data.read().get(identifier).cloned()
            },
            FontIdentifier::Local(_) => None,
        }
    }

    /// Returns a `FontGroup` representing fonts which can be used for layout, given the `style`.
    /// Font groups are cached, so subsequent calls with the same `style` will return a reference
    /// to an existing `FontGroup`.
    pub fn font_group(&self, style: ServoArc<FontStyleStruct>) -> FontGroupRef {
        let font_size = style.font_size.computed_size().into();
        self.font_group_with_size(style, font_size)
    }

    /// Like [`Self::font_group`], but overriding the size found in the [`FontStyleStruct`] with the given size
    /// in pixels.
    pub fn font_group_with_size(
        &self,
        style: ServoArc<FontStyleStruct>,
        size: Au,
    ) -> Arc<FontGroup> {
        let cache_key = FontGroupCacheKey { size, style };
        if let Some(font_group) = self.resolved_font_groups.read().get(&cache_key) {
            return font_group.clone();
        }

        let mut descriptor = FontDescriptor::from(&*cache_key.style);
        descriptor.pt_size = size;

        let font_group = Arc::new(FontGroup::new(&cache_key.style, descriptor));
        self.resolved_font_groups
            .write()
            .insert(cache_key, font_group.clone());
        font_group
    }

    /// Returns a font matching the parameters. Fonts are cached, so repeated calls will return a
    /// reference to the same underlying `Font`.
    pub fn font(
        &self,
        font_template: FontTemplateRef,
        font_descriptor: &FontDescriptor,
    ) -> Option<FontRef> {
        let font_descriptor = if servo_config::pref!(layout_variable_fonts_enabled) {
            let variation_settings = font_template.borrow().compute_variations(font_descriptor);
            &font_descriptor.with_variation_settings(variation_settings)
        } else {
            font_descriptor
        };

        self.get_font_maybe_synthesizing_small_caps(
            font_template,
            font_descriptor,
            true, /* synthesize_small_caps */
        )
    }

    fn get_font_maybe_synthesizing_small_caps(
        &self,
        font_template: FontTemplateRef,
        font_descriptor: &FontDescriptor,
        synthesize_small_caps: bool,
    ) -> Option<FontRef> {
        // TODO: (Bug #3463): Currently we only support fake small-caps
        // painting. We should also support true small-caps (where the
        // font supports it) in the future.
        let synthesized_small_caps_font =
            if font_descriptor.variant == FontVariantCaps::SmallCaps && synthesize_small_caps {
                let mut small_caps_descriptor = font_descriptor.clone();
                small_caps_descriptor.pt_size =
                    font_descriptor.pt_size.scale_by(SMALL_CAPS_SCALE_FACTOR);
                self.get_font_maybe_synthesizing_small_caps(
                    font_template.clone(),
                    &small_caps_descriptor,
                    false, /* synthesize_small_caps */
                )
            } else {
                None
            };

        let cache_key = FontCacheKey {
            font_identifier: font_template.identifier(),
            font_descriptor: font_descriptor.clone(),
        };

        if let Some(font) = self.fonts.read().get(&cache_key).cloned() {
            return font;
        }

        debug!(
            "FontContext::font cache miss for font_template={:?} font_descriptor={:?}",
            font_template, font_descriptor
        );

        // Check one more time whether the font is cached or not. There's a potential race
        // condition, where between the time we took the read lock above and now, another thread
        // added the font to the cache. This check makes sense, because loading a font has memory
        // implications and is much slower than checking the map again.
        let mut fonts = self.fonts.write();
        if let Some(font) = fonts.get(&cache_key).cloned() {
            return font;
        }

        // TODO: Inserting `None` into the cache here is a bit bogus. Instead we should somehow
        // mark this template as invalid so it isn't tried again.
        let font = self
            .create_font(
                font_template,
                font_descriptor.to_owned(),
                synthesized_small_caps_font,
            )
            .ok();
        fonts.insert(cache_key, font.clone());
        font
    }

    fn matching_web_font_templates(
        &self,
        descriptor_to_match: &FontDescriptor,
        family_descriptor: &FontFamilyDescriptor,
    ) -> Option<Vec<FontTemplateRef>> {
        if family_descriptor.scope != FontSearchScope::Any {
            return None;
        }

        // Do not look for generic fonts in our list of web fonts.
        let SingleFontFamily::FamilyName(ref family_name) = family_descriptor.family else {
            return None;
        };

        self.web_fonts
            .read()
            .families
            .get(&family_name.name.clone().into())
            .map(|templates| templates.find_for_descriptor(Some(descriptor_to_match)))
    }

    /// Try to find matching templates in this [`FontContext`], first looking in the list of web fonts and
    /// falling back to asking the [`super::SystemFontService`] for a matching system font.
    pub fn matching_templates(
        &self,
        descriptor_to_match: &FontDescriptor,
        family_descriptor: &FontFamilyDescriptor,
    ) -> Vec<FontTemplateRef> {
        self.matching_web_font_templates(descriptor_to_match, family_descriptor)
            .unwrap_or_else(|| {
                self.system_font_service_proxy.find_matching_font_templates(
                    Some(descriptor_to_match),
                    &family_descriptor.family,
                )
            })
    }

    /// Create a `Font` for use in layout calculations, from a `FontTemplateData` returned by the
    /// cache thread and a `FontDescriptor` which contains the styling parameters.
    #[servo_tracing::instrument(skip_all)]
    fn create_font(
        &self,
        font_template: FontTemplateRef,
        font_descriptor: FontDescriptor,
        synthesized_small_caps: Option<FontRef>,
    ) -> Result<FontRef, &'static str> {
        Ok(FontRef(Arc::new(Font::new(
            font_template.clone(),
            font_descriptor,
            self.get_font_data(&font_template.identifier()),
            synthesized_small_caps,
        )?)))
    }

    pub(crate) fn create_font_instance_key(
        &self,
        font: &Font,
        painter_id: PainterId,
    ) -> FontInstanceKey {
        match font.template.identifier() {
            FontIdentifier::Local(_) => self.system_font_service_proxy.get_system_font_instance(
                font.template.identifier(),
                font.descriptor.pt_size,
                font.webrender_font_instance_flags(),
                font.variations().to_owned(),
                painter_id,
            ),
            FontIdentifier::Web(_) | FontIdentifier::ArrayBuffer(_) => self
                .create_web_font_instance(
                    font.template.clone(),
                    font.descriptor.pt_size,
                    font.webrender_font_instance_flags(),
                    font.variations().to_owned(),
                    painter_id,
                ),
        }
    }

    fn create_web_font_instance(
        &self,
        font_template: FontTemplateRef,
        pt_size: Au,
        flags: FontInstanceFlags,
        variations: Vec<FontVariation>,
        painter_id: PainterId,
    ) -> FontInstanceKey {
        let identifier = font_template.identifier();
        let font_data = self
            .get_font_data(&identifier)
            .expect("Web font should have associated font data");
        let font_key = *self
            .webrender_font_keys
            .write()
            .entry(identifier.clone())
            .or_insert_with(|| {
                let font_key = self.system_font_service_proxy.generate_font_key(painter_id);
                self.paint_api.lock().add_font(
                    font_key,
                    font_data.as_ipc_shared_memory(),
                    identifier.index(),
                );
                font_key
            });

        let entry_key = FontParameters {
            font_key,
            pt_size,
            variations: variations.clone(),
            flags,
        };
        *self
            .webrender_font_instance_keys
            .write()
            .entry(entry_key)
            .or_insert_with(|| {
                let font_instance_key = self
                    .system_font_service_proxy
                    .generate_font_instance_key(painter_id);
                self.paint_api.lock().add_font_instance(
                    font_instance_key,
                    font_key,
                    pt_size.to_f32_px(),
                    flags,
                    variations,
                );
                font_instance_key
            })
    }

    fn invalidate_font_groups_after_web_font_load(&self) {
        self.resolved_font_groups.write().clear();
    }

    pub fn is_supported_web_font_source(source: &&Source) -> bool {
        let url_source = match &source {
            Source::Url(url_source) => url_source,
            Source::Local(_) => return true,
        };
        let format_hint = match url_source.format_hint {
            Some(ref format_hint) => format_hint,
            None => return true,
        };

        if matches!(
            format_hint,
            FontFaceSourceFormat::Keyword(
                FontFaceSourceFormatKeyword::Truetype |
                    FontFaceSourceFormatKeyword::Opentype |
                    FontFaceSourceFormatKeyword::Woff |
                    FontFaceSourceFormatKeyword::Woff2
            )
        ) {
            return true;
        }

        if let FontFaceSourceFormat::String(string) = format_hint {
            if string == "truetype" || string == "opentype" || string == "woff" || string == "woff2"
            {
                return true;
            }

            return pref!(layout_variable_fonts_enabled) &&
                (string == "truetype-variations" ||
                    string == "opentype-variations" ||
                    string == "woff-variations" ||
                    string == "woff2-variations");
        }

        false
    }

    fn is_local_or_unknown_url_font(
        &self,
        family_name: &LowercaseFontFamilyName,
        source: &Source,
    ) -> bool {
        match source {
            Source::Url(url) => !url
                .url
                .url()
                .cloned()
                .map(ServoUrl::from)
                .map(FontIdentifier::Web)
                .filter(|font_identifier| self.font_data.read().contains_key(font_identifier))
                .is_some_and(|font_identifier| {
                    self.web_fonts
                        .read()
                        .families
                        .get(family_name)
                        .is_some_and(|templates| {
                            templates
                                .templates
                                .iter()
                                .any(|template| template.borrow().identifier == font_identifier)
                        })
                }),
            Source::Local(_) => true,
        }
    }

    pub(crate) fn handle_web_font_request_started(
        &self,
        url: ServoUrl,
        state: WebFontDownloadState,
    ) {
        self.currently_downloading_fonts
            .lock()
            .entry(url)
            .or_default()
            .push(state);
    }

    /// Handle a web font load finishing, adding the new font to the [`FontStore`]. If the web font
    /// load was canceled (for instance, if the stylesheet was removed), then do nothing and return
    /// false.
    ///
    /// All download states waiting for this entry to load will have their promise fulfilled.
    pub(crate) fn handle_web_font_request_succeeded(
        &self,
        font_data: FontData,
        url: ServoUrl,
    ) -> bool {
        let Some(download_states) = self.currently_downloading_fonts.lock().remove(&url) else {
            // No one is waiting for this web font to load ):
            return false;
        };
        debug_assert!(
            !download_states.is_empty(),
            "Should have removed this entry"
        );

        let identifier = FontIdentifier::Web(url);
        let Ok(handle) =
            PlatformFont::new_from_data(identifier.clone(), &font_data, None, &[], false)
        else {
            return false;
        };

        self.font_data.write().insert(identifier.clone(), font_data);
        let descriptor = handle.descriptor();
        for download_state in download_states {
            let mut descriptor = descriptor.clone();
            descriptor.override_values_with_css_font_template_descriptors(
                &download_state.css_font_face_descriptors,
            );

            let new_template = FontTemplate::new(
                identifier.clone(),
                descriptor,
                download_state.initiator.font_face_rule().cloned(),
            );

            download_state.handle_web_font_load_success(new_template);
        }

        true
    }

    pub(crate) fn has_pending_font_requests_for_url(&self, url: ServoArc<Url>) -> bool {
        self.currently_downloading_fonts
            .lock()
            .contains_key(&url.into())
    }
}

/// Tracks the progress of loading a single `@font-face` rule by trying all specified
/// sources in order.
#[derive(MallocSizeOf)]
pub(crate) struct WebFontDownloadState {
    webview_id: Option<WebViewId>,
    css_font_face_descriptors: CSSFontFaceDescriptors,
    remaining_sources: Vec<Source>,
    local_fonts: HashMap<Atom, Option<FontTemplateRef>>,
    #[conditional_malloc_size_of]
    pub(crate) font_context: Arc<FontContext>,
    initiator: WebFontLoadInitiator,
    document_context: WebFontDocumentContext,
}

impl WebFontDownloadState {
    fn new(
        webview_id: Option<WebViewId>,
        font_context: Arc<FontContext>,
        css_font_face_descriptors: CSSFontFaceDescriptors,
        initiator: WebFontLoadInitiator,
        sources: Vec<Source>,
        local_fonts: HashMap<Atom, Option<FontTemplateRef>>,
        document_context: WebFontDocumentContext,
    ) -> WebFontDownloadState {
        WebFontDownloadState {
            webview_id,
            css_font_face_descriptors,
            remaining_sources: sources,
            local_fonts,
            font_context,
            initiator,
            document_context,
        }
    }

    pub(crate) fn handle_web_font_load_success(self, new_template: FontTemplate) {
        let family_name = self.css_font_face_descriptors.family_name.clone();
        match self.initiator {
            WebFontLoadInitiator::Stylesheet(initiator) => {
                self.font_context
                    .web_fonts
                    .write()
                    .add_new_template(family_name, new_template);
                self.font_context
                    .invalidate_font_groups_after_web_font_load();
                (initiator.callback)(true);
            },
            WebFontLoadInitiator::Script(callback) => {
                callback(family_name, Some(new_template));
            },
        }
    }

    /// Called when we've tried all available sources and none were usable.
    pub(crate) fn handle_web_font_load_failure(self) {
        let family_name = self.css_font_face_descriptors.family_name.clone();
        match self.initiator {
            WebFontLoadInitiator::Stylesheet(initiator) => {
                (initiator.callback)(false);
            },
            WebFontLoadInitiator::Script(callback) => {
                callback(family_name, None);
            },
        }
    }

    fn was_created_for_font_face_rule(&self, font_face_rule: &FontFaceRuleDescriptors) -> bool {
        self.initiator
            .font_face_rule()
            .is_some_and(|initiating_rule| initiating_rule == font_face_rule)
    }
}

pub trait FontContextWebFontMethods {
    fn rebuild_font_face_set(
        &self,
        webview_id: WebViewId,
        stylist: &Stylist,
        guards: &StylesheetGuards<'_>,
        callback: StylesheetWebFontLoadFinishedCallback,
        document_context: &WebFontDocumentContext,
    ) -> WebFontSetDifference;
    fn load_single_font_face_rule(
        &self,
        font_face_rule: &FontFaceRule,
        webview_id: WebViewId,
        callback: StylesheetWebFontLoadFinishedCallback,
        document_context: &WebFontDocumentContext,
    );
    fn load_web_font_for_script(
        &self,
        webview_id: Option<WebViewId>,
        sources: SourceList,
        descriptors: CSSFontFaceDescriptors,
        finished_callback: ScriptWebFontLoadFinishedCallback,
        document_context: &WebFontDocumentContext,
    );
    fn handle_web_font_request_failed(&self, url: ServoUrl);
}

impl FontContextWebFontMethods for Arc<FontContext> {
    fn load_single_font_face_rule(
        &self,
        font_face_rule: &FontFaceRule,
        webview_id: WebViewId,
        callback: StylesheetWebFontLoadFinishedCallback,
        document_context: &WebFontDocumentContext,
    ) {
        let Some(ref sources) = font_face_rule.descriptors.src else {
            return;
        };

        let css_font_face_descriptors = font_face_rule.into();

        let initiator = FontFaceRuleInitiator {
            font_face_rule: font_face_rule.descriptors.clone(),
            callback: callback.clone(),
        };

        self.start_loading_one_web_font(
            Some(webview_id),
            sources,
            css_font_face_descriptors,
            WebFontLoadInitiator::Stylesheet(Box::new(initiator)),
            document_context,
        );
    }
    fn rebuild_font_face_set(
        &self,
        webview_id: WebViewId,
        stylist: &Stylist,
        guards: &StylesheetGuards<'_>,
        callback: StylesheetWebFontLoadFinishedCallback,
        document_context: &WebFontDocumentContext,
    ) -> WebFontSetDifference {
        let difference = self
            .known_font_face_rules
            .lock()
            .diff_old_and_new_font_face_rules(stylist, guards);

        for added_rule in &difference.added_font_faces {
            let added_rule = added_rule.read_with(guards);
            self.load_single_font_face_rule(
                added_rule,
                webview_id,
                callback.clone(),
                document_context,
            );
        }
        for removed_rule in &difference.removed_font_faces {
            let removed_rule = removed_rule.read_with(guards);
            self.remove_single_font_face_rule(
                &removed_rule.descriptors,
                &mut self.web_fonts.write(),
            );
        }

        if !difference.removed_font_faces.is_empty() {
            // We modified the list of available fonts, so invalidate resolved font groups.
            self.resolved_font_groups.write().clear();

            // Ensure that we clean up any WebRender resources on the next display list update.
            self.have_removed_web_fonts.store(true, Ordering::Relaxed);
        }

        difference
    }

    fn load_web_font_for_script(
        &self,
        webview_id: Option<WebViewId>,
        sources: SourceList,
        descriptors: CSSFontFaceDescriptors,
        finished_callback: ScriptWebFontLoadFinishedCallback,
        document_context: &WebFontDocumentContext,
    ) {
        let completion_handler = WebFontLoadInitiator::Script(finished_callback);
        self.start_loading_one_web_font(
            webview_id,
            &sources,
            descriptors,
            completion_handler,
            document_context,
        );
    }

    /// Called when a single URL for a `@font-face` failed to load.
    fn handle_web_font_request_failed(&self, url: ServoUrl) {
        let Some(subscribers) = self.currently_downloading_fonts.lock().remove(&url) else {
            return;
        };

        for subscriber in subscribers {
            self.process_next_web_font_source(subscriber);
        }
    }
}

impl FontContext {
    pub fn collect_unused_webrender_resources(
        &self,
        all: bool,
    ) -> (Vec<FontKey>, Vec<FontInstanceKey>) {
        if all {
            let mut webrender_font_keys = self.webrender_font_keys.write();
            let mut webrender_font_instance_keys = self.webrender_font_instance_keys.write();
            self.have_removed_web_fonts.store(false, Ordering::Relaxed);
            return (
                webrender_font_keys.drain().map(|(_, key)| key).collect(),
                webrender_font_instance_keys
                    .drain()
                    .map(|(_, key)| key)
                    .collect(),
            );
        }

        if !self.have_removed_web_fonts.load(Ordering::Relaxed) {
            return (Vec::new(), Vec::new());
        }

        // Lock everything to prevent adding new fonts while we are cleaning up the old ones.
        let web_fonts = self.web_fonts.write();
        let mut font_data = self.font_data.write();
        let _fonts = self.fonts.write();
        let _font_groups = self.resolved_font_groups.write();
        let mut webrender_font_keys = self.webrender_font_keys.write();
        let mut webrender_font_instance_keys = self.webrender_font_instance_keys.write();

        let mut unused_identifiers: HashSet<FontIdentifier> =
            webrender_font_keys.keys().cloned().collect();
        for templates in web_fonts.families.values() {
            templates.for_all_identifiers(|identifier| {
                unused_identifiers.remove(identifier);
            });
        }

        font_data.retain(|font_identifier, _| !unused_identifiers.contains(font_identifier));

        self.have_removed_web_fonts.store(false, Ordering::Relaxed);

        let mut removed_keys: FxHashSet<FontKey> = FxHashSet::default();
        webrender_font_keys.retain(|identifier, font_key| {
            if unused_identifiers.contains(identifier) {
                removed_keys.insert(*font_key);
                false
            } else {
                true
            }
        });

        let mut removed_instance_keys: HashSet<FontInstanceKey> = HashSet::new();
        webrender_font_instance_keys.retain(|font_param, instance_key| {
            if removed_keys.contains(&font_param.font_key) {
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

    /// Returns `true` if any font templates were removed.
    fn remove_single_font_face_rule(
        &self,
        font_face_rule: &FontFaceRuleDescriptors,
        font_store: &mut FontStore,
    ) -> bool {
        let Some(family) = font_face_rule.font_family.as_ref() else {
            return false;
        };

        // Mark any ongoing load operations for this font as cancelled.
        self.currently_downloading_fonts
            .lock()
            .retain(|_, download_states| {
                download_states.retain(|download_state| {
                    !download_state.was_created_for_font_face_rule(font_face_rule)
                });

                !download_states.is_empty()
            });

        let lowercase_family_name: LowercaseFontFamilyName = family.name.clone().into();
        let Some(known_family) = font_store.families.get_mut(&lowercase_family_name) else {
            return false;
        };
        if !known_family.remove_template_for_font_face_rule(font_face_rule) {
            return false;
        }
        self.fonts.write().retain(|_, font| match font {
            Some(font) => !font
                .template
                .borrow()
                .is_defined_by_font_face_rule(font_face_rule),
            _ => true,
        });

        true
    }

    pub fn add_template_to_font_context(
        &self,
        family_name: LowercaseFontFamilyName,
        new_template: FontTemplate,
    ) {
        self.web_fonts
            .write()
            .add_new_template(family_name, new_template);
        self.invalidate_font_groups_after_web_font_load();
    }

    pub fn construct_web_font_from_data(
        &self,
        data: &[u8],
        descriptors: CSSFontFaceDescriptors,
    ) -> Option<(LowercaseFontFamilyName, FontTemplate)> {
        let bytes = fontsan::process(data)
            .inspect_err(|error| {
                debug!(
                    "Sanitiser rejected FontFace font: family={} with {error:?}",
                    descriptors.family_name,
                );
            })
            .ok()?;
        let font_data = FontData::from_bytes(&bytes);

        let identifier = FontIdentifier::ArrayBuffer(Uuid::new_v4());
        let handle =
            PlatformFont::new_from_data(identifier.clone(), &font_data, None, &[], false).ok()?;

        let new_template = FontTemplate::new(identifier.clone(), handle.descriptor(), None);

        self.font_data.write().insert(identifier, font_data);

        Some((descriptors.family_name, new_template))
    }

    fn start_loading_one_web_font(
        self: &Arc<FontContext>,
        webview_id: Option<WebViewId>,
        source_list: &SourceList,
        css_font_face_descriptors: CSSFontFaceDescriptors,
        completion_handler: WebFontLoadInitiator,
        document_context: &WebFontDocumentContext,
    ) {
        let sources: Vec<Source> = source_list
            .0
            .iter()
            .rev()
            .filter(Self::is_supported_web_font_source)
            .filter(|source| {
                self.is_local_or_unknown_url_font(&css_font_face_descriptors.family_name, source)
            })
            .cloned()
            .collect();

        // Fetch all local fonts first, beacause if we try to fetch them later on during the process of
        // loading the list of web font `src`s we may be running in the context of the router thread, which
        // means we won't be able to seend IPC messages to the FontCacheThread.
        //
        // TODO: This is completely wrong. The specification says that `local()` font-family should match
        // against full PostScript names, but this is matching against font family names. This works...
        // sometimes.
        let mut local_fonts = HashMap::new();
        for source in sources.iter() {
            if let Source::Local(family_name) = source {
                local_fonts
                    .entry(family_name.name.clone())
                    .or_insert_with(|| {
                        let family = SingleFontFamily::FamilyName(FamilyName {
                            name: family_name.name.clone(),
                            syntax: FontFamilyNameSyntax::Quoted,
                        });
                        self.system_font_service_proxy
                            .find_matching_font_templates(None, &family)
                            .first()
                            .cloned()
                    });
            }
        }

        self.process_next_web_font_source(WebFontDownloadState::new(
            webview_id,
            self.clone(),
            css_font_face_descriptors,
            completion_handler,
            sources,
            local_fonts,
            document_context.clone(),
        ));
    }

    pub(crate) fn process_next_web_font_source(
        self: &Arc<FontContext>,
        mut state: WebFontDownloadState,
    ) {
        let Some(source) = state.remaining_sources.pop() else {
            state.handle_web_font_load_failure();
            return;
        };

        let this = self.clone();
        let web_font_family_name = state.css_font_face_descriptors.family_name.clone();
        match source {
            Source::Url(url_source) => {
                RemoteWebFontDownloader::download(url_source, this, web_font_family_name, state)
            },
            Source::Local(ref local_family_name) => {
                if let Some(new_template) = state
                    .local_fonts
                    .get(&local_family_name.name)
                    .cloned()
                    .flatten()
                    .and_then(|local_template| {
                        let template = FontTemplate::new_for_local_web_font(
                            local_template,
                            &state.css_font_face_descriptors,
                            state.initiator.font_face_rule().cloned(),
                        )
                        .ok()?;
                        Some(template)
                    })
                {
                    state.handle_web_font_load_success(new_template);
                } else {
                    this.process_next_web_font_source(state);
                }
            },
        }
    }
}

pub(crate) type ScriptWebFontLoadFinishedCallback =
    Box<dyn FnOnce(LowercaseFontFamilyName, Option<FontTemplate>) + Send>;

#[derive(MallocSizeOf)]
pub(crate) struct FontFaceRuleInitiator {
    font_face_rule: FontFaceRuleDescriptors,
    #[ignore_malloc_size_of = "dyn Fn"]
    callback: StylesheetWebFontLoadFinishedCallback,
}

#[derive(MallocSizeOf)]
pub(crate) enum WebFontLoadInitiator {
    Stylesheet(Box<FontFaceRuleInitiator>),
    Script(#[ignore_malloc_size_of = "dyn Fn"] ScriptWebFontLoadFinishedCallback),
}

impl WebFontLoadInitiator {
    pub(crate) fn font_face_rule(&self) -> Option<&FontFaceRuleDescriptors> {
        match self {
            Self::Stylesheet(initiator) => Some(&initiator.font_face_rule),
            Self::Script(_) => None,
        }
    }
}

struct RemoteWebFontDownloader {
    /// The URL of the font currently being loaded.
    url: ServoArc<Url>,
    web_font_family_name: LowercaseFontFamilyName,
    response_valid: bool,
    /// The data that has been received from the network thread so far.
    response_data: Vec<u8>,
    document_context: WebFontDocumentContext,
    font_context: Arc<FontContext>,
}

enum DownloaderResponseResult {
    InProcess,
    Finished,
    Failure,
}

impl RemoteWebFontDownloader {
    fn download(
        url_source: UrlSource,
        font_context: Arc<FontContext>,
        web_font_family_name: LowercaseFontFamilyName,
        state: WebFontDownloadState,
    ) {
        // https://drafts.csswg.org/css-fonts/#font-fetching-requirements
        let url = match url_source.url.url() {
            Some(url) => url.clone(),
            None => return,
        };

        let document_context = &state.document_context;

        let request = RequestBuilder::new(
            state.webview_id,
            UrlWithBlobClaim::from_url_without_having_claimed_blob(url.clone().into()),
            Referrer::ReferrerUrl(document_context.document_url.clone()),
        )
        .destination(Destination::Font)
        .mode(RequestMode::CorsMode)
        .credentials_mode(CredentialsMode::CredentialsSameOrigin)
        .service_workers_mode(ServiceWorkersMode::All)
        .policy_container(document_context.policy_container.clone())
        .client(document_context.request_client.clone());

        let core_resource_thread_clone = font_context.resource_threads.lock().clone();

        debug!("Loading @font-face {} from {}", web_font_family_name, url);
        let mut downloader = Self {
            url: url.clone(),
            web_font_family_name,
            response_valid: false,
            response_data: Vec::new(),
            document_context: document_context.clone(),
            font_context: font_context.clone(),
        };

        font_context.handle_web_font_request_started(url.into(), state);
        fetch_async(
            &core_resource_thread_clone,
            request,
            None,
            Box::new(move |response_message| {
                match downloader.handle_web_font_fetch_message(response_message) {
                    DownloaderResponseResult::InProcess => {},
                    DownloaderResponseResult::Finished => {
                        downloader.process_downloaded_font_and_signal_completion()
                    },
                    DownloaderResponseResult::Failure => {
                        font_context.handle_web_font_request_failed(downloader.url.clone().into());
                    },
                }
            }),
        )
    }

    /// After a download finishes, try to process the downloaded data, returning true if
    /// the font is added successfully to the [`FontContext`] or false if it isn't.
    fn process_downloaded_font_and_signal_completion(&mut self) {
        // Check if we still need this web font. If the stylesheet has been removed in the meantime
        // then there is no need to process it any further.
        if !self
            .font_context
            .has_pending_font_requests_for_url(self.url.clone())
        {
            return;
        }

        let font_data = std::mem::take(&mut self.response_data);
        trace!(
            "Downloaded @font-face {} ({} bytes)",
            self.web_font_family_name,
            font_data.len()
        );

        let font_data = match fontsan::process(&font_data) {
            Ok(bytes) => FontData::from_bytes(&bytes),
            Err(error) => {
                debug!(
                    "Sanitiser rejected web font url={:?} with {error:?}",
                    self.url.as_str(),
                );
                return self
                    .font_context
                    .handle_web_font_request_failed(self.url.clone().into());
            },
        };

        let url: ServoUrl = self.url.clone().into();
        self.font_context
            .handle_web_font_request_succeeded(font_data, url);
    }

    fn handle_web_font_fetch_message(
        &mut self,
        response_message: FetchResponseMsg,
    ) -> DownloaderResponseResult {
        match response_message {
            FetchResponseMsg::ProcessRequestBody(..) => DownloaderResponseResult::InProcess,
            FetchResponseMsg::ProcessCspViolations(_request_id, violations) => {
                self.document_context
                    .csp_handler
                    .process_violations(violations);
                DownloaderResponseResult::InProcess
            },
            FetchResponseMsg::ProcessResponse(_, meta_result) => {
                trace!(
                    "@font-face {} metadata ok={:?}",
                    self.web_font_family_name,
                    meta_result.is_ok()
                );
                self.response_valid = meta_result.is_ok();
                DownloaderResponseResult::InProcess
            },
            FetchResponseMsg::ProcessResponseChunk(_, new_bytes) => {
                trace!(
                    "@font-face {} chunk={:?}",
                    self.web_font_family_name, new_bytes
                );
                if self.response_valid {
                    self.response_data.extend(new_bytes.0)
                }
                DownloaderResponseResult::InProcess
            },
            FetchResponseMsg::ProcessResponseEOF(_, response, timing) => {
                trace!(
                    "@font-face {} EOF={:?}",
                    self.web_font_family_name, response
                );
                if response.is_err() || !self.response_valid {
                    return DownloaderResponseResult::Failure;
                }
                self.document_context
                    .network_timing_handler
                    .submit_timing(ServoUrl::from_url(self.url.as_ref().clone()), timing);
                DownloaderResponseResult::Finished
            },
            FetchResponseMsg::ProcessContentLength(_request_id, size) => {
                self.response_data.reserve(size - self.response_data.len());
                DownloaderResponseResult::InProcess
            },
        }
    }
}

#[derive(Debug, Eq, Hash, MallocSizeOf, PartialEq)]
struct FontCacheKey {
    font_identifier: FontIdentifier,
    font_descriptor: FontDescriptor,
}

#[derive(Debug, MallocSizeOf)]
struct FontGroupCacheKey {
    #[ignore_malloc_size_of = "This is also stored as part of styling."]
    style: ServoArc<FontStyleStruct>,
    size: Au,
}

impl PartialEq for FontGroupCacheKey {
    fn eq(&self, other: &FontGroupCacheKey) -> bool {
        self.style == other.style && self.size == other.size
    }
}

impl Eq for FontGroupCacheKey {}

impl Hash for FontGroupCacheKey {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.style.hash.hash(hasher)
    }
}

#[derive(Default, MallocSizeOf)]
struct KnownFontFaceRules {
    /// Used to distinguish new, incoming `@font-face` rules from existing ones.
    ///
    /// Generations alternate between true and false, which is enough to tell one generation apart from
    /// the next.
    generation: bool,
    /// Maps from a font family name to a list of `@font-face` rules declaring fonts
    /// that belong to said family.
    contents: HashMap<Atom, Vec<KnownFontFaceRule>>,
}

#[derive(MallocSizeOf)]
struct KnownFontFaceRule {
    rule_with_origin: FontFaceRuleWithOrigin,
    generation: bool,
}

impl KnownFontFaceRules {
    /// Computes the difference between the `@font-face `rules that are currently in effect
    /// and the ones that the `Stylist` knows about. The caller is notified about new or removed rules
    /// with callbacks.
    fn diff_old_and_new_font_face_rules(
        &mut self,
        stylist: &Stylist,
        guards: &StylesheetGuards<'_>,
    ) -> WebFontSetDifference {
        let mut difference = WebFontSetDifference::default();
        self.generation = !self.generation;

        let font_face_rules_in_cascade_order = stylist
            .iter_extra_data_origins()
            .flat_map(|(extra_data, origin)| {
                extra_data.font_faces.iter().rev().zip(iter::repeat(origin))
            })
            .map(|((rule, _layer), origin)| FontFaceRuleWithOrigin::new(rule.clone(), origin));

        // First, find any *new* font families that were not defined previously
        let mut number_of_unchanged_rules = 0;
        let number_of_previously_known_rules: usize = self
            .contents
            .values()
            .map(|fonts_from_family| fonts_from_family.len())
            .sum();
        for rule_with_origin in font_face_rules_in_cascade_order {
            let borrowed_rule = rule_with_origin.read_with(guards);

            let Some(font_family) = borrowed_rule.descriptors.font_family.as_ref() else {
                // Per https://github.com/w3c/csswg-drafts/issues/1133 an @font-face rule
                // is valid as far as the CSS parser is concerned even if it doesn’t have
                // a font-family or src declaration.
                // However, both are required for the rule to represent an actual font face.
                continue;
            };
            if borrowed_rule.descriptors.src.is_none() {
                // @font-face rules without a src don't constitute usable font faces.
                continue;
            }

            let known_font_faces_for_family =
                self.contents.entry(font_family.name.clone()).or_default();

            let mut conflicting_declaration_with_higher_priority_exists = false;
            let mut index_of_existing_entry_for_this_rule = None;
            for (index, known_font_face) in known_font_faces_for_family.iter().enumerate() {
                // See if this is a entry for this @font-face that existed prior to the current update
                if FontFaceRuleWithOrigin::ptr_eq(
                    &known_font_face.rule_with_origin,
                    &rule_with_origin,
                ) {
                    index_of_existing_entry_for_this_rule = Some(index);
                }

                // Check if there are existing declarations with higher priority that conflict
                if conflicting_declaration_with_higher_priority_exists {
                    // We already found one conflict, no need to search for more.
                    continue;
                }
                if known_font_face.generation != self.generation {
                    // This rule was not inserted yet during this update, so it was either removed or
                    // has lower priority than the one currently being inserted.
                    continue;
                }
                if font_face_rules_conflict(
                    &known_font_face
                        .rule_with_origin
                        .read_with(guards)
                        .descriptors,
                    &borrowed_rule.descriptors,
                ) {
                    conflicting_declaration_with_higher_priority_exists = true;
                }
            }

            if let Some(index_of_existing_entry_for_this_rule) =
                index_of_existing_entry_for_this_rule
            {
                // This @font-face rule was already present in the cascade prior to this update.
                // But if during this update we inserted a rule with higher priority that overrides this one
                // then we should not update its generation so it will be dropped at the end.
                if conflicting_declaration_with_higher_priority_exists {
                    let stale_rule =
                        known_font_faces_for_family.remove(index_of_existing_entry_for_this_rule);
                    difference
                        .removed_font_faces
                        .push(stale_rule.rule_with_origin);
                } else {
                    number_of_unchanged_rules += 1;
                    known_font_faces_for_family[index_of_existing_entry_for_this_rule].generation =
                        self.generation;
                }
            } else if conflicting_declaration_with_higher_priority_exists {
                // This (new) rule does not apply to the document because another rule with higher cascade priority
                // overrides it. We can simply ignore this declaration.
                continue;
            } else {
                // This is a new rule that does not conflict with anything that previously existed, so insert it.
                difference.added_font_faces.push(rule_with_origin.clone());
                known_font_faces_for_family.push(KnownFontFaceRule {
                    rule_with_origin,
                    generation: self.generation,
                });
            }
        }

        if number_of_unchanged_rules == number_of_previously_known_rules {
            // This is the common case, where the new set of known @font-face rules is a superset of
            // the old one after applying the cascade. In this case there is nothing more to do,
            // because all old @font-face rules are still present.
            return difference;
        }

        // Remove all `@font-face` rules that were not updated - those no longer exist on the stylist.
        self.contents.retain(|_, known_font_faces_for_family| {
            known_font_faces_for_family
                .extract_if(.., |rule| rule.generation != self.generation)
                .for_each(|removed_rule| {
                    difference
                        .removed_font_faces
                        .push(removed_rule.rule_with_origin);
                });

            !known_font_faces_for_family.is_empty()
        });

        difference
    }
}

/// Returns `true` if the two `@font-face` rules cannot both apply at the same time.
///
/// Two font faces can coexist if they are different for the purposes of font matching:
/// <https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm>
///
/// This method does assume that the family names have already been verified to be equal.
fn font_face_rules_conflict(
    first_rule: &FontFaceRuleDescriptors,
    second_rule: &FontFaceRuleDescriptors,
) -> bool {
    first_rule.font_stretch == second_rule.font_stretch &&
        first_rule.font_style == second_rule.font_style &&
        first_rule.font_weight == second_rule.font_weight &&
        first_rule.unicode_range == second_rule.unicode_range
}
