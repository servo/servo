/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::{Arc, Weak};

use fonts_traits::{SystemFontServiceMessage, SystemFontServiceProxySender};
use hashlink::LruCache;
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::{Mutex, RwLock};
use servo_base::generic_channel::{self, GenericSender};
use servo_base::id::PainterId;
use style::values::computed::font::SingleFontFamily;
use webrender_api::units::Au;
use webrender_api::{
    FontInstanceFlags, FontInstanceKey, FontInstancePlatformOptions, FontKey, FontVariation,
};

use crate::font_context::FontCacheKey;
use crate::{FontDescriptor, FontIdentifier, FontRef, FontTemplateRef, SkrifaFont};

const SYSTEM_FONT_CACHE_CAPACITY: usize = 256;

#[derive(Debug, Eq, Hash, MallocSizeOf, PartialEq)]
struct FontTemplateCacheKey {
    font_descriptor: Option<FontDescriptor>,
    family_descriptor: SingleFontFamily,
}

#[derive(MallocSizeOf)]
enum SkrifaFontCacheEntry {
    /// A cache entry exists for a particular [`FontIdentifier`], but the `skrifa::Font`
    /// for it could not be created. This typically means the font data is corrupted.
    CouldNotCreate,
    /// A cache entry exists for a [`SkrifaFont`], which may or may not be upgradeable
    /// if there are any surviving references to it elsewhere in Servo.
    Font(Weak<SkrifaFont>),
}

impl SkrifaFontCacheEntry {
    /// Returns `Some(Some(...))` when this entry can be converted into a non-weak [`SkrifaFont`]
    /// reference, `Some(None)` when the font entry represents a font that could not be created
    /// properly, and `None` when the entry is for a dropped [`SkrifaFont`].
    fn get(&self) -> Option<Option<Arc<SkrifaFont>>> {
        match self {
            Self::CouldNotCreate => Some(None),
            Self::Font(weak) => weak.upgrade().map(Some),
        }
    }

    fn is_stale(&self) -> bool {
        matches!(self, Self::Font(weak_font) if weak_font.strong_count() == 0)
    }
}

impl From<Option<&Arc<SkrifaFont>>> for SkrifaFontCacheEntry {
    fn from(maybe_font: Option<&Arc<SkrifaFont>>) -> Self {
        match maybe_font {
            Some(font) => Self::Font(Arc::downgrade(font)),
            None => Self::CouldNotCreate,
        }
    }
}

/// The public interface to the [`SystemFontService`], used by per-Document
/// `FontContext` instances.
#[derive(MallocSizeOf)]
pub struct SystemFontServiceProxy {
    sender: GenericSender<SystemFontServiceMessage>,
    templates: RwLock<HashMap<FontTemplateCacheKey, Vec<FontTemplateRef>>>,
    system_font_cache: Mutex<LruCache<FontCacheKey, Option<FontRef>>>,
    skrifa_fonts: RwLock<HashMap<FontIdentifier, SkrifaFontCacheEntry>>,
}

impl SystemFontServiceProxy {
    pub fn new(sender: &SystemFontServiceProxySender) -> Arc<Self> {
        Self {
            sender: sender.0.clone(),
            templates: Default::default(),
            system_font_cache: Mutex::new(LruCache::new(SYSTEM_FONT_CACHE_CAPACITY)),
            skrifa_fonts: Default::default(),
        }
        .into()
    }

    pub fn exit(&self) {
        let (response_chan, response_port) = generic_channel::channel().unwrap();
        self.sender
            .send(SystemFontServiceMessage::Exit(response_chan))
            .expect("Couldn't send SystemFontService exit message");
        response_port
            .recv()
            .expect("Couldn't receive SystemFontService reply");
    }

    pub fn to_sender(&self) -> SystemFontServiceProxySender {
        SystemFontServiceProxySender(self.sender.clone())
    }

    pub fn get_system_font_instance(
        &self,
        identifier: FontIdentifier,
        size: Au,
        flags: FontInstanceFlags,
        options: FontInstancePlatformOptions,
        variations: Vec<FontVariation>,
        painter_id: PainterId,
    ) -> FontInstanceKey {
        let (response_chan, response_port) =
            generic_channel::channel().expect("failed to create IPC channel");
        self.sender
            .send(SystemFontServiceMessage::GetFontInstance(
                painter_id,
                identifier,
                size,
                flags,
                options,
                variations,
                response_chan,
            ))
            .expect("failed to send message to system font service");

        let instance_key = response_port.recv();
        if instance_key.is_err() {
            let font_thread_has_closed = self.sender.send(SystemFontServiceMessage::Ping).is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("SystemFontService has already exited.");
        }
        instance_key.unwrap()
    }

    pub fn find_matching_font_templates(
        &self,
        descriptor_to_match: Option<&FontDescriptor>,
        family_descriptor: &SingleFontFamily,
    ) -> Vec<FontTemplateRef> {
        let cache_key = FontTemplateCacheKey {
            font_descriptor: descriptor_to_match.cloned(),
            family_descriptor: family_descriptor.clone(),
        };
        if let Some(templates) = self.templates.read().get(&cache_key).cloned() {
            return templates;
        }

        debug!(
            "SystemFontServiceProxy: cache miss for template_descriptor={:?} family_descriptor={:?}",
            descriptor_to_match, family_descriptor
        );

        let (response_chan, response_port) =
            generic_channel::channel().expect("failed to create IPC channel");
        self.sender
            .send(SystemFontServiceMessage::GetFontTemplates(
                descriptor_to_match.cloned(),
                family_descriptor.clone(),
                response_chan,
            ))
            .expect("failed to send message to system font service");

        let Ok(templates) = response_port.recv() else {
            let font_thread_has_closed = self.sender.send(SystemFontServiceMessage::Ping).is_err();
            assert!(
                font_thread_has_closed,
                "Failed to receive a response from live font cache"
            );
            panic!("SystemFontService has already exited.");
        };

        let templates: Vec<_> = templates.into_iter().map(FontTemplateRef::new).collect();
        self.templates.write().insert(cache_key, templates.clone());

        templates
    }

    pub fn generate_font_key(&self, painter_id: PainterId) -> FontKey {
        let (result_sender, result_receiver) =
            generic_channel::channel().expect("failed to create IPC channel");
        self.sender
            .send(SystemFontServiceMessage::GetFontKey(
                painter_id,
                result_sender,
            ))
            .expect("failed to send message to system font service");
        result_receiver
            .recv()
            .expect("Failed to communicate with system font service.")
    }

    pub fn generate_font_instance_key(&self, painter_id: PainterId) -> FontInstanceKey {
        let (result_sender, result_receiver) =
            generic_channel::channel().expect("failed to create IPC channel");
        self.sender
            .send(SystemFontServiceMessage::GetFontInstanceKey(
                painter_id,
                result_sender,
            ))
            .expect("failed to send message to system font service");
        result_receiver
            .recv()
            .expect("Failed to communicate with system font service.")
    }

    pub fn prefetch_font_keys_for_painter(&self, painter_id: PainterId) {
        let _ = self
            .sender
            .send(SystemFontServiceMessage::PrefetchFontKeys(painter_id));
    }

    pub fn skrifa_font(
        &self,
        font_identifier: &FontIdentifier,
        create_function: impl FnOnce() -> Option<Arc<SkrifaFont>>,
    ) -> Option<Arc<SkrifaFont>> {
        if let Some(font) = self
            .skrifa_fonts
            .read()
            .get(font_identifier)
            .and_then(SkrifaFontCacheEntry::get)
        {
            return font;
        }

        let mut cache = self.skrifa_fonts.write();
        if let Some(font) = cache
            .get(font_identifier)
            .and_then(SkrifaFontCacheEntry::get)
        {
            return font;
        }

        let font = create_function();
        cache.retain(|_, entry| !entry.is_stale());
        cache.insert(font_identifier.clone(), font.as_ref().into());
        font
    }

    pub(crate) fn system_font_cache(&self) -> &Mutex<LruCache<FontCacheKey, Option<FontRef>>> {
        &self.system_font_cache
    }
}
