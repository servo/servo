/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ColorF, YuvColorSpace, YuvFormat, ImageRendering};
use api::units::{DeviceRect, DeviceIntSize, DeviceIntRect, DeviceIntPoint, WorldRect};
use api::units::{DevicePixelScale, DevicePoint, PictureRect, TexelRect};
use crate::batch::{resolve_image, get_buffer_kind};
use crate::gpu_cache::GpuCache;
use crate::gpu_types::{ZBufferId, ZBufferIdGenerator};
use crate::internal_types::TextureSource;
use crate::picture::{ImageDependency, ResolvedSurfaceTexture, TileCacheInstance, TileId, TileSurface};
use crate::prim_store::DeferredResolve;
use crate::renderer::ImageBufferKind;
use crate::resource_cache::{ImageRequest, ResourceCache};
use std::{ops, u64};

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
    DestroySurface {
        id: NativeSurfaceId,
    },
    CreateTile {
        id: NativeTileId,
    },
    DestroyTile {
        id: NativeTileId,
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

/// Describes the geometry and surface of a tile to be composited
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct CompositeTile {
    pub surface: CompositeTileSurface,
    pub rect: DeviceRect,
    pub clip_rect: DeviceRect,
    pub dirty_rect: DeviceRect,
    pub valid_rect: DeviceRect,
    pub z_id: ZBufferId,
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
    pub local_rect: PictureRect,
    pub world_rect: WorldRect,
    pub device_rect: DeviceRect,
    pub local_clip_rect: PictureRect,
    pub clip_rect: DeviceRect,
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
    pub texture_layer: i32,
    pub uv_rect: TexelRect,
}

impl ExternalPlaneDescriptor {
    fn invalid() -> Self {
        ExternalPlaneDescriptor {
            texture: TextureSource::Invalid,
            texture_layer: 0,
            uv_rect: TexelRect::invalid(),
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct ResolvedExternalSurfaceIndex(pub usize);

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
        /// If this is true, WR would draw the previous frame's dirty region when
        /// doing a partial present. This is used for EGL which requires the front
        /// buffer to always be fully consistent.
        draw_previous_partial_present_regions: bool,
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
}

impl Default for CompositorConfig {
    /// Default compositor config is full present without partial present.
    fn default() -> Self {
        CompositorConfig::Draw {
            max_partial_present_rects: 0,
            draw_previous_partial_present_regions: false,
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
        /// The virtual surface size used by underlying platform.
        virtual_surface_size: i32,
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
            CompositorKind::Native { virtual_surface_size, .. } => *virtual_surface_size,
        }
    }
}

/// Information about an opaque surface used to occlude tiles.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct Occluder {
    z_id: ZBufferId,
    device_rect: DeviceIntRect,
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
    pub offset: DevicePoint,
    pub clip_rect: DeviceRect,
    // A list of image keys and generations that this compositor surface
    // depends on. This avoids composites being skipped when the only
    // thing that has changed is the generation of an compositor surface
    // image dependency.
    pub image_dependencies: [ImageDependency; 3],
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

/// The list of tiles to be drawn this frame
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct CompositeState {
    // TODO(gw): Consider splitting up CompositeState into separate struct types depending
    //           on the selected compositing mode. Many of the fields in this state struct
    //           are only applicable to either Native or Draw compositing mode.
    /// List of opaque tiles to be drawn by the Draw compositor.
    pub opaque_tiles: Vec<CompositeTile>,
    /// List of alpha tiles to be drawn by the Draw compositor.
    pub alpha_tiles: Vec<CompositeTile>,
    /// List of clear tiles to be drawn by the Draw compositor.
    pub clear_tiles: Vec<CompositeTile>,
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
    /// Picture caching may be disabled dynamically, based on debug flags, pinch zoom etc.
    pub picture_caching_is_enabled: bool,
    /// The overall device pixel scale, used for tile occlusion conversions.
    global_device_pixel_scale: DevicePixelScale,
    /// List of registered occluders
    occluders: Vec<Occluder>,
    /// Description of the surfaces and properties that are being composited.
    pub descriptor: CompositeDescriptor,
}

impl CompositeState {
    /// Construct a new state for compositing picture tiles. This is created
    /// during each frame construction and passed to the renderer.
    pub fn new(
        compositor_kind: CompositorKind,
        mut picture_caching_is_enabled: bool,
        global_device_pixel_scale: DevicePixelScale,
        max_depth_ids: i32,
    ) -> Self {
        // The native compositor interface requires picture caching to work, so
        // force it here and warn if it was disabled.
        if let CompositorKind::Native { .. } = compositor_kind {
            if !picture_caching_is_enabled {
                warn!("Picture caching cannot be disabled in native compositor config");
            }
            picture_caching_is_enabled = true;
        }

        CompositeState {
            opaque_tiles: Vec::new(),
            alpha_tiles: Vec::new(),
            clear_tiles: Vec::new(),
            z_generator: ZBufferIdGenerator::new(0, max_depth_ids),
            dirty_rects_are_valid: true,
            compositor_kind,
            picture_caching_is_enabled,
            global_device_pixel_scale,
            occluders: Vec::new(),
            descriptor: CompositeDescriptor::empty(),
            external_surfaces: Vec::new(),
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

        self.occluders.push(Occluder {
            device_rect,
            z_id,
        });
    }

    /// Returns true if a tile with the specified rectangle and z_id
    /// is occluded by an opaque surface in front of it.
    pub fn is_tile_occluded(
        &self,
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
        let cover_area = area_of_occluders(&self.occluders, z_id, &device_rect);
        debug_assert!(cover_area <= ref_area);

        // Check if the tile area is completely covered
        ref_area == cover_area
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
        let mut visible_opaque_tile_count = 0;
        let mut visible_alpha_tile_count = 0;
        let mut opaque_tile_descriptors = Vec::new();
        let mut alpha_tile_descriptors = Vec::new();

        for tile in tile_cache.tiles.values() {
            if !tile.is_visible {
                // This can occur when a tile is found to be occluded during frame building.
                continue;
            }

            let device_rect = (tile.world_tile_rect * global_device_pixel_scale).round();
            let surface = tile.surface.as_ref().expect("no tile surface set!");

            let descriptor = CompositeTileDescriptor {
                surface_kind: surface.into(),
                tile_id: tile.id,
            };

            let (surface, is_opaque) = match surface {
                TileSurface::Color { color } => {
                    (CompositeTileSurface::Color { color: *color }, true)
                }
                TileSurface::Clear => {
                    (CompositeTileSurface::Clear, false)
                }
                TileSurface::Texture { descriptor, .. } => {
                    let surface = descriptor.resolve(resource_cache, tile_cache.current_tile_size);
                    (
                        CompositeTileSurface::Texture { surface },
                        // If a tile has compositor surface intersecting with it, we need to
                        // respect the tile.is_opaque property even if the overall tile cache
                        // is opaque. In this case, the tile.is_opaque property is required
                        // in order to ensure correct draw order with compositor surfaces.
                        tile.is_opaque || (!tile.has_compositor_surface && tile_cache.is_opaque()),
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
                surface,
                rect: device_rect,
                valid_rect: tile.device_valid_rect.translate(-device_rect.origin.to_vector()),
                dirty_rect: tile.device_dirty_rect.translate(-device_rect.origin.to_vector()),
                clip_rect: device_clip_rect,
                z_id: tile.z_id,
            };

            self.push_tile(tile, is_opaque);
        }

        // Sort the tile descriptor lists, since iterating values in the tile_cache.tiles
        // hashmap doesn't provide any ordering guarantees, but we want to detect the
        // composite descriptor as equal if the tiles list is the same, regardless of
        // ordering.
        opaque_tile_descriptors.sort_by_key(|desc| desc.tile_id);
        alpha_tile_descriptors.sort_by_key(|desc| desc.tile_id);

        // Add opaque surface before any compositor surfaces
        if visible_opaque_tile_count > 0 {
            self.descriptor.surfaces.push(
                CompositeSurfaceDescriptor {
                    surface_id: tile_cache.native_surface.as_ref().map(|s| s.opaque),
                    offset: tile_cache.device_position,
                    clip_rect: device_clip_rect,
                    image_dependencies: [ImageDependency::INVALID; 3],
                    tile_descriptors: opaque_tile_descriptors,
                }
            );
        }

        // For each compositor surface that was promoted, build the
        // information required for the compositor to draw it
        for external_surface in &tile_cache.external_surfaces {

            let mut planes = [
                ExternalPlaneDescriptor::invalid(),
                ExternalPlaneDescriptor::invalid(),
                ExternalPlaneDescriptor::invalid(),
            ];

            // Step through the image keys, and build a plane descriptor for each
            let required_plane_count =
                match external_surface.dependency {
                    ExternalSurfaceDependency::Yuv { format, .. } => {
                        format.get_plane_num()
                    },
                    ExternalSurfaceDependency::Rgb { .. } => {
                        1
                    }
                };
            let mut valid_plane_count = 0;

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

                let request = ImageRequest {
                    key: dependency.key,
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
                        texture_layer: cache_item.texture_layer,
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
                continue;
            }

            let clip_rect = external_surface
                .clip_rect
                .intersection(&device_clip_rect)
                .unwrap_or_else(DeviceRect::zero);

            // Get a new z_id for each compositor surface, to ensure correct ordering
            // when drawing with the simple (Draw) compositor.

            let surface = CompositeTileSurface::ExternalSurface {
                external_surface_index: ResolvedExternalSurfaceIndex(self.external_surfaces.len()),
            };

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

                    let image_buffer_kind = get_buffer_kind(planes[0].texture);

                    self.external_surfaces.push(ResolvedExternalSurface {
                        color_data: ResolvedExternalSurfaceColorData::Yuv {
                            image_dependencies,
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

                    let image_buffer_kind = get_buffer_kind(planes[0].texture);

                    self.external_surfaces.push(ResolvedExternalSurface {
                        color_data: ResolvedExternalSurfaceColorData::Rgb {
                            image_dependency: image_dependencies[0],
                            plane: planes[0],
                            flip_y,
                        },
                        image_buffer_kind,
                        update_params,
                    });
                },
            }

            let tile = CompositeTile {
                surface,
                rect: external_surface.device_rect,
                valid_rect: external_surface.device_rect.translate(-external_surface.device_rect.origin.to_vector()),
                dirty_rect: external_surface.device_rect.translate(-external_surface.device_rect.origin.to_vector()),
                clip_rect,
                z_id: external_surface.z_id,
            };

            // Add a surface descriptor for each compositor surface. For the Draw
            // compositor, this is used to avoid composites being skipped by adding
            // a dependency on the compositor surface external image keys / generations.
            self.descriptor.surfaces.push(
                CompositeSurfaceDescriptor {
                    surface_id: external_surface.native_surface_id,
                    offset: tile.rect.origin,
                    clip_rect: tile.clip_rect,
                    image_dependencies: image_dependencies,
                    tile_descriptors: Vec::new(),
                }
            );

            self.push_tile(tile, true);
        }

        // Add alpha / overlay tiles after compositor surfaces
        if visible_alpha_tile_count > 0 {
            self.descriptor.surfaces.push(
                CompositeSurfaceDescriptor {
                    surface_id: tile_cache.native_surface.as_ref().map(|s| s.alpha),
                    offset: tile_cache.device_position,
                    clip_rect: device_clip_rect,
                    image_dependencies: [ImageDependency::INVALID; 3],
                    tile_descriptors: alpha_tile_descriptors,
                }
            );
        }
    }

    /// Add a tile to the appropriate array, depending on tile properties and compositor mode.
    fn push_tile(
        &mut self,
        tile: CompositeTile,
        is_opaque: bool,
    ) {
        match tile.surface {
            CompositeTileSurface::Color { .. } => {
                // Color tiles are, by definition, opaque. We might support non-opaque color
                // tiles if we ever find pages that have a lot of these.
                self.opaque_tiles.push(tile);
            }
            CompositeTileSurface::Clear => {
                // Clear tiles have a special bucket
                self.clear_tiles.push(tile);
            }
            CompositeTileSurface::Texture { .. } => {
                // Texture surfaces get bucketed by opaque/alpha, for z-rejection
                // on the Draw compositor mode.
                if is_opaque {
                    self.opaque_tiles.push(tile);
                } else {
                    self.alpha_tiles.push(tile);
                }
            }
            CompositeTileSurface::ExternalSurface { .. } => {
                self.opaque_tiles.push(tile);
            }
        }
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
pub struct CompositorCapabilities {
    pub virtual_surface_size: i32,
}

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
    // TODO(gw): In future, expand to support a more complete transform matrix.
    fn add_surface(
        &mut self,
        id: NativeSurfaceId,
        position: DeviceIntPoint,
        clip_rect: DeviceIntRect,
    );

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

/// Return the total area covered by a set of occluders, accounting for
/// overlapping areas between those rectangles.
fn area_of_occluders(
    occluders: &[Occluder],
    z_id: ZBufferId,
    clip_rect: &DeviceIntRect,
) -> i32 {
    // This implementation is based on the article https://leetcode.com/articles/rectangle-area-ii/.
    // This is not a particularly efficient implementation (it skips building segment trees), however
    // we typically use this where the length of the rectangles array is < 10, so simplicity is more important.

    let mut area = 0;

    // Whether this event is the start or end of a rectangle
    #[derive(Debug)]
    enum EventKind {
        Begin,
        End,
    }

    // A list of events on the y-axis, with the rectangle range that it affects on the x-axis
    #[derive(Debug)]
    struct Event {
        y: i32,
        x_range: ops::Range<i32>,
        kind: EventKind,
    }

    impl Event {
        fn new(y: i32, kind: EventKind, x0: i32, x1: i32) -> Self {
            Event {
                y,
                x_range: ops::Range {
                    start: x0,
                    end: x1,
                },
                kind,
            }
        }
    }

    // Step through each rectangle and build the y-axis event list
    let mut events = Vec::with_capacity(occluders.len() * 2);
    for occluder in occluders {
        // Only consider occluders in front of this rect
        if occluder.z_id.0 > z_id.0 {
            // Clip the source rect to the rectangle we care about, since we only
            // want to record area for the tile we are comparing to.
            if let Some(rect) = occluder.device_rect.intersection(clip_rect) {
                let x0 = rect.origin.x;
                let x1 = x0 + rect.size.width;
                events.push(Event::new(rect.origin.y, EventKind::Begin, x0, x1));
                events.push(Event::new(rect.origin.y + rect.size.height, EventKind::End, x0, x1));
            }
        }
    }

    // If we didn't end up with any valid events, the area must be 0
    if events.is_empty() {
        return 0;
    }

    // Sort the events by y-value
    events.sort_by_key(|e| e.y);
    let mut active: Vec<ops::Range<i32>> = Vec::new();
    let mut cur_y = events[0].y;

    // Step through each y interval
    for event in &events {
        // This is the dimension of the y-axis we are accumulating areas for
        let dy = event.y - cur_y;

        // If we have active events covering x-ranges in this y-interval, process them
        if dy != 0 && !active.is_empty() {
            assert!(dy > 0);

            // Step through the x-ranges, ordered by x0 of each event
            active.sort_by_key(|i| i.start);
            let mut query = 0;
            let mut cur = active[0].start;

            // Accumulate the non-overlapping x-interval that contributes to area for this y-interval.
            for interval in &active {
                cur = interval.start.max(cur);
                query += (interval.end - cur).max(0);
                cur = cur.max(interval.end);
            }

            // Accumulate total area for this y-interval
            area += query * dy;
        }

        // Update the active events list
        match event.kind {
            EventKind::Begin => {
                active.push(event.x_range.clone());
            }
            EventKind::End => {
                let index = active.iter().position(|i| *i == event.x_range).unwrap();
                active.remove(index);
            }
        }

        cur_y = event.y;
    }

    area
}
