/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ColorF, YuvColorSpace, YuvFormat, ImageRendering, ExternalImageId, ImageBufferKind};
use api::units::*;
use api::ColorDepth;
use crate::image_source::resolve_image;
use euclid::Transform3D;
use crate::gpu_cache::GpuCache;
use crate::gpu_types::{ZBufferId, ZBufferIdGenerator};
use crate::internal_types::TextureSource;
use crate::picture::{ImageDependency, ResolvedSurfaceTexture, TileCacheInstance, TileId, TileSurface};
use crate::prim_store::DeferredResolve;
use crate::resource_cache::{ImageRequest, ResourceCache};
use crate::util::Preallocator;
use crate::tile_cache::PictureCacheDebugInfo;
use std::{ops, u64, os::raw::c_void};

/*
 Types and definitions related to compositing picture cache tiles
 and/or OS compositor integration.
 */

/// Describes details of an operation to apply to a native surface
#[derive(Debug, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum NativeSurfaceOperationDetails {
    CreateSurface {
        id: NativeSurfaceId,
        virtual_offset: DeviceIntPoint,
        tile_size: DeviceIntSize,
        is_opaque: bool,
    },
    CreateExternalSurface {
        id: NativeSurfaceId,
        is_opaque: bool,
    },
    DestroySurface {
        id: NativeSurfaceId,
    },
    CreateTile {
        id: NativeTileId,
    },
    DestroyTile {
        id: NativeTileId,
    },
    AttachExternalImage {
        id: NativeSurfaceId,
        external_image: ExternalImageId,
    }
}

/// Describes an operation to apply to a native surface
#[derive(Debug, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct NativeSurfaceOperation {
    pub details: NativeSurfaceOperationDetails,
}

/// Describes the source surface information for a tile to be composited. This
/// is the analog of the TileSurface type, with target surface information
/// resolved such that it can be used by the renderer.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum CompositeTileSurface {
    Texture {
        surface: ResolvedSurfaceTexture,
    },
    Color {
        color: ColorF,
    },
    Clear,
    ExternalSurface {
        external_surface_index: ResolvedExternalSurfaceIndex,
    },
}

/// The surface format for a tile being composited.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompositeSurfaceFormat {
    Rgba,
    Yuv,
}

bitflags! {
    /// Optional features that can be opted-out of when compositing,
    /// possibly allowing a fast path to be selected.
    pub struct CompositeFeatures: u8 {
        // UV coordinates do not require clamping, for example because the
        // entire texture is being composited.
        const NO_UV_CLAMP = 1 << 0;
        // The texture sample should not be modulated by a specified color.
        const NO_COLOR_MODULATION = 1 << 1;
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum TileKind {
    Opaque,
    Alpha,
    Clear,
}

/// Describes the geometry and surface of a tile to be composited
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct CompositeTile {
    pub surface: CompositeTileSurface,
    pub rect: DeviceRect,
    pub clip_rect: DeviceRect,
    pub dirty_rect: DeviceRect,
    pub valid_rect: DeviceRect,
    pub transform: Option<CompositorSurfaceTransform>,
    pub z_id: ZBufferId,
    pub kind: TileKind,
}

fn tile_kind(surface: &CompositeTileSurface, is_opaque: bool) -> TileKind {
    match surface {
        // Color tiles are, by definition, opaque. We might support non-opaque color
        // tiles if we ever find pages that have a lot of these.
        CompositeTileSurface::Color { .. } => TileKind::Opaque,
        // Clear tiles have a special bucket
        CompositeTileSurface::Clear => TileKind::Clear,
        CompositeTileSurface::Texture { .. }
        | CompositeTileSurface::ExternalSurface { .. } => {
            // Texture surfaces get bucketed by opaque/alpha, for z-rejection
            // on the Draw compositor mode.
            if is_opaque {
                TileKind::Opaque
            } else {
                TileKind::Alpha
            }
        }
    }
}

pub enum ExternalSurfaceDependency {
    Yuv {
        image_dependencies: [ImageDependency; 3],
        color_space: YuvColorSpace,
        format: YuvFormat,
        rescale: f32,
    },
    Rgb {
        image_dependency: ImageDependency,
        flip_y: bool,
    },
}

/// Describes information about drawing a primitive as a compositor surface.
/// For now, we support only YUV images as compositor surfaces, but in future
/// this will also support RGBA images.
pub struct ExternalSurfaceDescriptor {
    // Rectangle of this surface in owning picture's coordinate space
    pub local_rect: PictureRect,
    // Rectangle of this surface in the compositor local space
    // TODO(gw): Switch this to CompositorSurfaceRect (CompositorSurfacePixel) in compositor trait.
    pub surface_rect: DeviceRect,
    // Rectangle of this surface in true device pixels
    pub device_rect: DeviceRect,
    pub local_clip_rect: PictureRect,
    pub clip_rect: DeviceRect,
    pub transform: CompositorSurfaceTransform,
    pub image_rendering: ImageRendering,
    pub z_id: ZBufferId,
    pub dependency: ExternalSurfaceDependency,
    /// If native compositing is enabled, the native compositor surface handle.
    /// Otherwise, this will be None
    pub native_surface_id: Option<NativeSurfaceId>,
    /// If the native surface needs to be updated, this will contain the size
    /// of the native surface as Some(size). If not dirty, this is None.
    pub update_params: Option<DeviceIntSize>,
}

/// Information about a plane in a YUV or RGB surface.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct ExternalPlaneDescriptor {
    pub texture: TextureSource,
    pub uv_rect: TexelRect,
}

impl ExternalPlaneDescriptor {
    fn invalid() -> Self {
        ExternalPlaneDescriptor {
            texture: TextureSource::Invalid,
            uv_rect: TexelRect::invalid(),
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ResolvedExternalSurfaceIndex(pub usize);

impl ResolvedExternalSurfaceIndex {
    pub const INVALID: ResolvedExternalSurfaceIndex = ResolvedExternalSurfaceIndex(usize::MAX);
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum ResolvedExternalSurfaceColorData {
    Yuv {
        // YUV specific information
        image_dependencies: [ImageDependency; 3],
        planes: [ExternalPlaneDescriptor; 3],
        color_space: YuvColorSpace,
        format: YuvFormat,
        rescale: f32,
    },
    Rgb {
        image_dependency: ImageDependency,
        plane: ExternalPlaneDescriptor,
        flip_y: bool,
    },
}

/// An ExternalSurfaceDescriptor that has had image keys
/// resolved to texture handles. This contains all the
/// information that the compositor step in renderer
/// needs to know.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ResolvedExternalSurface {
    pub color_data: ResolvedExternalSurfaceColorData,
    pub image_buffer_kind: ImageBufferKind,
    // Update information for a native surface if it's dirty
    pub update_params: Option<(NativeSurfaceId, DeviceIntSize)>,
}

/// Public interface specified in `RendererOptions` that configures
/// how WR compositing will operate.
pub enum CompositorConfig {
    /// Let WR draw tiles via normal batching. This requires no special OS support.
    Draw {
        /// If this is zero, a full screen present occurs at the end of the
        /// frame. This is the simplest and default mode. If this is non-zero,
        /// then the operating system supports a form of 'partial present' where
        /// only dirty regions of the framebuffer need to be updated.
        max_partial_present_rects: usize,
        /// If this is true, WR must draw the previous frames' dirty regions when
        /// doing a partial present. This is used for EGL which requires the front
        /// buffer to always be fully consistent.
        draw_previous_partial_present_regions: bool,
        /// A client provided interface to a compositor handling partial present.
        /// Required if webrender must query the backbuffer's age.
        partial_present: Option<Box<dyn PartialPresentCompositor>>,
    },
    /// Use a native OS compositor to draw tiles. This requires clients to implement
    /// the Compositor trait, but can be significantly more power efficient on operating
    /// systems that support it.
    Native {
        /// The maximum number of dirty rects that can be provided per compositor
        /// surface update. If this is zero, the entire compositor surface for
        /// a given tile will be drawn if it's dirty.
        max_update_rects: usize,
        /// A client provided interface to a native / OS compositor.
        compositor: Box<dyn Compositor>,
    }
}

impl CompositorConfig {
    pub fn compositor(&mut self) -> Option<&mut Box<dyn Compositor>> {
        match self {
            CompositorConfig::Native { ref mut compositor, .. } => {
                Some(compositor)
            }
            CompositorConfig::Draw { .. } => {
                None
            }
        }
    }

    pub fn partial_present(&mut self) -> Option<&mut Box<dyn PartialPresentCompositor>> {
        match self {
            CompositorConfig::Native { .. } => {
                None
            }
            CompositorConfig::Draw { ref mut partial_present, .. } => {
                partial_present.as_mut()
            }
        }
    }

}

impl Default for CompositorConfig {
    /// Default compositor config is full present without partial present.
    fn default() -> Self {
        CompositorConfig::Draw {
            max_partial_present_rects: 0,
            draw_previous_partial_present_regions: false,
            partial_present: None,
        }
    }
}

/// This is a representation of `CompositorConfig` without the `Compositor` trait
/// present. This allows it to be freely copied to other threads, such as the render
/// backend where the frame builder can access it.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompositorKind {
    /// WR handles compositing via drawing.
    Draw {
        /// Partial present support.
        max_partial_present_rects: usize,
        /// Draw previous regions when doing partial present.
        draw_previous_partial_present_regions: bool,
    },
    /// Native OS compositor.
    Native {
        /// Maximum dirty rects per compositor surface.
        max_update_rects: usize,
        /// The capabilities of the underlying platform.
        capabilities: CompositorCapabilities,
    },
}

impl Default for CompositorKind {
    /// Default compositor config is full present without partial present.
    fn default() -> Self {
        CompositorKind::Draw {
            max_partial_present_rects: 0,
            draw_previous_partial_present_regions: false,
        }
    }
}

impl CompositorKind {
    pub fn get_virtual_surface_size(&self) -> i32 {
        match self {
            CompositorKind::Draw { .. } => 0,
            CompositorKind::Native { capabilities, .. } => capabilities.virtual_surface_size,
        }
    }

    // We currently only support transforms for Native compositors,
    // bug 1655639 is filed for adding support to Draw.
    pub fn supports_transforms(&self) -> bool {
        match self {
            CompositorKind::Draw { .. } => false,
            CompositorKind::Native { .. } => true,
        }
    }

    pub fn should_redraw_on_invalidation(&self) -> bool {
        match self {
            CompositorKind::Draw { max_partial_present_rects, .. } => {
                // When partial present is enabled, we need to force redraw.
                *max_partial_present_rects > 0
            }
            CompositorKind::Native { capabilities, .. } => capabilities.redraw_on_invalidation,
        }
    }
}

/// The backing surface kind for a tile. Same as `TileSurface`, minus
/// the texture cache handles, visibility masks etc.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(PartialEq, Clone)]
pub enum TileSurfaceKind {
    Texture,
    Color {
        color: ColorF,
    },
    Clear,
}

impl From<&TileSurface> for TileSurfaceKind {
    fn from(surface: &TileSurface) -> Self {
        match surface {
            TileSurface::Texture { .. } => TileSurfaceKind::Texture,
            TileSurface::Color { color } => TileSurfaceKind::Color { color: *color },
            TileSurface::Clear => TileSurfaceKind::Clear,
        }
    }
}

/// Describes properties that identify a tile composition uniquely.
/// The backing surface for this tile.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(PartialEq, Clone)]
pub struct CompositeTileDescriptor {
    pub tile_id: TileId,
    pub surface_kind: TileSurfaceKind,
}

/// Describes the properties that identify a surface composition uniquely.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(PartialEq, Clone)]
pub struct CompositeSurfaceDescriptor {
    pub surface_id: Option<NativeSurfaceId>,
    pub clip_rect: DeviceRect,
    pub transform: CompositorSurfaceTransform,
    // A list of image keys and generations that this compositor surface
    // depends on. This avoids composites being skipped when the only
    // thing that has changed is the generation of an compositor surface
    // image dependency.
    pub image_dependencies: [ImageDependency; 3],
    pub image_rendering: ImageRendering,
    // List of the surface information for each tile added to this virtual surface
    pub tile_descriptors: Vec<CompositeTileDescriptor>,
}

/// Describes surface properties used to composite a frame. This
/// is used to compare compositions between frames.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(PartialEq, Clone)]
pub struct CompositeDescriptor {
    pub surfaces: Vec<CompositeSurfaceDescriptor>,
}

impl CompositeDescriptor {
    /// Construct an empty descriptor.
    pub fn empty() -> Self {
        CompositeDescriptor {
            surfaces: Vec::new(),
        }
    }
}

pub struct CompositeStatePreallocator {
    tiles: Preallocator,
    external_surfaces: Preallocator,
    occluders: Preallocator,
    occluders_events: Preallocator,
    occluders_active: Preallocator,
    descriptor_surfaces: Preallocator,
}

impl CompositeStatePreallocator {
    pub fn record(&mut self, state: &CompositeState) {
        self.tiles.record_vec(&state.tiles);
        self.external_surfaces.record_vec(&state.external_surfaces);
        self.occluders.record_vec(&state.occluders.occluders);
        self.occluders_events.record_vec(&state.occluders.events);
        self.occluders_active.record_vec(&state.occluders.active);
        self.descriptor_surfaces.record_vec(&state.descriptor.surfaces);
    }

    pub fn preallocate(&self, state: &mut CompositeState) {
        self.tiles.preallocate_vec(&mut state.tiles);
        self.external_surfaces.preallocate_vec(&mut state.external_surfaces);
        self.occluders.preallocate_vec(&mut state.occluders.occluders);
        self.occluders_events.preallocate_vec(&mut state.occluders.events);
        self.occluders_active.preallocate_vec(&mut state.occluders.active);
        self.descriptor_surfaces.preallocate_vec(&mut state.descriptor.surfaces);
    }
}

impl Default for CompositeStatePreallocator {
    fn default() -> Self {
        CompositeStatePreallocator {
            tiles: Preallocator::new(56),
            external_surfaces: Preallocator::new(0),
            occluders: Preallocator::new(16),
            occluders_events: Preallocator::new(32),
            occluders_active: Preallocator::new(16),
            descriptor_surfaces: Preallocator::new(8),
        }
    }
}

/// The list of tiles to be drawn this frame
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct CompositeState {
    // TODO(gw): Consider splitting up CompositeState into separate struct types depending
    //           on the selected compositing mode. Many of the fields in this state struct
    //           are only applicable to either Native or Draw compositing mode.
    /// List of tiles to be drawn by the Draw compositor.
    /// Tiles are accumulated in this vector and sorted from front to back at the end of the
    /// frame.
    pub tiles: Vec<CompositeTile>,
    /// List of primitives that were promoted to be compositor surfaces.
    pub external_surfaces: Vec<ResolvedExternalSurface>,
    /// Used to generate z-id values for tiles in the Draw compositor mode.
    pub z_generator: ZBufferIdGenerator,
    // If false, we can't rely on the dirty rects in the CompositeTile
    // instances. This currently occurs during a scroll event, as a
    // signal to refresh the whole screen. This is only a temporary
    // measure until we integrate with OS compositors. In the meantime
    // it gives us the ability to partial present for any non-scroll
    // case as a simple win (e.g. video, animation etc).
    pub dirty_rects_are_valid: bool,
    /// The kind of compositor for picture cache tiles (e.g. drawn by WR, or OS compositor)
    pub compositor_kind: CompositorKind,
    /// The overall device pixel scale, used for tile occlusion conversions.
    global_device_pixel_scale: DevicePixelScale,
    /// List of registered occluders
    pub occluders: Occluders,
    /// Description of the surfaces and properties that are being composited.
    pub descriptor: CompositeDescriptor,
    /// Debugging information about the state of the pictures cached for regression testing.
    pub picture_cache_debug: PictureCacheDebugInfo,
}

impl CompositeState {
    /// Construct a new state for compositing picture tiles. This is created
    /// during each frame construction and passed to the renderer.
    pub fn new(
        compositor_kind: CompositorKind,
        global_device_pixel_scale: DevicePixelScale,
        max_depth_ids: i32,
        dirty_rects_are_valid: bool,
    ) -> Self {
        CompositeState {
            tiles: Vec::new(),
            z_generator: ZBufferIdGenerator::new(max_depth_ids),
            dirty_rects_are_valid,
            compositor_kind,
            global_device_pixel_scale,
            occluders: Occluders::new(),
            descriptor: CompositeDescriptor::empty(),
            external_surfaces: Vec::new(),
            picture_cache_debug: PictureCacheDebugInfo::new(),
        }
    }

    /// Register an occluder during picture cache updates that can be
    /// used during frame building to occlude tiles.
    pub fn register_occluder(
        &mut self,
        z_id: ZBufferId,
        rect: WorldRect,
    ) {
        let device_rect = (rect * self.global_device_pixel_scale).round().to_i32();

        self.occluders.push(device_rect, z_id);
    }

    /// Add a picture cache to be composited
    pub fn push_surface(
        &mut self,
        tile_cache: &TileCacheInstance,
        device_clip_rect: DeviceRect,
        global_device_pixel_scale: DevicePixelScale,
        resource_cache: &ResourceCache,
        gpu_cache: &mut GpuCache,
        deferred_resolves: &mut Vec<DeferredResolve>,
    ) {
        for sub_slice in &tile_cache.sub_slices {
            let mut visible_opaque_tile_count = 0;
            let mut visible_alpha_tile_count = 0;
            let mut opaque_tile_descriptors = Vec::new();
            let mut alpha_tile_descriptors = Vec::new();
            let mut surface_device_rect = DeviceRect::zero();

            for tile in sub_slice.tiles.values() {
                if !tile.is_visible {
                    // This can occur when a tile is found to be occluded during frame building.
                    continue;
                }

                let device_rect = (tile.world_tile_rect * global_device_pixel_scale).round();
                let surface = tile.surface.as_ref().expect("no tile surface set!");

                // Accumulate this tile into the overall surface bounds. This is used below
                // to clamp the size of the supplied clip rect to a reasonable value.
                // NOTE: This clip rect must include the device_valid_rect rather than
                //       the tile device rect. This ensures that in the case of a picture
                //       cache slice that is smaller than a single tile, the clip rect in
                //       the composite descriptor will change if the position of that slice
                //       is changed. Otherwise, WR may conclude that no composite is needed
                //       if the tile itself was not invalidated due to changing content.
                //       See bug #1675414 for more detail.
                surface_device_rect = surface_device_rect.union(&tile.device_valid_rect);

                let descriptor = CompositeTileDescriptor {
                    surface_kind: surface.into(),
                    tile_id: tile.id,
                };

                let (surface, is_opaque) = match surface {
                    TileSurface::Color { color } => {
                        (CompositeTileSurface::Color { color: *color }, true)
                    }
                    TileSurface::Clear => {
                        // Clear tiles are rendered with blend mode pre-multiply-dest-out.
                        (CompositeTileSurface::Clear, false)
                    }
                    TileSurface::Texture { descriptor, .. } => {
                        let surface = descriptor.resolve(resource_cache, tile_cache.current_tile_size);
                        (
                            CompositeTileSurface::Texture { surface },
                            tile.is_opaque 
                        )
                    }
                };

                if is_opaque {
                    opaque_tile_descriptors.push(descriptor);
                    visible_opaque_tile_count += 1;
                } else {
                    alpha_tile_descriptors.push(descriptor);
                    visible_alpha_tile_count += 1;
                }

                let tile = CompositeTile {
                    kind: tile_kind(&surface, is_opaque),
                    surface,
                    rect: device_rect,
                    valid_rect: tile.device_valid_rect.translate(-device_rect.origin.to_vector()),
                    dirty_rect: tile.device_dirty_rect.translate(-device_rect.origin.to_vector()),
                    clip_rect: device_clip_rect,
                    transform: None,
                    z_id: tile.z_id,
                };

                self.tiles.push(tile);
            }

            // Sort the tile descriptor lists, since iterating values in the tile_cache.tiles
            // hashmap doesn't provide any ordering guarantees, but we want to detect the
            // composite descriptor as equal if the tiles list is the same, regardless of
            // ordering.
            opaque_tile_descriptors.sort_by_key(|desc| desc.tile_id);
            alpha_tile_descriptors.sort_by_key(|desc| desc.tile_id);

            // If the clip rect is too large, it can cause accuracy and correctness problems
            // for some native compositors (specifically, CoreAnimation in this case). To
            // work around that, intersect the supplied clip rect with the current bounds
            // of the native surface, which ensures it is a reasonable size.
            let surface_clip_rect = device_clip_rect
                .intersection(&surface_device_rect)
                .unwrap_or(DeviceRect::zero());

            // Add opaque surface before any compositor surfaces
            if visible_opaque_tile_count > 0 {
                self.descriptor.surfaces.push(
                    CompositeSurfaceDescriptor {
                        surface_id: sub_slice.native_surface.as_ref().map(|s| s.opaque),
                        clip_rect: surface_clip_rect,
                        transform: CompositorSurfaceTransform::translation(
                            tile_cache.device_position.x,
                            tile_cache.device_position.y,
                            0.0,
                        ),
                        image_dependencies: [ImageDependency::INVALID; 3],
                        image_rendering: ImageRendering::CrispEdges,
                        tile_descriptors: opaque_tile_descriptors,
                    }
                );
            }

            // Add alpha tiles after opaque surfaces
            if visible_alpha_tile_count > 0 {
                self.descriptor.surfaces.push(
                    CompositeSurfaceDescriptor {
                        surface_id: sub_slice.native_surface.as_ref().map(|s| s.alpha),
                        clip_rect: surface_clip_rect,
                        transform: CompositorSurfaceTransform::translation(
                            tile_cache.device_position.x,
                            tile_cache.device_position.y,
                            0.0,
                        ),
                        image_dependencies: [ImageDependency::INVALID; 3],
                        image_rendering: ImageRendering::CrispEdges,
                        tile_descriptors: alpha_tile_descriptors,
                    }
                );
            }

            // For each compositor surface that was promoted, build the
            // information required for the compositor to draw it
            for compositor_surface in &sub_slice.compositor_surfaces {
                let external_surface = &compositor_surface.descriptor;

                let clip_rect = external_surface
                    .clip_rect
                    .intersection(&device_clip_rect)
                    .unwrap_or_else(DeviceRect::zero);

                let required_plane_count =
                    match external_surface.dependency {
                        ExternalSurfaceDependency::Yuv { format, .. } => {
                            format.get_plane_num()
                        },
                        ExternalSurfaceDependency::Rgb { .. } => {
                            1
                        }
                    };

                let mut image_dependencies = [ImageDependency::INVALID; 3];

                for i in 0 .. required_plane_count {
                    let dependency = match external_surface.dependency {
                        ExternalSurfaceDependency::Yuv { image_dependencies, .. } => {
                            image_dependencies[i]
                        },
                        ExternalSurfaceDependency::Rgb { image_dependency, .. } => {
                            image_dependency
                        }
                    };
                    image_dependencies[i] = dependency;
                }

                // Get a new z_id for each compositor surface, to ensure correct ordering
                // when drawing with the simple (Draw) compositor, and to schedule compositing
                // of any required updates into the surfaces.
                let needs_external_surface_update = match self.compositor_kind {
                    CompositorKind::Draw { .. } => true,
                    _ => external_surface.update_params.is_some(),
                };
                let external_surface_index = if needs_external_surface_update {
                    let external_surface_index = self.compute_external_surface_dependencies(
                        &external_surface,
                        &image_dependencies,
                        required_plane_count,
                        resource_cache,
                        gpu_cache,
                        deferred_resolves,
                    );
                    if external_surface_index == ResolvedExternalSurfaceIndex::INVALID {
                        continue;
                    }
                    external_surface_index
                } else {
                    ResolvedExternalSurfaceIndex::INVALID
                };

                let surface = CompositeTileSurface::ExternalSurface { external_surface_index };
                let tile = CompositeTile {
                    kind: tile_kind(&surface, compositor_surface.is_opaque),
                    surface,
                    rect: external_surface.surface_rect,
                    valid_rect: external_surface.surface_rect.translate(-external_surface.surface_rect.origin.to_vector()),
                    dirty_rect: external_surface.surface_rect.translate(-external_surface.surface_rect.origin.to_vector()),
                    clip_rect,
                    transform: Some(external_surface.transform),
                    z_id: external_surface.z_id,
                };

                // Add a surface descriptor for each compositor surface. For the Draw
                // compositor, this is used to avoid composites being skipped by adding
                // a dependency on the compositor surface external image keys / generations.
                self.descriptor.surfaces.push(
                    CompositeSurfaceDescriptor {
                        surface_id: external_surface.native_surface_id,
                        clip_rect,
                        transform: external_surface.transform,
                        image_dependencies: image_dependencies,
                        image_rendering: external_surface.image_rendering,
                        tile_descriptors: Vec::new(),
                    }
                );

                self.tiles.push(tile);
            }
        }
    }

    fn compute_external_surface_dependencies(
        &mut self,
        external_surface: &ExternalSurfaceDescriptor,
        image_dependencies: &[ImageDependency; 3],
        required_plane_count: usize,
        resource_cache: &ResourceCache,
        gpu_cache: &mut GpuCache,
        deferred_resolves: &mut Vec<DeferredResolve>,
    ) -> ResolvedExternalSurfaceIndex {
        let mut planes = [
            ExternalPlaneDescriptor::invalid(),
            ExternalPlaneDescriptor::invalid(),
            ExternalPlaneDescriptor::invalid(),
        ];

        let mut valid_plane_count = 0;
        for i in 0 .. required_plane_count {
            let request = ImageRequest {
                key: image_dependencies[i].key,
                rendering: external_surface.image_rendering,
                tile: None,
            };

            let cache_item = resolve_image(
                request,
                resource_cache,
                gpu_cache,
                deferred_resolves,
            );

            if cache_item.texture_id != TextureSource::Invalid {
                valid_plane_count += 1;
                let plane = &mut planes[i];
                *plane = ExternalPlaneDescriptor {
                    texture: cache_item.texture_id,
                    uv_rect: cache_item.uv_rect.into(),
                };
            }
        }

        // Check if there are valid images added for each YUV plane
        if valid_plane_count < required_plane_count {
            warn!("Warnings: skip a YUV/RGB compositor surface, found {}/{} valid images",
                valid_plane_count,
                required_plane_count,
            );
            return ResolvedExternalSurfaceIndex::INVALID;
        }

        let external_surface_index = ResolvedExternalSurfaceIndex(self.external_surfaces.len());

        // If the external surface descriptor reports that the native surface
        // needs to be updated, create an update params tuple for the renderer
        // to use.
        let update_params = external_surface.update_params.map(|surface_size| {
            (
                external_surface.native_surface_id.expect("bug: no native surface!"),
                surface_size
            )
        });

        match external_surface.dependency {
            ExternalSurfaceDependency::Yuv{ color_space, format, rescale, .. } => {

                let image_buffer_kind = planes[0].texture.image_buffer_kind();

                self.external_surfaces.push(ResolvedExternalSurface {
                    color_data: ResolvedExternalSurfaceColorData::Yuv {
                        image_dependencies: *image_dependencies,
                        planes,
                        color_space,
                        format,
                        rescale,
                    },
                    image_buffer_kind,
                    update_params,
                });
            },
            ExternalSurfaceDependency::Rgb{ flip_y, .. } => {

                let image_buffer_kind = planes[0].texture.image_buffer_kind();

                // Only propagate flip_y if the compositor doesn't support transforms,
                // since otherwise it'll be handled as part of the transform.
                self.external_surfaces.push(ResolvedExternalSurface {
                    color_data: ResolvedExternalSurfaceColorData::Rgb {
                        image_dependency: image_dependencies[0],
                        plane: planes[0],
                        flip_y: flip_y && !self.compositor_kind.supports_transforms(),
                    },
                    image_buffer_kind,
                    update_params,
                });
            },
        }
        external_surface_index
    }

    pub fn end_frame(&mut self) {
        // Sort tiles from front to back.
        self.tiles.sort_by_key(|tile| tile.z_id.0);
    }
}

/// An arbitrary identifier for a native (OS compositor) surface
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct NativeSurfaceId(pub u64);

impl NativeSurfaceId {
    /// A special id for the native surface that is used for debug / profiler overlays.
    pub const DEBUG_OVERLAY: NativeSurfaceId = NativeSurfaceId(u64::MAX);
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct NativeTileId {
    pub surface_id: NativeSurfaceId,
    pub x: i32,
    pub y: i32,
}

impl NativeTileId {
    /// A special id for the native surface that is used for debug / profiler overlays.
    pub const DEBUG_OVERLAY: NativeTileId = NativeTileId {
        surface_id: NativeSurfaceId::DEBUG_OVERLAY,
        x: 0,
        y: 0,
    };
}

/// Information about a bound surface that the native compositor
/// returns to WR.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct NativeSurfaceInfo {
    /// An offset into the surface that WR should draw. Some compositing
    /// implementations (notably, DirectComposition) use texture atlases
    /// when the surface sizes are small. In this case, an offset can
    /// be returned into the larger texture where WR should draw. This
    /// can be (0, 0) if texture atlases are not used.
    pub origin: DeviceIntPoint,
    /// The ID of the FBO that WR should bind to, in order to draw to
    /// the bound surface. On Windows (ANGLE) this will always be 0,
    /// since creating a p-buffer sets the default framebuffer to
    /// be the DirectComposition surface. On Mac, this will be non-zero,
    /// since it identifies the IOSurface that has been bound to draw to.
    // TODO(gw): This may need to be a larger / different type for WR
    //           backends that are not GL.
    pub fbo_id: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct CompositorCapabilities {
    /// The virtual surface size used by the underlying platform.
    pub virtual_surface_size: i32,
    /// Whether the compositor requires redrawing on invalidation.
    pub redraw_on_invalidation: bool,
}

impl Default for CompositorCapabilities {
    fn default() -> Self {
        // The default set of compositor capabilities for a given platform.
        // These should only be modified if a compositor diverges specifically
        // from the default behavior so that compositors don't have to track
        // which changes to this structure unless necessary.
        CompositorCapabilities {
            virtual_surface_size: 0,
            redraw_on_invalidation: false,
        }
    }
}

/// The transform type to apply to Compositor surfaces.
// TODO: Should transform from CompositorSurfacePixel instead, but this requires a cleanup of the
// Compositor API to use CompositorSurface-space geometry instead of Device-space where necessary
// to avoid a bunch of noisy cast_unit calls and make it actually type-safe. May be difficult due
// to pervasive use of Device-space nomenclature inside WR.
// pub struct CompositorSurfacePixel;
// pub type CompositorSurfaceTransform = Transform3D<f32, CompositorSurfacePixel, DevicePixel>;
pub type CompositorSurfaceTransform = Transform3D<f32, DevicePixel, DevicePixel>;

/// Defines an interface to a native (OS level) compositor. If supplied
/// by the client application, then picture cache slices will be
/// composited by the OS compositor, rather than drawn via WR batches.
pub trait Compositor {
    /// Create a new OS compositor surface with the given properties.
    fn create_surface(
        &mut self,
        id: NativeSurfaceId,
        virtual_offset: DeviceIntPoint,
        tile_size: DeviceIntSize,
        is_opaque: bool,
    );

    /// Create a new OS compositor surface that can be used with an
    /// existing ExternalImageId, instead of being drawn to by WebRender.
    /// Surfaces created by this can only be used with attach_external_image,
    /// and not create_tile/destroy_tile/bind/unbind.
    fn create_external_surface(
        &mut self,
        id: NativeSurfaceId,
        is_opaque: bool,
    );

    /// Destroy the surface with the specified id. WR may call this
    /// at any time the surface is no longer required (including during
    /// renderer deinit). It's the responsibility of the embedder
    /// to ensure that the surface is only freed once the GPU is
    /// no longer using the surface (if this isn't already handled
    /// by the operating system).
    fn destroy_surface(
        &mut self,
        id: NativeSurfaceId,
    );

    /// Create a new OS compositor tile with the given properties.
    fn create_tile(
        &mut self,
        id: NativeTileId,
    );

    /// Destroy an existing compositor tile.
    fn destroy_tile(
        &mut self,
        id: NativeTileId,
    );

    /// Attaches an ExternalImageId to an OS compositor surface created
    /// by create_external_surface, and uses that as the contents of
    /// the surface. It is expected that a single surface will have
    /// many different images attached (like one for each video frame).
    fn attach_external_image(
        &mut self,
        id: NativeSurfaceId,
        external_image: ExternalImageId
    );

    /// Mark a tile as invalid before any surfaces are queued for
    /// composition and before it is updated with bind. This is useful
    /// for early composition, allowing for dependency tracking of which
    /// surfaces can be composited early while others are still updating.
    fn invalidate_tile(
        &mut self,
        _id: NativeTileId,
        _valid_rect: DeviceIntRect
    ) {}

    /// Bind this surface such that WR can issue OpenGL commands
    /// that will target the surface. Returns an (x, y) offset
    /// where WR should draw into the surface. This can be set
    /// to (0, 0) if the OS doesn't use texture atlases. The dirty
    /// rect is a local surface rect that specifies which part
    /// of the surface needs to be updated. If max_update_rects
    /// in CompositeConfig is 0, this will always be the size
    /// of the entire surface. The returned offset is only
    /// relevant to compositors that store surfaces in a texture
    /// atlas (that is, WR expects that the dirty rect doesn't
    /// affect the coordinates of the returned origin).
    fn bind(
        &mut self,
        id: NativeTileId,
        dirty_rect: DeviceIntRect,
        valid_rect: DeviceIntRect,
    ) -> NativeSurfaceInfo;

    /// Unbind the surface. This is called by WR when it has
    /// finished issuing OpenGL commands on the current surface.
    fn unbind(
        &mut self,
    );

    /// Begin the frame
    fn begin_frame(&mut self);

    /// Add a surface to the visual tree to be composited. Visuals must
    /// be added every frame, between the begin/end transaction call. The
    /// z-order of the surfaces is determined by the order they are added
    /// to the visual tree.
    // TODO(gw): Adding visuals every frame makes the interface simple,
    //           but may have performance implications on some compositors?
    //           We might need to change the interface to maintain a visual
    //           tree that can be mutated?
    // TODO(gw): We might need to add a concept of a hierachy in future.
    fn add_surface(
        &mut self,
        id: NativeSurfaceId,
        transform: CompositorSurfaceTransform,
        clip_rect: DeviceIntRect,
        image_rendering: ImageRendering,
    );

    /// Notify the compositor that all tiles have been invalidated and all
    /// native surfaces have been added, thus it is safe to start compositing
    /// valid surfaces. The dirty rects array allows native compositors that
    /// support partial present to skip copying unchanged areas.
    /// Optionally provides a set of rectangles for the areas known to be
    /// opaque, this is currently only computed if the caller is SwCompositor.
    fn start_compositing(
        &mut self,
        _dirty_rects: &[DeviceIntRect],
        _opaque_rects: &[DeviceIntRect],
    ) {}

    /// Commit any changes in the compositor tree for this frame. WR calls
    /// this once when all surface and visual updates are complete, to signal
    /// that the OS composite transaction should be applied.
    fn end_frame(&mut self);

    /// Enable/disable native compositor usage
    fn enable_native_compositor(&mut self, enable: bool);

    /// Safely deinitialize any remaining resources owned by the compositor.
    fn deinit(&mut self);

    /// Get the capabilities struct for this compositor. This is used to
    /// specify what features a compositor supports, depending on the
    /// underlying platform
    fn get_capabilities(&self) -> CompositorCapabilities;
}

/// Information about the underlying data buffer of a mapped tile.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MappedTileInfo {
    pub data: *mut c_void,
    pub stride: i32,
}

/// Descriptor for a locked surface that will be directly composited by SWGL.
#[repr(C)]
pub struct SWGLCompositeSurfaceInfo {
    /// The number of YUV planes in the surface. 0 indicates non-YUV BGRA.
    /// 1 is interleaved YUV. 2 is NV12. 3 is planar YUV.
    pub yuv_planes: u32,
    /// Textures for planes of the surface, or 0 if not applicable.
    pub textures: [u32; 3],
    /// Color space of surface if using a YUV format.
    pub color_space: YuvColorSpace,
    /// Color depth of surface if using a YUV format.
    pub color_depth: ColorDepth,
    /// The actual source surface size before transformation.
    pub size: DeviceIntSize,
}

/// A Compositor variant that supports mapping tiles into CPU memory.
pub trait MappableCompositor: Compositor {
    /// Map a tile's underlying buffer so it can be used as the backing for
    /// a SWGL framebuffer. This is intended to be a replacement for 'bind'
    /// in any compositors that intend to directly interoperate with SWGL
    /// while supporting some form of native layers.
    fn map_tile(
        &mut self,
        id: NativeTileId,
        dirty_rect: DeviceIntRect,
        valid_rect: DeviceIntRect,
    ) -> Option<MappedTileInfo>;

    /// Unmap a tile that was was previously mapped via map_tile to signal
    /// that SWGL is done rendering to the buffer.
    fn unmap_tile(&mut self);

    fn lock_composite_surface(
        &mut self,
        ctx: *mut c_void,
        external_image_id: ExternalImageId,
        composite_info: *mut SWGLCompositeSurfaceInfo,
    ) -> bool;
    fn unlock_composite_surface(&mut self, ctx: *mut c_void, external_image_id: ExternalImageId);
}

/// Defines an interface to a non-native (application-level) Compositor which handles
/// partial present. This is required if webrender must query the backbuffer's age.
/// TODO: Use the Compositor trait for native and non-native compositors, and integrate
/// this functionality there.
pub trait PartialPresentCompositor {
    /// Allows webrender to specify the total region that will be rendered to this frame,
    /// ie the frame's dirty region and some previous frames' dirty regions, if applicable
    /// (calculated using the buffer age). Must be called before anything has been rendered
    /// to the main framebuffer.
    fn set_buffer_damage_region(&mut self, rects: &[DeviceIntRect]);
}

/// Information about an opaque surface used to occlude tiles.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct Occluder {
    z_id: ZBufferId,
    device_rect: DeviceIntRect,
}

// Whether this event is the start or end of a rectangle
#[derive(Debug)]
enum OcclusionEventKind {
    Begin,
    End,
}

// A list of events on the y-axis, with the rectangle range that it affects on the x-axis
#[derive(Debug)]
struct OcclusionEvent {
    y: i32,
    x_range: ops::Range<i32>,
    kind: OcclusionEventKind,
}

impl OcclusionEvent {
    fn new(y: i32, kind: OcclusionEventKind, x0: i32, x1: i32) -> Self {
        OcclusionEvent {
            y,
            x_range: ops::Range {
                start: x0,
                end: x1,
            },
            kind,
        }
    }
}

/// List of registered occluders.
///
/// Also store a couple of vectors for reuse.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct Occluders {
    occluders: Vec<Occluder>,

    // The two vectors below are kept to avoid unnecessary reallocations in area().

    #[cfg_attr(feature = "serde", serde(skip))]
    events: Vec<OcclusionEvent>,

    #[cfg_attr(feature = "serde", serde(skip))]
    active: Vec<ops::Range<i32>>,
}

impl Occluders {
    fn new() -> Self {
        Occluders {
            occluders: Vec::new(),
            events: Vec::new(),
            active: Vec::new(),
        }
    }

    fn push(&mut self, device_rect: DeviceIntRect, z_id: ZBufferId) {
        self.occluders.push(Occluder { device_rect, z_id });
    }

    /// Returns true if a tile with the specified rectangle and z_id
    /// is occluded by an opaque surface in front of it.
    pub fn is_tile_occluded(
        &mut self,
        z_id: ZBufferId,
        device_rect: DeviceRect,
    ) -> bool {
        // It's often the case that a tile is only occluded by considering multiple
        // picture caches in front of it (for example, the background tiles are
        // often occluded by a combination of the content slice + the scrollbar slices).

        // The basic algorithm is:
        //    For every occluder:
        //      If this occluder is in front of the tile we are querying:
        //         Clip the occluder rectangle to the query rectangle.
        //    Calculate the total non-overlapping area of those clipped occluders.
        //    If the cumulative area of those occluders is the same as the area of the query tile,
        //       Then the entire tile must be occluded and can be skipped during rasterization and compositing.

        // Get the reference area we will compare against.
        let device_rect = device_rect.round().to_i32();
        let ref_area = device_rect.size.width * device_rect.size.height;

        // Calculate the non-overlapping area of the valid occluders.
        let cover_area = self.area(z_id, &device_rect);
        debug_assert!(cover_area <= ref_area);

        // Check if the tile area is completely covered
        ref_area == cover_area
    }

    /// Return the total area covered by a set of occluders, accounting for
    /// overlapping areas between those rectangles.
    fn area(
        &mut self,
        z_id: ZBufferId,
        clip_rect: &DeviceIntRect,
    ) -> i32 {
        // This implementation is based on the article https://leetcode.com/articles/rectangle-area-ii/.
        // This is not a particularly efficient implementation (it skips building segment trees), however
        // we typically use this where the length of the rectangles array is < 10, so simplicity is more important.

        self.events.clear();
        self.active.clear();

        let mut area = 0;

        // Step through each rectangle and build the y-axis event list
        for occluder in &self.occluders {
            // Only consider occluders in front of this rect
            if occluder.z_id.0 < z_id.0 {
                // Clip the source rect to the rectangle we care about, since we only
                // want to record area for the tile we are comparing to.
                if let Some(rect) = occluder.device_rect.intersection(clip_rect) {
                    let x0 = rect.origin.x;
                    let x1 = x0 + rect.size.width;
                    self.events.push(OcclusionEvent::new(rect.origin.y, OcclusionEventKind::Begin, x0, x1));
                    self.events.push(OcclusionEvent::new(rect.origin.y + rect.size.height, OcclusionEventKind::End, x0, x1));
                }
            }
        }

        // If we didn't end up with any valid events, the area must be 0
        if self.events.is_empty() {
            return 0;
        }

        // Sort the events by y-value
        self.events.sort_by_key(|e| e.y);
        let mut cur_y = self.events[0].y;

        // Step through each y interval
        for event in &self.events {
            // This is the dimension of the y-axis we are accumulating areas for
            let dy = event.y - cur_y;

            // If we have active events covering x-ranges in this y-interval, process them
            if dy != 0 && !self.active.is_empty() {
                assert!(dy > 0);

                // Step through the x-ranges, ordered by x0 of each event
                self.active.sort_by_key(|i| i.start);
                let mut query = 0;
                let mut cur = self.active[0].start;

                // Accumulate the non-overlapping x-interval that contributes to area for this y-interval.
                for interval in &self.active {
                    cur = interval.start.max(cur);
                    query += (interval.end - cur).max(0);
                    cur = cur.max(interval.end);
                }

                // Accumulate total area for this y-interval
                area += query * dy;
            }

            // Update the active events list
            match event.kind {
                OcclusionEventKind::Begin => {
                    self.active.push(event.x_range.clone());
                }
                OcclusionEventKind::End => {
                    let index = self.active.iter().position(|i| *i == event.x_range).unwrap();
                    self.active.remove(index);
                }
            }

            cur_y = event.y;
        }

        area
    }
}
