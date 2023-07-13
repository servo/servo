/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ColorF, DocumentId, ExternalImageId, PrimitiveFlags};
use api::{ImageFormat, NotificationRequest, Shadow, FilterOp, ImageBufferKind};
use api::units::*;
use api;
use crate::render_api::DebugCommand;
use crate::composite::NativeSurfaceOperation;
use crate::device::TextureFilter;
use crate::renderer::{FullFrameStats, PipelineInfo};
use crate::gpu_cache::GpuCacheUpdateList;
use crate::frame_builder::Frame;
use crate::profiler::TransactionProfile;
use fxhash::FxHasher;
use plane_split::BspSplitter;
use smallvec::SmallVec;
use std::{usize, i32};
use std::collections::{HashMap, HashSet};
use std::f32;
use std::hash::BuildHasherDefault;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(any(feature = "capture", feature = "replay"))]
use crate::capture::CaptureConfig;
#[cfg(feature = "capture")]
use crate::capture::ExternalCaptureImage;
#[cfg(feature = "replay")]
use crate::capture::PlainExternalImage;

pub type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FastHashSet<K> = HashSet<K, BuildHasherDefault<FxHasher>>;

/// Custom field embedded inside the Polygon struct of the plane-split crate.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct PlaneSplitAnchor {
    pub cluster_index: usize,
    pub instance_index: usize,
}

impl PlaneSplitAnchor {
    pub fn new(cluster_index: usize, instance_index: usize) -> Self {
        PlaneSplitAnchor {
            cluster_index,
            instance_index,
        }
    }
}

impl Default for PlaneSplitAnchor {
    fn default() -> Self {
        PlaneSplitAnchor {
            cluster_index: 0,
            instance_index: 0,
        }
    }
}

/// A concrete plane splitter type used in WebRender.
pub type PlaneSplitter = BspSplitter<f64, WorldPixel, PlaneSplitAnchor>;

/// An arbitrary number which we assume opacity is invisible below.
const OPACITY_EPSILON: f32 = 0.001;

/// Equivalent to api::FilterOp with added internal information
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum Filter {
    Identity,
    Blur(f32, f32),
    Brightness(f32),
    Contrast(f32),
    Grayscale(f32),
    HueRotate(f32),
    Invert(f32),
    Opacity(api::PropertyBinding<f32>, f32),
    Saturate(f32),
    Sepia(f32),
    DropShadows(SmallVec<[Shadow; 1]>),
    ColorMatrix(Box<[f32; 20]>),
    SrgbToLinear,
    LinearToSrgb,
    ComponentTransfer,
    Flood(ColorF),
}

impl Filter {
    pub fn is_visible(&self) -> bool {
        match *self {
            Filter::Identity |
            Filter::Blur(..) |
            Filter::Brightness(..) |
            Filter::Contrast(..) |
            Filter::Grayscale(..) |
            Filter::HueRotate(..) |
            Filter::Invert(..) |
            Filter::Saturate(..) |
            Filter::Sepia(..) |
            Filter::DropShadows(..) |
            Filter::ColorMatrix(..) |
            Filter::SrgbToLinear |
            Filter::LinearToSrgb |
            Filter::ComponentTransfer  => true,
            Filter::Opacity(_, amount) => {
                amount > OPACITY_EPSILON
            },
            Filter::Flood(color) => {
                color.a > OPACITY_EPSILON
            }
        }
    }

    pub fn is_noop(&self) -> bool {
        match *self {
            Filter::Identity => false, // this is intentional
            Filter::Blur(width, height) => width == 0.0 && height == 0.0,
            Filter::Brightness(amount) => amount == 1.0,
            Filter::Contrast(amount) => amount == 1.0,
            Filter::Grayscale(amount) => amount == 0.0,
            Filter::HueRotate(amount) => amount == 0.0,
            Filter::Invert(amount) => amount == 0.0,
            Filter::Opacity(api::PropertyBinding::Value(amount), _) => amount >= 1.0,
            Filter::Saturate(amount) => amount == 1.0,
            Filter::Sepia(amount) => amount == 0.0,
            Filter::DropShadows(ref shadows) => {
                for shadow in shadows {
                    if shadow.offset.x != 0.0 || shadow.offset.y != 0.0 || shadow.blur_radius != 0.0 {
                        return false;
                    }
                }

                true
            }
            Filter::ColorMatrix(ref matrix) => {
                **matrix == [
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0,
                    0.0, 0.0, 0.0, 0.0
                ]
            }
            Filter::Opacity(api::PropertyBinding::Binding(..), _) |
            Filter::SrgbToLinear |
            Filter::LinearToSrgb |
            Filter::ComponentTransfer |
            Filter::Flood(..) => false,
        }
    }


    pub fn as_int(&self) -> i32 {
        // Must be kept in sync with brush_blend.glsl
        match *self {
            Filter::Identity => 0, // matches `Contrast(1)`
            Filter::Contrast(..) => 0,
            Filter::Grayscale(..) => 1,
            Filter::HueRotate(..) => 2,
            Filter::Invert(..) => 3,
            Filter::Saturate(..) => 4,
            Filter::Sepia(..) => 5,
            Filter::Brightness(..) => 6,
            Filter::ColorMatrix(..) => 7,
            Filter::SrgbToLinear => 8,
            Filter::LinearToSrgb => 9,
            Filter::Flood(..) => 10,
            Filter::ComponentTransfer => 11,
            Filter::Blur(..) => 12,
            Filter::DropShadows(..) => 13,
            Filter::Opacity(..) => 14,
        }
    }
}

impl From<FilterOp> for Filter {
    fn from(op: FilterOp) -> Self {
        match op {
            FilterOp::Identity => Filter::Identity,
            FilterOp::Blur(w, h) => Filter::Blur(w, h),
            FilterOp::Brightness(b) => Filter::Brightness(b),
            FilterOp::Contrast(c) => Filter::Contrast(c),
            FilterOp::Grayscale(g) => Filter::Grayscale(g),
            FilterOp::HueRotate(h) => Filter::HueRotate(h),
            FilterOp::Invert(i) => Filter::Invert(i),
            FilterOp::Opacity(binding, opacity) => Filter::Opacity(binding, opacity),
            FilterOp::Saturate(s) => Filter::Saturate(s),
            FilterOp::Sepia(s) => Filter::Sepia(s),
            FilterOp::ColorMatrix(mat) => Filter::ColorMatrix(Box::new(mat)),
            FilterOp::SrgbToLinear => Filter::SrgbToLinear,
            FilterOp::LinearToSrgb => Filter::LinearToSrgb,
            FilterOp::ComponentTransfer => Filter::ComponentTransfer,
            FilterOp::DropShadow(shadow) => Filter::DropShadows(smallvec![shadow]),
            FilterOp::Flood(color) => Filter::Flood(color),
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub enum Swizzle {
    Rgba,
    Bgra,
}

impl Default for Swizzle {
    fn default() -> Self {
        Swizzle::Rgba
    }
}

/// Swizzle settings of the texture cache.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub struct SwizzleSettings {
    /// Swizzle required on sampling a texture with BGRA8 format.
    pub bgra8_sampling_swizzle: Swizzle,
}

/// An ID for a texture that is owned by the `texture_cache` module.
///
/// This can include atlases or standalone textures allocated via the texture
/// cache (e.g.  if an image is too large to be added to an atlas). The texture
/// cache manages the allocation and freeing of these IDs, and the rendering
/// thread maintains a map from cache texture ID to native texture.
///
/// We never reuse IDs, so we use a u64 here to be safe.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct CacheTextureId(pub u32);

impl CacheTextureId {
    pub const INVALID: CacheTextureId = CacheTextureId(!0);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct DeferredResolveIndex(pub u32);

/// Identifies the source of an input texture to a shader.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum TextureSource {
    /// Equivalent to `None`, allowing us to avoid using `Option`s everywhere.
    Invalid,
    /// An entry in the texture cache.
    TextureCache(CacheTextureId, Swizzle),
    /// An external image texture, mananged by the embedding.
    External(DeferredResolveIndex, ImageBufferKind),
    /// Select a dummy 1x1 white texture. This can be used by image
    /// shaders that want to draw a solid color.
    Dummy,
}

impl TextureSource {
    pub fn image_buffer_kind(&self) -> ImageBufferKind {
        match *self {
            TextureSource::TextureCache(..) => ImageBufferKind::Texture2D,

            TextureSource::External(_, image_buffer_kind) => image_buffer_kind,

            // Render tasks use texture arrays for now.
            TextureSource::Dummy => ImageBufferKind::Texture2D,

            TextureSource::Invalid => ImageBufferKind::Texture2D,
        }
    }

    #[inline]
    pub fn is_compatible(
        &self,
        other: &TextureSource,
    ) -> bool {
        *self == TextureSource::Invalid ||
        *other == TextureSource::Invalid ||
        self == other
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderTargetInfo {
    pub has_depth: bool,
}

#[derive(Debug)]
pub enum TextureUpdateSource {
    External {
        id: ExternalImageId,
        channel_index: u8,
    },
    Bytes { data: Arc<Vec<u8>> },
    /// Clears the target area, rather than uploading any pixels. Used when the
    /// texture cache debug display is active.
    DebugClear,
}

/// Command to allocate, reallocate, or free a texture for the texture cache.
#[derive(Debug)]
pub struct TextureCacheAllocation {
    /// The virtual ID (i.e. distinct from device ID) of the texture.
    pub id: CacheTextureId,
    /// Details corresponding to the operation in question.
    pub kind: TextureCacheAllocationKind,
}

/// Information used when allocating / reallocating.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TextureCacheAllocInfo {
    pub width: i32,
    pub height: i32,
    pub format: ImageFormat,
    pub filter: TextureFilter,
    pub target: ImageBufferKind,
    /// Indicates whether this corresponds to one of the shared texture caches.
    pub is_shared_cache: bool,
    /// If true, this texture requires a depth target.
    pub has_depth: bool,
}

/// Sub-operation-specific information for allocation operations.
#[derive(Debug)]
pub enum TextureCacheAllocationKind {
    /// Performs an initial texture allocation.
    Alloc(TextureCacheAllocInfo),
    /// Reallocates the texture without preserving its contents.
    Reset(TextureCacheAllocInfo),
    /// Frees the texture and the corresponding cache ID.
    Free,
}

/// Command to update the contents of the texture cache.
#[derive(Debug)]
pub struct TextureCacheUpdate {
    pub rect: DeviceIntRect,
    pub stride: Option<i32>,
    pub offset: i32,
    pub format_override: Option<ImageFormat>,
    pub source: TextureUpdateSource,
}

/// Atomic set of commands to manipulate the texture cache, generated on the
/// RenderBackend thread and executed on the Renderer thread.
///
/// The list of allocation operations is processed before the updates. This is
/// important to allow coalescing of certain allocation operations.
#[derive(Default)]
pub struct TextureUpdateList {
    /// Indicates that there was some kind of cleanup clear operation. Used for
    /// sanity checks.
    pub clears_shared_cache: bool,
    /// Commands to alloc/realloc/free the textures. Processed first.
    pub allocations: Vec<TextureCacheAllocation>,
    /// Commands to update the contents of the textures. Processed second.
    pub updates: FastHashMap<CacheTextureId, Vec<TextureCacheUpdate>>,
}

impl TextureUpdateList {
    /// Mints a new `TextureUpdateList`.
    pub fn new() -> Self {
        TextureUpdateList {
            clears_shared_cache: false,
            allocations: Vec::new(),
            updates: FastHashMap::default(),
        }
    }

    /// Returns true if this is a no-op (no updates to be applied).
    pub fn is_nop(&self) -> bool {
        self.allocations.is_empty() && self.updates.is_empty()
    }

    /// Sets the clears_shared_cache flag for renderer-side sanity checks.
    #[inline]
    pub fn note_clear(&mut self) {
        self.clears_shared_cache = true;
    }

    /// Pushes an update operation onto the list.
    #[inline]
    pub fn push_update(&mut self, id: CacheTextureId, update: TextureCacheUpdate) {
        self.updates
            .entry(id)
            .or_default()
            .push(update);
    }

    /// Sends a command to the Renderer to clear the portion of the shared region
    /// we just freed. Used when the texture cache debugger is enabled.
    #[cold]
    pub fn push_debug_clear(
        &mut self,
        id: CacheTextureId,
        origin: DeviceIntPoint,
        width: i32,
        height: i32,
    ) {
        let size = DeviceIntSize::new(width, height);
        let rect = DeviceIntRect::new(origin, size);
        self.push_update(id, TextureCacheUpdate {
            rect,
            stride: None,
            offset: 0,
            format_override: None,
            source: TextureUpdateSource::DebugClear,
        });
    }


    /// Pushes an allocation operation onto the list.
    pub fn push_alloc(&mut self, id: CacheTextureId, info: TextureCacheAllocInfo) {
        debug_assert!(!self.allocations.iter().any(|x| x.id == id));
        self.allocations.push(TextureCacheAllocation {
            id,
            kind: TextureCacheAllocationKind::Alloc(info),
        });
    }

    /// Pushes a reallocation operation onto the list, potentially coalescing
    /// with previous operations.
    pub fn push_reset(&mut self, id: CacheTextureId, info: TextureCacheAllocInfo) {
        self.debug_assert_coalesced(id);

        // Drop any unapplied updates to the to-be-freed texture.
        self.updates.remove(&id);

        // Coallesce this realloc into a previous alloc or realloc, if available.
        if let Some(cur) = self.allocations.iter_mut().find(|x| x.id == id) {
            match cur.kind {
                TextureCacheAllocationKind::Alloc(ref mut i) => *i = info,
                TextureCacheAllocationKind::Reset(ref mut i) => *i = info,
                TextureCacheAllocationKind::Free => panic!("Resetting freed texture"),
            }
            return
        }

        self.allocations.push(TextureCacheAllocation {
            id,
            kind: TextureCacheAllocationKind::Reset(info),
        });
    }

    /// Pushes a free operation onto the list, potentially coalescing with
    /// previous operations.
    pub fn push_free(&mut self, id: CacheTextureId) {
        self.debug_assert_coalesced(id);

        // Drop any unapplied updates to the to-be-freed texture.
        self.updates.remove(&id);

        // Drop any allocations for it as well. If we happen to be allocating and
        // freeing in the same batch, we can collapse them to a no-op.
        let idx = self.allocations.iter().position(|x| x.id == id);
        let removed_kind = idx.map(|i| self.allocations.remove(i).kind);
        match removed_kind {
            Some(TextureCacheAllocationKind::Alloc(..)) => { /* no-op! */ },
            Some(TextureCacheAllocationKind::Free) => panic!("Double free"),
            Some(TextureCacheAllocationKind::Reset(..)) |
            None => {
                self.allocations.push(TextureCacheAllocation {
                    id,
                    kind: TextureCacheAllocationKind::Free,
                });
            }
        };
    }

    fn debug_assert_coalesced(&self, id: CacheTextureId) {
        debug_assert!(
            self.allocations.iter().filter(|x| x.id == id).count() <= 1,
            "Allocations should have been coalesced",
        );
    }
}

/// A list of updates built by the render backend that should be applied
/// by the renderer thread.
pub struct ResourceUpdateList {
    /// List of OS native surface create / destroy operations to apply.
    pub native_surface_updates: Vec<NativeSurfaceOperation>,

    /// Atomic set of texture cache updates to apply.
    pub texture_updates: TextureUpdateList,
}

impl ResourceUpdateList {
    /// Returns true if this update list has no effect.
    pub fn is_nop(&self) -> bool {
        self.texture_updates.is_nop() && self.native_surface_updates.is_empty()
    }
}

/// Wraps a frame_builder::Frame, but conceptually could hold more information
pub struct RenderedDocument {
    pub frame: Frame,
    pub is_new_scene: bool,
    pub profile: TransactionProfile,
    pub frame_stats: Option<FullFrameStats>
}

pub enum DebugOutput {
    #[cfg(feature = "capture")]
    SaveCapture(CaptureConfig, Vec<ExternalCaptureImage>),
    #[cfg(feature = "replay")]
    LoadCapture(CaptureConfig, Vec<PlainExternalImage>),
}

#[allow(dead_code)]
pub enum ResultMsg {
    DebugCommand(DebugCommand),
    DebugOutput(DebugOutput),
    RefreshShader(PathBuf),
    UpdateGpuCache(GpuCacheUpdateList),
    UpdateResources {
        resource_updates: ResourceUpdateList,
        memory_pressure: bool,
    },
    PublishPipelineInfo(PipelineInfo),
    PublishDocument(
        DocumentId,
        RenderedDocument,
        ResourceUpdateList,
    ),
    AppendNotificationRequests(Vec<NotificationRequest>),
    ForceRedraw,
}

#[derive(Clone, Debug)]
pub struct ResourceCacheError {
    description: String,
}

impl ResourceCacheError {
    pub fn new(description: String) -> ResourceCacheError {
        ResourceCacheError {
            description,
        }
    }
}

/// Primitive metadata we pass around in a bunch of places
#[derive(Copy, Clone, Debug)]
pub struct LayoutPrimitiveInfo {
    /// NOTE: this is *ideally* redundant with the clip_rect
    /// but that's an ongoing project, so for now it exists and is used :(
    pub rect: LayoutRect,
    pub clip_rect: LayoutRect,
    pub flags: PrimitiveFlags,
}

impl LayoutPrimitiveInfo {
    pub fn with_clip_rect(rect: LayoutRect, clip_rect: LayoutRect) -> Self {
        Self {
            rect,
            clip_rect,
            flags: PrimitiveFlags::default(),
        }
    }
}
