/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ColorF, FontInstanceFlags, GlyphInstance, RasterSpace, Shadow};
use api::units::{LayoutToWorldTransform, LayoutVector2D};
use crate::scene_building::{CreateShadow, IsVisible};
use crate::frame_builder::FrameBuildingState;
use crate::glyph_rasterizer::{FontInstance, FontTransform, GlyphKey, FONT_SIZE_LIMIT};
use crate::gpu_cache::GpuCache;
use crate::intern;
use crate::internal_types::LayoutPrimitiveInfo;
use crate::picture::SurfaceInfo;
use crate::prim_store::{PrimitiveOpacity,  PrimitiveScratchBuffer};
use crate::prim_store::{PrimitiveStore, PrimKeyCommonData, PrimTemplateCommonData};
use crate::renderer::{MAX_VERTEX_TEXTURE_WIDTH};
use crate::resource_cache::{ResourceCache};
use crate::util::{MatrixHelpers};
use crate::prim_store::{InternablePrimitive, PrimitiveInstanceKind};
use crate::spatial_tree::{SpatialTree, SpatialNodeIndex, ROOT_SPATIAL_NODE_INDEX};
use crate::space::SpaceSnapper;
use crate::util::PrimaryArc;

use std::ops;
use std::sync::Arc;

use super::storage;

/// A run of glyphs, with associated font information.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
pub struct TextRunKey {
    pub common: PrimKeyCommonData,
    pub font: FontInstance,
    pub glyphs: PrimaryArc<Vec<GlyphInstance>>,
    pub shadow: bool,
    pub requested_raster_space: RasterSpace,
}

impl TextRunKey {
    pub fn new(
        info: &LayoutPrimitiveInfo,
        text_run: TextRun,
    ) -> Self {
        TextRunKey {
            common: info.into(),
            font: text_run.font,
            glyphs: PrimaryArc(text_run.glyphs),
            shadow: text_run.shadow,
            requested_raster_space: text_run.requested_raster_space,
        }
    }
}

impl intern::InternDebug for TextRunKey {}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct TextRunTemplate {
    pub common: PrimTemplateCommonData,
    pub font: FontInstance,
    #[ignore_malloc_size_of = "Measured via PrimaryArc"]
    pub glyphs: Arc<Vec<GlyphInstance>>,
}

impl ops::Deref for TextRunTemplate {
    type Target = PrimTemplateCommonData;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl ops::DerefMut for TextRunTemplate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

impl From<TextRunKey> for TextRunTemplate {
    fn from(item: TextRunKey) -> Self {
        let common = PrimTemplateCommonData::with_key_common(item.common);
        TextRunTemplate {
            common,
            font: item.font,
            glyphs: item.glyphs.0,
        }
    }
}

impl TextRunTemplate {
    /// Update the GPU cache for a given primitive template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        frame_state: &mut FrameBuildingState,
    ) {
        self.write_prim_gpu_blocks(frame_state);
        self.opacity = PrimitiveOpacity::translucent();
    }

    fn write_prim_gpu_blocks(
        &mut self,
        frame_state: &mut FrameBuildingState,
    ) {
        // corresponds to `fetch_glyph` in the shaders
        if let Some(mut request) = frame_state.gpu_cache.request(&mut self.common.gpu_cache_handle) {
            request.push(ColorF::from(self.font.color).premultiplied());
            // this is the only case where we need to provide plain color to GPU
            let bg_color = ColorF::from(self.font.bg_color);
            request.push([bg_color.r, bg_color.g, bg_color.b, 1.0]);

            let mut gpu_block = [0.0; 4];
            for (i, src) in self.glyphs.iter().enumerate() {
                // Two glyphs are packed per GPU block.

                if (i & 1) == 0 {
                    gpu_block[0] = src.point.x;
                    gpu_block[1] = src.point.y;
                } else {
                    gpu_block[2] = src.point.x;
                    gpu_block[3] = src.point.y;
                    request.push(gpu_block);
                }
            }

            // Ensure the last block is added in the case
            // of an odd number of glyphs.
            if (self.glyphs.len() & 1) != 0 {
                request.push(gpu_block);
            }

            assert!(request.current_used_block_num() <= MAX_VERTEX_TEXTURE_WIDTH);
        }
    }
}

pub type TextRunDataHandle = intern::Handle<TextRun>;

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct TextRun {
    pub font: FontInstance,
    #[ignore_malloc_size_of = "Measured via PrimaryArc"]
    pub glyphs: Arc<Vec<GlyphInstance>>,
    pub shadow: bool,
    pub requested_raster_space: RasterSpace,
}

impl intern::Internable for TextRun {
    type Key = TextRunKey;
    type StoreData = TextRunTemplate;
    type InternData = ();
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_TEXT_RUNS;
}

impl InternablePrimitive for TextRun {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> TextRunKey {
        TextRunKey::new(
            info,
            self,
        )
    }

    fn make_instance_kind(
        key: TextRunKey,
        data_handle: TextRunDataHandle,
        prim_store: &mut PrimitiveStore,
        reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        let run_index = prim_store.text_runs.push(TextRunPrimitive {
            used_font: key.font.clone(),
            glyph_keys_range: storage::Range::empty(),
            reference_frame_relative_offset,
            snapped_reference_frame_relative_offset: reference_frame_relative_offset,
            shadow: key.shadow,
            raster_scale: 1.0,
            requested_raster_space: key.requested_raster_space,
        });

        PrimitiveInstanceKind::TextRun{ data_handle, run_index }
    }
}

impl CreateShadow for TextRun {
    fn create_shadow(
        &self,
        shadow: &Shadow,
        blur_is_noop: bool,
        current_raster_space: RasterSpace,
    ) -> Self {
        let mut font = FontInstance {
            color: shadow.color.into(),
            ..self.font.clone()
        };
        if shadow.blur_radius > 0.0 {
            font.disable_subpixel_aa();
        }

        let requested_raster_space = if blur_is_noop {
            current_raster_space
        } else {
            RasterSpace::Local(1.0)
        };

        TextRun {
            font,
            glyphs: self.glyphs.clone(),
            shadow: true,
            requested_raster_space,
        }
    }
}

impl IsVisible for TextRun {
    fn is_visible(&self) -> bool {
        self.font.color.a > 0
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct TextRunPrimitive {
    pub used_font: FontInstance,
    pub glyph_keys_range: storage::Range<GlyphKey>,
    pub reference_frame_relative_offset: LayoutVector2D,
    pub snapped_reference_frame_relative_offset: LayoutVector2D,
    pub shadow: bool,
    pub raster_scale: f32,
    pub requested_raster_space: RasterSpace,
}

impl TextRunPrimitive {
    pub fn update_font_instance(
        &mut self,
        specified_font: &FontInstance,
        surface: &SurfaceInfo,
        spatial_node_index: SpatialNodeIndex,
        transform: &LayoutToWorldTransform,
        mut allow_subpixel: bool,
        raster_space: RasterSpace,
        root_scaling_factor: f32,
        spatial_tree: &SpatialTree,
    ) -> bool {
        // If local raster space is specified, include that in the scale
        // of the glyphs that get rasterized.
        // TODO(gw): Once we support proper local space raster modes, this
        //           will implicitly be part of the device pixel ratio for
        //           the (cached) local space surface, and so this code
        //           will no longer be required.
        let raster_scale = raster_space.local_scale().unwrap_or(1.0).max(0.001);

        // root_scaling_factor is used to scale very large pictures that establish
        // a raster root back to something sane, thus scale the device size accordingly.
        // to the shader it looks like a change in DPI which it already supports.
        let dps = surface.device_pixel_scale.0 * root_scaling_factor;
        let font_size = specified_font.size.to_f32_px();
        let mut device_font_size = font_size * dps * raster_scale;

        // Check there is a valid transform that doesn't exceed the font size limit.
        // Ensure the font is supposed to be rasterized in screen-space.
        // Only support transforms that can be coerced to simple 2D transforms.
        // Add texture padding to the rasterized glyph buffer when one anticipates
        // the glyph will need to be scaled when rendered.
        let (use_subpixel_aa, transform_glyphs, texture_padding, oversized) = if raster_space != RasterSpace::Screen ||
            transform.has_perspective_component() || !transform.has_2d_inverse()
        {
            (false, false, true, device_font_size > FONT_SIZE_LIMIT)
        } else if transform.exceeds_2d_scale((FONT_SIZE_LIMIT / device_font_size) as f64) {
            (false, false, true, true)
        } else {
            (true, !transform.is_simple_2d_translation(), false, false)
        };

        let font_transform = if transform_glyphs {
            // Get the font transform matrix (skew / scale) from the complete transform.
            // Fold in the device pixel scale.
            self.raster_scale = 1.0;
            FontTransform::from(transform)
        } else {
            if oversized {
                // Font sizes larger than the limit need to be scaled, thus can't use subpixels.
                // In this case we adjust the font size and raster space to ensure
                // we rasterize at the limit, to minimize the amount of scaling.
                let limited_raster_scale = FONT_SIZE_LIMIT / (font_size * dps);
                device_font_size = FONT_SIZE_LIMIT;

                // Record the raster space the text needs to be snapped in. The original raster
                // scale would have been too big.
                self.raster_scale = limited_raster_scale;
            } else {
                // Record the raster space the text needs to be snapped in. We may have changed
                // from RasterSpace::Screen due to a transform with perspective or without a 2d
                // inverse, or it may have been RasterSpace::Local all along.
                self.raster_scale = raster_scale;
            }

            // Rasterize the glyph without any transform
            FontTransform::identity()
        };

        // TODO(aosmond): Snapping really ought to happen during scene building
        // as much as possible. This will allow clips to be already adjusted
        // based on the snapping requirements of the primitive. This may affect
        // complex clips that create a different task, and when we rasterize
        // glyphs without the transform (because the shader doesn't have the
        // snap offsets to adjust its clip). These rects are fairly conservative
        // to begin with and do not appear to be causing significant issues at
        // this time.
        self.snapped_reference_frame_relative_offset = if transform_glyphs {
            // Don't touch the reference frame relative offset. We'll let the
            // shader do the snapping in device pixels.
            self.reference_frame_relative_offset
        } else {
            // There may be an animation, so snap the reference frame relative
            // offset such that it excludes the impact, if any.
            let snap_to_device = SpaceSnapper::new_with_target(
                surface.raster_spatial_node_index,
                spatial_node_index,
                surface.device_pixel_scale,
                spatial_tree,
            );
            snap_to_device.snap_point(&self.reference_frame_relative_offset.to_point()).to_vector()
        };

        let mut flags = specified_font.flags;
        if transform_glyphs {
            flags |= FontInstanceFlags::TRANSFORM_GLYPHS;
        }
        if texture_padding {
            flags |= FontInstanceFlags::TEXTURE_PADDING;
        }

        // If the transform or device size is different, then the caller of
        // this method needs to know to rebuild the glyphs.
        let cache_dirty =
            self.used_font.transform != font_transform ||
            self.used_font.size != device_font_size.into() ||
            self.used_font.flags != flags;

        // Construct used font instance from the specified font instance
        self.used_font = FontInstance {
            transform: font_transform,
            size: device_font_size.into(),
            flags,
            ..specified_font.clone()
        };

        // If we are using special estimated background subpixel blending, then
        // we can allow it regardless of what the surface says.
        allow_subpixel |= self.used_font.bg_color.a != 0;

        // If using local space glyphs, we don't want subpixel AA.
        if !allow_subpixel || !use_subpixel_aa {
            self.used_font.disable_subpixel_aa();

            // Disable subpixel positioning for oversized glyphs to avoid
            // thrashing the glyph cache with many subpixel variations of
            // big glyph textures. A possible subpixel positioning error
            // is small relative to the maximum font size and thus should
            // not be very noticeable.
            if oversized {
                self.used_font.disable_subpixel_position();
            }
        }

        cache_dirty
    }

    /// Gets the raster space to use when rendering this primitive.
    /// Usually this would be the requested raster space. However, if
    /// the primitive's spatial node or one of its ancestors is being pinch zoomed
    /// then we round it. This prevents us rasterizing glyphs for every minor
    /// change in zoom level, as that would be too expensive.
    fn get_raster_space_for_prim(
        &self,
        prim_spatial_node_index: SpatialNodeIndex,
        spatial_tree: &SpatialTree,
    ) -> RasterSpace {
        let prim_spatial_node = &spatial_tree.spatial_nodes[prim_spatial_node_index.0 as usize];
        if prim_spatial_node.is_ancestor_or_self_zooming {
            let scale_factors = spatial_tree
                .get_relative_transform(prim_spatial_node_index, ROOT_SPATIAL_NODE_INDEX)
                .scale_factors();

            // Round the scale up to the nearest power of 2, but don't exceed 8.
            let scale = scale_factors.0.max(scale_factors.1).min(8.0);
            let rounded_up = 2.0f32.powf(scale.log2().ceil());

            RasterSpace::Local(rounded_up)
        } else {
            self.requested_raster_space
        }
    }

    pub fn request_resources(
        &mut self,
        prim_offset: LayoutVector2D,
        specified_font: &FontInstance,
        glyphs: &[GlyphInstance],
        transform: &LayoutToWorldTransform,
        surface: &SurfaceInfo,
        spatial_node_index: SpatialNodeIndex,
        root_scaling_factor: f32,
        allow_subpixel: bool,
        resource_cache: &mut ResourceCache,
        gpu_cache: &mut GpuCache,
        spatial_tree: &SpatialTree,
        scratch: &mut PrimitiveScratchBuffer,
    ) {
        let raster_space = self.get_raster_space_for_prim(
            spatial_node_index,
            spatial_tree,
        );

        let cache_dirty = self.update_font_instance(
            specified_font,
            surface,
            spatial_node_index,
            transform,
            allow_subpixel,
            raster_space,
            root_scaling_factor,
            spatial_tree,
        );

        if self.glyph_keys_range.is_empty() || cache_dirty {
            let subpx_dir = self.used_font.get_subpx_dir();

            let dps = surface.device_pixel_scale.0 * root_scaling_factor;
            let transform = match raster_space {
                RasterSpace::Local(scale) => FontTransform::new(scale * dps, 0.0, 0.0, scale * dps),
                RasterSpace::Screen => self.used_font.transform.scale(dps),
            };

            self.glyph_keys_range = scratch.glyph_keys.extend(
                glyphs.iter().map(|src| {
                    let src_point = src.point + prim_offset;
                    let device_offset = transform.transform(&src_point);
                    GlyphKey::new(src.index, device_offset, subpx_dir)
                }));
        }

        resource_cache.request_glyphs(
            self.used_font.clone(),
            &scratch.glyph_keys[self.glyph_keys_range],
            gpu_cache,
        );
    }
}

/// These are linux only because FontInstancePlatformOptions varies in size by platform.
#[test]
#[cfg(target_os = "linux")]
fn test_struct_sizes() {
    use std::mem;
    // The sizes of these structures are critical for performance on a number of
    // talos stress tests. If you get a failure here on CI, there's two possibilities:
    // (a) You made a structure smaller than it currently is. Great work! Update the
    //     test expectations and move on.
    // (b) You made a structure larger. This is not necessarily a problem, but should only
    //     be done with care, and after checking if talos performance regresses badly.
    assert_eq!(mem::size_of::<TextRun>(), 64, "TextRun size changed");
    assert_eq!(mem::size_of::<TextRunTemplate>(), 80, "TextRunTemplate size changed");
    assert_eq!(mem::size_of::<TextRunKey>(), 80, "TextRunKey size changed");
    assert_eq!(mem::size_of::<TextRunPrimitive>(), 80, "TextRunPrimitive size changed");
}
