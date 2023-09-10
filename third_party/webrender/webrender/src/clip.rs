/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Internal representation of clips in WebRender.
//!
//! # Data structures
//!
//! There are a number of data structures involved in the clip module:
//!
//! - ClipStore - Main interface used by other modules.
//!
//! - ClipItem - A single clip item (e.g. a rounded rect, or a box shadow).
//!              These are an exposed API type, stored inline in a ClipNode.
//!
//! - ClipNode - A ClipItem with an attached GPU handle. The GPU handle is populated
//!              when a ClipNodeInstance is built from this node (which happens while
//!              preparing primitives for render).
//!
//! ClipNodeInstance - A ClipNode with attached positioning information (a spatial
//!                    node index). This is stored as a contiguous array of nodes
//!                    within the ClipStore.
//!
//! ```ascii
//! +-----------------------+-----------------------+-----------------------+
//! | ClipNodeInstance      | ClipNodeInstance      | ClipNodeInstance      |
//! +-----------------------+-----------------------+-----------------------+
//! | ClipItem              | ClipItem              | ClipItem              |
//! | Spatial Node Index    | Spatial Node Index    | Spatial Node Index    |
//! | GPU cache handle      | GPU cache handle      | GPU cache handle      |
//! | ...                   | ...                   | ...                   |
//! +-----------------------+-----------------------+-----------------------+
//!            0                        1                       2
//!    +----------------+    |                                              |
//!    | ClipNodeRange  |____|                                              |
//!    |    index: 1    |                                                   |
//!    |    count: 2    |___________________________________________________|
//!    +----------------+
//! ```
//!
//! - ClipNodeRange - A clip item range identifies a range of clip nodes instances.
//!                   It is stored as an (index, count).
//!
//! - ClipChainNode - A clip chain node contains a handle to an interned clip item,
//!                   positioning information (from where the clip was defined), and
//!                   an optional parent link to another ClipChainNode. ClipChainId
//!                   is an index into an array, or ClipChainId::NONE for no parent.
//!
//! ```ascii
//! +----------------+    ____+----------------+    ____+----------------+   /---> ClipChainId::NONE
//! | ClipChainNode  |   |    | ClipChainNode  |   |    | ClipChainNode  |   |
//! +----------------+   |    +----------------+   |    +----------------+   |
//! | ClipDataHandle |   |    | ClipDataHandle |   |    | ClipDataHandle |   |
//! | Spatial index  |   |    | Spatial index  |   |    | Spatial index  |   |
//! | Parent Id      |___|    | Parent Id      |___|    | Parent Id      |___|
//! | ...            |        | ...            |        | ...            |
//! +----------------+        +----------------+        +----------------+
//! ```
//!
//! - ClipChainInstance - A ClipChain that has been built for a specific primitive + positioning node.
//!
//!    When given a clip chain ID, and a local primitive rect and its spatial node, the clip module
//!    creates a clip chain instance. This is a struct with various pieces of useful information
//!    (such as a local clip rect). It also contains a (index, count)
//!    range specifier into an index buffer of the ClipNodeInstance structures that are actually relevant
//!    for this clip chain instance. The index buffer structure allows a single array to be used for
//!    all of the clip-chain instances built in a single frame. Each entry in the index buffer
//!    also stores some flags relevant to the clip node in this positioning context.
//!
//! ```ascii
//! +----------------------+
//! | ClipChainInstance    |
//! +----------------------+
//! | ...                  |
//! | local_clip_rect      |________________________________________________________________________
//! | clips_range          |_______________                                                        |
//! +----------------------+              |                                                        |
//!                                       |                                                        |
//! +------------------+------------------+------------------+------------------+------------------+
//! | ClipNodeInstance | ClipNodeInstance | ClipNodeInstance | ClipNodeInstance | ClipNodeInstance |
//! +------------------+------------------+------------------+------------------+------------------+
//! | flags            | flags            | flags            | flags            | flags            |
//! | ...              | ...              | ...              | ...              | ...              |
//! +------------------+------------------+------------------+------------------+------------------+
//! ```
//!
//! # Rendering clipped primitives
//!
//! See the [`segment` module documentation][segment.rs].
//!
//!
//! [segment.rs]: ../segment/index.html
//!

use api::{BorderRadius, ClipMode, ComplexClipRegion, ImageMask};
use api::{BoxShadowClipMode, ClipId, FillRule, ImageKey, ImageRendering, PipelineId};
use api::units::*;
use crate::image_tiling::{self, Repetition};
use crate::border::{ensure_no_corner_overlap, BorderRadiusAu};
use crate::box_shadow::{BLUR_SAMPLE_SCALE, BoxShadowClipSource, BoxShadowCacheKey};
use crate::spatial_tree::{ROOT_SPATIAL_NODE_INDEX, SpatialTree, SpatialNodeIndex, CoordinateSystemId};
use crate::ellipse::Ellipse;
use crate::gpu_cache::GpuCache;
use crate::gpu_types::{BoxShadowStretchMode};
use crate::intern::{self, ItemUid};
use crate::internal_types::{FastHashMap, FastHashSet};
use crate::prim_store::{VisibleMaskImageTile};
use crate::prim_store::{PointKey, SizeKey, RectangleKey, PolygonKey};
use crate::render_task_cache::to_cache_size;
use crate::resource_cache::{ImageRequest, ResourceCache};
use crate::space::SpaceMapper;
use crate::util::{clamp_to_scale_factor, MaxRect, extract_inner_rect_safe, project_rect, ScaleOffset, VecHelper};
use euclid::approxeq::ApproxEq;
use std::{iter, ops, u32, mem};

// Type definitions for interning clip nodes.

#[derive(Copy, Clone, Debug, MallocSizeOf, PartialEq)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub enum ClipIntern {}

pub type ClipDataStore = intern::DataStore<ClipIntern>;
pub type ClipDataHandle = intern::Handle<ClipIntern>;

/// Defines a clip that is positioned by a specific spatial node
#[cfg_attr(feature = "capture", derive(Serialize))]
#[derive(Copy, Clone, PartialEq)]
#[derive(MallocSizeOf)]
pub struct ClipInstance {
    /// Handle to the interned clip
    pub handle: ClipDataHandle,
    /// Positioning node for this clip
    pub spatial_node_index: SpatialNodeIndex,
}

impl ClipInstance {
    /// Construct a new positioned clip
    pub fn new(
        handle: ClipDataHandle,
        spatial_node_index: SpatialNodeIndex,
    ) -> Self {
        ClipInstance {
            handle,
            spatial_node_index,
        }
    }
}

/// Defines a clip instance with some extra information that is available
/// during scene building (since interned clips cannot retrieve the underlying
/// data from the scene building thread).
#[cfg_attr(feature = "capture", derive(Serialize))]
#[derive(MallocSizeOf)]
#[derive(Copy, Clone)]
pub struct SceneClipInstance {
    /// The interned clip + positioning information that is used during frame building.
    pub clip: ClipInstance,
    /// The definition of the clip, used during scene building to optimize clip-chains.
    pub key: ClipItemKey,
}

/// A clip template defines clips in terms of the public API. Specifically,
/// this is a parent `ClipId` and some number of clip instances. See the
/// CLIPPING_AND_POSITIONING.md document in doc/ for more information.
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ClipTemplate {
    /// Parent of this clip, in terms of the public clip API
    pub parent: ClipId,
    /// Range of instances that define this clip template
    pub clips: ops::Range<u32>,
}

/// A helper used during scene building to construct (internal) clip chains from
/// the public API definitions (a hierarchy of ClipIds)
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ClipChainBuilder {
    /// The built clip chain id for this level of the stack
    clip_chain_id: ClipChainId,
    /// A list of parent clips in the current clip chain, to de-duplicate clips as
    /// we build child chains from this level.
    parent_clips: FastHashSet<(ItemUid, SpatialNodeIndex)>,
    /// A cache used during building child clip chains. Retained here to avoid
    /// extra memory allocations each time we build a clip.
    existing_clips_cache: FastHashSet<(ItemUid, SpatialNodeIndex)>,
    /// Cache the previous ClipId we built, since it's quite common to share clip
    /// id between primitives.
    prev_clip_id: ClipId,
    prev_clip_chain_id: ClipChainId,
}

impl ClipChainBuilder {
    /// Construct a new clip chain builder with specified parent clip chain. If
    /// the clip_id is Some(..), the clips in that template will be added to the
    /// clip chain at this level (this functionality isn't currently used, but will
    /// be in the follow up patches).
    fn new(
        parent_clip_chain_id: ClipChainId,
        clip_id: Option<ClipId>,
        clip_chain_nodes: &mut Vec<ClipChainNode>,
        templates: &FastHashMap<ClipId, ClipTemplate>,
        instances: &[SceneClipInstance],
    ) -> Self {
        let mut parent_clips = FastHashSet::default();

        // Walk the current clip chain ID, building a set of existing clips
        let mut current_clip_chain_id = parent_clip_chain_id;
        while current_clip_chain_id != ClipChainId::NONE {
            let clip_chain_node = &clip_chain_nodes[current_clip_chain_id.0 as usize];
            parent_clips.insert((clip_chain_node.handle.uid(), clip_chain_node.spatial_node_index));
            current_clip_chain_id = clip_chain_node.parent_clip_chain_id;
        }

        // If specified, add the clips from the supplied template to this builder
        let clip_chain_id = match clip_id {
            Some(clip_id) => {
                ClipChainBuilder::add_new_clips_to_chain(
                    clip_id,
                    parent_clip_chain_id,
                    &mut parent_clips,
                    clip_chain_nodes,
                    templates,
                    instances,
                )
            }
            None => {
                // Even if the clip id is None, it's possible that there were parent clips in the builder
                // that need to be applied and set as the root of this clip-chain builder.
                parent_clip_chain_id
            }
        };

        ClipChainBuilder {
            clip_chain_id,
            existing_clips_cache: parent_clips.clone(),
            parent_clips,
            prev_clip_id: ClipId::root(PipelineId::dummy()),
            prev_clip_chain_id: ClipChainId::NONE,
        }
    }

    /// Internal helper function that appends all clip instances from a template
    /// to a clip-chain (if they don't already exist in this chain).
    fn add_new_clips_to_chain(
        clip_id: ClipId,
        parent_clip_chain_id: ClipChainId,
        existing_clips: &mut FastHashSet<(ItemUid, SpatialNodeIndex)>,
        clip_chain_nodes: &mut Vec<ClipChainNode>,
        templates: &FastHashMap<ClipId, ClipTemplate>,
        clip_instances: &[SceneClipInstance],
    ) -> ClipChainId {
        let template = &templates[&clip_id];
        let instances = &clip_instances[template.clips.start as usize .. template.clips.end as usize];
        let mut clip_chain_id = parent_clip_chain_id;

        for clip in instances {
            let key = (clip.clip.handle.uid(), clip.clip.spatial_node_index);

            // If this clip chain already has this clip instance, skip it
            if existing_clips.contains(&key) {
                continue;
            }

            // Create a new clip-chain entry for this instance
            let new_clip_chain_id = ClipChainId(clip_chain_nodes.len() as u32);
            existing_clips.insert(key);
            clip_chain_nodes.push(ClipChainNode {
                handle: clip.clip.handle,
                spatial_node_index: clip.clip.spatial_node_index,
                parent_clip_chain_id: clip_chain_id,
            });
            clip_chain_id = new_clip_chain_id;
        }

        // The ClipId parenting is terminated when we reach the root ClipId
        if clip_id == template.parent {
            return clip_chain_id;
        }

        ClipChainBuilder::add_new_clips_to_chain(
            template.parent,
            clip_chain_id,
            existing_clips,
            clip_chain_nodes,
            templates,
            clip_instances,
        )
    }

    /// Return true if any of the clips in the hierarchy from clip_id to the
    /// root clip are complex.
    // TODO(gw): This method should only be required until the shared_clip
    //           optimization patches are complete, and can then be removed.
    fn has_complex_clips(
        &self,
        clip_id: ClipId,
        templates: &FastHashMap<ClipId, ClipTemplate>,
        instances: &[SceneClipInstance],
    ) -> bool {
        let template = &templates[&clip_id];

        // Check if any of the clips in this template are complex
        let clips = &instances[template.clips.start as usize .. template.clips.end as usize];
        for clip in clips {
            if let ClipNodeKind::Complex = clip.key.kind.node_kind() {
                return true;
            }
        }

        // The ClipId parenting is terminated when we reach the root ClipId
        if clip_id == template.parent {
            return false;
        }

        // Recurse into parent clip template to also check those
        self.has_complex_clips(
            template.parent,
            templates,
            instances,
        )
    }

    /// This is the main method used to get a clip chain for a primitive. Given a
    /// clip id, it builds a clip-chain for that primitive, parented to the current
    /// root clip chain hosted in this builder.
    fn get_or_build_clip_chain_id(
        &mut self,
        clip_id: ClipId,
        clip_chain_nodes: &mut Vec<ClipChainNode>,
        templates: &FastHashMap<ClipId, ClipTemplate>,
        instances: &[SceneClipInstance],
    ) -> ClipChainId {
        if self.prev_clip_id == clip_id {
            return self.prev_clip_chain_id;
        }

        // Instead of cloning here, do a clear and manual insertions, to
        // avoid any extra heap allocations each time we build a clip-chain here.
        // Maybe there is a better way to do this?
        self.existing_clips_cache.clear();
        self.existing_clips_cache.reserve(self.parent_clips.len());
        for clip in &self.parent_clips {
            self.existing_clips_cache.insert(*clip);
        }

        let clip_chain_id = ClipChainBuilder::add_new_clips_to_chain(
            clip_id,
            self.clip_chain_id,
            &mut self.existing_clips_cache,
            clip_chain_nodes,
            templates,
            instances,
        );

        self.prev_clip_id = clip_id;
        self.prev_clip_chain_id = clip_chain_id;

        clip_chain_id
    }
}

/// Helper to identify simple clips (normal rects) from other kinds of clips,
/// which can often be handled via fast code paths.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, MallocSizeOf)]
pub enum ClipNodeKind {
    /// A normal clip rectangle, with Clip mode.
    Rectangle,
    /// A rectangle with ClipOut, or any other kind of clip.
    Complex,
}

// Result of comparing a clip node instance against a local rect.
#[derive(Debug)]
enum ClipResult {
    // The clip does not affect the region at all.
    Accept,
    // The clip prevents the region from being drawn.
    Reject,
    // The clip affects part of the region. This may
    // require a clip mask, depending on other factors.
    Partial,
}

// A clip node is a single clip source, along with some
// positioning information and implementation details
// that control where the GPU data for this clip source
// can be found.
#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct ClipNode {
    pub item: ClipItem,
}

// Convert from an interning key for a clip item
// to a clip node, which is cached in the document.
impl From<ClipItemKey> for ClipNode {
    fn from(item: ClipItemKey) -> Self {
        let kind = match item.kind {
            ClipItemKeyKind::Rectangle(rect, mode) => {
                ClipItemKind::Rectangle { rect: rect.into(), mode }
            }
            ClipItemKeyKind::RoundedRectangle(rect, radius, mode) => {
                ClipItemKind::RoundedRectangle {
                    rect: rect.into(),
                    radius: radius.into(),
                    mode,
                }
            }
            ClipItemKeyKind::ImageMask(rect, image, repeat, polygon_handle) => {
                ClipItemKind::Image {
                    image,
                    rect: rect.into(),
                    repeat,
                    polygon_handle,
                }
            }
            ClipItemKeyKind::BoxShadow(shadow_rect_fract_offset, shadow_rect_size, shadow_radius, prim_shadow_rect, blur_radius, clip_mode) => {
                ClipItemKind::new_box_shadow(
                    shadow_rect_fract_offset.into(),
                    shadow_rect_size.into(),
                    shadow_radius.into(),
                    prim_shadow_rect.into(),
                    blur_radius.to_f32_px(),
                    clip_mode,
                )
            }
        };

        ClipNode {
            item: ClipItem {
                kind,
            },
        }
    }
}

// Flags that are attached to instances of clip nodes.
bitflags! {
    #[cfg_attr(feature = "capture", derive(Serialize))]
    #[cfg_attr(feature = "replay", derive(Deserialize))]
    #[derive(MallocSizeOf)]
    pub struct ClipNodeFlags: u8 {
        const SAME_SPATIAL_NODE = 0x1;
        const SAME_COORD_SYSTEM = 0x2;
        const USE_FAST_PATH = 0x4;
    }
}

// Identifier for a clip chain. Clip chains are stored
// in a contiguous array in the clip store. They are
// identified by a simple index into that array.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ClipChainId(pub u32);

// The root of each clip chain is the NONE id. The
// value is specifically set to u32::MAX so that if
// any code accidentally tries to access the root
// node, a bounds error will occur.
impl ClipChainId {
    pub const NONE: Self = ClipChainId(u32::MAX);
    pub const INVALID: Self = ClipChainId(0xDEADBEEF);
}

// A clip chain node is an id for a range of clip sources,
// and a link to a parent clip chain node, or ClipChainId::NONE.
#[derive(Clone, Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ClipChainNode {
    pub handle: ClipDataHandle,
    pub spatial_node_index: SpatialNodeIndex,
    pub parent_clip_chain_id: ClipChainId,
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ClipSet {
    /// Local space clip rect
    pub local_clip_rect: LayoutRect,

    /// ID of the clip chain that this set is clipped by.
    pub clip_chain_id: ClipChainId,
}

// When a clip node is found to be valid for a
// clip chain instance, it's stored in an index
// buffer style structure. This struct contains
// an index to the node data itself, as well as
// some flags describing how this clip node instance
// is positioned.
#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ClipNodeInstance {
    pub handle: ClipDataHandle,
    pub spatial_node_index: SpatialNodeIndex,
    pub flags: ClipNodeFlags,
    pub visible_tiles: Option<ops::Range<usize>>,
}

impl ClipNodeInstance {
    pub fn has_visible_tiles(&self) -> bool {
        self.visible_tiles.is_some()
    }
}

// A range of clip node instances that were found by
// building a clip chain instance.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ClipNodeRange {
    pub first: u32,
    pub count: u32,
}

impl ClipNodeRange {
    pub fn to_range(&self) -> ops::Range<usize> {
        let start = self.first as usize;
        let end = start + self.count as usize;

        ops::Range {
            start,
            end,
        }
    }
}

/// A helper struct for converting between coordinate systems
/// of clip sources and primitives.
// todo(gw): optimize:
//  separate arrays for matrices
//  cache and only build as needed.
//TODO: merge with `CoordinateSpaceMapping`?
#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
enum ClipSpaceConversion {
    Local,
    ScaleOffset(ScaleOffset),
    Transform(LayoutToWorldTransform),
}

impl ClipSpaceConversion {
    /// Construct a new clip space converter between two spatial nodes.
    fn new(
        prim_spatial_node_index: SpatialNodeIndex,
        clip_spatial_node_index: SpatialNodeIndex,
        spatial_tree: &SpatialTree,
    ) -> Self {
        //Note: this code is different from `get_relative_transform` in a way that we only try
        // getting the relative transform if it's Local or ScaleOffset,
        // falling back to the world transform otherwise.
        let clip_spatial_node = &spatial_tree
            .spatial_nodes[clip_spatial_node_index.0 as usize];
        let prim_spatial_node = &spatial_tree
            .spatial_nodes[prim_spatial_node_index.0 as usize];

        if prim_spatial_node_index == clip_spatial_node_index {
            ClipSpaceConversion::Local
        } else if prim_spatial_node.coordinate_system_id == clip_spatial_node.coordinate_system_id {
            let scale_offset = prim_spatial_node.content_transform
                .inverse()
                .accumulate(&clip_spatial_node.content_transform);
            ClipSpaceConversion::ScaleOffset(scale_offset)
        } else {
            ClipSpaceConversion::Transform(
                spatial_tree
                    .get_world_transform(clip_spatial_node_index)
                    .into_transform()
            )
        }
    }

    fn to_flags(&self) -> ClipNodeFlags {
        match *self {
            ClipSpaceConversion::Local => {
                ClipNodeFlags::SAME_SPATIAL_NODE | ClipNodeFlags::SAME_COORD_SYSTEM
            }
            ClipSpaceConversion::ScaleOffset(..) => {
                ClipNodeFlags::SAME_COORD_SYSTEM
            }
            ClipSpaceConversion::Transform(..) => {
                ClipNodeFlags::empty()
            }
        }
    }
}

// Temporary information that is cached and reused
// during building of a clip chain instance.
#[derive(MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
struct ClipNodeInfo {
    conversion: ClipSpaceConversion,
    handle: ClipDataHandle,
    spatial_node_index: SpatialNodeIndex,
}

impl ClipNodeInfo {
    fn create_instance(
        &self,
        node: &ClipNode,
        clipped_rect: &LayoutRect,
        gpu_cache: &mut GpuCache,
        resource_cache: &mut ResourceCache,
        mask_tiles: &mut Vec<VisibleMaskImageTile>,
        spatial_tree: &SpatialTree,
        request_resources: bool,
    ) -> Option<ClipNodeInstance> {
        // Calculate some flags that are required for the segment
        // building logic.
        let mut flags = self.conversion.to_flags();

        // Some clip shaders support a fast path mode for simple clips.
        // TODO(gw): We could also apply fast path when segments are created, since we only write
        //           the mask for a single corner at a time then, so can always consider radii uniform.
        let is_raster_2d =
            flags.contains(ClipNodeFlags::SAME_COORD_SYSTEM) ||
            spatial_tree
                .get_world_viewport_transform(self.spatial_node_index)
                .is_2d_axis_aligned();
        if is_raster_2d && node.item.kind.supports_fast_path_rendering() {
            flags |= ClipNodeFlags::USE_FAST_PATH;
        }

        let mut visible_tiles = None;

        if let ClipItemKind::Image { rect, image, repeat, .. } = node.item.kind {
            let request = ImageRequest {
                key: image,
                rendering: ImageRendering::Auto,
                tile: None,
            };

            if let Some(props) = resource_cache.get_image_properties(image) {
                if let Some(tile_size) = props.tiling {
                    let tile_range_start = mask_tiles.len();

                    let visible_rect = if repeat {
                        *clipped_rect
                    } else {
                        // Bug 1648323 - It is unclear why on rare occasions we get
                        // a clipped_rect that does not intersect the clip's mask rect.
                        // defaulting to clipped_rect here results in zero repetitions
                        // which clips the primitive entirely.
                        clipped_rect.intersection(&rect).unwrap_or(*clipped_rect)
                    };

                    let repetitions = image_tiling::repetitions(
                        &rect,
                        &visible_rect,
                        rect.size,
                    );

                    for Repetition { origin, .. } in repetitions {
                        let layout_image_rect = LayoutRect {
                            origin,
                            size: rect.size,
                        };
                        let tiles = image_tiling::tiles(
                            &layout_image_rect,
                            &visible_rect,
                            &props.visible_rect,
                            tile_size as i32,
                        );
                        for tile in tiles {
                            if request_resources {
                                resource_cache.request_image(
                                    request.with_tile(tile.offset),
                                    gpu_cache,
                                );
                            }
                            mask_tiles.push(VisibleMaskImageTile {
                                tile_offset: tile.offset,
                                tile_rect: tile.rect,
                            });
                        }
                    }
                    visible_tiles = Some(tile_range_start..mask_tiles.len());
                } else if request_resources {
                    resource_cache.request_image(request, gpu_cache);
                }
            } else {
                // If the supplied image key doesn't exist in the resource cache,
                // skip the clip node since there is nothing to mask with.
                warn!("Clip mask with missing image key {:?}", request.key);
                return None;
            }
        }

        Some(ClipNodeInstance {
            handle: self.handle,
            flags,
            visible_tiles,
            spatial_node_index: self.spatial_node_index,
        })
    }
}

impl ClipNode {
    pub fn update(
        &mut self,
        device_pixel_scale: DevicePixelScale,
    ) {
        match self.item.kind {
            ClipItemKind::Image { .. } |
            ClipItemKind::Rectangle { .. } |
            ClipItemKind::RoundedRectangle { .. } => {}

            ClipItemKind::BoxShadow { ref mut source } => {
                // Quote from https://drafts.csswg.org/css-backgrounds-3/#shadow-blur
                // "the image that would be generated by applying to the shadow a
                // Gaussian blur with a standard deviation equal to half the blur radius."
                let blur_radius_dp = source.blur_radius * 0.5;

                // Create scaling from requested size to cache size.
                let mut content_scale = LayoutToWorldScale::new(1.0) * device_pixel_scale;
                content_scale.0 = clamp_to_scale_factor(content_scale.0, false);

                // Create the cache key for this box-shadow render task.
                let cache_size = to_cache_size(source.shadow_rect_alloc_size, &mut content_scale);

                let bs_cache_key = BoxShadowCacheKey {
                    blur_radius_dp: (blur_radius_dp * content_scale.0).round() as i32,
                    clip_mode: source.clip_mode,
                    original_alloc_size: (source.original_alloc_size * content_scale).round().to_i32(),
                    br_top_left: (source.shadow_radius.top_left * content_scale).round().to_i32(),
                    br_top_right: (source.shadow_radius.top_right * content_scale).round().to_i32(),
                    br_bottom_right: (source.shadow_radius.bottom_right * content_scale).round().to_i32(),
                    br_bottom_left: (source.shadow_radius.bottom_left * content_scale).round().to_i32(),
                    device_pixel_scale: Au::from_f32_px(content_scale.0),
                };

                source.cache_key = Some((cache_size, bs_cache_key));
            }
        }
    }
}

pub struct ClipStoreStats {
    templates_capacity: usize,
    instances_capacity: usize,
}

impl ClipStoreStats {
    pub fn empty() -> Self {
        ClipStoreStats {
            templates_capacity: 0,
            instances_capacity: 0,
        }
    }
}

#[derive(Default)]
pub struct ClipStoreScratchBuffer {
    clip_node_instances: Vec<ClipNodeInstance>,
    mask_tiles: Vec<VisibleMaskImageTile>,
}

/// The main clipping public interface that other modules access.
#[derive(MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ClipStore {
    pub clip_chain_nodes: Vec<ClipChainNode>,
    pub clip_node_instances: Vec<ClipNodeInstance>,
    mask_tiles: Vec<VisibleMaskImageTile>,

    active_clip_node_info: Vec<ClipNodeInfo>,
    active_local_clip_rect: Option<LayoutRect>,
    active_pic_clip_rect: PictureRect,

    // No malloc sizeof since it's not implemented for ops::Range, but these
    // allocations are tiny anyway.

    /// Map of all clip templates defined by the public API to templates
    #[ignore_malloc_size_of = "range missing"]
    pub templates: FastHashMap<ClipId, ClipTemplate>,
    pub instances: Vec<SceneClipInstance>,

    /// A stack of current clip-chain builders. A new clip-chain builder is
    /// typically created each time a clip root (such as an iframe or stacking
    /// context) is defined.
    #[ignore_malloc_size_of = "range missing"]
    chain_builder_stack: Vec<ClipChainBuilder>,
}

// A clip chain instance is what gets built for a given clip
// chain id + local primitive region + positioning node.
#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ClipChainInstance {
    pub clips_range: ClipNodeRange,
    // Combined clip rect for clips that are in the
    // same coordinate system as the primitive.
    pub local_clip_rect: LayoutRect,
    pub has_non_local_clips: bool,
    // If true, this clip chain requires allocation
    // of a clip mask.
    pub needs_mask: bool,
    // Combined clip rect in picture space (may
    // be more conservative that local_clip_rect).
    pub pic_clip_rect: PictureRect,
    // Space, in which the `pic_clip_rect` is defined.
    pub pic_spatial_node_index: SpatialNodeIndex,
}

impl ClipChainInstance {
    pub fn empty() -> Self {
        ClipChainInstance {
            clips_range: ClipNodeRange {
                first: 0,
                count: 0,
            },
            local_clip_rect: LayoutRect::zero(),
            has_non_local_clips: false,
            needs_mask: false,
            pic_clip_rect: PictureRect::zero(),
            pic_spatial_node_index: ROOT_SPATIAL_NODE_INDEX,
        }
    }
}

/// Maintains a (flattened) list of clips for a given level in the surface level stack.
pub struct ClipChainLevel {
    /// These clips will be handled when compositing this surface into the parent,
    /// and can thus be ignored on the primitives that are drawn as part of this surface.
    shared_clips: Vec<ClipInstance>,

    /// Index of the first element in ClipChainStack::clip that belongs to this level.
    first_clip_index: usize,
    /// Used to sanity check push/pop balance.
    initial_clip_counts_len: usize,
}

/// Maintains a stack of clip chain ids that are currently active,
/// when a clip exists on a picture that has no surface, and is passed
/// on down to the child primitive(s).
///
///
/// In order to avoid many small vector allocations, all clip chain ids are
/// stored in a single vector instead of per-level.
/// Since we only work with the top-most level of the stack, we only need to
/// know the first index in the clips vector that belongs to each level. The
/// last index for the top-most level is always the end of the clips array.
///
/// Likewise, we push several clip chain ids to the clips array at each
/// push_clip, and the number of clip chain ids removed during pop_clip
/// must match. This is done by having a separate stack of clip counts
/// in the clip-stack rather than per-level to avoid vector allocations.
///
/// ```ascii
///              +----+----+---
///      levels: |    |    | ...
///              +----+----+---
///               |first   \
///               |         \
///               |          \
///              +--+--+--+--+--+--+--
///       clips: |  |  |  |  |  |  | ...
///              +--+--+--+--+--+--+--
///               |     /     /
///               |    /    /
///               |   /   /
///              +--+--+--+--
/// clip_counts: | 1| 2| 2| ...
///              +--+--+--+--
/// ```
pub struct ClipChainStack {
    /// A stack of clip chain lists. Each time a new surface is pushed,
    /// a new level is added. Each time a new picture without surface is
    /// pushed, it adds the picture clip chain to the clips vector in the
    /// range belonging to the level (always the top-most level, so always
    /// at the end of the clips array).
    levels: Vec<ClipChainLevel>,
    /// The actual stack of clip ids.
    clips: Vec<ClipChainId>,
    /// How many clip ids to pop from the vector each time we call pop_clip.
    clip_counts: Vec<usize>,
}

impl ClipChainStack {
    pub fn new() -> Self {
        ClipChainStack {
            levels: vec![
                ClipChainLevel {
                    shared_clips: Vec::new(),
                    first_clip_index: 0,
                    initial_clip_counts_len: 0,
                }
            ],
            clips: Vec::new(),
            clip_counts: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.clips.clear();
        self.clip_counts.clear();
        self.levels.clear();
        self.levels.push(ClipChainLevel {
            shared_clips: Vec::new(),
            first_clip_index: 0,
            initial_clip_counts_len: 0,
        });
    }

    pub fn take(&mut self) -> Self {
        ClipChainStack {
            levels: self.levels.take(),
            clips: self.clips.take(),
            clip_counts: self.clip_counts.take(),
        }
    }

    /// Push a clip chain root onto the currently active list.
    pub fn push_clip(
        &mut self,
        clip_chain_id: ClipChainId,
        clip_store: &ClipStore,
    ) {
        let mut clip_count = 0;

        let mut current_clip_chain_id = clip_chain_id;
        while current_clip_chain_id != ClipChainId::NONE {
            let clip_chain_node = &clip_store.clip_chain_nodes[current_clip_chain_id.0 as usize];
            let clip_uid = clip_chain_node.handle.uid();

            // The clip is required, so long as it doesn't exist in any of the shared_clips
            // array from this or any parent surfaces.
            // TODO(gw): We could consider making this a HashSet if it ever shows up in
            //           profiles, but the typical array length is 2-3 elements.
            let mut valid_clip = true;
            for level in &self.levels {
                if level.shared_clips.iter().any(|instance| {
                    instance.handle.uid() == clip_uid &&
                    instance.spatial_node_index == clip_chain_node.spatial_node_index
                }) {
                    valid_clip = false;
                    break;
                }
            }

            if valid_clip {
                self.clips.push(current_clip_chain_id);
                clip_count += 1;
            }

            current_clip_chain_id = clip_chain_node.parent_clip_chain_id;
        }

        self.clip_counts.push(clip_count);
    }

    /// Pop a clip chain root from the currently active list.
    pub fn pop_clip(&mut self) {
        let count = self.clip_counts.pop().unwrap();
        for _ in 0 .. count {
            self.clips.pop().unwrap();
        }
    }

    /// When a surface is created, it takes all clips and establishes a new
    /// stack of clips to be propagated.
    pub fn push_surface(
        &mut self,
        maybe_shared_clips: &[ClipInstance],
        spatial_tree: &SpatialTree,
    ) {
        let mut shared_clips = Vec::with_capacity(maybe_shared_clips.len());

        // If there are clips in the shared list for a picture cache, only include
        // them if they are simple, axis-aligned clips (i.e. in the root coordinate
        // system). This is necessary since when compositing picture cache tiles
        // into the parent, we don't support applying a clip mask. This only ever
        // occurs in wrench tests, not in display lists supplied by Gecko.
        // TODO(gw): We can remove this when we update the WR API to have better
        //           knowledge of what coordinate system a clip must be in (by
        //           knowing if a reference frame exists in the chain between the
        //           clip's spatial node and the picture cache reference spatial node).
        for clip in maybe_shared_clips {
            let spatial_node = &spatial_tree.spatial_nodes[clip.spatial_node_index.0 as usize];
            if spatial_node.coordinate_system_id == CoordinateSystemId::root() {
                shared_clips.push(*clip);
            }
        }

        let level = ClipChainLevel {
            shared_clips,
            first_clip_index: self.clips.len(),
            initial_clip_counts_len: self.clip_counts.len(),
        };

        self.levels.push(level);
    }

    /// Pop a surface from the clip chain stack
    pub fn pop_surface(&mut self) {
        let level = self.levels.pop().unwrap();
        assert!(self.clip_counts.len() == level.initial_clip_counts_len);
        assert!(self.clips.len() == level.first_clip_index);
    }

    /// Get the list of currently active clip chains
    pub fn current_clips_array(&self) -> &[ClipChainId] {
        let first = self.levels.last().unwrap().first_clip_index;
        &self.clips[first..]
    }
}

impl ClipStore {
    pub fn new(stats: &ClipStoreStats) -> Self {
        let mut templates = FastHashMap::default();
        templates.reserve(stats.templates_capacity);

        ClipStore {
            clip_chain_nodes: Vec::new(),
            clip_node_instances: Vec::new(),
            mask_tiles: Vec::new(),
            active_clip_node_info: Vec::new(),
            active_local_clip_rect: None,
            active_pic_clip_rect: PictureRect::max_rect(),
            templates,
            instances: Vec::with_capacity(stats.instances_capacity),
            chain_builder_stack: Vec::new(),
        }
    }

    pub fn get_stats(&self) -> ClipStoreStats {
        // Selecting the smaller of the current capacity and 2*len ensures we don't
        // retain a huge hashmap alloc after navigating away from a page with a large
        // number of clip templates.
        let templates_capacity = self.templates.capacity().min(self.templates.len() * 2);
        let instances_capacity = self.instances.capacity().min(self.instances.len() * 2);

        ClipStoreStats {
            templates_capacity,
            instances_capacity,
        }
    }

    /// Register a new clip template for the clip_id defined in the display list.
    pub fn register_clip_template(
        &mut self,
        clip_id: ClipId,
        parent: ClipId,
        clips: &[SceneClipInstance],
    ) {
        let start = self.instances.len() as u32;
        self.instances.extend_from_slice(clips);
        let end = self.instances.len() as u32;

        self.templates.insert(clip_id, ClipTemplate {
            parent,
            clips: start..end,
        });
    }

    pub fn get_template(
        &self,
        clip_id: ClipId,
    ) -> &ClipTemplate {
        &self.templates[&clip_id]
    }

    /// The main method used to build a clip-chain for a given ClipId on a primitive
    pub fn get_or_build_clip_chain_id(
        &mut self,
        clip_id: ClipId,
    ) -> ClipChainId {
        // TODO(gw): If many primitives reference the same ClipId, it might be worth
        //           maintaining a hash map cache of ClipId -> ClipChainId in each
        //           ClipChainBuilder

        self.chain_builder_stack
            .last_mut()
            .unwrap()
            .get_or_build_clip_chain_id(
                clip_id,
                &mut self.clip_chain_nodes,
                &self.templates,
                &self.instances,
            )
    }

    /// Return true if any of the clips in the hierarchy from clip_id to the
    /// root clip are complex.
    // TODO(gw): This method should only be required until the shared_clip
    //           optimization patches are complete, and can then be removed.
    pub fn has_complex_clips(
        &self,
        clip_id: ClipId,
    ) -> bool {
        self.chain_builder_stack
            .last()
            .unwrap()
            .has_complex_clips(
                clip_id,
                &self.templates,
                &self.instances,
            )
    }

    /// Push a new clip root. This is used at boundaries of clips (such as iframes
    /// and stacking contexts). This means that any clips on the existing clip
    /// chain builder will not be added to clip-chains defined within this level,
    /// since the clips will be applied by the parent.
    pub fn push_clip_root(
        &mut self,
        clip_id: Option<ClipId>,
        link_to_parent: bool,
    ) {
        let parent_clip_chain_id = if link_to_parent {
            self.chain_builder_stack.last().unwrap().clip_chain_id
        } else {
            ClipChainId::NONE
        };

        let builder = ClipChainBuilder::new(
            parent_clip_chain_id,
            clip_id,
            &mut self.clip_chain_nodes,
            &self.templates,
            &self.instances,
        );

        self.chain_builder_stack.push(builder);
    }

    /// On completion of a stacking context or iframe, pop the current clip root.
    pub fn pop_clip_root(
        &mut self,
    ) {
        self.chain_builder_stack.pop().unwrap();
    }

    pub fn get_clip_chain(&self, clip_chain_id: ClipChainId) -> &ClipChainNode {
        &self.clip_chain_nodes[clip_chain_id.0 as usize]
    }

    pub fn add_clip_chain_node(
        &mut self,
        handle: ClipDataHandle,
        spatial_node_index: SpatialNodeIndex,
        parent_clip_chain_id: ClipChainId,
    ) -> ClipChainId {
        let id = ClipChainId(self.clip_chain_nodes.len() as u32);
        self.clip_chain_nodes.push(ClipChainNode {
            handle,
            spatial_node_index,
            parent_clip_chain_id,
        });
        id
    }

    pub fn get_instance_from_range(
        &self,
        node_range: &ClipNodeRange,
        index: u32,
    ) -> &ClipNodeInstance {
        &self.clip_node_instances[(node_range.first + index) as usize]
    }

    /// Setup the active clip chains for building a clip chain instance.
    pub fn set_active_clips(
        &mut self,
        local_prim_clip_rect: LayoutRect,
        prim_spatial_node_index: SpatialNodeIndex,
        pic_spatial_node_index: SpatialNodeIndex,
        clip_chains: &[ClipChainId],
        spatial_tree: &SpatialTree,
        clip_data_store: &ClipDataStore,
    ) {
        self.active_clip_node_info.clear();
        self.active_local_clip_rect = None;
        self.active_pic_clip_rect = PictureRect::max_rect();

        let mut local_clip_rect = local_prim_clip_rect;

        for clip_chain_id in clip_chains {
            let clip_chain_node = &self.clip_chain_nodes[clip_chain_id.0 as usize];

            if !add_clip_node_to_current_chain(
                clip_chain_node,
                prim_spatial_node_index,
                pic_spatial_node_index,
                &mut local_clip_rect,
                &mut self.active_clip_node_info,
                &mut self.active_pic_clip_rect,
                clip_data_store,
                spatial_tree,
            ) {
                return;
            }
        }

        self.active_local_clip_rect = Some(local_clip_rect);
    }

    /// Setup the active clip chains, based on an existing primitive clip chain instance.
    pub fn set_active_clips_from_clip_chain(
        &mut self,
        prim_clip_chain: &ClipChainInstance,
        prim_spatial_node_index: SpatialNodeIndex,
        spatial_tree: &SpatialTree,
    ) {
        // TODO(gw): Although this does less work than set_active_clips(), it does
        //           still do some unnecessary work (such as the clip space conversion).
        //           We could consider optimizing this if it ever shows up in a profile.

        self.active_clip_node_info.clear();
        self.active_local_clip_rect = Some(prim_clip_chain.local_clip_rect);
        self.active_pic_clip_rect = prim_clip_chain.pic_clip_rect;

        let clip_instances = &self
            .clip_node_instances[prim_clip_chain.clips_range.to_range()];
        for clip_instance in clip_instances {
            let conversion = ClipSpaceConversion::new(
                prim_spatial_node_index,
                clip_instance.spatial_node_index,
                spatial_tree,
            );
            self.active_clip_node_info.push(ClipNodeInfo {
                handle: clip_instance.handle,
                spatial_node_index: clip_instance.spatial_node_index,
                conversion,
            });
        }
    }

    /// The main interface external code uses. Given a local primitive, positioning
    /// information, and a clip chain id, build an optimized clip chain instance.
    pub fn build_clip_chain_instance(
        &mut self,
        local_prim_rect: LayoutRect,
        prim_to_pic_mapper: &SpaceMapper<LayoutPixel, PicturePixel>,
        pic_to_world_mapper: &SpaceMapper<PicturePixel, WorldPixel>,
        spatial_tree: &SpatialTree,
        gpu_cache: &mut GpuCache,
        resource_cache: &mut ResourceCache,
        device_pixel_scale: DevicePixelScale,
        world_rect: &WorldRect,
        clip_data_store: &mut ClipDataStore,
        request_resources: bool,
        is_chased: bool,
    ) -> Option<ClipChainInstance> {
        let local_clip_rect = match self.active_local_clip_rect {
            Some(rect) => rect,
            None => return None,
        };
        profile_scope!("build_clip_chain_instance");
        if is_chased {
            println!("\tbuilding clip chain instance with local rect {:?}", local_prim_rect);
        }

        let local_bounding_rect = local_prim_rect.intersection(&local_clip_rect)?;
        let mut pic_clip_rect = prim_to_pic_mapper.map(&local_bounding_rect)?;
        let world_clip_rect = pic_to_world_mapper.map(&pic_clip_rect)?;

        // Now, we've collected all the clip nodes that *potentially* affect this
        // primitive region, and reduced the size of the prim region as much as possible.

        // Run through the clip nodes, and see which ones affect this prim region.

        let first_clip_node_index = self.clip_node_instances.len() as u32;
        let mut has_non_local_clips = false;
        let mut needs_mask = false;

        // For each potential clip node
        for node_info in self.active_clip_node_info.drain(..) {
            let node = &mut clip_data_store[node_info.handle];

            // See how this clip affects the prim region.
            let clip_result = match node_info.conversion {
                ClipSpaceConversion::Local => {
                    node.item.kind.get_clip_result(&local_bounding_rect)
                }
                ClipSpaceConversion::ScaleOffset(ref scale_offset) => {
                    has_non_local_clips = true;
                    node.item.kind.get_clip_result(&scale_offset.unmap_rect(&local_bounding_rect))
                }
                ClipSpaceConversion::Transform(ref transform) => {
                    has_non_local_clips = true;
                    node.item.kind.get_clip_result_complex(
                        transform,
                        &world_clip_rect,
                        world_rect,
                    )
                }
            };

            if is_chased {
                println!("\t\tclip {:?}", node.item);
                println!("\t\tflags {:?}, resulted in {:?}", node_info.conversion.to_flags(), clip_result);
            }

            match clip_result {
                ClipResult::Accept => {
                    // Doesn't affect the primitive at all, so skip adding to list
                }
                ClipResult::Reject => {
                    // Completely clips the supplied prim rect
                    return None;
                }
                ClipResult::Partial => {
                    // Needs a mask -> add to clip node indices

                    // TODO(gw): Ensure this only runs once on each node per frame?
                    node.update(device_pixel_scale);

                    // Create the clip node instance for this clip node
                    if let Some(instance) = node_info.create_instance(
                        node,
                        &local_bounding_rect,
                        gpu_cache,
                        resource_cache,
                        &mut self.mask_tiles,
                        spatial_tree,
                        request_resources,
                    ) {
                        // As a special case, a partial accept of a clip rect that is
                        // in the same coordinate system as the primitive doesn't need
                        // a clip mask. Instead, it can be handled by the primitive
                        // vertex shader as part of the local clip rect. This is an
                        // important optimization for reducing the number of clip
                        // masks that are allocated on common pages.
                        needs_mask |= match node.item.kind {
                            ClipItemKind::Rectangle { mode: ClipMode::ClipOut, .. } |
                            ClipItemKind::RoundedRectangle { .. } |
                            ClipItemKind::Image { .. } |
                            ClipItemKind::BoxShadow { .. } => {
                                true
                            }

                            ClipItemKind::Rectangle { mode: ClipMode::Clip, .. } => {
                                !instance.flags.contains(ClipNodeFlags::SAME_COORD_SYSTEM)
                            }
                        };

                        // Store this in the index buffer for this clip chain instance.
                        self.clip_node_instances.push(instance);
                    }
                }
            }
        }

        // Get the range identifying the clip nodes in the index buffer.
        let clips_range = ClipNodeRange {
            first: first_clip_node_index,
            count: self.clip_node_instances.len() as u32 - first_clip_node_index,
        };

        // If this clip chain needs a mask, reduce the size of the mask allocation
        // by any clips that were in the same space as the picture. This can result
        // in much smaller clip mask allocations in some cases. Note that the ordering
        // here is important - the reduction must occur *after* the clip item accept
        // reject checks above, so that we don't eliminate masks accidentally (since
        // we currently only support a local clip rect in the vertex shader).
        if needs_mask {
            pic_clip_rect = pic_clip_rect.intersection(&self.active_pic_clip_rect)?;
        }

        // Return a valid clip chain instance
        Some(ClipChainInstance {
            clips_range,
            has_non_local_clips,
            local_clip_rect,
            pic_clip_rect,
            pic_spatial_node_index: prim_to_pic_mapper.ref_spatial_node_index,
            needs_mask,
        })
    }

    pub fn begin_frame(&mut self, scratch: &mut ClipStoreScratchBuffer) {
        mem::swap(&mut self.clip_node_instances, &mut scratch.clip_node_instances);
        mem::swap(&mut self.mask_tiles, &mut scratch.mask_tiles);
        self.clip_node_instances.clear();
        self.mask_tiles.clear();
    }

    pub fn end_frame(&mut self, scratch: &mut ClipStoreScratchBuffer) {
        mem::swap(&mut self.clip_node_instances, &mut scratch.clip_node_instances);
        mem::swap(&mut self.mask_tiles, &mut scratch.mask_tiles);
    }

    pub fn visible_mask_tiles(&self, instance: &ClipNodeInstance) -> &[VisibleMaskImageTile] {
        if let Some(range) = &instance.visible_tiles {
            &self.mask_tiles[range.clone()]
        } else {
            &[]
        }
    }
}

pub struct ComplexTranslateIter<I> {
    source: I,
    offset: LayoutVector2D,
}

impl<I: Iterator<Item = ComplexClipRegion>> Iterator for ComplexTranslateIter<I> {
    type Item = ComplexClipRegion;
    fn next(&mut self) -> Option<Self::Item> {
        self.source
            .next()
            .map(|mut complex| {
                complex.rect = complex.rect.translate(self.offset);
                complex
            })
    }
}

#[derive(Clone, Debug)]
pub struct ClipRegion<I> {
    pub main: LayoutRect,
    pub complex_clips: I,
}

impl<J> ClipRegion<ComplexTranslateIter<J>> {
    pub fn create_for_clip_node(
        rect: LayoutRect,
        complex_clips: J,
        reference_frame_relative_offset: &LayoutVector2D,
    ) -> Self
    where
        J: Iterator<Item = ComplexClipRegion>
    {
        ClipRegion {
            main: rect.translate(*reference_frame_relative_offset),
            complex_clips: ComplexTranslateIter {
                source: complex_clips,
                offset: *reference_frame_relative_offset,
            },
        }
    }
}

// The ClipItemKey is a hashable representation of the contents
// of a clip item. It is used during interning to de-duplicate
// clip nodes between frames and display lists. This allows quick
// comparison of clip node equality by handle, and also allows
// the uploaded GPU cache handle to be retained between display lists.
// TODO(gw): Maybe we should consider constructing these directly
//           in the DL builder?
#[derive(Copy, Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum ClipItemKeyKind {
    Rectangle(RectangleKey, ClipMode),
    RoundedRectangle(RectangleKey, BorderRadiusAu, ClipMode),
    ImageMask(RectangleKey, ImageKey, bool, Option<PolygonDataHandle>),
    BoxShadow(PointKey, SizeKey, BorderRadiusAu, RectangleKey, Au, BoxShadowClipMode),
}

impl ClipItemKeyKind {
    pub fn rectangle(rect: LayoutRect, mode: ClipMode) -> Self {
        ClipItemKeyKind::Rectangle(rect.into(), mode)
    }

    pub fn rounded_rect(rect: LayoutRect, mut radii: BorderRadius, mode: ClipMode) -> Self {
        if radii.is_zero() {
            ClipItemKeyKind::rectangle(rect, mode)
        } else {
            ensure_no_corner_overlap(&mut radii, rect.size);
            ClipItemKeyKind::RoundedRectangle(
                rect.into(),
                radii.into(),
                mode,
            )
        }
    }

    pub fn image_mask(image_mask: &ImageMask, mask_rect: LayoutRect,
                      polygon_handle: Option<PolygonDataHandle>) -> Self {
        ClipItemKeyKind::ImageMask(
            mask_rect.into(),
            image_mask.image,
            image_mask.repeat,
            polygon_handle,
        )
    }

    pub fn box_shadow(
        shadow_rect: LayoutRect,
        shadow_radius: BorderRadius,
        prim_shadow_rect: LayoutRect,
        blur_radius: f32,
        clip_mode: BoxShadowClipMode,
    ) -> Self {
        // Get the fractional offsets required to match the
        // source rect with a minimal rect.
        let fract_offset = LayoutPoint::new(
            shadow_rect.origin.x.fract().abs(),
            shadow_rect.origin.y.fract().abs(),
        );

        ClipItemKeyKind::BoxShadow(
            fract_offset.into(),
            shadow_rect.size.into(),
            shadow_radius.into(),
            prim_shadow_rect.into(),
            Au::from_f32_px(blur_radius),
            clip_mode,
        )
    }

    pub fn node_kind(&self) -> ClipNodeKind {
        match *self {
            ClipItemKeyKind::Rectangle(_, ClipMode::Clip) => ClipNodeKind::Rectangle,

            ClipItemKeyKind::Rectangle(_, ClipMode::ClipOut) |
            ClipItemKeyKind::RoundedRectangle(..) |
            ClipItemKeyKind::ImageMask(..) |
            ClipItemKeyKind::BoxShadow(..) => ClipNodeKind::Complex,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ClipItemKey {
    pub kind: ClipItemKeyKind,
}

/// The data available about an interned clip node during scene building
#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ClipInternData {
    /// Whether this is a simple rectangle clip
    pub clip_node_kind: ClipNodeKind,
}

impl intern::InternDebug for ClipItemKey {}

impl intern::Internable for ClipIntern {
    type Key = ClipItemKey;
    type StoreData = ClipNode;
    type InternData = ClipInternData;
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_CLIPS;
}

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum ClipItemKind {
    Rectangle {
        rect: LayoutRect,
        mode: ClipMode,
    },
    RoundedRectangle {
        rect: LayoutRect,
        radius: BorderRadius,
        mode: ClipMode,
    },
    Image {
        image: ImageKey,
        rect: LayoutRect,
        repeat: bool,
        polygon_handle: Option<PolygonDataHandle>,
    },
    BoxShadow {
        source: BoxShadowClipSource,
    },
}

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ClipItem {
    pub kind: ClipItemKind,
}

fn compute_box_shadow_parameters(
    shadow_rect_fract_offset: LayoutPoint,
    shadow_rect_size: LayoutSize,
    mut shadow_radius: BorderRadius,
    prim_shadow_rect: LayoutRect,
    blur_radius: f32,
    clip_mode: BoxShadowClipMode,
) -> BoxShadowClipSource {
    // Make sure corners don't overlap.
    ensure_no_corner_overlap(&mut shadow_radius, shadow_rect_size);

    let fract_size = LayoutSize::new(
        shadow_rect_size.width.fract().abs(),
        shadow_rect_size.height.fract().abs(),
    );

    // Create a minimal size primitive mask to blur. In this
    // case, we ensure the size of each corner is the same,
    // to simplify the shader logic that stretches the blurred
    // result across the primitive.
    let max_corner_width = shadow_radius.top_left.width
                                .max(shadow_radius.bottom_left.width)
                                .max(shadow_radius.top_right.width)
                                .max(shadow_radius.bottom_right.width);
    let max_corner_height = shadow_radius.top_left.height
                                .max(shadow_radius.bottom_left.height)
                                .max(shadow_radius.top_right.height)
                                .max(shadow_radius.bottom_right.height);

    // Get maximum distance that can be affected by given blur radius.
    let blur_region = (BLUR_SAMPLE_SCALE * blur_radius).ceil();

    // If the largest corner is smaller than the blur radius, we need to ensure
    // that it's big enough that the corners don't affect the middle segments.
    let used_corner_width = max_corner_width.max(blur_region);
    let used_corner_height = max_corner_height.max(blur_region);

    // Minimal nine-patch size, corner + internal + corner.
    let min_shadow_rect_size = LayoutSize::new(
        2.0 * used_corner_width + blur_region,
        2.0 * used_corner_height + blur_region,
    );

    // The minimal rect to blur.
    let mut minimal_shadow_rect = LayoutRect::new(
        LayoutPoint::new(
            blur_region + shadow_rect_fract_offset.x,
            blur_region + shadow_rect_fract_offset.y,
        ),
        LayoutSize::new(
            min_shadow_rect_size.width + fract_size.width,
            min_shadow_rect_size.height + fract_size.height,
        ),
    );

    // If the width or height ends up being bigger than the original
    // primitive shadow rect, just blur the entire rect along that
    // axis and draw that as a simple blit. This is necessary for
    // correctness, since the blur of one corner may affect the blur
    // in another corner.
    let mut stretch_mode_x = BoxShadowStretchMode::Stretch;
    if shadow_rect_size.width < minimal_shadow_rect.size.width {
        minimal_shadow_rect.size.width = shadow_rect_size.width;
        stretch_mode_x = BoxShadowStretchMode::Simple;
    }

    let mut stretch_mode_y = BoxShadowStretchMode::Stretch;
    if shadow_rect_size.height < minimal_shadow_rect.size.height {
        minimal_shadow_rect.size.height = shadow_rect_size.height;
        stretch_mode_y = BoxShadowStretchMode::Simple;
    }

    // Expand the shadow rect by enough room for the blur to take effect.
    let shadow_rect_alloc_size = LayoutSize::new(
        2.0 * blur_region + minimal_shadow_rect.size.width.ceil(),
        2.0 * blur_region + minimal_shadow_rect.size.height.ceil(),
    );

    BoxShadowClipSource {
        original_alloc_size: shadow_rect_alloc_size,
        shadow_rect_alloc_size,
        shadow_radius,
        prim_shadow_rect,
        blur_radius,
        clip_mode,
        stretch_mode_x,
        stretch_mode_y,
        render_task: None,
        cache_key: None,
        minimal_shadow_rect,
    }
}

impl ClipItemKind {
    pub fn new_box_shadow(
        shadow_rect_fract_offset: LayoutPoint,
        shadow_rect_size: LayoutSize,
        mut shadow_radius: BorderRadius,
        prim_shadow_rect: LayoutRect,
        blur_radius: f32,
        clip_mode: BoxShadowClipMode,
    ) -> Self {
        let mut source = compute_box_shadow_parameters(
            shadow_rect_fract_offset,
            shadow_rect_size,
            shadow_radius,
            prim_shadow_rect,
            blur_radius,
            clip_mode,
        );

        fn needed_downscaling(source: &BoxShadowClipSource) -> Option<f32> {
            // This size is fairly arbitrary, but it's the same as the size that
            // we use to avoid caching big blurred stacking contexts.
            //
            // If you change it, ensure that the reftests
            // box-shadow-large-blur-radius-* still hit the downscaling path,
            // and that they render correctly.
            const MAX_SIZE: f32 = 2048.;

            let max_dimension =
                source.shadow_rect_alloc_size.width.max(source.shadow_rect_alloc_size.height);

            if max_dimension > MAX_SIZE {
                Some(MAX_SIZE / max_dimension)
            } else {
                None
            }
        }

        if let Some(downscale) = needed_downscaling(&source) {
            shadow_radius.bottom_left.height *= downscale;
            shadow_radius.bottom_left.width *= downscale;
            shadow_radius.bottom_right.height *= downscale;
            shadow_radius.bottom_right.width *= downscale;
            shadow_radius.top_left.height *= downscale;
            shadow_radius.top_left.width *= downscale;
            shadow_radius.top_right.height *= downscale;
            shadow_radius.top_right.width *= downscale;

            let original_alloc_size = source.shadow_rect_alloc_size;

            source = compute_box_shadow_parameters(
                shadow_rect_fract_offset * downscale,
                shadow_rect_size * downscale,
                shadow_radius,
                prim_shadow_rect,
                blur_radius * downscale,
                clip_mode,
            );
            source.original_alloc_size = original_alloc_size;
        }
        ClipItemKind::BoxShadow { source }
    }

    /// Returns true if this clip mask can run through the fast path
    /// for the given clip item type.
    ///
    /// Note: this logic has to match `ClipBatcher::add` behavior.
    fn supports_fast_path_rendering(&self) -> bool {
        match *self {
            ClipItemKind::Rectangle { .. } |
            ClipItemKind::Image { .. } |
            ClipItemKind::BoxShadow { .. } => {
                false
            }
            ClipItemKind::RoundedRectangle { ref radius, .. } => {
                // The rounded clip rect fast path shader can only work
                // if the radii are uniform.
                radius.is_uniform().is_some()
            }
        }
    }

    // Get an optional clip rect that a clip source can provide to
    // reduce the size of a primitive region. This is typically
    // used to eliminate redundant clips, and reduce the size of
    // any clip mask that eventually gets drawn.
    pub fn get_local_clip_rect(&self) -> Option<LayoutRect> {
        match *self {
            ClipItemKind::Rectangle { rect, mode: ClipMode::Clip } => Some(rect),
            ClipItemKind::Rectangle { mode: ClipMode::ClipOut, .. } => None,
            ClipItemKind::RoundedRectangle { rect, mode: ClipMode::Clip, .. } => Some(rect),
            ClipItemKind::RoundedRectangle { mode: ClipMode::ClipOut, .. } => None,
            ClipItemKind::Image { repeat, rect, .. } => {
                if repeat {
                    None
                } else {
                    Some(rect)
                }
            }
            ClipItemKind::BoxShadow { .. } => None,
        }
    }

    fn get_clip_result_complex(
        &self,
        transform: &LayoutToWorldTransform,
        prim_world_rect: &WorldRect,
        world_rect: &WorldRect,
    ) -> ClipResult {
        let visible_rect = match prim_world_rect.intersection(world_rect) {
            Some(rect) => rect,
            None => return ClipResult::Reject,
        };

        let (clip_rect, inner_rect, mode) = match *self {
            ClipItemKind::Rectangle { rect, mode } => {
                (rect, Some(rect), mode)
            }
            ClipItemKind::RoundedRectangle { rect, ref radius, mode } => {
                let inner_clip_rect = extract_inner_rect_safe(&rect, radius);
                (rect, inner_clip_rect, mode)
            }
            ClipItemKind::Image { rect, repeat: false, .. } => {
                (rect, None, ClipMode::Clip)
            }
            ClipItemKind::Image { repeat: true, .. } |
            ClipItemKind::BoxShadow { .. } => {
                return ClipResult::Partial;
            }
        };

        if let Some(ref inner_clip_rect) = inner_rect {
            if let Some(()) = projected_rect_contains(inner_clip_rect, transform, &visible_rect) {
                return match mode {
                    ClipMode::Clip => ClipResult::Accept,
                    ClipMode::ClipOut => ClipResult::Reject,
                };
            }
        }

        match mode {
            ClipMode::Clip => {
                let outer_clip_rect = match project_rect(
                    transform,
                    &clip_rect,
                    world_rect,
                ) {
                    Some(outer_clip_rect) => outer_clip_rect,
                    None => return ClipResult::Partial,
                };

                match outer_clip_rect.intersection(prim_world_rect) {
                    Some(..) => {
                        ClipResult::Partial
                    }
                    None => {
                        ClipResult::Reject
                    }
                }
            }
            ClipMode::ClipOut => ClipResult::Partial,
        }
    }

    // Check how a given clip source affects a local primitive region.
    fn get_clip_result(
        &self,
        prim_rect: &LayoutRect,
    ) -> ClipResult {
        match *self {
            ClipItemKind::Rectangle { rect, mode: ClipMode::Clip } => {
                if rect.contains_rect(prim_rect) {
                    return ClipResult::Accept;
                }

                match rect.intersection(prim_rect) {
                    Some(..) => {
                        ClipResult::Partial
                    }
                    None => {
                        ClipResult::Reject
                    }
                }
            }
            ClipItemKind::Rectangle { rect, mode: ClipMode::ClipOut } => {
                if rect.contains_rect(prim_rect) {
                    return ClipResult::Reject;
                }

                match rect.intersection(prim_rect) {
                    Some(_) => {
                        ClipResult::Partial
                    }
                    None => {
                        ClipResult::Accept
                    }
                }
            }
            ClipItemKind::RoundedRectangle { rect, ref radius, mode: ClipMode::Clip } => {
                // TODO(gw): Consider caching this in the ClipNode
                //           if it ever shows in profiles.
                if rounded_rectangle_contains_rect_quick(&rect, radius, &prim_rect) {
                    return ClipResult::Accept;
                }

                match rect.intersection(prim_rect) {
                    Some(..) => {
                        ClipResult::Partial
                    }
                    None => {
                        ClipResult::Reject
                    }
                }
            }
            ClipItemKind::RoundedRectangle { rect, ref radius, mode: ClipMode::ClipOut } => {
                // TODO(gw): Consider caching this in the ClipNode
                //           if it ever shows in profiles.
                if rounded_rectangle_contains_rect_quick(&rect, radius, &prim_rect) {
                    return ClipResult::Reject;
                }

                match rect.intersection(prim_rect) {
                    Some(_) => {
                        ClipResult::Partial
                    }
                    None => {
                        ClipResult::Accept
                    }
                }
            }
            ClipItemKind::Image { rect, repeat, .. } => {
                if repeat {
                    ClipResult::Partial
                } else {
                    match rect.intersection(prim_rect) {
                        Some(..) => {
                            ClipResult::Partial
                        }
                        None => {
                            ClipResult::Reject
                        }
                    }
                }
            }
            ClipItemKind::BoxShadow { .. } => {
                ClipResult::Partial
            }
        }
    }
}

/// Represents a local rect and a device space
/// rectangles that are either outside or inside bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub local_rect: LayoutRect,
    pub device_rect: DeviceIntRect,
}

impl From<LayoutRect> for Geometry {
    fn from(local_rect: LayoutRect) -> Self {
        Geometry {
            local_rect,
            device_rect: DeviceIntRect::zero(),
        }
    }
}

pub fn rounded_rectangle_contains_point(
    point: &LayoutPoint,
    rect: &LayoutRect,
    radii: &BorderRadius
) -> bool {
    if !rect.contains(*point) {
        return false;
    }

    let top_left_center = rect.origin + radii.top_left.to_vector();
    if top_left_center.x > point.x && top_left_center.y > point.y &&
       !Ellipse::new(radii.top_left).contains(*point - top_left_center.to_vector()) {
        return false;
    }

    let bottom_right_center = rect.bottom_right() - radii.bottom_right.to_vector();
    if bottom_right_center.x < point.x && bottom_right_center.y < point.y &&
       !Ellipse::new(radii.bottom_right).contains(*point - bottom_right_center.to_vector()) {
        return false;
    }

    let top_right_center = rect.top_right() +
                           LayoutVector2D::new(-radii.top_right.width, radii.top_right.height);
    if top_right_center.x < point.x && top_right_center.y > point.y &&
       !Ellipse::new(radii.top_right).contains(*point - top_right_center.to_vector()) {
        return false;
    }

    let bottom_left_center = rect.bottom_left() +
                             LayoutVector2D::new(radii.bottom_left.width, -radii.bottom_left.height);
    if bottom_left_center.x > point.x && bottom_left_center.y < point.y &&
       !Ellipse::new(radii.bottom_left).contains(*point - bottom_left_center.to_vector()) {
        return false;
    }

    true
}

/// Return true if the rounded rectangle described by `container` and `radii`
/// definitely contains `containee`. May return false negatives, but never false
/// positives.
fn rounded_rectangle_contains_rect_quick(
    container: &LayoutRect,
    radii: &BorderRadius,
    containee: &LayoutRect,
) -> bool {
    if !container.contains_rect(containee) {
        return false;
    }

    /// Return true if `point` falls within `corner`. This only covers the
    /// upper-left case; we transform the other corners into that form.
    fn foul(point: LayoutPoint, corner: LayoutPoint) -> bool {
        point.x < corner.x && point.y < corner.y
    }

    /// Flip `pt` about the y axis (i.e. negate `x`).
    fn flip_x(pt: LayoutPoint) -> LayoutPoint {
        LayoutPoint { x: -pt.x, .. pt }
    }

    /// Flip `pt` about the x axis (i.e. negate `y`).
    fn flip_y(pt: LayoutPoint) -> LayoutPoint {
        LayoutPoint { y: -pt.y, .. pt }
    }

    if foul(containee.top_left(), container.top_left() + radii.top_left) ||
        foul(flip_x(containee.top_right()), flip_x(container.top_right()) + radii.top_right) ||
        foul(flip_y(containee.bottom_left()), flip_y(container.bottom_left()) + radii.bottom_left) ||
        foul(-containee.bottom_right(), -container.bottom_right() + radii.bottom_right)
    {
        return false;
    }

    true
}

/// Test where point p is relative to the infinite line that passes through the segment
/// defined by p0 and p1. Point p is on the "left" of the line if the triangle (p0, p1, p)
/// forms a counter-clockwise triangle.
/// > 0 is left of the line
/// < 0 is right of the line
/// == 0 is on the line
pub fn is_left_of_line(
    p_x: f32,
    p_y: f32,
    p0_x: f32,
    p0_y: f32,
    p1_x: f32,
    p1_y: f32,
) -> f32 {
    (p1_x - p0_x) * (p_y - p0_y) - (p_x - p0_x) * (p1_y - p0_y)
}

pub fn polygon_contains_point(
    point: &LayoutPoint,
    rect: &LayoutRect,
    polygon: &PolygonKey,
) -> bool {
    if !rect.contains(*point) {
        return false;
    }

    // p is a LayoutPoint that we'll be comparing to dimensionless PointKeys,
    // which were created from LayoutPoints, so it all works out.
    let p = LayoutPoint::new(point.x - rect.origin.x, point.y - rect.origin.y);

    // Calculate a winding number for this point.
    let mut winding_number: i32 = 0;

    let count = polygon.point_count as usize;

    for i in 0..count {
        let p0 = polygon.points[i];
        let p1 = polygon.points[(i + 1) % count];

        if p0.y <= p.y {
            if p1.y > p.y {
                if is_left_of_line(p.x, p.y, p0.x, p0.y, p1.x, p1.y) > 0.0 {
                    winding_number = winding_number + 1;
                }
            }
        } else if p1.y <= p.y {
            if is_left_of_line(p.x, p.y, p0.x, p0.y, p1.x, p1.y) < 0.0 {
                winding_number = winding_number - 1;
            }
        }
    }

    match polygon.fill_rule {
        FillRule::Nonzero => winding_number != 0,
        FillRule::Evenodd => winding_number.abs() % 2 == 1,
    }
}

pub fn projected_rect_contains(
    source_rect: &LayoutRect,
    transform: &LayoutToWorldTransform,
    target_rect: &WorldRect,
) -> Option<()> {
    let points = [
        transform.transform_point2d(source_rect.origin)?,
        transform.transform_point2d(source_rect.top_right())?,
        transform.transform_point2d(source_rect.bottom_right())?,
        transform.transform_point2d(source_rect.bottom_left())?,
    ];
    let target_points = [
        target_rect.origin,
        target_rect.top_right(),
        target_rect.bottom_right(),
        target_rect.bottom_left(),
    ];
    // iterate the edges of the transformed polygon
    for (a, b) in points
        .iter()
        .cloned()
        .zip(points[1..].iter().cloned().chain(iter::once(points[0])))
    {
        // If this edge is redundant, it's a weird, case, and we shouldn't go
        // length in trying to take the fast path (e.g. when the whole rectangle is a point).
        // If any of edges of the target rectangle crosses the edge, it's not completely
        // inside our transformed polygon either.
        if a.approx_eq(&b) || target_points.iter().any(|&c| (b - a).cross(c - a) < 0.0) {
            return None
        }
    }

    Some(())
}


// Add a clip node into the list of clips to be processed
// for the current clip chain. Returns false if the clip
// results in the entire primitive being culled out.
fn add_clip_node_to_current_chain(
    node: &ClipChainNode,
    prim_spatial_node_index: SpatialNodeIndex,
    pic_spatial_node_index: SpatialNodeIndex,
    local_clip_rect: &mut LayoutRect,
    clip_node_info: &mut Vec<ClipNodeInfo>,
    current_pic_clip_rect: &mut PictureRect,
    clip_data_store: &ClipDataStore,
    spatial_tree: &SpatialTree,
) -> bool {
    let clip_node = &clip_data_store[node.handle];

    // Determine the most efficient way to convert between coordinate
    // systems of the primitive and clip node.
    let conversion = ClipSpaceConversion::new(
        prim_spatial_node_index,
        node.spatial_node_index,
        spatial_tree,
    );

    // If we can convert spaces, try to reduce the size of the region
    // requested, and cache the conversion information for the next step.
    if let Some(clip_rect) = clip_node.item.kind.get_local_clip_rect() {
        match conversion {
            ClipSpaceConversion::Local => {
                *local_clip_rect = match local_clip_rect.intersection(&clip_rect) {
                    Some(rect) => rect,
                    None => return false,
                };
            }
            ClipSpaceConversion::ScaleOffset(ref scale_offset) => {
                let clip_rect = scale_offset.map_rect(&clip_rect);
                *local_clip_rect = match local_clip_rect.intersection(&clip_rect) {
                    Some(rect) => rect,
                    None => return false,
                };
            }
            ClipSpaceConversion::Transform(..) => {
                // Map the local clip rect directly into the same space as the picture
                // surface. This will often be the same space as the clip itself, which
                // results in a reduction in allocated clip mask size.

                // For simplicity, only apply this optimization if the clip is in the
                // same coord system as the picture. There are some 'advanced' perspective
                // clip tests in wrench that break without this check. Those cases are
                // never used in Gecko, and we aim to remove support in WR for that
                // in future to simplify the clipping pipeline.
                let pic_coord_system = spatial_tree
                    .spatial_nodes[pic_spatial_node_index.0 as usize]
                    .coordinate_system_id;

                let clip_coord_system = spatial_tree
                    .spatial_nodes[node.spatial_node_index.0 as usize]
                    .coordinate_system_id;

                if pic_coord_system == clip_coord_system {
                    let mapper = SpaceMapper::new_with_target(
                        pic_spatial_node_index,
                        node.spatial_node_index,
                        PictureRect::max_rect(),
                        spatial_tree,
                    );

                    if let Some(pic_clip_rect) = mapper.map(&clip_rect) {
                        *current_pic_clip_rect = pic_clip_rect
                            .intersection(current_pic_clip_rect)
                            .unwrap_or(PictureRect::zero());
                    }
                }
            }
        }
    }

    clip_node_info.push(ClipNodeInfo {
        conversion,
        spatial_node_index: node.spatial_node_index,
        handle: node.handle,
    });

    true
}

#[cfg(test)]
mod tests {
    use super::projected_rect_contains;
    use euclid::{Transform3D, rect};

    #[test]
    fn test_empty_projected_rect() {
        assert_eq!(
            None,
            projected_rect_contains(
                &rect(10.0, 10.0, 0.0, 0.0),
                &Transform3D::identity(),
                &rect(20.0, 20.0, 10.0, 10.0),
            ),
            "Empty rectangle is considered to include a non-empty!"
        );
    }
}

/// PolygonKeys get interned, because it's a convenient way to move the data
/// for the polygons out of the ClipItemKind and ClipItemKeyKind enums. The
/// polygon data is both interned and retrieved by the scene builder, and not
/// accessed at all by the frame builder. Another oddity is that the
/// PolygonKey contains the totality of the information about the polygon, so
/// the InternData and StoreData types are both PolygonKey.
#[derive(Copy, Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub enum PolygonIntern {}

pub type PolygonDataHandle = intern::Handle<PolygonIntern>;

impl intern::InternDebug for PolygonKey {}

impl intern::Internable for PolygonIntern {
    type Key = PolygonKey;
    type StoreData = PolygonKey;
    type InternData = PolygonKey;
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_POLYGONS;
}
