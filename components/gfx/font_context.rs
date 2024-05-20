/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::default::Default;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::sync::Arc;

use app_units::Au;
use fnv::FnvHasher;
use ipc_channel::ipc::{self, IpcSender};
use log::{debug, trace};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::request::{Destination, Referrer, RequestBuilder};
use net_traits::{fetch_async, CoreResourceThread, FetchResponseMsg, ResourceThreads};
use parking_lot::{Mutex, ReentrantMutex, RwLock};
use servo_arc::Arc as ServoArc;
use style::computed_values::font_variant_caps::T as FontVariantCaps;
use style::font_face::{FontFaceSourceFormat, FontFaceSourceFormatKeyword, Source, UrlSource};
use style::media_queries::Device;
use style::properties::style_structs::Font as FontStyleStruct;
use style::shared_lock::SharedRwLockReadGuard;
use style::stylesheets::{Stylesheet, StylesheetInDocument};
use style::Atom;
use url::Url;

use crate::font::{
    Font, FontDescriptor, FontFamilyDescriptor, FontFamilyName, FontGroup, FontRef, FontSearchScope,
};
use crate::font_cache_thread::{
    CSSFontFaceDescriptors, FontIdentifier, FontSource, LowercaseString,
};
use crate::font_store::{CrossThreadFontStore, CrossThreadWebRenderFontStore};
use crate::font_template::{FontTemplate, FontTemplateRef, FontTemplateRefMethods};
#[cfg(target_os = "macos")]
use crate::platform::core_text_font_cache::CoreTextFontCache;

static SMALL_CAPS_SCALE_FACTOR: f32 = 0.8; // Matches FireFox (see gfxFont.h)

/// The FontContext represents the per-thread/thread state necessary for
/// working with fonts. It is the public API used by the layout and
/// paint code. It talks directly to the font cache thread where
/// required.
pub struct FontContext<S: FontSource> {
    font_source: ReentrantMutex<S>,
    resource_threads: ReentrantMutex<CoreResourceThread>,
    cache: CachingFontSource<S>,
    web_fonts: CrossThreadFontStore,
    webrender_font_store: CrossThreadWebRenderFontStore,
}

impl<S: FontSource> MallocSizeOf for FontContext<S> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.cache.size_of(ops)
    }
}

impl<S: FontSource> FontContext<S> {
    pub fn new(font_source: S, resource_threads: ResourceThreads) -> FontContext<S> {
        #[allow(clippy::default_constructed_unit_structs)]
        FontContext {
            font_source: ReentrantMutex::new(font_source.clone()),
            resource_threads: ReentrantMutex::new(resource_threads.core_thread),
            cache: CachingFontSource::new(font_source),
            web_fonts: Arc::new(RwLock::default()),
            webrender_font_store: Arc::new(RwLock::default()),
        }
    }

    /// Invalidate all caches that this [`FontContext`] holds and any in-process platform-specific
    /// caches.
    pub fn invalidate_caches(&self) {
        #[cfg(target_os = "macos")]
        CoreTextFontCache::clear_core_text_font_cache();
        self.cache.invalidate()
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
        self.cache.font_group_with_size(style, size)
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

        if let Some(font) = self.cache.fonts.read().get(&cache_key).cloned() {
            return font;
        }

        debug!(
            "FontContext::font cache miss for font_template={:?} font_descriptor={:?}",
            font_template, font_descriptor
        );

        let font = self
            .create_font(
                font_template,
                font_descriptor.to_owned(),
                synthesized_small_caps_font,
            )
            .ok();
        self.cache.fonts.write().insert(cache_key, font.clone());
        font
    }

    pub fn matching_templates(
        &self,
        descriptor_to_match: &FontDescriptor,
        family_descriptor: &FontFamilyDescriptor,
    ) -> Vec<FontTemplateRef> {
        // First try to find an appropriate web font that matches this descriptor.
        if family_descriptor.scope == FontSearchScope::Any {
            let family_name = LowercaseString::from(&family_descriptor.name);
            if let Some(templates) = self
                .web_fonts
                .read()
                .families
                .get(&family_name)
                .map(|templates| templates.find_for_descriptor(Some(descriptor_to_match)))
            {
                return templates;
            }
        }

        // If not request a matching font from the system font cache.
        self.cache
            .matching_templates(descriptor_to_match, family_descriptor)
    }

    /// Create a `Font` for use in layout calculations, from a `FontTemplateData` returned by the
    /// cache thread and a `FontDescriptor` which contains the styling parameters.
    fn create_font(
        &self,
        font_template: FontTemplateRef,
        font_descriptor: FontDescriptor,
        synthesized_small_caps: Option<FontRef>,
    ) -> Result<FontRef, &'static str> {
        let mut font = Font::new(
            font_template.clone(),
            font_descriptor.clone(),
            synthesized_small_caps,
        )?;

        let font_source = self.font_source.lock();
        font.font_key = match font_template.identifier() {
            FontIdentifier::Local(_) => font_source.get_system_font_instance(
                font_template.identifier(),
                font_descriptor.pt_size,
                font.webrender_font_instance_flags(),
            ),
            FontIdentifier::Web(_) => self.webrender_font_store.write().get_font_instance(
                &*font_source,
                font_template.clone(),
                font_descriptor.pt_size,
                font.webrender_font_instance_flags(),
            ),
        };

        Ok(Arc::new(font))
    }
}

#[derive(Clone)]
pub struct WebFontDownloadState {
    css_font_face_descriptors: Arc<CSSFontFaceDescriptors>,
    remaining_sources: Vec<Source>,
    result_sender: IpcSender<()>,
    core_resource_thread: CoreResourceThread,
    local_fonts: Arc<HashMap<Atom, Option<FontTemplateRef>>>,
}

pub trait FontContextWebFontMethods {
    fn add_all_web_fonts_from_stylesheet(
        &self,
        stylesheet: &Stylesheet,
        guard: &SharedRwLockReadGuard,
        device: &Device,
        font_cache_sender: &IpcSender<()>,
        synchronous: bool,
    ) -> usize;
    fn process_next_web_font_source(&self, web_font_download_state: WebFontDownloadState);
}

impl<S: FontSource + Send + 'static> FontContextWebFontMethods for Arc<FontContext<S>> {
    fn add_all_web_fonts_from_stylesheet(
        &self,
        stylesheet: &Stylesheet,
        guard: &SharedRwLockReadGuard,
        device: &Device,
        font_cache_sender: &IpcSender<()>,
        synchronous: bool,
    ) -> usize {
        let (result_sender, receiver) = if synchronous {
            let (sender, receiver) = ipc::channel().unwrap();
            (Some(sender), Some(receiver))
        } else {
            (None, None)
        };

        let mut number_loading = 0;
        stylesheet.effective_font_face_rules(device, guard, |rule| {
            let font_face = match rule.font_face() {
                Some(font_face) => font_face,
                None => return,
            };

            let sources: Vec<Source> = font_face
                .sources()
                .0
                .iter()
                .rev()
                .filter(is_supported_web_font_source)
                .cloned()
                .collect();
            if sources.is_empty() {
                return;
            }

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
                            let family_name = FontFamilyName::Specific(family_name.name.clone());
                            self.font_source
                                .lock()
                                .find_matching_font_templates(None, &family_name)
                                .first()
                                .cloned()
                        });
                }
            }

            let result_sender = result_sender.as_ref().unwrap_or(font_cache_sender).clone();
            self.process_next_web_font_source(WebFontDownloadState {
                css_font_face_descriptors: Arc::new(rule.into()),
                remaining_sources: sources,
                result_sender,
                core_resource_thread: self.resource_threads.lock().clone(),
                local_fonts: Arc::new(local_fonts),
            });

            // Either increment the count of loading web fonts, or wait for a synchronous load.
            if let Some(ref receiver) = receiver {
                receiver.recv().unwrap();
            }
            number_loading += 1;
        });

        number_loading
    }

    fn process_next_web_font_source(&self, mut state: WebFontDownloadState) {
        let Some(source) = state.remaining_sources.pop() else {
            state.result_sender.send(()).unwrap();
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
                        FontTemplate::new_for_local_web_font(
                            local_template,
                            &state.css_font_face_descriptors,
                        )
                        .ok()
                    })
                {
                    this.web_fonts
                        .write()
                        .families
                        .entry(web_font_family_name.clone())
                        .or_default()
                        .add_template(new_template);
                    drop(state.result_sender.send(()));
                } else {
                    this.process_next_web_font_source(state);
                }
            },
        }
    }
}

struct RemoteWebFontDownloader<FCT: FontSource> {
    font_context: Arc<FontContext<FCT>>,
    url: ServoArc<Url>,
    web_font_family_name: LowercaseString,
    response_valid: Mutex<bool>,
    response_data: Mutex<Vec<u8>>,
}

enum DownloaderResponseResult {
    InProcess,
    Finished,
    Failure,
}

impl<FCT: FontSource + Send + 'static> RemoteWebFontDownloader<FCT> {
    fn download(
        url_source: UrlSource,
        font_context: Arc<FontContext<FCT>>,
        web_font_family_name: LowercaseString,
        state: WebFontDownloadState,
    ) {
        // https://drafts.csswg.org/css-fonts/#font-fetching-requirements
        let url = match url_source.url.url() {
            Some(url) => url.clone(),
            None => return,
        };

        // FIXME: This shouldn't use NoReferrer, but the current documents url
        let request = RequestBuilder::new(url.clone().into(), Referrer::NoReferrer)
            .destination(Destination::Font);

        debug!("Loading @font-face {} from {}", web_font_family_name, url);
        let downloader = Self {
            font_context,
            url,
            web_font_family_name,
            response_valid: Mutex::new(false),
            response_data: Mutex::default(),
        };

        let core_resource_thread_clone = state.core_resource_thread.clone();
        fetch_async(
            request,
            &core_resource_thread_clone,
            move |response_message| match downloader.handle_web_font_fetch_message(response_message)
            {
                DownloaderResponseResult::InProcess => {},
                DownloaderResponseResult::Finished => {
                    if !downloader.process_downloaded_font_and_signal_completion(&state) {
                        downloader
                            .font_context
                            .process_next_web_font_source(state.clone())
                    }
                },
                DownloaderResponseResult::Failure => downloader
                    .font_context
                    .process_next_web_font_source(state.clone()),
            },
        )
    }

    /// After a download finishes, try to process the downloaded data, returning true if
    /// the font is added successfully to the [`FontContext`] or false if it isn't.
    fn process_downloaded_font_and_signal_completion(&self, state: &WebFontDownloadState) -> bool {
        let font_data = std::mem::take(&mut *self.response_data.lock());
        trace!(
            "@font-face {} data={:?}",
            self.web_font_family_name,
            font_data
        );

        let font_data = match fontsan::process(&font_data) {
            Ok(bytes) => bytes,
            Err(error) => {
                debug!(
                    "Sanitiser rejected web font: family={} url={:?} with {error:?}",
                    self.web_font_family_name, self.url,
                );
                return false;
            },
        };

        let Ok(new_template) = FontTemplate::new_for_remote_web_font(
            self.url.clone().into(),
            Arc::new(font_data),
            &state.css_font_face_descriptors,
        ) else {
            return false;
        };

        let family_name = state.css_font_face_descriptors.family_name.clone();
        self.font_context
            .web_fonts
            .write()
            .families
            .entry(family_name.clone())
            .or_default()
            .add_template(new_template);

        // Signal the Document that we have finished trying to load this web font.
        drop(state.result_sender.send(()));
        true
    }

    fn handle_web_font_fetch_message(
        &self,
        response_message: FetchResponseMsg,
    ) -> DownloaderResponseResult {
        match response_message {
            FetchResponseMsg::ProcessRequestBody | FetchResponseMsg::ProcessRequestEOF => {
                DownloaderResponseResult::InProcess
            },
            FetchResponseMsg::ProcessResponse(meta_result) => {
                trace!(
                    "@font-face {} metadata ok={:?}",
                    self.web_font_family_name,
                    meta_result.is_ok()
                );
                *self.response_valid.lock() = meta_result.is_ok();
                DownloaderResponseResult::InProcess
            },
            FetchResponseMsg::ProcessResponseChunk(new_bytes) => {
                trace!(
                    "@font-face {} chunk={:?}",
                    self.web_font_family_name,
                    new_bytes
                );
                if *self.response_valid.lock() {
                    self.response_data.lock().extend(new_bytes)
                }
                DownloaderResponseResult::InProcess
            },
            FetchResponseMsg::ProcessResponseEOF(response) => {
                trace!(
                    "@font-face {} EOF={:?}",
                    self.web_font_family_name,
                    response
                );
                if response.is_err() || !*self.response_valid.lock() {
                    return DownloaderResponseResult::Failure;
                }
                DownloaderResponseResult::Finished
            },
        }
    }
}

#[derive(Default)]
pub struct CachingFontSource<FCT: FontSource> {
    font_cache_thread: ReentrantMutex<FCT>,
    fonts: RwLock<HashMap<FontCacheKey, Option<FontRef>>>,
    templates: RwLock<HashMap<FontTemplateCacheKey, Vec<FontTemplateRef>>>,
    resolved_font_groups:
        RwLock<HashMap<FontGroupCacheKey, Arc<RwLock<FontGroup>>, BuildHasherDefault<FnvHasher>>>,
}

impl<FCT: FontSource> CachingFontSource<FCT> {
    fn new(font_cache_thread: FCT) -> Self {
        Self {
            font_cache_thread: ReentrantMutex::new(font_cache_thread),
            fonts: Default::default(),
            templates: Default::default(),
            resolved_font_groups: Default::default(),
        }
    }

    fn invalidate(&self) {
        self.fonts.write().clear();
        self.templates.write().clear();
        self.resolved_font_groups.write().clear();
    }

    pub fn matching_templates(
        &self,
        descriptor_to_match: &FontDescriptor,
        family_descriptor: &FontFamilyDescriptor,
    ) -> Vec<FontTemplateRef> {
        let cache_key = FontTemplateCacheKey {
            font_descriptor: descriptor_to_match.clone(),
            family_descriptor: family_descriptor.clone(),
        };
        if let Some(templates) = self.templates.read().get(&cache_key).cloned() {
            return templates;
        }

        debug!(
            "CachingFontSource: cache miss for template_descriptor={:?} family_descriptor={:?}",
            descriptor_to_match, family_descriptor
        );
        let templates = self
            .font_cache_thread
            .lock()
            .find_matching_font_templates(Some(descriptor_to_match), &family_descriptor.name);
        self.templates.write().insert(cache_key, templates.clone());

        templates
    }

    pub fn font_group_with_size(
        &self,
        style: ServoArc<FontStyleStruct>,
        size: Au,
    ) -> Arc<RwLock<FontGroup>> {
        let cache_key = FontGroupCacheKey { size, style };
        if let Some(font_group) = self.resolved_font_groups.read().get(&cache_key) {
            return font_group.clone();
        }
        let font_group = Arc::new(RwLock::new(FontGroup::new(&cache_key.style)));
        self.resolved_font_groups
            .write()
            .insert(cache_key, font_group.clone());
        font_group
    }
}

impl<FCT: FontSource> MallocSizeOf for CachingFontSource<FCT> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let font_cache_size = self
            .fonts
            .read()
            .iter()
            .map(|(key, font)| {
                key.size_of(ops) + font.as_ref().map_or(0, |font| (*font).size_of(ops))
            })
            .sum::<usize>();
        let font_template_cache_size = self
            .templates
            .read()
            .iter()
            .map(|(key, templates)| {
                let templates_size = templates
                    .iter()
                    .map(|template| template.borrow().size_of(ops))
                    .sum::<usize>();
                key.size_of(ops) + templates_size
            })
            .sum::<usize>();
        let font_group_cache_size = self
            .resolved_font_groups
            .read()
            .iter()
            .map(|(key, font_group)| key.size_of(ops) + (*font_group.read()).size_of(ops))
            .sum::<usize>();

        font_cache_size + font_template_cache_size + font_group_cache_size
    }
}

#[derive(Debug, Eq, Hash, MallocSizeOf, PartialEq)]
struct FontCacheKey {
    font_identifier: FontIdentifier,
    font_descriptor: FontDescriptor,
}

#[derive(Debug, Eq, Hash, MallocSizeOf, PartialEq)]
struct FontTemplateCacheKey {
    font_descriptor: FontDescriptor,
    family_descriptor: FontFamilyDescriptor,
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

fn is_supported_web_font_source(source: &&Source) -> bool {
    let url_source = match source {
        Source::Url(ref url_source) => url_source,
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
