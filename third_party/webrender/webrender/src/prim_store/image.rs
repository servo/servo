/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{
    AlphaType, ColorDepth, ColorF, ColorU, ExternalImageData, ExternalImageType,
    ImageKey as ApiImageKey, ImageBufferKind, ImageRendering, PremultipliedColorF,
    RasterSpace, Shadow, YuvColorSpace, ColorRange, YuvFormat,
};
use api::units::*;
use crate::scene_building::{CreateShadow, IsVisible};
use crate::frame_builder::{FrameBuildingContext, FrameBuildingState, add_child_render_task};
use crate::gpu_cache::{GpuCache, GpuDataRequest};
use crate::intern::{Internable, InternDebug, Handle as InternHandle};
use crate::internal_types::{LayoutPrimitiveInfo};
use crate::picture::SurfaceIndex;
use crate::prim_store::{
    EdgeAaSegmentMask, PrimitiveInstanceKind,
    PrimitiveOpacity, PrimKey,
    PrimTemplate, PrimTemplateCommonData, PrimitiveStore, SegmentInstanceIndex,
    SizeKey, InternablePrimitive,
};
use crate::render_target::RenderTargetKind;
use crate::render_task_graph::RenderTaskId;
use crate::render_task::RenderTask;
use crate::render_task_cache::{
    RenderTaskCacheKey, RenderTaskCacheKeyKind, RenderTaskParent
};
use crate::resource_cache::{ImageRequest, ImageProperties, ResourceCache};
use crate::util::pack_as_float;
use crate::visibility::{PrimitiveVisibility, compute_conservative_visible_rect};
use crate::spatial_tree::SpatialNodeIndex;
use crate::image_tiling;

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct VisibleImageTile {
    pub src_color: RenderTaskId,
    pub edge_flags: EdgeAaSegmentMask,
    pub local_rect: LayoutRect,
    pub local_clip_rect: LayoutRect,
}

// Key that identifies a unique (partial) image that is being
// stored in the render task cache.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ImageCacheKey {
    pub request: ImageRequest,
    pub texel_rect: Option<DeviceIntRect>,
}

/// Instance specific fields for an image primitive. These are
/// currently stored in a separate array to avoid bloating the
/// size of PrimitiveInstance. In the future, we should be able
/// to remove this and store the information inline, by:
/// (a) Removing opacity collapse / binding support completely.
///     Once we have general picture caching, we don't need this.
/// (b) Change visible_tiles to use Storage in the primitive
///     scratch buffer. This will reduce the size of the
///     visible_tiles field here, and save memory allocation
///     when image tiling is used. I've left it as a Vec for
///     now to reduce the number of changes, and because image
///     tiling is very rare on real pages.
#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ImageInstance {
    pub segment_instance_index: SegmentInstanceIndex,
    pub tight_local_clip_rect: LayoutRect,
    pub visible_tiles: Vec<VisibleImageTile>,
    pub src_color: Option<RenderTaskId>,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, MallocSizeOf, Hash)]
pub struct Image {
    pub key: ApiImageKey,
    pub stretch_size: SizeKey,
    pub tile_spacing: SizeKey,
    pub color: ColorU,
    pub image_rendering: ImageRendering,
    pub alpha_type: AlphaType,
}

pub type ImageKey = PrimKey<Image>;

impl ImageKey {
    pub fn new(
        info: &LayoutPrimitiveInfo,
        image: Image,
    ) -> Self {
        ImageKey {
            common: info.into(),
            kind: image,
        }
    }
}

impl InternDebug for ImageKey {}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, MallocSizeOf)]
pub struct ImageData {
    pub key: ApiImageKey,
    pub stretch_size: LayoutSize,
    pub tile_spacing: LayoutSize,
    pub color: ColorF,
    pub image_rendering: ImageRendering,
    pub alpha_type: AlphaType,
}

impl From<Image> for ImageData {
    fn from(image: Image) -> Self {
        ImageData {
            key: image.key,
            color: image.color.into(),
            stretch_size: image.stretch_size.into(),
            tile_spacing: image.tile_spacing.into(),
            image_rendering: image.image_rendering,
            alpha_type: image.alpha_type,
        }
    }
}

impl ImageData {
    /// Update the GPU cache for a given primitive template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        common: &mut PrimTemplateCommonData,
        image_instance: &mut ImageInstance,
        parent_surface: SurfaceIndex,
        prim_spatial_node_index: SpatialNodeIndex,
        frame_state: &mut FrameBuildingState,
        frame_context: &FrameBuildingContext,
        visibility: &mut PrimitiveVisibility,
    ) {

        let image_properties = frame_state
            .resource_cache
            .get_image_properties(self.key);

        common.opacity = match &image_properties {
            Some(properties) => {
                if properties.descriptor.is_opaque() {
                    PrimitiveOpacity::from_alpha(self.color.a)
                } else {
                    PrimitiveOpacity::translucent()
                }
            }
            None => PrimitiveOpacity::opaque(),
        };

        if self.stretch_size.width >= common.prim_rect.size.width &&
            self.stretch_size.height >= common.prim_rect.size.height {

            common.may_need_repetition = false;
        }

        let request = ImageRequest {
            key: self.key,
            rendering: self.image_rendering,
            tile: None,
        };

        match image_properties {
            // Non-tiled (most common) path.
            Some(ImageProperties { tiling: None, ref descriptor, ref external_image, .. }) => {
                let mut size = frame_state.resource_cache.request_image(
                    request,
                    frame_state.gpu_cache,
                );

                let orig_task_id = frame_state.rg_builder.add().init(
                    RenderTask::new_image(size, request)
                );

                // On some devices we cannot render from an ImageBufferKind::TextureExternal
                // source using most shaders, so must peform a copy to a regular texture first.
                let task_id = if frame_context.fb_config.external_images_require_copy
                    && matches!(
                        external_image,
                        Some(ExternalImageData {
                            image_type: ExternalImageType::TextureHandle(
                                ImageBufferKind::TextureExternal
                            ),
                            ..
                        })
                    )
                {
                    let target_kind = if descriptor.format.bytes_per_pixel() == 1 {
                        RenderTargetKind::Alpha
                    } else {
                        RenderTargetKind::Color
                    };

                    let task_id = RenderTask::new_scaling(
                        orig_task_id,
                        frame_state.rg_builder,
                        target_kind,
                        size
                    );

                    add_child_render_task(
                        parent_surface,
                        task_id,
                        frame_state.surfaces,
                        frame_state.rg_builder,
                    );

                    task_id
                } else {
                    orig_task_id
                };

                // Every frame, for cached items, we need to request the render
                // task cache item. The closure will be invoked on the first
                // time through, and any time the render task output has been
                // evicted from the texture cache.
                if self.tile_spacing == LayoutSize::zero() {
                    // Most common case.
                    image_instance.src_color = Some(task_id);
                } else {
                    let padding = DeviceIntSideOffsets::new(
                        0,
                        (self.tile_spacing.width * size.width as f32 / self.stretch_size.width) as i32,
                        (self.tile_spacing.height * size.height as f32 / self.stretch_size.height) as i32,
                        0,
                    );

                    size.width += padding.horizontal();
                    size.height += padding.vertical();

                    if padding != DeviceIntSideOffsets::zero() {
                        common.opacity = PrimitiveOpacity::translucent();
                    }

                    let image_cache_key = ImageCacheKey {
                        request,
                        texel_rect: None,
                    };
                    let target_kind = if descriptor.format.bytes_per_pixel() == 1 {
                        RenderTargetKind::Alpha
                    } else {
                        RenderTargetKind::Color
                    };

                    // Request a pre-rendered image task.
                    let cached_task_handle = frame_state.resource_cache.request_render_task(
                        RenderTaskCacheKey {
                            size,
                            kind: RenderTaskCacheKeyKind::Image(image_cache_key),
                        },
                        frame_state.gpu_cache,
                        frame_state.rg_builder,
                        None,
                        descriptor.is_opaque(),
                        RenderTaskParent::Surface(parent_surface),
                        frame_state.surfaces,
                        |rg_builder| {
                            // Create a task to blit from the texture cache to
                            // a normal transient render task surface.
                            // TODO: figure out if/when we can do a blit instead.
                            let cache_to_target_task_id = RenderTask::new_scaling_with_padding(
                                task_id,
                                rg_builder,
                                target_kind,
                                size,
                                padding,
                            );

                            // Create a task to blit the rect from the child render
                            // task above back into the right spot in the persistent
                            // render target cache.
                            RenderTask::new_blit(
                                size,
                                cache_to_target_task_id,
                                rg_builder,
                            )
                        }
                    );

                    image_instance.src_color = Some(cached_task_handle);
                }
            }
            // Tiled image path.
            Some(ImageProperties { tiling: Some(tile_size), visible_rect, .. }) => {
                // we'll  have a source handle per visible tile instead.
                image_instance.src_color = None;

                image_instance.visible_tiles.clear();
                // TODO: rename the blob's visible_rect into something that doesn't conflict
                // with the terminology we use during culling since it's not really the same
                // thing.
                let active_rect = visible_rect;

                // Tighten the clip rect because decomposing the repeated image can
                // produce primitives that are partially covering the original image
                // rect and we want to clip these extra parts out.
                let tight_clip_rect = visibility
                    .combined_local_clip_rect
                    .intersection(&common.prim_rect).unwrap();
                image_instance.tight_local_clip_rect = tight_clip_rect;

                let visible_rect = compute_conservative_visible_rect(
                    &visibility.clip_chain,
                    frame_state.current_dirty_region().combined,
                    prim_spatial_node_index,
                    frame_context.spatial_tree,
                );

                let base_edge_flags = edge_flags_for_tile_spacing(&self.tile_spacing);

                let stride = self.stretch_size + self.tile_spacing;

                // We are performing the decomposition on the CPU here, no need to
                // have it in the shader.
                common.may_need_repetition = false;

                let repetitions = image_tiling::repetitions(
                    &common.prim_rect,
                    &visible_rect,
                    stride,
                );

                for image_tiling::Repetition { origin, edge_flags } in repetitions {
                    let edge_flags = base_edge_flags | edge_flags;

                    let layout_image_rect = LayoutRect {
                        origin,
                        size: self.stretch_size,
                    };

                    let tiles = image_tiling::tiles(
                        &layout_image_rect,
                        &visible_rect,
                        &active_rect,
                        tile_size as i32,
                    );

                    for tile in tiles {
                        let request = request.with_tile(tile.offset);
                        let size = frame_state.resource_cache.request_image(
                            request,
                            frame_state.gpu_cache,
                        );

                        let task_id = frame_state.rg_builder.add().init(
                            RenderTask::new_image(size, request)
                        );

                        image_instance.visible_tiles.push(VisibleImageTile {
                            src_color: task_id,
                            edge_flags: tile.edge_flags & edge_flags,
                            local_rect: tile.rect,
                            local_clip_rect: tight_clip_rect,
                        });
                    }
                }

                if image_instance.visible_tiles.is_empty() {
                    // Mark as invisible
                    visibility.reset();
                }
            }
            None => {
                image_instance.src_color = None;
            }
        }

        if let Some(mut request) = frame_state.gpu_cache.request(&mut common.gpu_cache_handle) {
            self.write_prim_gpu_blocks(&mut request);
        }
    }

    pub fn write_prim_gpu_blocks(&self, request: &mut GpuDataRequest) {
        // Images are drawn as a white color, modulated by the total
        // opacity coming from any collapsed property bindings.
        // Size has to match `VECS_PER_SPECIFIC_BRUSH` from `brush_image.glsl` exactly.
        request.push(self.color.premultiplied());
        request.push(PremultipliedColorF::WHITE);
        request.push([
            self.stretch_size.width + self.tile_spacing.width,
            self.stretch_size.height + self.tile_spacing.height,
            0.0,
            0.0,
        ]);
    }
}

fn edge_flags_for_tile_spacing(tile_spacing: &LayoutSize) -> EdgeAaSegmentMask {
    let mut flags = EdgeAaSegmentMask::empty();

    if tile_spacing.width > 0.0 {
        flags |= EdgeAaSegmentMask::LEFT | EdgeAaSegmentMask::RIGHT;
    }
    if tile_spacing.height > 0.0 {
        flags |= EdgeAaSegmentMask::TOP | EdgeAaSegmentMask::BOTTOM;
    }

    flags
}

pub type ImageTemplate = PrimTemplate<ImageData>;

impl From<ImageKey> for ImageTemplate {
    fn from(image: ImageKey) -> Self {
        let common = PrimTemplateCommonData::with_key_common(image.common);

        ImageTemplate {
            common,
            kind: image.kind.into(),
        }
    }
}

pub type ImageDataHandle = InternHandle<Image>;

impl Internable for Image {
    type Key = ImageKey;
    type StoreData = ImageTemplate;
    type InternData = ();
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_IMAGES;
}

impl InternablePrimitive for Image {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> ImageKey {
        ImageKey::new(info, self)
    }

    fn make_instance_kind(
        _key: ImageKey,
        data_handle: ImageDataHandle,
        prim_store: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        // TODO(gw): Refactor this to not need a separate image
        //           instance (see ImageInstance struct).
        let image_instance_index = prim_store.images.push(ImageInstance {
            segment_instance_index: SegmentInstanceIndex::INVALID,
            tight_local_clip_rect: LayoutRect::zero(),
            visible_tiles: Vec::new(),
            src_color: None,
        });

        PrimitiveInstanceKind::Image {
            data_handle,
            image_instance_index,
            is_compositor_surface: false,
        }
    }
}

impl CreateShadow for Image {
    fn create_shadow(
        &self,
        shadow: &Shadow,
        _: bool,
        _: RasterSpace,
    ) -> Self {
        Image {
            tile_spacing: self.tile_spacing,
            stretch_size: self.stretch_size,
            key: self.key,
            image_rendering: self.image_rendering,
            alpha_type: self.alpha_type,
            color: shadow.color.into(),
        }
    }
}

impl IsVisible for Image {
    fn is_visible(&self) -> bool {
        true
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
pub struct YuvImage {
    pub color_depth: ColorDepth,
    pub yuv_key: [ApiImageKey; 3],
    pub format: YuvFormat,
    pub color_space: YuvColorSpace,
    pub color_range: ColorRange,
    pub image_rendering: ImageRendering,
}

pub type YuvImageKey = PrimKey<YuvImage>;

impl YuvImageKey {
    pub fn new(
        info: &LayoutPrimitiveInfo,
        yuv_image: YuvImage,
    ) -> Self {
        YuvImageKey {
            common: info.into(),
            kind: yuv_image,
        }
    }
}

impl InternDebug for YuvImageKey {}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct YuvImageData {
    pub color_depth: ColorDepth,
    pub yuv_key: [ApiImageKey; 3],
    pub src_yuv: [Option<RenderTaskId>; 3],
    pub format: YuvFormat,
    pub color_space: YuvColorSpace,
    pub color_range: ColorRange,
    pub image_rendering: ImageRendering,
}

impl From<YuvImage> for YuvImageData {
    fn from(image: YuvImage) -> Self {
        YuvImageData {
            color_depth: image.color_depth,
            yuv_key: image.yuv_key,
            src_yuv: [None, None, None],
            format: image.format,
            color_space: image.color_space,
            color_range: image.color_range,
            image_rendering: image.image_rendering,
        }
    }
}

impl YuvImageData {
    /// Update the GPU cache for a given primitive template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        common: &mut PrimTemplateCommonData,
        frame_state: &mut FrameBuildingState,
    ) {

        self.src_yuv = [ None, None, None ];

        let channel_num = self.format.get_plane_num();
        debug_assert!(channel_num <= 3);
        for channel in 0 .. channel_num {
            let request = ImageRequest {
                key: self.yuv_key[channel],
                rendering: self.image_rendering,
                tile: None,
            };

            let size = frame_state.resource_cache.request_image(
                request,
                frame_state.gpu_cache,
            );

            let task_id = frame_state.rg_builder.add().init(
                RenderTask::new_image(size, request)
            );

            self.src_yuv[channel] = Some(task_id);
        }

        if let Some(mut request) = frame_state.gpu_cache.request(&mut common.gpu_cache_handle) {
            self.write_prim_gpu_blocks(&mut request);
        };

        // YUV images never have transparency
        common.opacity = PrimitiveOpacity::opaque();
    }

    pub fn request_resources(
        &mut self,
        resource_cache: &mut ResourceCache,
        gpu_cache: &mut GpuCache,
    ) {
        let channel_num = self.format.get_plane_num();
        debug_assert!(channel_num <= 3);
        for channel in 0 .. channel_num {
            resource_cache.request_image(
                ImageRequest {
                    key: self.yuv_key[channel],
                    rendering: self.image_rendering,
                    tile: None,
                },
                gpu_cache,
            );
        }
    }

    pub fn write_prim_gpu_blocks(&self, request: &mut GpuDataRequest) {
        request.push([
            self.color_depth.rescaling_factor(),
            pack_as_float(self.color_space as u32),
            pack_as_float(self.format as u32),
            0.0
        ]);
    }
}

pub type YuvImageTemplate = PrimTemplate<YuvImageData>;

impl From<YuvImageKey> for YuvImageTemplate {
    fn from(image: YuvImageKey) -> Self {
        let common = PrimTemplateCommonData::with_key_common(image.common);

        YuvImageTemplate {
            common,
            kind: image.kind.into(),
        }
    }
}

pub type YuvImageDataHandle = InternHandle<YuvImage>;

impl Internable for YuvImage {
    type Key = YuvImageKey;
    type StoreData = YuvImageTemplate;
    type InternData = ();
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_YUV_IMAGES;
}

impl InternablePrimitive for YuvImage {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> YuvImageKey {
        YuvImageKey::new(info, self)
    }

    fn make_instance_kind(
        _key: YuvImageKey,
        data_handle: YuvImageDataHandle,
        _prim_store: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        PrimitiveInstanceKind::YuvImage {
            data_handle,
            segment_instance_index: SegmentInstanceIndex::INVALID,
            is_compositor_surface: false,
        }
    }
}

impl IsVisible for YuvImage {
    fn is_visible(&self) -> bool {
        true
    }
}

#[test]
#[cfg(target_pointer_width = "64")]
fn test_struct_sizes() {
    use std::mem;
    // The sizes of these structures are critical for performance on a number of
    // talos stress tests. If you get a failure here on CI, there's two possibilities:
    // (a) You made a structure smaller than it currently is. Great work! Update the
    //     test expectations and move on.
    // (b) You made a structure larger. This is not necessarily a problem, but should only
    //     be done with care, and after checking if talos performance regresses badly.
    assert_eq!(mem::size_of::<Image>(), 32, "Image size changed");
    assert_eq!(mem::size_of::<ImageTemplate>(), 72, "ImageTemplate size changed");
    assert_eq!(mem::size_of::<ImageKey>(), 52, "ImageKey size changed");
    assert_eq!(mem::size_of::<YuvImage>(), 32, "YuvImage size changed");
    assert_eq!(mem::size_of::<YuvImageTemplate>(), 72, "YuvImageTemplate size changed");
    assert_eq!(mem::size_of::<YuvImageKey>(), 52, "YuvImageKey size changed");
}
