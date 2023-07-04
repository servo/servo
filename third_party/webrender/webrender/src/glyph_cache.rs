/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::glyph_rasterizer::{FontInstance, GlyphFormat, GlyphKey, GlyphRasterizer};
use crate::internal_types::FastHashMap;
use crate::render_backend::{FrameId, FrameStamp};
use crate::render_task_cache::RenderTaskCache;
use crate::resource_cache::ResourceClassCache;
use std::sync::Arc;
use crate::texture_cache::{EvictionNotice, TextureCache};
use crate::texture_cache::TextureCacheHandle;

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Clone, Debug)]
pub struct CachedGlyphInfo {
    pub format: GlyphFormat,
    pub texture_cache_handle: TextureCacheHandle,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum GlyphCacheEntry {
    // A glyph that has been successfully rasterized.
    Cached(CachedGlyphInfo),
    // A glyph that should not be rasterized (i.e. a space).
    Blank,
    // A glyph that has been submitted to the font backend for rasterization,
    // but is still pending a result.
    #[allow(dead_code)]
    Pending,
}

impl GlyphCacheEntry {
    fn get_allocated_size(&self, texture_cache: &TextureCache, _: &RenderTaskCache)
                          -> Option<usize> {
        match *self {
            GlyphCacheEntry::Cached(ref glyph) => {
                texture_cache.get_allocated_size(&glyph.texture_cache_handle)
            }
            GlyphCacheEntry::Pending | GlyphCacheEntry::Blank => Some(0),
        }
    }

    fn is_recently_used(&self, texture_cache: &mut TextureCache) -> bool {
        if let GlyphCacheEntry::Cached(ref glyph) = *self {
            texture_cache.is_recently_used(&glyph.texture_cache_handle, 1)
        } else {
            false
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Clone)]
pub enum CachedGlyphData {
    Memory(Arc<Vec<u8>>),
    Gpu,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Default)]
pub struct GlyphKeyCacheInfo {
    eviction_notice: EvictionNotice,
    last_frame_used: FrameId,
    bytes_used: usize,
}

pub type GlyphKeyCache = ResourceClassCache<GlyphKey, GlyphCacheEntry, GlyphKeyCacheInfo>;

impl GlyphKeyCache {
    const DIRTY: usize = !0;

    pub fn eviction_notice(&self) -> &EvictionNotice {
        &self.user_data.eviction_notice
    }

    fn is_recently_used(&self, current_frame: FrameId) -> bool {
        self.user_data.last_frame_used + 1 >= current_frame
    }

    fn clear_glyphs(&mut self) -> usize {
        let pruned = self.user_data.bytes_used;
        self.clear();
        self.user_data.bytes_used = 0;
        pruned
    }

    fn prune_glyphs(
        &mut self,
        skip_recent: bool,
        excess_bytes_used: usize,
        texture_cache: &mut TextureCache,
        render_task_cache: &RenderTaskCache,
    ) -> usize {
        let mut pruned = 0;
        self.retain(|_, entry| {
            if pruned <= excess_bytes_used &&
               (!skip_recent || !entry.is_recently_used(texture_cache)) {
                match entry.get_allocated_size(texture_cache, render_task_cache) {
                    Some(size) => {
                        pruned += size;
                        false
                    }
                    None => true,
                }
            } else {
                true
            }
        });
        self.user_data.bytes_used -= pruned;
        pruned
    }

    pub fn add_glyph(&mut self, key: GlyphKey, value: GlyphCacheEntry) {
        self.insert(key, value);
        self.user_data.bytes_used = Self::DIRTY;
    }

    fn clear_evicted(
        &mut self,
        texture_cache: &TextureCache,
        render_task_cache: &RenderTaskCache,
    ) {
        if self.eviction_notice().check() || self.user_data.bytes_used == Self::DIRTY {
            // If there are evictions, filter out any glyphs evicted from the
            // texture cache from the glyph key cache.
            let mut usage = 0;
            self.retain(|_, entry| {
                let size = entry.get_allocated_size(texture_cache, render_task_cache);
                usage += size.unwrap_or(0);
                size.is_some()
            });
            self.user_data.bytes_used = usage;
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GlyphCache {
    glyph_key_caches: FastHashMap<FontInstance, GlyphKeyCache>,
    current_frame: FrameId,
    bytes_used: usize,
    max_bytes_used: usize,
}

impl GlyphCache {
    /// The default space usage threshold, in bytes, after which to start pruning away old fonts.
    pub const DEFAULT_MAX_BYTES_USED: usize = 6 * 1024 * 1024;

    pub fn new(max_bytes_used: usize) -> Self {
        GlyphCache {
            glyph_key_caches: FastHashMap::default(),
            current_frame: Default::default(),
            bytes_used: 0,
            max_bytes_used,
        }
    }

    pub fn get_glyph_key_cache_for_font_mut(&mut self, font: FontInstance) -> &mut GlyphKeyCache {
        let cache = self.glyph_key_caches
                        .entry(font)
                        .or_insert_with(GlyphKeyCache::new);
        cache.user_data.last_frame_used = self.current_frame;
        cache
    }

    pub fn get_glyph_key_cache_for_font(&self, font: &FontInstance) -> &GlyphKeyCache {
        self.glyph_key_caches
            .get(font)
            .expect("BUG: Unable to find glyph key cache!")
    }

    pub fn clear(&mut self) {
        for (_, glyph_key_cache) in &mut self.glyph_key_caches {
            glyph_key_cache.clear()
        }
        // We use this in on_memory_pressure where retaining memory allocations
        // isn't desirable, so we completely remove the hash map instead of clearing it.
        self.glyph_key_caches = FastHashMap::default();
    }

    pub fn clear_fonts<F>(&mut self, key_fun: F)
    where
        for<'r> F: Fn(&'r &FontInstance) -> bool,
    {
        self.glyph_key_caches.retain(|k, cache| {
            let should_clear = key_fun(&k);
            if !should_clear {
                return true;
            }

            cache.clear_glyphs();
            false
        })
    }

    /// Clear out evicted entries from glyph key caches.
    fn clear_evicted(
        &mut self,
        texture_cache: &TextureCache,
        render_task_cache: &RenderTaskCache,
    ) {
        let mut usage = 0;
        for cache in self.glyph_key_caches.values_mut() {
            // Scan for any glyph key caches that have evictions.
            cache.clear_evicted(texture_cache, render_task_cache);
            usage += cache.user_data.bytes_used;
        }
        self.bytes_used = usage;
    }

    /// If possible, remove entirely any empty glyph key caches.
    fn clear_empty_caches(&mut self, glyph_rasterizer: &mut GlyphRasterizer) {
        self.glyph_key_caches.retain(|key, cache| {
            // Discard the glyph key cache if it has no valid glyphs.
            if cache.is_empty() {
                glyph_rasterizer.delete_font_instance(key);
                false
            } else {
                true
            }
        });
    }

    /// Check the total space usage of the glyph cache. If it exceeds the maximum usage threshold,
    /// then start clearing the oldest glyphs until below the threshold.
    fn prune_excess_usage(
        &mut self,
        texture_cache: &mut TextureCache,
        render_task_cache: &RenderTaskCache,
    ) {
        if self.bytes_used < self.max_bytes_used {
            return;
        }
        // Usage is above the threshold. Get a last-recently-used ordered list of caches to clear.
        let mut caches: Vec<_> = self.glyph_key_caches.values_mut().collect();
        caches.sort_unstable_by(|a, b| {
            a.user_data.last_frame_used.cmp(&b.user_data.last_frame_used)
        });
        // Clear out the oldest caches until below the threshold.
        for cache in caches {
            if self.bytes_used < self.max_bytes_used {
                break;
            }
            let recent = cache.is_recently_used(self.current_frame);
            let excess = self.bytes_used - self.max_bytes_used;
            if !recent && excess >= cache.user_data.bytes_used {
                // If the excess is greater than the cache's size, just clear the whole thing.
                self.bytes_used -= cache.clear_glyphs();
            } else {
                // Otherwise, just clear as little of the cache as needed to remove the excess
                // and avoid rematerialization costs.
                self.bytes_used -= cache.prune_glyphs(
                    recent,
                    excess,
                    texture_cache,
                    render_task_cache,
                );
            }
        }
    }

    pub fn begin_frame(
        &mut self,
        stamp: FrameStamp,
        texture_cache: &mut TextureCache,
        render_task_cache: &RenderTaskCache,
        glyph_rasterizer: &mut GlyphRasterizer,
    ) {
        profile_scope!("begin_frame");
        self.current_frame = stamp.frame_id();
        self.clear_evicted(texture_cache, render_task_cache);
        self.prune_excess_usage(texture_cache, render_task_cache);
        // Clearing evicted glyphs and pruning excess usage might have produced empty caches,
        // so get rid of them if possible.
        self.clear_empty_caches(glyph_rasterizer);
    }
}
