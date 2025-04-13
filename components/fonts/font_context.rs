/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use app_units::Au;
use base::id::WebViewId;
use compositing_traits::CrossProcessCompositorApi;
use fnv::FnvHasher;
use fonts_traits::StylesheetWebFontLoadFinishedCallback;
use log::{debug, trace};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::request::{Destination, Referrer, RequestBuilder};
use net_traits::{CoreResourceThread, FetchResponseMsg, ResourceThreads, fetch_async};
use parking_lot::{Mutex, RwLock};
use servo_arc::Arc as ServoArc;
use servo_url::ServoUrl;
use style::Atom;
use style::computed_values::font_variant_caps::T as FontVariantCaps;
use style::font_face::{
    FontFaceSourceFormat, FontFaceSourceFormatKeyword, Source, SourceList, UrlSource,
};
use style::media_queries::Device;
use style::properties::style_structs::Font as FontStyleStruct;
use style::shared_lock::SharedRwLockReadGuard;
use style::stylesheets::{CssRule, DocumentStyleSheet, FontFaceRule, StylesheetInDocument};
use style::values::computed::font::{FamilyName, FontFamilyNameSyntax, SingleFontFamily};
use url::Url;
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontKey};

use crate::font::{
    Font, FontDescriptor, FontFamilyDescriptor, FontGroup, FontRef, FontSearchScope,
};
use crate::font_store::CrossThreadFontStore;
use crate::font_template::{FontTemplate, FontTemplateRef, FontTemplateRefMethods};
use crate::platform::font::PlatformFont;
use crate::system_font_service::{CSSFontFaceDescriptors, FontIdentifier};
use crate::{FontData, LowercaseFontFamilyName, PlatformFontMethods, SystemFontServiceProxy};

static SMALL_CAPS_SCALE_FACTOR: f32 = 0.8; // Matches FireFox (see gfxFont.h)

/// The FontContext represents the per-thread/thread state necessary for
/// working with fonts. It is the public API used by the layout and
/// paint code. It talks directly to the system font service where
/// required.
pub struct FontContext {
    system_font_service_proxy: Arc<SystemFontServiceProxy>,
    resource_threads: Mutex<CoreResourceThread>,

    /// A sender that can send messages and receive replies from the compositor.
    compositor_api: Mutex<CrossProcessCompositorApi>,

    /// The actual instances of fonts ie a [`FontTemplate`] combined with a size and
    /// other font properties, along with the font data and a platform font instance.
    fonts: RwLock<HashMap<FontCacheKey, Option<FontRef>>>,

    /// A caching map between the specification of a font in CSS style and
    /// resolved [`FontGroup`] which contains information about all fonts that
    /// can be selected with that style.
    resolved_font_groups:
        RwLock<HashMap<FontGroupCacheKey, Arc<RwLock<FontGroup>>, BuildHasherDefault<FnvHasher>>>,

    web_fonts: CrossThreadFontStore,

    /// A collection of WebRender [`FontKey`]s generated for the web fonts that this
    /// [`FontContext`] controls.
    webrender_font_keys: RwLock<HashMap<FontIdentifier, FontKey>>,

    /// A collection of WebRender [`FontInstanceKey`]s generated for the web fonts that
    /// this [`FontContext`] controls.
    webrender_font_instance_keys: RwLock<HashMap<(FontKey, Au), FontInstanceKey>>,

    /// The data for each web font [`FontIdentifier`]. This data might be used by more than one
    /// [`FontTemplate`] as each identifier refers to a URL.
    font_data: RwLock<HashMap<FontIdentifier, FontData>>,

    have_removed_web_fonts: AtomicBool,
}

impl MallocSizeOf for FontContext {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let font_cache_size = self
            .fonts
            .read()
            .iter()
            .map(|(key, font)| {
                key.size_of(ops) + font.as_ref().map_or(0, |font| (*font).size_of(ops))
            })
            .sum::<usize>();
        let font_group_cache_size = self
            .resolved_font_groups
            .read()
            .iter()
            .map(|(key, font_group)| key.size_of(ops) + (*font_group.read()).size_of(ops))
            .sum::<usize>();
        font_cache_size + font_group_cache_size
    }
}

impl FontContext {
    pub fn new(
        system_font_service_proxy: Arc<SystemFontServiceProxy>,
        compositor_api: CrossProcessCompositorApi,
        resource_threads: ResourceThreads,
    ) -> Self {
        #[allow(clippy::default_constructed_unit_structs)]
        Self {
            system_font_service_proxy,
            resource_threads: Mutex::new(resource_threads.core_thread),
            compositor_api: Mutex::new(compositor_api),
            fonts: Default::default(),
            resolved_font_groups: Default::default(),
            web_fonts: Arc::new(RwLock::default()),
            webrender_font_keys: RwLock::default(),
            webrender_font_instance_keys: RwLock::default(),
            have_removed_web_fonts: AtomicBool::new(false),
            font_data: RwLock::default(),
        }
    }

    pub fn web_fonts_still_loading(&self) -> usize {
        self.web_fonts.read().number_of_fonts_still_loading()
    }

    fn get_font_data(&self, identifier: &FontIdentifier) -> Option<FontData> {
        match identifier {
            FontIdentifier::Web(_) => self.font_data.read().get(identifier).cloned(),
            FontIdentifier::Local(_) => None,
        }
    }

    /// Returns a `FontGroup` representing fonts which can be used for layout, given the `style`.
    /// Font groups are cached, so subsequent calls with the same `style` will return a reference
    /// to an existing `FontGroup`.
    pub fn font_group(&self, style: ServoArc<FontStyleStruct>) -> Arc<RwLock<FontGroup>> {
        let font_size = style.font_size.computed_size().into();
        self.font_group_with_size(style, font_size)
    }

    /// Like [`Self::font_group`], but overriding the size found in the [`FontStyleStruct`] with the given size
    /// in pixels.
    pub fn font_group_with_size(
        &self,
        style: ServoArc<FontStyleStruct>,
        size: Au,
    ) -> Arc<RwLock<FontGroup>> {
        let cache_key = FontGroupCacheKey { size, style };
        if let Some(font_group) = self.resolved_font_groups.read().get(&cache_key) {
            return font_group.clone();
        }

        let mut descriptor = FontDescriptor::from(&*cache_key.style);
        descriptor.pt_size = size;

        let font_group = Arc::new(RwLock::new(FontGroup::new(&cache_key.style, descriptor)));
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

        // TODO: Inserting `None` into the cache here is a bit bogus. Instead we should somehow
        // mark this template as invalid so it isn't tried again.
        let font = self
            .create_font(
                font_template,
                font_descriptor.to_owned(),
                synthesized_small_caps_font,
            )
            .ok();
        self.fonts.write().insert(cache_key, font.clone());
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn create_font(
        &self,
        font_template: FontTemplateRef,
        font_descriptor: FontDescriptor,
        synthesized_small_caps: Option<FontRef>,
    ) -> Result<FontRef, &'static str> {
        Ok(Arc::new(Font::new(
            font_template.clone(),
            font_descriptor.clone(),
            self.get_font_data(&font_template.identifier()),
            synthesized_small_caps,
        )?))
    }

    pub(crate) fn create_font_instance_key(&self, font: &Font) -> FontInstanceKey {
        match font.template.identifier() {
            FontIdentifier::Local(_) => self.system_font_service_proxy.get_system_font_instance(
                font.template.identifier(),
                font.descriptor.pt_size,
                font.webrender_font_instance_flags(),
            ),
            FontIdentifier::Web(_) => self.create_web_font_instance(
                font.template.clone(),
                font.descriptor.pt_size,
                font.webrender_font_instance_flags(),
            ),
        }
    }

    fn create_web_font_instance(
        &self,
        font_template: FontTemplateRef,
        pt_size: Au,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        let identifier = font_template.identifier().clone();
        let font_data = self
            .get_font_data(&identifier)
            .expect("Web font should have associated font data");
        let font_key = *self
            .webrender_font_keys
            .write()
            .entry(identifier.clone())
            .or_insert_with(|| {
                let font_key = self.system_font_service_proxy.generate_font_key();
                self.compositor_api.lock().add_font(
                    font_key,
                    font_data.as_ipc_shared_memory(),
                    identifier.index(),
                );
                font_key
            });

        let key = *self
            .webrender_font_instance_keys
            .write()
            .entry((font_key, pt_size))
            .or_insert_with(|| {
                let font_instance_key = self.system_font_service_proxy.generate_font_instance_key();
                self.compositor_api.lock().add_font_instance(
                    font_instance_key,
                    font_key,
                    pt_size.to_f32_px(),
                    flags,
                );
                font_instance_key
            });
        key
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
            return string == "truetype" ||
                string == "opentype" ||
                string == "woff" ||
                string == "woff2";
        }

        false
    }
}

pub(crate) struct WebFontDownloadState {
    webview_id: Option<WebViewId>,
    css_font_face_descriptors: CSSFontFaceDescriptors,
    remaining_sources: Vec<Source>,
    core_resource_thread: CoreResourceThread,
    local_fonts: HashMap<Atom, Option<FontTemplateRef>>,
    font_context: Arc<FontContext>,
    initiator: WebFontLoadInitiator,
}

impl WebFontDownloadState {
    fn new(
        webview_id: Option<WebViewId>,
        font_context: Arc<FontContext>,
        css_font_face_descriptors: CSSFontFaceDescriptors,
        initiator: WebFontLoadInitiator,
        sources: Vec<Source>,
        local_fonts: HashMap<Atom, Option<FontTemplateRef>>,
    ) -> WebFontDownloadState {
        match initiator {
            WebFontLoadInitiator::Stylesheet(ref stylesheet, _) => {
                font_context
                    .web_fonts
                    .write()
                    .handle_web_font_load_started_for_stylesheet(stylesheet);
            },
            WebFontLoadInitiator::Script(_) => {
                font_context
                    .web_fonts
                    .write()
                    .handle_web_font_load_started_for_script();
            },
        };
        let core_resource_thread = font_context.resource_threads.lock().clone();
        WebFontDownloadState {
            webview_id,
            css_font_face_descriptors,
            remaining_sources: sources,
            core_resource_thread,
            local_fonts,
            font_context,
            initiator,
        }
    }

    fn handle_web_font_load_success(self, new_template: FontTemplate) {
        let family_name = self.css_font_face_descriptors.family_name.clone();
        match self.initiator {
            WebFontLoadInitiator::Stylesheet(ref stylesheet, ref callback) => {
                let not_cancelled = self
                    .font_context
                    .web_fonts
                    .write()
                    .handle_web_font_loaded_for_stylesheet(stylesheet, family_name, new_template);
                self.font_context
                    .invalidate_font_groups_after_web_font_load();
                callback(not_cancelled);
            },
            WebFontLoadInitiator::Script(callback) => {
                self.font_context
                    .web_fonts
                    .write()
                    .handle_web_font_load_finished_for_script();
                callback(family_name, Some(new_template));
            },
        }
    }

    fn handle_web_font_load_failure(self) {
        let family_name = self.css_font_face_descriptors.family_name.clone();
        match self.initiator {
            WebFontLoadInitiator::Stylesheet(ref stylesheet, ref callback) => {
                self.font_context
                    .web_fonts
                    .write()
                    .handle_web_font_load_failed_for_stylesheet(stylesheet);
                callback(false);
            },
            WebFontLoadInitiator::Script(callback) => {
                self.font_context
                    .web_fonts
                    .write()
                    .handle_web_font_load_finished_for_script();
                callback(family_name, None);
            },
        }
    }

    fn font_load_cancelled(&self) -> bool {
        match self.initiator {
            WebFontLoadInitiator::Stylesheet(ref stylesheet, _) => self
                .font_context
                .web_fonts
                .read()
                .font_load_cancelled_for_stylesheet(stylesheet),
            WebFontLoadInitiator::Script(_) => false,
        }
    }
}

pub trait FontContextWebFontMethods {
    fn add_all_web_fonts_from_stylesheet(
        &self,
        webview_id: WebViewId,
        stylesheet: &DocumentStyleSheet,
        guard: &SharedRwLockReadGuard,
        device: &Device,
        finished_callback: StylesheetWebFontLoadFinishedCallback,
    ) -> usize;
    fn load_web_font_for_script(
        &self,
        webview_id: Option<WebViewId>,
        source_list: SourceList,
        descriptors: CSSFontFaceDescriptors,
        finished_callback: ScriptWebFontLoadFinishedCallback,
    );
    fn add_template_to_font_context(
        &self,
        family_name: LowercaseFontFamilyName,
        font_template: FontTemplate,
    );
    fn remove_all_web_fonts_from_stylesheet(&self, stylesheet: &DocumentStyleSheet);
    fn collect_unused_webrender_resources(&self, all: bool)
    -> (Vec<FontKey>, Vec<FontInstanceKey>);
}

impl FontContextWebFontMethods for Arc<FontContext> {
    fn add_all_web_fonts_from_stylesheet(
        &self,
        webview_id: WebViewId,
        stylesheet: &DocumentStyleSheet,
        guard: &SharedRwLockReadGuard,
        device: &Device,
        finished_callback: StylesheetWebFontLoadFinishedCallback,
    ) -> usize {
        let mut number_loading = 0;
        for rule in stylesheet.effective_rules(device, guard) {
            let CssRule::FontFace(ref lock) = *rule else {
                continue;
            };

            let rule: &FontFaceRule = lock.read_with(guard);
            let Some(font_face) = rule.font_face() else {
                continue;
            };

            let css_font_face_descriptors = rule.into();
            let completion_handler =
                WebFontLoadInitiator::Stylesheet(stylesheet.clone(), finished_callback.clone());

            number_loading += 1;
            self.start_loading_one_web_font(
                Some(webview_id),
                font_face.sources(),
                css_font_face_descriptors,
                completion_handler,
            );
        }

        number_loading
    }

    fn remove_all_web_fonts_from_stylesheet(&self, stylesheet: &DocumentStyleSheet) {
        let mut web_fonts = self.web_fonts.write();
        let mut fonts = self.fonts.write();
        let mut font_groups = self.resolved_font_groups.write();

        // Cancel any currently in-progress web font loads.
        web_fonts.handle_stylesheet_removed(stylesheet);

        let mut removed_any = false;
        for family in web_fonts.families.values_mut() {
            removed_any |= family.remove_templates_for_stylesheet(stylesheet);
        }
        if !removed_any {
            return;
        };

        fonts.retain(|_, font| match font {
            Some(font) => font.template.borrow().stylesheet.as_ref() != Some(stylesheet),
            _ => true,
        });

        // Removing this stylesheet modified the available fonts, so invalidate the cache
        // of resolved font groups.
        font_groups.clear();

        // Ensure that we clean up any WebRender resources on the next display list update.
        self.have_removed_web_fonts.store(true, Ordering::Relaxed);
    }

    fn collect_unused_webrender_resources(
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

        let mut removed_keys: HashSet<FontKey> = HashSet::new();
        webrender_font_keys.retain(|identifier, font_key| {
            if unused_identifiers.contains(identifier) {
                removed_keys.insert(*font_key);
                false
            } else {
                true
            }
        });

        let mut removed_instance_keys: HashSet<FontInstanceKey> = HashSet::new();
        webrender_font_instance_keys.retain(|(font_key, _), instance_key| {
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

    fn load_web_font_for_script(
        &self,
        webview_id: Option<WebViewId>,
        sources: SourceList,
        descriptors: CSSFontFaceDescriptors,
        finished_callback: ScriptWebFontLoadFinishedCallback,
    ) {
        let completion_handler = WebFontLoadInitiator::Script(finished_callback);
        self.start_loading_one_web_font(webview_id, &sources, descriptors, completion_handler);
    }

    fn add_template_to_font_context(
        &self,
        family_name: LowercaseFontFamilyName,
        new_template: FontTemplate,
    ) {
        self.web_fonts
            .write()
            .add_new_template(family_name, new_template);
        self.invalidate_font_groups_after_web_font_load();
    }
}

impl FontContext {
    fn start_loading_one_web_font(
        self: &Arc<FontContext>,
        webview_id: Option<WebViewId>,
        source_list: &SourceList,
        css_font_face_descriptors: CSSFontFaceDescriptors,
        completion_handler: WebFontLoadInitiator,
    ) {
        let sources: Vec<Source> = source_list
            .0
            .iter()
            .rev()
            .filter(Self::is_supported_web_font_source)
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
        ));
    }

    fn process_next_web_font_source(self: &Arc<FontContext>, mut state: WebFontDownloadState) {
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
                            local_template.clone(),
                            &state.css_font_face_descriptors,
                            state.initiator.stylesheet().cloned(),
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

pub type ScriptWebFontLoadFinishedCallback =
    Box<dyn FnOnce(LowercaseFontFamilyName, Option<FontTemplate>) + Send>;

pub(crate) enum WebFontLoadInitiator {
    Stylesheet(DocumentStyleSheet, StylesheetWebFontLoadFinishedCallback),
    Script(ScriptWebFontLoadFinishedCallback),
}

impl WebFontLoadInitiator {
    pub(crate) fn stylesheet(&self) -> Option<&DocumentStyleSheet> {
        match self {
            Self::Stylesheet(stylesheet, _) => Some(stylesheet),
            Self::Script(_) => None,
        }
    }
}

struct RemoteWebFontDownloader {
    state: Option<WebFontDownloadState>,
    url: ServoArc<Url>,
    web_font_family_name: LowercaseFontFamilyName,
    response_valid: bool,
    response_data: Vec<u8>,
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

        // FIXME: This shouldn't use NoReferrer, but the current documents url
        let request =
            RequestBuilder::new(state.webview_id, url.clone().into(), Referrer::NoReferrer)
                .destination(Destination::Font);

        let core_resource_thread_clone = state.core_resource_thread.clone();

        debug!("Loading @font-face {} from {}", web_font_family_name, url);
        let mut downloader = Self {
            url,
            web_font_family_name,
            response_valid: false,
            response_data: Vec::new(),
            state: Some(state),
        };

        fetch_async(
            &core_resource_thread_clone,
            request,
            None,
            Box::new(move |response_message| {
                match downloader.handle_web_font_fetch_message(response_message) {
                    DownloaderResponseResult::InProcess => {},
                    DownloaderResponseResult::Finished => {
                        if !downloader.process_downloaded_font_and_signal_completion() {
                            font_context.process_next_web_font_source(downloader.take_state())
                        }
                    },
                    DownloaderResponseResult::Failure => {
                        font_context.process_next_web_font_source(downloader.take_state())
                    },
                }
            }),
        )
    }

    fn take_state(&mut self) -> WebFontDownloadState {
        self.state
            .take()
            .expect("must be non-None until download either succeeds or fails")
    }

    /// After a download finishes, try to process the downloaded data, returning true if
    /// the font is added successfully to the [`FontContext`] or false if it isn't.
    fn process_downloaded_font_and_signal_completion(&mut self) -> bool {
        let state = self
            .state
            .as_ref()
            .expect("must be non-None until processing is completed");
        if state.font_load_cancelled() {
            self.take_state().handle_web_font_load_failure();
            // Returning true here prevents trying to load the next font on the source list.
            return true;
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
                    "Sanitiser rejected web font: family={} url={:?} with {error:?}",
                    self.web_font_family_name, self.url,
                );
                return false;
            },
        };

        let url: ServoUrl = self.url.clone().into();
        let identifier = FontIdentifier::Web(url.clone());
        let Ok(handle) = PlatformFont::new_from_data(identifier, &font_data, None) else {
            return false;
        };

        let state = self.take_state();
        let mut descriptor = handle.descriptor();
        descriptor
            .override_values_with_css_font_template_descriptors(&state.css_font_face_descriptors);

        let new_template = FontTemplate::new(
            FontIdentifier::Web(url),
            descriptor,
            state.initiator.stylesheet().cloned(),
        );

        state
            .font_context
            .font_data
            .write()
            .insert(new_template.identifier.clone(), font_data);

        state.handle_web_font_load_success(new_template);

        // If the load was canceled above, then we still want to return true from this function in
        // order to halt any attempt to load sources that come later on the source list.
        true
    }

    fn handle_web_font_fetch_message(
        &mut self,
        response_message: FetchResponseMsg,
    ) -> DownloaderResponseResult {
        match response_message {
            FetchResponseMsg::ProcessRequestBody(..) |
            FetchResponseMsg::ProcessRequestEOF(..) |
            FetchResponseMsg::ProcessCspViolations(..) => DownloaderResponseResult::InProcess,
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
                    self.response_data.extend(new_bytes)
                }
                DownloaderResponseResult::InProcess
            },
            FetchResponseMsg::ProcessResponseEOF(_, response) => {
                trace!(
                    "@font-face {} EOF={:?}",
                    self.web_font_family_name, response
                );
                if response.is_err() || !self.response_valid {
                    return DownloaderResponseResult::Failure;
                }
                DownloaderResponseResult::Finished
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
