/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{FontInstanceFlags, FontSize, BaseFontInstance};
use api::{FontKey, FontRenderMode, FontTemplate};
use api::{ColorU, GlyphIndex, GlyphDimensions, SyntheticItalics};
use api::channel::{unbounded_channel, Receiver, Sender};
use api::units::*;
use api::{ImageDescriptor, ImageDescriptorFlags, ImageFormat, DirtyRect};
use crate::internal_types::ResourceCacheError;
use crate::platform::font::FontContext;
use crate::device::TextureFilter;
use crate::gpu_types::UvRectKind;
use crate::glyph_cache::{GlyphCache, CachedGlyphInfo, GlyphCacheEntry};
use crate::internal_types::FastHashMap;
use crate::resource_cache::CachedImageData;
use crate::texture_cache::{TextureCache, TextureCacheHandle, Eviction, TargetShader};
use crate::gpu_cache::GpuCache;
use crate::profiler::{self, TransactionProfile};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use rayon::ThreadPool;
use rayon::prelude::*;
use euclid::approxeq::ApproxEq;
use euclid::size2;
use smallvec::SmallVec;
use std::cmp;
use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};

pub static GLYPH_FLASHING: AtomicBool = AtomicBool::new(false);

impl FontContexts {
    /// Get access to the font context associated to the current thread.
    pub fn lock_current_context(&self) -> MutexGuard<FontContext> {
        let id = self.current_worker_id();
        self.lock_context(id)
    }

    pub(in super) fn current_worker_id(&self) -> Option<usize> {
        self.workers.current_thread_index()
    }
}

thread_local! {
    pub static SEED: Cell<u32> = Cell::new(0);
}

// super simple random to avoid dependency on rand
fn random() -> u32 {
    SEED.with(|seed| {
        seed.set(seed.get().wrapping_mul(22695477).wrapping_add(1));
        seed.get()
    })
}

impl GlyphRasterizer {
    pub fn request_glyphs(
        &mut self,
        glyph_cache: &mut GlyphCache,
        font: FontInstance,
        glyph_keys: &[GlyphKey],
        texture_cache: &mut TextureCache,
        gpu_cache: &mut GpuCache,
    ) {
        assert!(
            self.font_contexts
                .lock_shared_context()
                .has_font(&font.font_key)
        );

        let glyph_key_cache = glyph_cache.get_glyph_key_cache_for_font_mut(font.clone());

        // select glyphs that have not been requested yet.
        for key in glyph_keys {
            if let Some(entry) = glyph_key_cache.try_get(key) {
                match entry {
                    GlyphCacheEntry::Cached(ref glyph) => {
                        // Skip the glyph if it is already has a valid texture cache handle.
                        if !texture_cache.request(&glyph.texture_cache_handle, gpu_cache) {
                            continue;
                        }
                        // This case gets hit when we already rasterized the glyph, but the
                        // glyph has been evicted from the texture cache. Just force it to
                        // pending so it gets rematerialized.
                    }
                    // Otherwise, skip the entry if it is blank or pending.
                    GlyphCacheEntry::Blank | GlyphCacheEntry::Pending => continue,
                }
            }

            // Increment the total number of glyphs that are pending. This is used to determine
            // later whether to use worker threads for the remaining glyphs during resolve time.
            self.pending_glyph_count += 1;
            self.glyph_request_count += 1;

            // Find a batch container for the font instance for this glyph. Use get_mut to avoid
            // cloning the font instance, since this is the common path.
            match self.pending_glyph_requests.get_mut(&font) {
                Some(container) => {
                    container.push(*key);

                    // If the batch for this font instance is big enough, kick off an async
                    // job to start rasterizing these glyphs on other threads now.
                    if container.len() == 8 {
                        let glyphs = mem::replace(container, SmallVec::new());
                        self.flush_glyph_requests(
                            font.clone(),
                            glyphs,
                            true,
                        );
                    }
                }
                None => {
                    // If no batch exists for this font instance, add the glyph to a new one.
                    self.pending_glyph_requests.insert(
                        font.clone(),
                        smallvec![*key],
                    );
                }
            }

            glyph_key_cache.add_glyph(*key, GlyphCacheEntry::Pending);
        }
    }

    pub fn enable_multithreading(&mut self, enable: bool) {
        self.enable_multithreading = enable;
    }

    /// Internal method to flush a list of glyph requests to a set of worker threads,
    /// or process on this thread if there isn't much work to do (in which case the
    /// overhead of processing these on a thread is unlikely to be a performance win).
    fn flush_glyph_requests(
        &mut self,
        font: FontInstance,
        glyphs: SmallVec<[GlyphKey; 16]>,
        use_workers: bool,
    ) {
        let font_contexts = Arc::clone(&self.font_contexts);
        let glyph_tx = self.glyph_tx.clone();
        self.pending_glyph_jobs += 1;
        self.pending_glyph_count -= glyphs.len();

        let can_use_r8_format = self.can_use_r8_format;

        let process_glyph = move |key: &GlyphKey, font_contexts: &FontContexts, font: &FontInstance| -> GlyphRasterJob {
            profile_scope!("glyph-raster");
            let mut context = font_contexts.lock_current_context();
            let mut job = GlyphRasterJob {
                key: key.clone(),
                result: context.rasterize_glyph(&font, key),
            };

            if let Ok(ref mut glyph) = job.result {
                // Sanity check.
                let bpp = 4; // We always render glyphs in 32 bits RGBA format.
                assert_eq!(
                    glyph.bytes.len(),
                    bpp * (glyph.width * glyph.height) as usize
                );

                // a quick-and-dirty monochrome over
                fn over(dst: u8, src: u8) -> u8 {
                    let a = src as u32;
                    let a = 256 - a;
                    let dst = ((dst as u32 * a) >> 8) as u8;
                    src + dst
                }

                if GLYPH_FLASHING.load(Ordering::Relaxed) {
                    let color = (random() & 0xff) as u8;
                    for i in &mut glyph.bytes {
                        *i = over(*i, color);
                    }
                }

                assert_eq!((glyph.left.fract(), glyph.top.fract()), (0.0, 0.0));

                // Check if the glyph has a bitmap that needs to be downscaled.
                glyph.downscale_bitmap_if_required(&font);

                // Convert from BGRA8 to R8 if required. In the future we can make it the
                // backends' responsibility to output glyphs in the desired format,
                // potentially reducing the number of copies.
                if glyph.format.image_format(can_use_r8_format).bytes_per_pixel() == 1 {
                    glyph.bytes = glyph.bytes
                        .chunks_mut(4)
                        .map(|pixel| pixel[3])
                        .collect::<Vec<_>>();
                }
            }

            job
        };

        // if the number of glyphs is small, do it inline to avoid the threading overhead;
        // send the result into glyph_tx so downstream code can't tell the difference.
        if self.enable_multithreading && use_workers {
            // spawn an async task to get off of the render backend thread as early as
            // possible and in that task use rayon's fork join dispatch to rasterize the
            // glyphs in the thread pool.
            profile_scope!("spawning process_glyph jobs");
            self.workers.spawn(move || {
                let jobs = glyphs
                    .par_iter()
                    .map(|key: &GlyphKey| process_glyph(key, &font_contexts, &font))
                    .collect();

                glyph_tx.send(GlyphRasterJobs { font, jobs }).unwrap();
            });
        } else {
            let jobs = glyphs.iter()
                             .map(|key: &GlyphKey| process_glyph(key, &font_contexts, &font))
                             .collect();
            glyph_tx.send(GlyphRasterJobs { font, jobs }).unwrap();
        }
    }

    pub fn resolve_glyphs(
        &mut self,
        glyph_cache: &mut GlyphCache,
        texture_cache: &mut TextureCache,
        gpu_cache: &mut GpuCache,
        profile: &mut TransactionProfile,
    ) {
        profile.start_time(profiler::GLYPH_RESOLVE_TIME);

        // Work around the borrow checker, since we call flush_glyph_requests below
        let mut pending_glyph_requests = mem::replace(
            &mut self.pending_glyph_requests,
            FastHashMap::default(),
        );
        // If we have a large amount of remaining work to do, spawn to worker threads,
        // even if that work is shared among a number of different font instances.
        let use_workers = self.pending_glyph_count >= 8;
        for (font, pending_glyphs) in pending_glyph_requests.drain() {
            self.flush_glyph_requests(
                font,
                pending_glyphs,
                use_workers,
            );
        }
        // Restore this so that we don't heap allocate next frame
        self.pending_glyph_requests = pending_glyph_requests;
        debug_assert_eq!(self.pending_glyph_count, 0);
        debug_assert!(self.pending_glyph_requests.is_empty());

        if self.glyph_request_count > 0 {
            profile.set(profiler::RASTERIZED_GLYPHS, self.glyph_request_count);
            self.glyph_request_count = 0;
        }

        profile_scope!("resolve_glyphs");
        // Pull rasterized glyphs from the queue and update the caches.
        while self.pending_glyph_jobs > 0 {
            self.pending_glyph_jobs -= 1;

            // TODO: rather than blocking until all pending glyphs are available
            // we could try_recv and steal work from the thread pool to take advantage
            // of the fact that this thread is alive and we avoid the added latency
            // of blocking it.

            let GlyphRasterJobs { font, mut jobs } = {
                profile_scope!("blocking wait on glyph_rx");
                self.glyph_rx
                .recv()
                .expect("BUG: Should be glyphs pending!")
            };

            // Ensure that the glyphs are always processed in the same
            // order for a given text run (since iterating a hash set doesn't
            // guarantee order). This can show up as very small float inaccuracy
            // differences in rasterizers due to the different coordinates
            // that text runs get associated with by the texture cache allocator.
            jobs.sort_by(|a, b| a.key.cmp(&b.key));

            let glyph_key_cache = glyph_cache.get_glyph_key_cache_for_font_mut(font);

            for GlyphRasterJob { key, result } in jobs {
                let glyph_info = match result {
                    Err(_) => GlyphCacheEntry::Blank,
                    Ok(ref glyph) if glyph.width == 0 || glyph.height == 0 => {
                        GlyphCacheEntry::Blank
                    }
                    Ok(glyph) => {
                        let mut texture_cache_handle = TextureCacheHandle::invalid();
                        texture_cache.request(&texture_cache_handle, gpu_cache);
                        texture_cache.update(
                            &mut texture_cache_handle,
                            ImageDescriptor {
                                size: size2(glyph.width, glyph.height),
                                stride: None,
                                format: glyph.format.image_format(self.can_use_r8_format),
                                flags: ImageDescriptorFlags::empty(),
                                offset: 0,
                            },
                            TextureFilter::Linear,
                            Some(CachedImageData::Raw(Arc::new(glyph.bytes))),
                            [glyph.left, -glyph.top, glyph.scale, 0.0],
                            DirtyRect::All,
                            gpu_cache,
                            Some(glyph_key_cache.eviction_notice()),
                            UvRectKind::Rect,
                            Eviction::Auto,
                            TargetShader::Text,
                        );
                        GlyphCacheEntry::Cached(CachedGlyphInfo {
                            texture_cache_handle,
                            format: glyph.format,
                        })
                    }
                };
                glyph_key_cache.insert(key, glyph_info);
            }
        }

        // Now that we are done with the critical path (rendering the glyphs),
        // we can schedule removing the fonts if needed.
        self.remove_dead_fonts();

        profile.end_time(profiler::GLYPH_RESOLVE_TIME);
    }
}

#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, PartialOrd)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct FontTransform {
    pub scale_x: f32,
    pub skew_x: f32,
    pub skew_y: f32,
    pub scale_y: f32,
}

// Floats don't impl Hash/Eq/Ord...
impl Eq for FontTransform {}
impl Ord for FontTransform {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap_or(cmp::Ordering::Equal)
    }
}
impl Hash for FontTransform {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Note: this is inconsistent with the Eq impl for -0.0 (don't care).
        self.scale_x.to_bits().hash(state);
        self.skew_x.to_bits().hash(state);
        self.skew_y.to_bits().hash(state);
        self.scale_y.to_bits().hash(state);
    }
}

impl FontTransform {
    const QUANTIZE_SCALE: f32 = 1024.0;

    pub fn new(scale_x: f32, skew_x: f32, skew_y: f32, scale_y: f32) -> Self {
        FontTransform { scale_x, skew_x, skew_y, scale_y }
    }

    pub fn identity() -> Self {
        FontTransform::new(1.0, 0.0, 0.0, 1.0)
    }

    #[allow(dead_code)]
    pub fn is_identity(&self) -> bool {
        *self == FontTransform::identity()
    }

    pub fn quantize(&self) -> Self {
        FontTransform::new(
            (self.scale_x * Self::QUANTIZE_SCALE).round() / Self::QUANTIZE_SCALE,
            (self.skew_x * Self::QUANTIZE_SCALE).round() / Self::QUANTIZE_SCALE,
            (self.skew_y * Self::QUANTIZE_SCALE).round() / Self::QUANTIZE_SCALE,
            (self.scale_y * Self::QUANTIZE_SCALE).round() / Self::QUANTIZE_SCALE,
        )
    }

    #[allow(dead_code)]
    pub fn determinant(&self) -> f64 {
        self.scale_x as f64 * self.scale_y as f64 - self.skew_y as f64 * self.skew_x as f64
    }

    #[allow(dead_code)]
    pub fn compute_scale(&self) -> Option<(f64, f64)> {
        let det = self.determinant();
        if det != 0.0 {
            let x_scale = (self.scale_x as f64).hypot(self.skew_y as f64);
            let y_scale = det.abs() / x_scale;
            Some((x_scale, y_scale))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn pre_scale(&self, scale_x: f32, scale_y: f32) -> Self {
        FontTransform::new(
            self.scale_x * scale_x,
            self.skew_x * scale_y,
            self.skew_y * scale_x,
            self.scale_y * scale_y,
        )
    }

    #[allow(dead_code)]
    pub fn scale(&self, scale: f32) -> Self { self.pre_scale(scale, scale) }

    #[allow(dead_code)]
    pub fn invert_scale(&self, x_scale: f64, y_scale: f64) -> Self {
        self.pre_scale(x_scale.recip() as f32, y_scale.recip() as f32)
    }

    pub fn synthesize_italics(&self, angle: SyntheticItalics, size: f64, vertical: bool) -> (Self, (f64, f64)) {
        let skew_factor = angle.to_skew();
        if vertical {
          // origin delta to be applied so that we effectively skew around
          // the middle rather than edge of the glyph
          let (tx, ty) = (0.0, -size * 0.5 * skew_factor as f64);
          (FontTransform::new(
              self.scale_x + self.skew_x * skew_factor,
              self.skew_x,
              self.skew_y + self.scale_y * skew_factor,
              self.scale_y,
          ), (self.scale_x as f64 * tx + self.skew_x as f64 * ty,
              self.skew_y as f64 * tx + self.scale_y as f64 * ty))
        } else {
          (FontTransform::new(
              self.scale_x,
              self.skew_x - self.scale_x * skew_factor,
              self.skew_y,
              self.scale_y - self.skew_y * skew_factor,
          ), (0.0, 0.0))
        }
    }

    pub fn swap_xy(&self) -> Self {
        FontTransform::new(self.skew_x, self.scale_x, self.scale_y, self.skew_y)
    }

    pub fn flip_x(&self) -> Self {
        FontTransform::new(-self.scale_x, self.skew_x, -self.skew_y, self.scale_y)
    }

    pub fn flip_y(&self) -> Self {
        FontTransform::new(self.scale_x, -self.skew_x, self.skew_y, -self.scale_y)
    }

    pub fn transform(&self, point: &LayoutPoint) -> DevicePoint {
        DevicePoint::new(
            self.scale_x * point.x + self.skew_x * point.y,
            self.skew_y * point.x + self.scale_y * point.y,
        )
    }

    pub fn get_subpx_dir(&self) -> SubpixelDirection {
        if self.skew_y.approx_eq(&0.0) {
            // The X axis is not projected onto the Y axis
            SubpixelDirection::Horizontal
        } else if self.scale_x.approx_eq(&0.0) {
            // The X axis has been swapped with the Y axis
            SubpixelDirection::Vertical
        } else {
            // Use subpixel precision on all axes
            SubpixelDirection::Mixed
        }
    }
}

impl<'a> From<&'a LayoutToWorldTransform> for FontTransform {
    fn from(xform: &'a LayoutToWorldTransform) -> Self {
        FontTransform::new(xform.m11, xform.m21, xform.m12, xform.m22)
    }
}

// Some platforms (i.e. Windows) may have trouble rasterizing glyphs above this size.
// Ensure glyph sizes are reasonably limited to avoid that scenario.
pub const FONT_SIZE_LIMIT: f32 = 320.0;

/// A mutable font instance description.
///
/// Performance is sensitive to the size of this structure, so it should only contain
/// the fields that we need to modify from the original base font instance.
#[derive(Clone, PartialEq, Eq, Debug, Ord, PartialOrd)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct FontInstance {
    pub base: Arc<BaseFontInstance>,
    pub transform: FontTransform,
    pub render_mode: FontRenderMode,
    pub flags: FontInstanceFlags,
    pub color: ColorU,
    // The font size is in *device/raster* pixels, not logical pixels.
    // It is stored as an f32 since we need sub-pixel sizes.
    pub size: FontSize,
}

impl Hash for FontInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash only the base instance's key to avoid the cost of hashing
        // the rest.
        self.base.instance_key.hash(state);
        self.transform.hash(state);
        self.render_mode.hash(state);
        self.flags.hash(state);
        self.color.hash(state);
        self.size.hash(state);
    }
}

impl Deref for FontInstance {
    type Target = BaseFontInstance;
    fn deref(&self) -> &BaseFontInstance {
        self.base.as_ref()
    }
}

impl MallocSizeOf for  FontInstance {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize { 0 }
}

impl FontInstance {
    pub fn new(
        base: Arc<BaseFontInstance>,
        color: ColorU,
        render_mode: FontRenderMode,
        flags: FontInstanceFlags,
    ) -> Self {
        FontInstance {
            transform: FontTransform::identity(),
            color,
            size: base.size,
            base,
            render_mode,
            flags,
        }
    }

    pub fn from_base(
        base: Arc<BaseFontInstance>,
    ) -> Self {
        FontInstance {
            transform: FontTransform::identity(),
            color: ColorU::new(0, 0, 0, 255),
            size: base.size,
            render_mode: base.render_mode,
            flags: base.flags,
            base,
        }
    }

    pub fn use_texture_padding(&self) -> bool {
        self.flags.contains(FontInstanceFlags::TEXTURE_PADDING)
    }

    pub fn use_transform_glyphs(&self) -> bool {
        self.flags.contains(FontInstanceFlags::TRANSFORM_GLYPHS)
    }

    pub fn get_alpha_glyph_format(&self) -> GlyphFormat {
        if self.use_transform_glyphs() { GlyphFormat::TransformedAlpha } else { GlyphFormat::Alpha }
    }

    pub fn get_subpixel_glyph_format(&self) -> GlyphFormat {
        if self.use_transform_glyphs() { GlyphFormat::TransformedSubpixel } else { GlyphFormat::Subpixel }
    }

    pub fn disable_subpixel_aa(&mut self) {
        self.render_mode = self.render_mode.limit_by(FontRenderMode::Alpha);
    }

    pub fn disable_subpixel_position(&mut self) {
        self.flags.remove(FontInstanceFlags::SUBPIXEL_POSITION);
    }

    pub fn use_subpixel_position(&self) -> bool {
        self.flags.contains(FontInstanceFlags::SUBPIXEL_POSITION) &&
        self.render_mode != FontRenderMode::Mono
    }

    pub fn get_subpx_dir(&self) -> SubpixelDirection {
        if self.use_subpixel_position() {
            let mut subpx_dir = self.transform.get_subpx_dir();
            if self.flags.contains(FontInstanceFlags::TRANSPOSE) {
                subpx_dir = subpx_dir.swap_xy();
            }
            subpx_dir
        } else {
            SubpixelDirection::None
        }
    }

    #[allow(dead_code)]
    pub fn get_subpx_offset(&self, glyph: &GlyphKey) -> (f64, f64) {
        if self.use_subpixel_position() {
            let (dx, dy) = glyph.subpixel_offset();
            (dx.into(), dy.into())
        } else {
            (0.0, 0.0)
        }
    }

    #[allow(dead_code)]
    pub fn get_glyph_format(&self) -> GlyphFormat {
        match self.render_mode {
            FontRenderMode::Mono | FontRenderMode::Alpha => self.get_alpha_glyph_format(),
            FontRenderMode::Subpixel => self.get_subpixel_glyph_format(),
        }
    }

    #[allow(dead_code)]
    pub fn get_extra_strikes(&self, x_scale: f64) -> usize {
        if self.flags.contains(FontInstanceFlags::SYNTHETIC_BOLD) {
            let mut bold_offset = self.size.to_f64_px() / 48.0;
            if bold_offset < 1.0 {
                bold_offset = 0.25 + 0.75 * bold_offset;
            }
            (bold_offset * x_scale).max(1.0).round() as usize
        } else {
            0
        }
    }

    pub fn synthesize_italics(&self, transform: FontTransform, size: f64) -> (FontTransform, (f64, f64)) {
        transform.synthesize_italics(self.synthetic_italics, size, self.flags.contains(FontInstanceFlags::VERTICAL))
    }

    #[allow(dead_code)]
    pub fn get_transformed_size(&self) -> f64 {
        let (_, y_scale) = self.transform.compute_scale().unwrap_or((1.0, 1.0));
        self.size.to_f64_px() * y_scale
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub enum SubpixelDirection {
    None = 0,
    Horizontal,
    Vertical,
    Mixed,
}

impl SubpixelDirection {
    // Limit the subpixel direction to what is supported by the glyph format.
    pub fn limit_by(self, glyph_format: GlyphFormat) -> Self {
        match glyph_format {
            GlyphFormat::Bitmap |
            GlyphFormat::ColorBitmap => SubpixelDirection::None,
            _ => self,
        }
    }

    pub fn swap_xy(self) -> Self {
        match self {
            SubpixelDirection::None | SubpixelDirection::Mixed => self,
            SubpixelDirection::Horizontal => SubpixelDirection::Vertical,
            SubpixelDirection::Vertical => SubpixelDirection::Horizontal,
        }
    }
}

#[repr(u8)]
#[derive(Hash, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum SubpixelOffset {
    Zero = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
}

impl SubpixelOffset {
    // Skia quantizes subpixel offsets into 1/4 increments.
    // Given the absolute position, return the quantized increment
    fn quantize(pos: f32) -> Self {
        // Following the conventions of Gecko and Skia, we want
        // to quantize the subpixel position, such that abs(pos) gives:
        // [0.0, 0.125) -> Zero
        // [0.125, 0.375) -> Quarter
        // [0.375, 0.625) -> Half
        // [0.625, 0.875) -> ThreeQuarters,
        // [0.875, 1.0) -> Zero
        // The unit tests below check for this.
        let apos = ((pos - pos.floor()) * 8.0) as i32;

        match apos {
            1..=2 => SubpixelOffset::Quarter,
            3..=4 => SubpixelOffset::Half,
            5..=6 => SubpixelOffset::ThreeQuarters,
            _ => SubpixelOffset::Zero,
        }
    }
}

impl Into<f64> for SubpixelOffset {
    fn into(self) -> f64 {
        match self {
            SubpixelOffset::Zero => 0.0,
            SubpixelOffset::Quarter => 0.25,
            SubpixelOffset::Half => 0.5,
            SubpixelOffset::ThreeQuarters => 0.75,
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Ord, PartialOrd)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GlyphKey(u32);

impl GlyphKey {
    pub fn new(
        index: u32,
        point: DevicePoint,
        subpx_dir: SubpixelDirection,
    ) -> Self {
        let (dx, dy) = match subpx_dir {
            SubpixelDirection::None => (0.0, 0.0),
            SubpixelDirection::Horizontal => (point.x, 0.0),
            SubpixelDirection::Vertical => (0.0, point.y),
            SubpixelDirection::Mixed => (point.x, point.y),
        };
        let sox = SubpixelOffset::quantize(dx);
        let soy = SubpixelOffset::quantize(dy);
        assert_eq!(0, index & 0xF0000000);

        GlyphKey(index | (sox as u32) << 28 | (soy as u32) << 30)
    }

    pub fn index(&self) -> GlyphIndex {
        self.0 & 0x0FFFFFFF
    }

    fn subpixel_offset(&self) -> (SubpixelOffset, SubpixelOffset) {
        let x = (self.0 >> 28) as u8 & 3;
        let y = (self.0 >> 30) as u8 & 3;
        unsafe {
            (mem::transmute(x), mem::transmute(y))
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[allow(dead_code)]
pub enum GlyphFormat {
    Alpha,
    TransformedAlpha,
    Subpixel,
    TransformedSubpixel,
    Bitmap,
    ColorBitmap,
}

impl GlyphFormat {
    /// Returns the ImageFormat that a glyph should be stored as in the texture cache.
    /// can_use_r8_format should be set false on platforms where we have encountered
    /// issues with R8 textures, so that we do not use them for glyphs.
    pub fn image_format(&self, can_use_r8_format: bool) -> ImageFormat {
        match *self {
            GlyphFormat::Alpha |
            GlyphFormat::TransformedAlpha |
            GlyphFormat::Bitmap => {
                if can_use_r8_format {
                    ImageFormat::R8
                } else {
                    ImageFormat::BGRA8
                }
            }
            GlyphFormat::Subpixel |
            GlyphFormat::TransformedSubpixel |
            GlyphFormat::ColorBitmap => ImageFormat::BGRA8,
        }
    }
}

pub struct RasterizedGlyph {
    pub top: f32,
    pub left: f32,
    pub width: i32,
    pub height: i32,
    pub scale: f32,
    pub format: GlyphFormat,
    pub bytes: Vec<u8>,
}

impl RasterizedGlyph {
    #[allow(dead_code)]
    pub fn downscale_bitmap_if_required(&mut self, font: &FontInstance) {
        // Check if the glyph is going to be downscaled in the shader. If the scaling is
        // less than 0.5, that means bilinear filtering can't effectively filter the glyph
        // without aliasing artifacts.
        //
        // Instead of fixing this by mipmapping the glyph cache texture, rather manually
        // produce the appropriate mip level for individual glyphs where bilinear filtering
        // will still produce acceptable results.
        match self.format {
            GlyphFormat::Bitmap | GlyphFormat::ColorBitmap => {},
            _ => return,
        }
        let (x_scale, y_scale) = font.transform.compute_scale().unwrap_or((1.0, 1.0));
        let upscaled = x_scale.max(y_scale) as f32;
        let mut new_scale = self.scale;
        if new_scale * upscaled <= 0.0 {
            return;
        }
        let mut steps = 0;
        while new_scale * upscaled <= 0.5 {
            new_scale *= 2.0;
            steps += 1;
        }
        // If no mipping is necessary, just bail.
        if steps == 0 {
            return;
        }

        // Calculate the actual size of the mip level.
        let new_width = (self.width as usize + (1 << steps) - 1) >> steps;
        let new_height = (self.height as usize + (1 << steps) - 1) >> steps;
        let mut new_bytes: Vec<u8> = Vec::with_capacity(new_width * new_height * 4);

        // Produce destination pixels by applying a box filter to the source pixels.
        // The box filter corresponds to how graphics drivers may generate mipmaps.
        for y in 0 .. new_height {
            for x in 0 .. new_width {
                // Calculate the number of source samples that contribute to the destination pixel.
                let src_y = y << steps;
                let src_x = x << steps;
                let y_samples = (1 << steps).min(self.height as usize - src_y);
                let x_samples = (1 << steps).min(self.width as usize - src_x);
                let num_samples = (x_samples * y_samples) as u32;

                let mut src_idx = (src_y * self.width as usize + src_x) * 4;
                // Initialize the accumulator with half an increment so that when later divided
                // by the sample count, it will effectively round the accumulator to the nearest
                // increment.
                let mut accum = [num_samples / 2; 4];
                // Accumulate all the contributing source sampless.
                for _ in 0 .. y_samples {
                    for _ in 0 .. x_samples {
                        accum[0] += self.bytes[src_idx + 0] as u32;
                        accum[1] += self.bytes[src_idx + 1] as u32;
                        accum[2] += self.bytes[src_idx + 2] as u32;
                        accum[3] += self.bytes[src_idx + 3] as u32;
                        src_idx += 4;
                    }
                    src_idx += (self.width as usize - x_samples) * 4;
                }

                // Finally, divide by the sample count to get the mean value for the new pixel.
                new_bytes.extend_from_slice(&[
                    (accum[0] / num_samples) as u8,
                    (accum[1] / num_samples) as u8,
                    (accum[2] / num_samples) as u8,
                    (accum[3] / num_samples) as u8,
                ]);
            }
        }

        // Fix the bounds for the new glyph data.
        self.top /= (1 << steps) as f32;
        self.left /= (1 << steps) as f32;
        self.width = new_width as i32;
        self.height = new_height as i32;
        self.scale = new_scale;
        self.bytes = new_bytes;
    }
}

pub struct FontContexts {
    // These worker are mostly accessed from their corresponding worker threads.
    // The goal is that there should be no noticeable contention on the mutexes.
    worker_contexts: Vec<Mutex<FontContext>>,
    // This worker should be accessed by threads that don't belong to the thread pool
    // (in theory that's only the render backend thread so no contention expected either).
    shared_context: Mutex<FontContext>,
    // Stored here as a convenience to get the current thread index.
    #[allow(dead_code)]
    workers: Arc<ThreadPool>,
    locked_mutex: Mutex<bool>,
    locked_cond: Condvar,
}

impl FontContexts {

    /// Get access to any particular font context.
    ///
    /// The id is ```Some(i)``` where i is an index between 0 and num_worker_contexts
    /// for font contexts associated to the thread pool, and None for the shared
    /// global font context for use outside of the thread pool.
    pub fn lock_context(&self, id: Option<usize>) -> MutexGuard<FontContext> {
        match id {
            Some(index) => self.worker_contexts[index].lock().unwrap(),
            None => self.shared_context.lock().unwrap(),
        }
    }

    /// Get access to the font context usable outside of the thread pool.
    pub fn lock_shared_context(&self) -> MutexGuard<FontContext> {
        self.shared_context.lock().unwrap()
    }

    // number of contexts associated to workers
    pub fn num_worker_contexts(&self) -> usize {
        self.worker_contexts.len()
    }
}

pub trait AsyncForEach<T> {
    fn async_for_each<F: Fn(MutexGuard<T>) + Send + 'static>(&self, f: F);
}

impl AsyncForEach<FontContext> for Arc<FontContexts> {
    fn async_for_each<F: Fn(MutexGuard<FontContext>) + Send + 'static>(&self, f: F) {
        // Reset the locked condition.
        let mut locked = self.locked_mutex.lock().unwrap();
        *locked = false;

        // Arc that can be safely moved into a spawn closure.
        let font_contexts = self.clone();
        // Spawn a new thread on which to run the for-each off the main thread.
        self.workers.spawn(move || {
            // Lock the shared and worker contexts up front.
            let mut locks = Vec::with_capacity(font_contexts.num_worker_contexts() + 1);
            locks.push(font_contexts.lock_shared_context());
            for i in 0 .. font_contexts.num_worker_contexts() {
                locks.push(font_contexts.lock_context(Some(i)));
            }

            // Signal the locked condition now that all contexts are locked.
            *font_contexts.locked_mutex.lock().unwrap() = true;
            font_contexts.locked_cond.notify_all();

            // Now that everything is locked, proceed to processing each locked context.
            for context in locks {
                f(context);
            }
        });

        // Wait for locked condition before resuming. Safe to proceed thereafter
        // since any other thread that needs to use a FontContext will try to lock
        // it first.
        while !*locked {
            locked = self.locked_cond.wait(locked).unwrap();
        }
    }
}

pub struct GlyphRasterizer {
    #[allow(dead_code)]
    workers: Arc<ThreadPool>,
    font_contexts: Arc<FontContexts>,

    /// The current number of individual glyphs waiting in pending batches.
    pending_glyph_count: usize,

    /// The current number of glyph request jobs that have been kicked to worker threads.
    pending_glyph_jobs: usize,

    /// The number of glyphs requested this frame.
    glyph_request_count: usize,

    /// A map of current glyph request batches.
    pending_glyph_requests: FastHashMap<FontInstance, SmallVec<[GlyphKey; 16]>>,

    // Receives the rendered glyphs.
    glyph_rx: Receiver<GlyphRasterJobs>,
    glyph_tx: Sender<GlyphRasterJobs>,

    // We defer removing fonts to the end of the frame so that:
    // - this work is done outside of the critical path,
    // - we don't have to worry about the ordering of events if a font is used on
    //   a frame where it is used (although it seems unlikely).
    fonts_to_remove: Vec<FontKey>,
    // Defer removal of font instances, as for fonts.
    font_instances_to_remove: Vec<FontInstance>,

    // Whether to parallelize glyph rasterization with rayon.
    enable_multithreading: bool,

    // Whether glyphs can be rasterized in r8 format when it makes sense.
    can_use_r8_format: bool,
}

impl GlyphRasterizer {
    pub fn new(workers: Arc<ThreadPool>, can_use_r8_format: bool) -> Result<Self, ResourceCacheError> {
        let (glyph_tx, glyph_rx) = unbounded_channel();

        let num_workers = workers.current_num_threads();
        let mut contexts = Vec::with_capacity(num_workers);

        let shared_context = FontContext::new()?;

        for _ in 0 .. num_workers {
            contexts.push(Mutex::new(FontContext::new()?));
        }

        let font_context = FontContexts {
                worker_contexts: contexts,
                shared_context: Mutex::new(shared_context),
                workers: Arc::clone(&workers),
                locked_mutex: Mutex::new(false),
                locked_cond: Condvar::new(),
        };

        Ok(GlyphRasterizer {
            font_contexts: Arc::new(font_context),
            pending_glyph_jobs: 0,
            pending_glyph_count: 0,
            glyph_request_count: 0,
            glyph_rx,
            glyph_tx,
            workers,
            fonts_to_remove: Vec::new(),
            font_instances_to_remove: Vec::new(),
            enable_multithreading: true,
            pending_glyph_requests: FastHashMap::default(),
            can_use_r8_format,
        })
    }

    pub fn add_font(&mut self, font_key: FontKey, template: FontTemplate) {
        self.font_contexts.async_for_each(move |mut context| {
            context.add_font(&font_key, &template);
        });
    }

    pub fn delete_font(&mut self, font_key: FontKey) {
        self.fonts_to_remove.push(font_key);
    }

    pub fn delete_font_instance(&mut self, instance: &FontInstance) {
        self.font_instances_to_remove.push(instance.clone());
    }

    pub fn prepare_font(&self, font: &mut FontInstance) {
        FontContext::prepare_font(font);

        // Quantize the transform to minimize thrashing of the glyph cache, but
        // only quantize the transform when preparing to access the glyph cache.
        // This way, the glyph subpixel positions, which are calculated before
        // this, can still use the precise transform which is required to match
        // the subpixel positions computed for glyphs in the text run shader.
        font.transform = font.transform.quantize();
    }

    pub fn get_glyph_dimensions(
        &mut self,
        font: &FontInstance,
        glyph_index: GlyphIndex,
    ) -> Option<GlyphDimensions> {
        let glyph_key = GlyphKey::new(
            glyph_index,
            DevicePoint::zero(),
            SubpixelDirection::None,
        );

        self.font_contexts
            .lock_shared_context()
            .get_glyph_dimensions(font, &glyph_key)
    }

    pub fn get_glyph_index(&mut self, font_key: FontKey, ch: char) -> Option<u32> {
        self.font_contexts
            .lock_shared_context()
            .get_glyph_index(font_key, ch)
    }

    fn remove_dead_fonts(&mut self) {
        if self.fonts_to_remove.is_empty() && self.font_instances_to_remove.is_empty() {
            return
        }

        profile_scope!("remove_dead_fonts");
        let fonts_to_remove = mem::replace(&mut self.fonts_to_remove, Vec::new());
        let font_instances_to_remove = mem::replace(& mut self.font_instances_to_remove, Vec::new());
        self.font_contexts.async_for_each(move |mut context| {
            for font_key in &fonts_to_remove {
                context.delete_font(font_key);
            }
            for instance in &font_instances_to_remove {
                context.delete_font_instance(instance);
            }
        });
    }

    #[cfg(feature = "replay")]
    pub fn reset(&mut self) {
        //TODO: any signals need to be sent to the workers?
        self.pending_glyph_jobs = 0;
        self.pending_glyph_count = 0;
        self.glyph_request_count = 0;
        self.fonts_to_remove.clear();
        self.font_instances_to_remove.clear();
    }
}

trait AddFont {
    fn add_font(&mut self, font_key: &FontKey, template: &FontTemplate);
}

impl AddFont for FontContext {
    fn add_font(&mut self, font_key: &FontKey, template: &FontTemplate) {
        match *template {
            FontTemplate::Raw(ref bytes, index) => {
                self.add_raw_font(font_key, bytes.clone(), index);
            }
            FontTemplate::Native(ref native_font_handle) => {
                self.add_native_font(font_key, (*native_font_handle).clone());
            }
        }
    }
}

#[allow(dead_code)]
pub(in crate::glyph_rasterizer) struct GlyphRasterJob {
    key: GlyphKey,
    result: GlyphRasterResult,
}

#[allow(dead_code)]
pub enum GlyphRasterError {
    LoadFailed,
}

#[allow(dead_code)]
pub type GlyphRasterResult = Result<RasterizedGlyph, GlyphRasterError>;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GpuGlyphCacheKey(pub u32);

#[allow(dead_code)]
struct GlyphRasterJobs {
    font: FontInstance,
    jobs: Vec<GlyphRasterJob>,
}

#[cfg(test)]
mod test_glyph_rasterizer {
    pub const FORMAT: api::ImageFormat = api::ImageFormat::BGRA8;

    #[test]
    fn rasterize_200_glyphs() {
        // This test loads a font from disc, the renders 4 requests containing
        // 50 glyphs each, deletes the font and waits for the result.

        use rayon::ThreadPoolBuilder;
        use std::fs::File;
        use std::io::Read;
        use crate::texture_cache::TextureCache;
        use crate::glyph_cache::GlyphCache;
        use crate::gpu_cache::GpuCache;
        use crate::profiler::TransactionProfile;
        use api::{FontKey, FontInstanceKey, FontSize, FontTemplate, FontRenderMode,
                  IdNamespace, ColorU};
        use api::units::DevicePoint;
        use std::sync::Arc;
        use crate::glyph_rasterizer::{FontInstance, BaseFontInstance, GlyphKey, GlyphRasterizer};

        let worker = ThreadPoolBuilder::new()
            .thread_name(|idx|{ format!("WRWorker#{}", idx) })
            .build();
        let workers = Arc::new(worker.unwrap());
        let mut glyph_rasterizer = GlyphRasterizer::new(workers, true).unwrap();
        let mut glyph_cache = GlyphCache::new();
        let mut gpu_cache = GpuCache::new_for_testing();
        let mut texture_cache = TextureCache::new_for_testing(2048, FORMAT);
        let mut font_file =
            File::open("../wrench/reftests/text/VeraBd.ttf").expect("Couldn't open font file");
        let mut font_data = vec![];
        font_file
            .read_to_end(&mut font_data)
            .expect("failed to read font file");

        let font_key = FontKey::new(IdNamespace(0), 0);
        glyph_rasterizer.add_font(font_key, FontTemplate::Raw(Arc::new(font_data), 0));

        let font = FontInstance::from_base(Arc::new(BaseFontInstance {
            instance_key: FontInstanceKey(IdNamespace(0), 0),
            font_key,
            size: FontSize::from_f32_px(32.0),
            bg_color: ColorU::new(0, 0, 0, 0),
            render_mode: FontRenderMode::Subpixel,
            flags: Default::default(),
            synthetic_italics: Default::default(),
            platform_options: None,
            variations: Vec::new(),
        }));

        let subpx_dir = font.get_subpx_dir();

        let mut glyph_keys = Vec::with_capacity(200);
        for i in 0 .. 200 {
            glyph_keys.push(GlyphKey::new(
                i,
                DevicePoint::zero(),
                subpx_dir,
            ));
        }

        for i in 0 .. 4 {
            glyph_rasterizer.request_glyphs(
                &mut glyph_cache,
                font.clone(),
                &glyph_keys[(50 * i) .. (50 * (i + 1))],
                &mut texture_cache,
                &mut gpu_cache,
            );
        }

        glyph_rasterizer.delete_font(font_key);

        glyph_rasterizer.resolve_glyphs(
            &mut glyph_cache,
            &mut TextureCache::new_for_testing(4096, FORMAT),
            &mut gpu_cache,
            &mut TransactionProfile::new(),
        );
    }

    #[test]
    fn rasterize_large_glyphs() {
        // This test loads a font from disc and rasterize a few glyphs with a size of 200px to check
        // that the texture cache handles them properly.
        use rayon::ThreadPoolBuilder;
        use std::fs::File;
        use std::io::Read;
        use crate::texture_cache::TextureCache;
        use crate::glyph_cache::GlyphCache;
        use crate::gpu_cache::GpuCache;
        use crate::profiler::TransactionProfile;
        use api::{FontKey, FontInstanceKey, FontSize, FontTemplate, FontRenderMode,
                  IdNamespace, ColorU};
        use api::units::DevicePoint;
        use std::sync::Arc;
        use crate::glyph_rasterizer::{FontInstance, BaseFontInstance, GlyphKey, GlyphRasterizer};

        let worker = ThreadPoolBuilder::new()
            .thread_name(|idx|{ format!("WRWorker#{}", idx) })
            .build();
        let workers = Arc::new(worker.unwrap());
        let mut glyph_rasterizer = GlyphRasterizer::new(workers, true).unwrap();
        let mut glyph_cache = GlyphCache::new();
        let mut gpu_cache = GpuCache::new_for_testing();
        let mut texture_cache = TextureCache::new_for_testing(2048, FORMAT);
        let mut font_file =
            File::open("../wrench/reftests/text/VeraBd.ttf").expect("Couldn't open font file");
        let mut font_data = vec![];
        font_file
            .read_to_end(&mut font_data)
            .expect("failed to read font file");

        let font_key = FontKey::new(IdNamespace(0), 0);
        glyph_rasterizer.add_font(font_key, FontTemplate::Raw(Arc::new(font_data), 0));

        let font = FontInstance::from_base(Arc::new(BaseFontInstance {
            instance_key: FontInstanceKey(IdNamespace(0), 0),
            font_key,
            size: FontSize::from_f32_px(200.0),
            bg_color: ColorU::new(0, 0, 0, 0),
            render_mode: FontRenderMode::Subpixel,
            flags: Default::default(),
            synthetic_italics: Default::default(),
            platform_options: None,
            variations: Vec::new(),
        }));

        let subpx_dir = font.get_subpx_dir();

        let mut glyph_keys = Vec::with_capacity(10);
        for i in 0 .. 10 {
            glyph_keys.push(GlyphKey::new(
                i,
                DevicePoint::zero(),
                subpx_dir,
            ));
        }

        glyph_rasterizer.request_glyphs(
            &mut glyph_cache,
            font.clone(),
            &glyph_keys,
            &mut texture_cache,
            &mut gpu_cache,
        );

        glyph_rasterizer.delete_font(font_key);

        glyph_rasterizer.resolve_glyphs(
            &mut glyph_cache,
            &mut TextureCache::new_for_testing(4096, FORMAT),
            &mut gpu_cache,
            &mut TransactionProfile::new(),
        );
    }

    #[test]
    fn test_subpx_quantize() {
        use crate::glyph_rasterizer::SubpixelOffset;

        assert_eq!(SubpixelOffset::quantize(0.0), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(-0.0), SubpixelOffset::Zero);

        assert_eq!(SubpixelOffset::quantize(0.1), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(0.01), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(0.05), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(0.12), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(0.124), SubpixelOffset::Zero);

        assert_eq!(SubpixelOffset::quantize(0.125), SubpixelOffset::Quarter);
        assert_eq!(SubpixelOffset::quantize(0.2), SubpixelOffset::Quarter);
        assert_eq!(SubpixelOffset::quantize(0.25), SubpixelOffset::Quarter);
        assert_eq!(SubpixelOffset::quantize(0.33), SubpixelOffset::Quarter);
        assert_eq!(SubpixelOffset::quantize(0.374), SubpixelOffset::Quarter);

        assert_eq!(SubpixelOffset::quantize(0.375), SubpixelOffset::Half);
        assert_eq!(SubpixelOffset::quantize(0.4), SubpixelOffset::Half);
        assert_eq!(SubpixelOffset::quantize(0.5), SubpixelOffset::Half);
        assert_eq!(SubpixelOffset::quantize(0.58), SubpixelOffset::Half);
        assert_eq!(SubpixelOffset::quantize(0.624), SubpixelOffset::Half);

        assert_eq!(SubpixelOffset::quantize(0.625), SubpixelOffset::ThreeQuarters);
        assert_eq!(SubpixelOffset::quantize(0.67), SubpixelOffset::ThreeQuarters);
        assert_eq!(SubpixelOffset::quantize(0.7), SubpixelOffset::ThreeQuarters);
        assert_eq!(SubpixelOffset::quantize(0.78), SubpixelOffset::ThreeQuarters);
        assert_eq!(SubpixelOffset::quantize(0.874), SubpixelOffset::ThreeQuarters);

        assert_eq!(SubpixelOffset::quantize(0.875), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(0.89), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(0.91), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(0.967), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(0.999), SubpixelOffset::Zero);

        assert_eq!(SubpixelOffset::quantize(-1.0), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(1.0), SubpixelOffset::Zero);
        assert_eq!(SubpixelOffset::quantize(1.5), SubpixelOffset::Half);
        assert_eq!(SubpixelOffset::quantize(-1.625), SubpixelOffset::Half);
        assert_eq!(SubpixelOffset::quantize(-4.33), SubpixelOffset::ThreeQuarters);
    }
}
