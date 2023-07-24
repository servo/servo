/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level module responsible for managing the pipeline and preparing
//! commands to be issued by the `Renderer`.
//!
//! See the comment at the top of the `renderer` module for a description of
//! how these two pieces interact.

use api::{ApiMsg, ClearCache, DebugCommand, DebugFlags, BlobImageHandler};
use api::{DocumentId, DocumentLayer, ExternalScrollId, FrameMsg, HitTestFlags, HitTestResult};
use api::{IdNamespace, MemoryReport, PipelineId, RenderNotifier, ScrollClamping};
use api::{ScrollLocation, TransactionMsg, ResourceUpdate};
use api::{NotificationRequest, Checkpoint, QualitySettings};
use api::{ClipIntern, FilterDataIntern, PrimitiveKeyKind};
use api::units::*;
#[cfg(any(feature = "capture", feature = "replay"))]
use api::CaptureBits;
#[cfg(feature = "replay")]
use api::CapturedDocument;
use crate::spatial_tree::SpatialNodeIndex;
#[cfg(any(feature = "capture", feature = "replay"))]
use crate::capture::CaptureConfig;
use crate::composite::{CompositorKind, CompositeDescriptor};
#[cfg(feature = "debugger")]
use crate::debug_server;
use crate::frame_builder::{FrameBuilder, FrameBuilderConfig};
use crate::glyph_rasterizer::{FontInstance};
use crate::gpu_cache::GpuCache;
use crate::hit_test::{HitTest, HitTester, SharedHitTester};
use crate::intern::DataStore;
use crate::internal_types::{DebugOutput, FastHashMap, RenderedDocument, ResultMsg};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use crate::picture::{RetainedTiles, TileCacheLogger};
use crate::prim_store::{PrimitiveScratchBuffer, PrimitiveInstance};
use crate::prim_store::{PrimitiveInstanceKind, PrimTemplateCommonData, PrimitiveStore};
use crate::prim_store::interned::*;
use crate::profiler::{BackendProfileCounters, ResourceProfileCounters};
use crate::render_task_graph::RenderTaskGraphCounters;
use crate::renderer::{AsyncPropertySampler, PipelineInfo};
use crate::resource_cache::ResourceCache;
#[cfg(feature = "replay")]
use crate::resource_cache::PlainCacheOwn;
#[cfg(feature = "replay")]
use crate::resource_cache::PlainResources;
#[cfg(feature = "replay")]
use crate::scene::Scene;
use crate::scene::{BuiltScene, SceneProperties};
use crate::scene_builder_thread::*;
#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "debugger")]
use serde_json;
#[cfg(feature = "replay")]
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::{UNIX_EPOCH, SystemTime};
use std::u32;
#[cfg(feature = "capture")]
use std::path::PathBuf;
#[cfg(feature = "replay")]
use crate::frame_builder::Frame;
use time::precise_time_ns;
use crate::util::{Recycler, VecHelper, drain_filter};

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Copy, Clone)]
pub struct DocumentView {
    scene: SceneView,
    frame: FrameView,
}

/// Some rendering parameters applying at the scene level.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Copy, Clone)]
pub struct SceneView {
    pub device_rect: DeviceIntRect,
    pub layer: DocumentLayer,
    pub device_pixel_ratio: f32,
    pub page_zoom_factor: f32,
    pub quality_settings: QualitySettings,
}

impl SceneView {
    pub fn accumulated_scale_factor_for_snapping(&self) -> DevicePixelScale {
        DevicePixelScale::new(
            self.device_pixel_ratio *
            self.page_zoom_factor
        )
    }
}

/// Some rendering parameters applying at the frame level.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Copy, Clone)]
pub struct FrameView {
    pan: DeviceIntPoint,
    pinch_zoom_factor: f32,
}

impl DocumentView {
    pub fn accumulated_scale_factor(&self) -> DevicePixelScale {
        DevicePixelScale::new(
            self.scene.device_pixel_ratio *
            self.scene.page_zoom_factor *
            self.frame.pinch_zoom_factor
        )
    }
}

#[derive(Copy, Clone, Hash, MallocSizeOf, PartialEq, PartialOrd, Debug, Eq, Ord)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct FrameId(usize);

impl FrameId {
    /// Returns a FrameId corresponding to the first frame.
    ///
    /// Note that we use 0 as the internal id here because the current code
    /// increments the frame id at the beginning of the frame, rather than
    /// at the end, and we want the first frame to be 1. It would probably
    /// be sensible to move the advance() call to after frame-building, and
    /// then make this method return FrameId(1).
    pub fn first() -> Self {
        FrameId(0)
    }

    /// Returns the backing usize for this FrameId.
    pub fn as_usize(&self) -> usize {
        self.0
    }

    /// Advances this FrameId to the next frame.
    pub fn advance(&mut self) {
        self.0 += 1;
    }

    /// An invalid sentinel FrameId, which will always compare less than
    /// any valid FrameId.
    pub const INVALID: FrameId = FrameId(0);
}

impl Default for FrameId {
    fn default() -> Self {
        FrameId::INVALID
    }
}

impl ::std::ops::Add<usize> for FrameId {
    type Output = Self;
    fn add(self, other: usize) -> FrameId {
        FrameId(self.0 + other)
    }
}

impl ::std::ops::Sub<usize> for FrameId {
    type Output = Self;
    fn sub(self, other: usize) -> FrameId {
        assert!(self.0 >= other, "Underflow subtracting FrameIds");
        FrameId(self.0 - other)
    }
}

enum RenderBackendStatus {
    Continue,
    ShutDown(Option<Sender<()>>),
}

/// Identifier to track a sequence of frames.
///
/// This is effectively a `FrameId` with a ridealong timestamp corresponding
/// to when advance() was called, which allows for more nuanced cache eviction
/// decisions. As such, we use the `FrameId` for equality and comparison, since
/// we should never have two `FrameStamps` with the same id but different
/// timestamps.
#[derive(Copy, Clone, Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct FrameStamp {
    id: FrameId,
    time: SystemTime,
    document_id: DocumentId,
}

impl Eq for FrameStamp {}

impl PartialEq for FrameStamp {
    fn eq(&self, other: &Self) -> bool {
        // We should not be checking equality unless the documents are the same
        debug_assert!(self.document_id == other.document_id);
        self.id == other.id
    }
}

impl PartialOrd for FrameStamp {
    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl FrameStamp {
    /// Gets the FrameId in this stamp.
    pub fn frame_id(&self) -> FrameId {
        self.id
    }

    /// Gets the time associated with this FrameStamp.
    pub fn time(&self) -> SystemTime {
        self.time
    }

    /// Gets the DocumentId in this stamp.
    pub fn document_id(&self) -> DocumentId {
        self.document_id
    }

    pub fn is_valid(&self) -> bool {
        // If any fields are their default values, the whole struct should equal INVALID
        debug_assert!((self.time != UNIX_EPOCH && self.id != FrameId(0) && self.document_id != DocumentId::INVALID) ||
                      *self == Self::INVALID);
        self.document_id != DocumentId::INVALID
    }

    /// Returns a FrameStamp corresponding to the first frame.
    pub fn first(document_id: DocumentId) -> Self {
        FrameStamp {
            id: FrameId::first(),
            time: SystemTime::now(),
            document_id,
        }
    }

    /// Advances to a new frame.
    pub fn advance(&mut self) {
        self.id.advance();
        self.time = SystemTime::now();
    }

    /// An invalid sentinel FrameStamp.
    pub const INVALID: FrameStamp = FrameStamp {
        id: FrameId(0),
        time: UNIX_EPOCH,
        document_id: DocumentId::INVALID,
    };
}

macro_rules! declare_data_stores {
    ( $( $name:ident : $ty:ty, )+ ) => {
        /// A collection of resources that are shared by clips, primitives
        /// between display lists.
        #[cfg_attr(feature = "capture", derive(Serialize))]
        #[cfg_attr(feature = "replay", derive(Deserialize))]
        #[derive(Default)]
        pub struct DataStores {
            $(
                pub $name: DataStore<$ty>,
            )+
        }

        impl DataStores {
            /// Reports CPU heap usage.
            fn report_memory(&self, ops: &mut MallocSizeOfOps, r: &mut MemoryReport) {
                $(
                    r.interning.data_stores.$name += self.$name.size_of(ops);
                )+
            }

            fn apply_updates(
                &mut self,
                updates: InternerUpdates,
                profile_counters: &mut BackendProfileCounters,
            ) {
                $(
                    self.$name.apply_updates(
                        updates.$name,
                        &mut profile_counters.intern.$name,
                    );
                )+
            }
        }
    }
}

enumerate_interners!(declare_data_stores);

impl DataStores {
    /// Returns the local rect for a primitive. For most primitives, this is
    /// stored in the template. For pictures, this is stored inside the picture
    /// primitive instance itself, since this is determined during frame building.
    pub fn get_local_prim_rect(
        &self,
        prim_instance: &PrimitiveInstance,
        prim_store: &PrimitiveStore,
    ) -> LayoutRect {
        match prim_instance.kind {
            PrimitiveInstanceKind::Picture { pic_index, .. } => {
                let pic = &prim_store.pictures[pic_index.0];
                pic.precise_local_rect
            }
            _ => {
                self.as_common_data(prim_instance).prim_rect
            }
        }
    }

    /// Returns true if this primitive might need repition.
    // TODO(gw): This seems like the wrong place for this - maybe this flag should
    //           not be in the common prim template data?
    pub fn prim_may_need_repetition(
        &self,
        prim_instance: &PrimitiveInstance,
    ) -> bool {
        match prim_instance.kind {
            PrimitiveInstanceKind::Picture { .. } => {
                false
            }
            _ => {
                self.as_common_data(prim_instance).may_need_repetition
            }
        }
    }

    pub fn as_common_data(
        &self,
        prim_inst: &PrimitiveInstance
    ) -> &PrimTemplateCommonData {
        match prim_inst.kind {
            PrimitiveInstanceKind::Rectangle { data_handle, .. } |
            PrimitiveInstanceKind::Clear { data_handle, .. } => {
                let prim_data = &self.prim[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::Image { data_handle, .. } => {
                let prim_data = &self.image[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::ImageBorder { data_handle, .. } => {
                let prim_data = &self.image_border[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::LineDecoration { data_handle, .. } => {
                let prim_data = &self.line_decoration[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::LinearGradient { data_handle, .. } => {
                let prim_data = &self.linear_grad[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::NormalBorder { data_handle, .. } => {
                let prim_data = &self.normal_border[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::Picture { .. } => {
                panic!("BUG: picture prims don't have common data!");
            }
            PrimitiveInstanceKind::RadialGradient { data_handle, .. } => {
                let prim_data = &self.radial_grad[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::ConicGradient { data_handle, .. } => {
                let prim_data = &self.conic_grad[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::TextRun { data_handle, .. }  => {
                let prim_data = &self.text_run[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::YuvImage { data_handle, .. } => {
                let prim_data = &self.yuv_image[data_handle];
                &prim_data.common
            }
            PrimitiveInstanceKind::Backdrop { data_handle, .. } => {
                let prim_data = &self.backdrop[data_handle];
                &prim_data.common
            }
        }
    }
}

struct Document {
    /// The id of this document
    id: DocumentId,

    /// Temporary list of removed pipelines received from the scene builder
    /// thread and forwarded to the renderer.
    removed_pipelines: Vec<(PipelineId, DocumentId)>,

    view: DocumentView,

    /// The id and time of the current frame.
    stamp: FrameStamp,

    /// The latest built scene, usable to build frames.
    /// received from the scene builder thread.
    scene: BuiltScene,

    /// The builder object that prodces frames, kept around to preserve some retained state.
    frame_builder: FrameBuilder,

    /// A data structure to allow hit testing against rendered frames. This is updated
    /// every time we produce a fully rendered frame.
    hit_tester: Option<Arc<HitTester>>,
    /// To avoid synchronous messaging we update a shared hit-tester that other threads
    /// can query.
    shared_hit_tester: Arc<SharedHitTester>,

    /// Properties that are resolved during frame building and can be changed at any time
    /// without requiring the scene to be re-built.
    dynamic_properties: SceneProperties,

    /// Track whether the last built frame is up to date or if it will need to be re-built
    /// before rendering again.
    frame_is_valid: bool,
    hit_tester_is_valid: bool,
    rendered_frame_is_valid: bool,
    /// We track this information to be able to display debugging information from the
    /// renderer.
    has_built_scene: bool,

    data_stores: DataStores,

    /// Contains various vecs of data that is used only during frame building,
    /// where we want to recycle the memory each new display list, to avoid constantly
    /// re-allocating and moving memory around.
    scratch: PrimitiveScratchBuffer,
    /// Keep track of the size of render task graph to pre-allocate memory up-front
    /// the next frame.
    render_task_counters: RenderTaskGraphCounters,

    #[cfg(feature = "replay")]
    loaded_scene: Scene,

    /// Tracks the state of the picture cache tiles that were composited on the previous frame.
    prev_composite_descriptor: CompositeDescriptor,
}

impl Document {
    pub fn new(
        id: DocumentId,
        size: DeviceIntSize,
        layer: DocumentLayer,
        default_device_pixel_ratio: f32,
    ) -> Self {
        Document {
            id,
            removed_pipelines: Vec::new(),
            view: DocumentView {
                scene: SceneView {
                    device_rect: size.into(),
                    layer,
                    page_zoom_factor: 1.0,
                    device_pixel_ratio: default_device_pixel_ratio,
                    quality_settings: QualitySettings::default(),
                },
                frame: FrameView {
                    pan: DeviceIntPoint::new(0, 0),
                    pinch_zoom_factor: 1.0,
                },
            },
            stamp: FrameStamp::first(id),
            scene: BuiltScene::empty(),
            frame_builder: FrameBuilder::new(),
            hit_tester: None,
            shared_hit_tester: Arc::new(SharedHitTester::new()),
            dynamic_properties: SceneProperties::new(),
            frame_is_valid: false,
            hit_tester_is_valid: false,
            rendered_frame_is_valid: false,
            has_built_scene: false,
            data_stores: DataStores::default(),
            scratch: PrimitiveScratchBuffer::new(),
            render_task_counters: RenderTaskGraphCounters::new(),
            #[cfg(feature = "replay")]
            loaded_scene: Scene::new(),
            prev_composite_descriptor: CompositeDescriptor::empty(),
        }
    }

    fn can_render(&self) -> bool {
        self.scene.has_root_pipeline
    }

    fn has_pixels(&self) -> bool {
        !self.view.scene.device_rect.size.is_empty()
    }

    fn process_frame_msg(
        &mut self,
        message: FrameMsg,
    ) -> DocumentOps {
        match message {
            FrameMsg::UpdateEpoch(pipeline_id, epoch) => {
                self.scene.pipeline_epochs.insert(pipeline_id, epoch);
            }
            FrameMsg::Scroll(delta, cursor) => {
                profile_scope!("Scroll");

                let node_index = match self.hit_tester {
                    Some(ref hit_tester) => {
                        // Ideally we would call self.scroll_nearest_scrolling_ancestor here, but
                        // we need have to avoid a double-borrow.
                        let test = HitTest::new(None, cursor, HitTestFlags::empty());
                        hit_tester.find_node_under_point(test)
                    }
                    None => {
                        None
                    }
                };

                if self.hit_tester.is_some()
                    && self.scroll_nearest_scrolling_ancestor(delta, node_index) {
                    self.hit_tester_is_valid = false;
                    self.frame_is_valid = false;
                }

                return DocumentOps {
                    // TODO: Does it make sense to track this as a scrolling even if we
                    // ended up not scrolling anything?
                    scroll: true,
                    ..DocumentOps::nop()
                };
            }
            FrameMsg::HitTest(pipeline_id, point, flags, tx) => {
                if !self.hit_tester_is_valid {
                    self.rebuild_hit_tester();
                }

                let result = match self.hit_tester {
                    Some(ref hit_tester) => {
                        hit_tester.hit_test(HitTest::new(pipeline_id, point, flags))
                    }
                    None => HitTestResult { items: Vec::new() },
                };

                tx.send(result).unwrap();
            }
            FrameMsg::RequestHitTester(tx) => {
                tx.send(self.shared_hit_tester.clone()).unwrap();
            }
            FrameMsg::SetPan(pan) => {
                if self.view.frame.pan != pan {
                    self.view.frame.pan = pan;
                    self.hit_tester_is_valid = false;
                    self.frame_is_valid = false;
                }
            }
            FrameMsg::ScrollNodeWithId(origin, id, clamp) => {
                profile_scope!("ScrollNodeWithScrollId");

                if self.scroll_node(origin, id, clamp) {
                    self.hit_tester_is_valid = false;
                    self.frame_is_valid = false;
                }

                return DocumentOps {
                    scroll: true,
                    ..DocumentOps::nop()
                };
            }
            FrameMsg::GetScrollNodeState(tx) => {
                profile_scope!("GetScrollNodeState");
                tx.send(self.scene.spatial_tree.get_scroll_node_state()).unwrap();
            }
            FrameMsg::UpdateDynamicProperties(property_bindings) => {
                self.dynamic_properties.set_properties(property_bindings);
            }
            FrameMsg::AppendDynamicTransformProperties(property_bindings) => {
                self.dynamic_properties.add_transforms(property_bindings);
            }
            FrameMsg::SetPinchZoom(factor) => {
                if self.view.frame.pinch_zoom_factor != factor.get() {
                    self.view.frame.pinch_zoom_factor = factor.get();
                    self.frame_is_valid = false;
                }
            }
            FrameMsg::SetIsTransformAsyncZooming(is_zooming, animation_id) => {
                let node = self.scene.spatial_tree.spatial_nodes.iter_mut()
                    .find(|node| node.is_transform_bound_to_property(animation_id));
                if let Some(node) = node {
                    if node.is_async_zooming != is_zooming {
                        node.is_async_zooming = is_zooming;
                        self.frame_is_valid = false;
                    }
                }
            }
        }

        DocumentOps::nop()
    }

    fn build_frame(
        &mut self,
        resource_cache: &mut ResourceCache,
        gpu_cache: &mut GpuCache,
        resource_profile: &mut ResourceProfileCounters,
        debug_flags: DebugFlags,
        tile_cache_logger: &mut TileCacheLogger,
    ) -> RenderedDocument {
        let accumulated_scale_factor = self.view.accumulated_scale_factor();
        let pan = self.view.frame.pan.to_f32() / accumulated_scale_factor;

        // Advance to the next frame.
        self.stamp.advance();

        assert!(self.stamp.frame_id() != FrameId::INVALID,
                "First frame increment must happen before build_frame()");

        let frame = {
            let frame = self.frame_builder.build(
                &mut self.scene,
                resource_cache,
                gpu_cache,
                self.stamp,
                accumulated_scale_factor,
                self.view.scene.layer,
                self.view.scene.device_rect.origin,
                pan,
                resource_profile,
                &self.dynamic_properties,
                &mut self.data_stores,
                &mut self.scratch,
                &mut self.render_task_counters,
                debug_flags,
                tile_cache_logger,
            );

            frame
        };

        self.frame_is_valid = true;

        let is_new_scene = self.has_built_scene;
        self.has_built_scene = false;

        RenderedDocument {
            frame,
            is_new_scene,
        }
    }

    fn rebuild_hit_tester(&mut self) {
        let accumulated_scale_factor = self.view.accumulated_scale_factor();
        let pan = self.view.frame.pan.to_f32() / accumulated_scale_factor;

        self.scene.spatial_tree.update_tree(
            pan,
            accumulated_scale_factor,
            &self.dynamic_properties,
        );

        let hit_tester = Arc::new(self.scene.create_hit_tester(&self.data_stores.clip));
        self.hit_tester = Some(Arc::clone(&hit_tester));
        self.shared_hit_tester.update(hit_tester);
        self.hit_tester_is_valid = true;
    }

    pub fn updated_pipeline_info(&mut self) -> PipelineInfo {
        let removed_pipelines = self.removed_pipelines.take_and_preallocate();
        PipelineInfo {
            epochs: self.scene.pipeline_epochs.iter()
                .map(|(&pipeline_id, &epoch)| ((pipeline_id, self.id), epoch)).collect(),
            removed_pipelines,
        }
    }

    /// Returns true if any nodes actually changed position or false otherwise.
    pub fn scroll_nearest_scrolling_ancestor(
        &mut self,
        scroll_location: ScrollLocation,
        scroll_node_index: Option<SpatialNodeIndex>,
    ) -> bool {
        self.scene.spatial_tree.scroll_nearest_scrolling_ancestor(scroll_location, scroll_node_index)
    }

    /// Returns true if the node actually changed position or false otherwise.
    pub fn scroll_node(
        &mut self,
        origin: LayoutPoint,
        id: ExternalScrollId,
        clamp: ScrollClamping
    ) -> bool {
        self.scene.spatial_tree.scroll_node(origin, id, clamp)
    }

    pub fn new_async_scene_ready(
        &mut self,
        built_scene: BuiltScene,
        recycler: &mut Recycler,
    ) {
        self.frame_is_valid = false;
        self.hit_tester_is_valid = false;

        // Give the old scene a chance to destroy any resources.
        // Right now, all this does is build a hash map of any cached
        // surface tiles, that can be provided to the next scene.
        // TODO(nical) - It's a bit awkward how these retained tiles live
        // in the scene's prim store then temporarily in the frame builder
        // and then presumably back in the prim store during the next frame
        // build.
        let mut retained_tiles = RetainedTiles::new();
        self.scene.prim_store.destroy(&mut retained_tiles);
        let old_scrolling_states = self.scene.spatial_tree.drain();

        self.scene = built_scene;

        // Provide any cached tiles from the previous scene to
        // the newly built one.
        self.frame_builder.set_retained_resources(retained_tiles);

        self.scratch.recycle(recycler);

        self.scene.spatial_tree.finalize_and_apply_pending_scroll_offsets(old_scrolling_states);
    }
}

struct DocumentOps {
    scroll: bool,
}

impl DocumentOps {
    fn nop() -> Self {
        DocumentOps {
            scroll: false,
        }
    }
}

/// The unique id for WR resource identification.
/// The namespace_id should start from 1.
static NEXT_NAMESPACE_ID: AtomicUsize = AtomicUsize::new(1);

#[cfg(any(feature = "capture", feature = "replay"))]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct PlainRenderBackend {
    default_device_pixel_ratio: f32,
    frame_config: FrameBuilderConfig,
    documents: FastHashMap<DocumentId, DocumentView>,
    resource_sequence_id: u32,
}

/// The render backend is responsible for transforming high level display lists into
/// GPU-friendly work which is then submitted to the renderer in the form of a frame::Frame.
///
/// The render backend operates on its own thread.
pub struct RenderBackend {
    api_rx: Receiver<ApiMsg>,
    result_tx: Sender<ResultMsg>,
    scene_tx: Sender<SceneBuilderRequest>,
    low_priority_scene_tx: Sender<SceneBuilderRequest>,
    backend_scene_tx: Sender<BackendSceneBuilderRequest>,
    scene_rx: Receiver<SceneBuilderResult>,

    default_device_pixel_ratio: f32,

    gpu_cache: GpuCache,
    resource_cache: ResourceCache,

    frame_config: FrameBuilderConfig,
    default_compositor_kind: CompositorKind,
    documents: FastHashMap<DocumentId, Document>,

    notifier: Box<dyn RenderNotifier>,
    tile_cache_logger: TileCacheLogger,
    sampler: Option<Box<dyn AsyncPropertySampler + Send>>,
    size_of_ops: Option<MallocSizeOfOps>,
    debug_flags: DebugFlags,
    namespace_alloc_by_client: bool,

    // We keep one around to be able to call clear_namespace
    // after the api object is deleted. For most purposes the
    // api object's blob handler should be used instead.
    blob_image_handler: Option<Box<dyn BlobImageHandler>>,

    recycler: Recycler,
    #[cfg(feature = "capture")]
    capture_config: Option<CaptureConfig>,
    #[cfg(feature = "replay")]
    loaded_resource_sequence_id: u32,
}

impl RenderBackend {
    pub fn new(
        api_rx: Receiver<ApiMsg>,
        result_tx: Sender<ResultMsg>,
        scene_tx: Sender<SceneBuilderRequest>,
        low_priority_scene_tx: Sender<SceneBuilderRequest>,
        backend_scene_tx: Sender<BackendSceneBuilderRequest>,
        scene_rx: Receiver<SceneBuilderResult>,
        default_device_pixel_ratio: f32,
        resource_cache: ResourceCache,
        notifier: Box<dyn RenderNotifier>,
        blob_image_handler: Option<Box<dyn BlobImageHandler>>,
        frame_config: FrameBuilderConfig,
        sampler: Option<Box<dyn AsyncPropertySampler + Send>>,
        size_of_ops: Option<MallocSizeOfOps>,
        debug_flags: DebugFlags,
        namespace_alloc_by_client: bool,
    ) -> RenderBackend {
        RenderBackend {
            api_rx,
            result_tx,
            scene_tx,
            low_priority_scene_tx,
            backend_scene_tx,
            scene_rx,
            default_device_pixel_ratio,
            resource_cache,
            gpu_cache: GpuCache::new(),
            frame_config,
            default_compositor_kind : frame_config.compositor_kind,
            documents: FastHashMap::default(),
            notifier,
            tile_cache_logger: TileCacheLogger::new(500usize),
            sampler,
            size_of_ops,
            debug_flags,
            namespace_alloc_by_client,
            recycler: Recycler::new(),
            blob_image_handler,
            #[cfg(feature = "capture")]
            capture_config: None,
            #[cfg(feature = "replay")]
            loaded_resource_sequence_id: 0,
        }
    }

    fn next_namespace_id(&self) -> IdNamespace {
        IdNamespace(NEXT_NAMESPACE_ID.fetch_add(1, Ordering::Relaxed) as u32)
    }

    pub fn run(&mut self, mut profile_counters: BackendProfileCounters) {
        let mut frame_counter: u32 = 0;
        let mut status = RenderBackendStatus::Continue;

        if let Some(ref sampler) = self.sampler {
            sampler.register();
        }

        while let RenderBackendStatus::Continue = status {
            while let Ok(msg) = self.scene_rx.try_recv() {
                profile_scope!("rb_msg");

                match msg {
                    SceneBuilderResult::Transactions(txns, result_tx) => {
                        self.process_transaction(
                            txns,
                            result_tx,
                            &mut frame_counter,
                            &mut profile_counters,
                        );
                        self.bookkeep_after_frames();
                    },
                    #[cfg(feature = "capture")]
                    SceneBuilderResult::CapturedTransactions(txns, capture_config, result_tx) => {
                        if let Some(ref mut old_config) = self.capture_config {
                            assert!(old_config.scene_id <= capture_config.scene_id);
                            if old_config.scene_id < capture_config.scene_id {
                                old_config.scene_id = capture_config.scene_id;
                                old_config.frame_id = 0;
                            }
                        } else {
                            self.capture_config = Some(capture_config);
                        }

                        let built_frame = self.process_transaction(
                            txns,
                            result_tx,
                            &mut frame_counter,
                            &mut profile_counters,
                        );

                        if built_frame {
                            self.save_capture_sequence();
                        }

                        self.bookkeep_after_frames();
                    },
                    SceneBuilderResult::GetGlyphDimensions(request) => {
                        let mut glyph_dimensions = Vec::with_capacity(request.glyph_indices.len());
                        if let Some(base) = self.resource_cache.get_font_instance(request.key) {
                            let font = FontInstance::from_base(Arc::clone(&base));
                            for glyph_index in &request.glyph_indices {
                                let glyph_dim = self.resource_cache.get_glyph_dimensions(&font, *glyph_index);
                                glyph_dimensions.push(glyph_dim);
                            }
                        }
                        request.sender.send(glyph_dimensions).unwrap();
                    }
                    SceneBuilderResult::GetGlyphIndices(request) => {
                        let mut glyph_indices = Vec::with_capacity(request.text.len());
                        for ch in request.text.chars() {
                            let index = self.resource_cache.get_glyph_index(request.key, ch);
                            glyph_indices.push(index);
                        }
                        request.sender.send(glyph_indices).unwrap();
                    }
                    SceneBuilderResult::FlushComplete(tx) => {
                        tx.send(()).ok();
                    }
                    SceneBuilderResult::ExternalEvent(evt) => {
                        self.notifier.external_event(evt);
                    }
                    SceneBuilderResult::ClearNamespace(id) => {
                        self.resource_cache.clear_namespace(id);
                        self.documents.retain(|doc_id, _doc| doc_id.namespace_id != id);
                        if let Some(handler) = &mut self.blob_image_handler {
                            handler.clear_namespace(id);
                        }
                    }
                    SceneBuilderResult::Stopped => {
                        panic!("We haven't sent a Stop yet, how did we get a Stopped back?");
                    }
                    SceneBuilderResult::DocumentsForDebugger(json) => {
                        let msg = ResultMsg::DebugOutput(DebugOutput::FetchDocuments(json));
                        self.result_tx.send(msg).unwrap();
                        self.notifier.wake_up();
                    }
                }
            }

            status = match self.api_rx.recv() {
                Ok(msg) => {
                    self.process_api_msg(msg, &mut profile_counters, &mut frame_counter)
                }
                Err(..) => { RenderBackendStatus::ShutDown(None) }
            };
        }

        let _ = self.low_priority_scene_tx.send(SceneBuilderRequest::Stop);
        // Ensure we read everything the scene builder is sending us from
        // inflight messages, otherwise the scene builder might panic.
        while let Ok(msg) = self.scene_rx.recv() {
            match msg {
                SceneBuilderResult::FlushComplete(tx) => {
                    // If somebody's blocked waiting for a flush, how did they
                    // trigger the RB thread to shut down? This shouldn't happen
                    // but handle it gracefully anyway.
                    debug_assert!(false);
                    tx.send(()).ok();
                }
                SceneBuilderResult::Stopped => break,
                _ => continue,
            }
        }

        self.documents.clear();

        self.notifier.shut_down();

        if let Some(ref sampler) = self.sampler {
            sampler.deregister();
        }


        if let RenderBackendStatus::ShutDown(Some(sender)) = status {
            let _ = sender.send(());
        }
    }

    fn process_transaction(
        &mut self,
        mut txns: Vec<Box<BuiltTransaction>>,
        result_tx: Option<Sender<SceneSwapResult>>,
        frame_counter: &mut u32,
        profile_counters: &mut BackendProfileCounters,
    ) -> bool {
        self.prepare_for_frames();
        self.maybe_force_nop_documents(
            frame_counter,
            profile_counters,
            |document_id| txns.iter().any(|txn| txn.document_id == document_id));

        let mut built_frame = false;
        for mut txn in txns.drain(..) {
           let has_built_scene = txn.built_scene.is_some();

           if let Some(timings) = txn.timings {
               if has_built_scene {
                   profile_counters.scene_changed = true;
               }

               profile_counters.txn.set(
                   timings.builder_start_time_ns,
                   timings.builder_end_time_ns,
                   timings.send_time_ns,
                   timings.scene_build_start_time_ns,
                   timings.scene_build_end_time_ns,
                   timings.display_list_len,
               );
            }

            if let Some(doc) = self.documents.get_mut(&txn.document_id) {
                doc.removed_pipelines.append(&mut txn.removed_pipelines);
                doc.view.scene = txn.view;

                if let Some(built_scene) = txn.built_scene.take() {
                    doc.new_async_scene_ready(
                        built_scene,
                        &mut self.recycler,
                    );
                }

                // If there are any additions or removals of clip modes
                // during the scene build, apply them to the data store now.
                // This needs to happen before we build the hit tester.
                if let Some(updates) = txn.interner_updates.take() {
                    #[cfg(feature = "capture")]
                    {
                        if self.debug_flags.contains(DebugFlags::TILE_CACHE_LOGGING_DBG) {
                            self.tile_cache_logger.serialize_updates(&updates);
                        }
                    }
                    doc.data_stores.apply_updates(updates, profile_counters);
                }

                // Build the hit tester while the APZ lock is held so that its content
                // is in sync with the gecko APZ tree.
                if !doc.hit_tester_is_valid {
                    doc.rebuild_hit_tester();
                }

                if let Some(ref tx) = result_tx {
                    let (resume_tx, resume_rx) = channel();
                    tx.send(SceneSwapResult::Complete(resume_tx)).unwrap();
                    // Block until the post-swap hook has completed on
                    // the scene builder thread. We need to do this before
                    // we can sample from the sampler hook which might happen
                    // in the update_document call below.
                    resume_rx.recv().ok();
                }

                for pipeline_id in &txn.discard_frame_state_for_pipelines {
                    doc.scene
                        .spatial_tree
                        .discard_frame_state_for_pipeline(*pipeline_id);
                }
            } else {
                // The document was removed while we were building it, skip it.
                // TODO: we might want to just ensure that removed documents are
                // always forwarded to the scene builder thread to avoid this case.
                if let Some(ref tx) = result_tx {
                    tx.send(SceneSwapResult::Aborted).unwrap();
                }
                continue;
            }

            self.resource_cache.add_rasterized_blob_images(
                txn.rasterized_blobs.take(),
                &mut profile_counters.resources.texture_cache,
            );

            built_frame |= self.update_document(
                txn.document_id,
                txn.resource_updates.take(),
                txn.frame_ops.take(),
                txn.notifications.take(),
                txn.render_frame,
                txn.invalidate_rendered_frame,
                frame_counter,
                profile_counters,
                has_built_scene,
            );
        }

        built_frame
    }

    fn process_api_msg(
        &mut self,
        msg: ApiMsg,
        profile_counters: &mut BackendProfileCounters,
        frame_counter: &mut u32,
    ) -> RenderBackendStatus {
        match msg {
            ApiMsg::WakeUp => {}
            ApiMsg::WakeSceneBuilder => {
                self.scene_tx.send(SceneBuilderRequest::WakeUp).unwrap();
            }
            ApiMsg::FlushSceneBuilder(tx) => {
                self.low_priority_scene_tx.send(SceneBuilderRequest::Flush(tx)).unwrap();
            }
            ApiMsg::GetGlyphDimensions(request) => {
                self.scene_tx.send(SceneBuilderRequest::GetGlyphDimensions(request)).unwrap();
            }
            ApiMsg::GetGlyphIndices(request) => {
                self.scene_tx.send(SceneBuilderRequest::GetGlyphIndices(request)).unwrap();
            }
            ApiMsg::CloneApi(sender) => {
                assert!(!self.namespace_alloc_by_client);
                sender.send(self.next_namespace_id()).unwrap();
            }
            ApiMsg::CloneApiByClient(namespace_id) => {
                assert!(self.namespace_alloc_by_client);
                debug_assert!(!self.documents.iter().any(|(did, _doc)| did.namespace_id == namespace_id));
            }
            ApiMsg::AddDocument(document_id, initial_size, layer) => {
                let document = Document::new(
                    document_id,
                    initial_size,
                    layer,
                    self.default_device_pixel_ratio,
                );
                let old = self.documents.insert(document_id, document);
                debug_assert!(old.is_none());

                self.scene_tx.send(
                    SceneBuilderRequest::AddDocument(document_id, initial_size, layer)
                ).unwrap();

            }
            ApiMsg::DeleteDocument(document_id) => {
                self.documents.remove(&document_id);
                self.low_priority_scene_tx.send(
                    SceneBuilderRequest::DeleteDocument(document_id)
                ).unwrap();
            }
            ApiMsg::ExternalEvent(evt) => {
                self.low_priority_scene_tx.send(SceneBuilderRequest::ExternalEvent(evt)).unwrap();
            }
            ApiMsg::ClearNamespace(id) => {
                self.low_priority_scene_tx.send(SceneBuilderRequest::ClearNamespace(id)).unwrap();
            }
            ApiMsg::MemoryPressure => {
                // This is drastic. It will basically flush everything out of the cache,
                // and the next frame will have to rebuild all of its resources.
                // We may want to look into something less extreme, but on the other hand this
                // should only be used in situations where are running low enough on memory
                // that we risk crashing if we don't do something about it.
                // The advantage of clearing the cache completely is that it gets rid of any
                // remaining fragmentation that could have persisted if we kept around the most
                // recently used resources.
                self.resource_cache.clear(ClearCache::all());

                self.gpu_cache.clear();

                let resource_updates = self.resource_cache.pending_updates();
                let msg = ResultMsg::UpdateResources {
                    resource_updates,
                    memory_pressure: true,
                };
                self.result_tx.send(msg).unwrap();
                self.notifier.wake_up();
            }
            ApiMsg::ReportMemory(tx) => {
                self.report_memory(tx);
            }
            ApiMsg::DebugCommand(option) => {
                let msg = match option {
                    DebugCommand::EnableDualSourceBlending(enable) => {
                        // Set in the config used for any future documents
                        // that are created.
                        self.frame_config
                            .dual_source_blending_is_enabled = enable;
                        self.update_frame_builder_config();

                        // We don't want to forward this message to the renderer.
                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::SetPictureTileSize(tile_size) => {
                        self.frame_config.tile_size_override = tile_size;
                        self.update_frame_builder_config();

                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::FetchDocuments => {
                        // Ask SceneBuilderThread to send JSON presentation of the documents,
                        // that will be forwarded to Renderer.
                        self.send_backend_message(BackendSceneBuilderRequest::DocumentsForDebugger);
                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::FetchClipScrollTree => {
                        let json = self.get_spatial_tree_for_debugger();
                        ResultMsg::DebugOutput(DebugOutput::FetchClipScrollTree(json))
                    }
                    #[cfg(feature = "capture")]
                    DebugCommand::SaveCapture(root, bits) => {
                        let output = self.save_capture(root, bits, profile_counters);
                        ResultMsg::DebugOutput(output)
                    },
                    #[cfg(feature = "capture")]
                    DebugCommand::StartCaptureSequence(root, bits) => {
                        self.start_capture_sequence(root, bits);
                        return RenderBackendStatus::Continue;
                    },
                    #[cfg(feature = "capture")]
                    DebugCommand::StopCaptureSequence => {
                        self.stop_capture_sequence();
                        return RenderBackendStatus::Continue;
                    },
                    #[cfg(feature = "replay")]
                    DebugCommand::LoadCapture(path, ids, tx) => {
                        NEXT_NAMESPACE_ID.fetch_add(1, Ordering::Relaxed);
                        *frame_counter += 1;

                        let mut config = CaptureConfig::new(path, CaptureBits::all());
                        if let Some((scene_id, frame_id)) = ids {
                            config.scene_id = scene_id;
                            config.frame_id = frame_id;
                        }

                        self.load_capture(config, profile_counters);

                        for (id, doc) in &self.documents {
                            let captured = CapturedDocument {
                                document_id: *id,
                                root_pipeline_id: doc.loaded_scene.root_pipeline_id,
                            };
                            tx.send(captured).unwrap();
                        }

                        // Note: we can't pass `LoadCapture` here since it needs to arrive
                        // before the `PublishDocument` messages sent by `load_capture`.
                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::ClearCaches(mask) => {
                        self.resource_cache.clear(mask);
                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::EnableNativeCompositor(enable) => {
                        // Default CompositorKind should be Native
                        if let CompositorKind::Draw { .. } = self.default_compositor_kind {
                            unreachable!();
                        }

                        let compositor_kind = if enable {
                            self.default_compositor_kind
                        } else {
                            CompositorKind::default()
                        };

                        for (_, doc) in &mut self.documents {
                            doc.scene.config.compositor_kind = compositor_kind;
                            doc.frame_is_valid = false;
                        }

                        self.frame_config.compositor_kind = compositor_kind;
                        self.update_frame_builder_config();

                        // We don't want to forward this message to the renderer.
                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::EnableMultithreading(enable) => {
                        self.resource_cache.enable_multithreading(enable);
                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::SetBatchingLookback(count) => {
                        self.frame_config.batch_lookback_count = count as usize;
                        self.update_frame_builder_config();

                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::SimulateLongSceneBuild(time_ms) => {
                        self.scene_tx.send(SceneBuilderRequest::SimulateLongSceneBuild(time_ms)).unwrap();
                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::SimulateLongLowPrioritySceneBuild(time_ms) => {
                        self.low_priority_scene_tx.send(
                            SceneBuilderRequest::SimulateLongLowPrioritySceneBuild(time_ms)
                        ).unwrap();
                        return RenderBackendStatus::Continue;
                    }
                    DebugCommand::SetFlags(flags) => {
                        self.resource_cache.set_debug_flags(flags);
                        self.gpu_cache.set_debug_flags(flags);

                        // If we're toggling on the GPU cache debug display, we
                        // need to blow away the cache. This is because we only
                        // send allocation/free notifications to the renderer
                        // thread when the debug display is enabled, and thus
                        // enabling it when the cache is partially populated will
                        // give the renderer an incomplete view of the world.
                        // And since we might as well drop all the debugging state
                        // from the renderer when we disable the debug display,
                        // we just clear the cache on toggle.
                        let changed = self.debug_flags ^ flags;
                        if changed.contains(DebugFlags::GPU_CACHE_DBG) {
                            self.gpu_cache.clear();
                        }
                        self.debug_flags = flags;

                        ResultMsg::DebugCommand(option)
                    }
                    _ => ResultMsg::DebugCommand(option),
                };
                self.result_tx.send(msg).unwrap();
                self.notifier.wake_up();
            }
            ApiMsg::ShutDown(sender) => {
                info!("Recycling stats: {:?}", self.recycler);
                return RenderBackendStatus::ShutDown(sender);
            }
            ApiMsg::UpdateDocuments(transaction_msgs) => {
                self.prepare_transactions(
                    transaction_msgs,
                    frame_counter,
                    profile_counters,
                );
            }
        }

        RenderBackendStatus::Continue
    }

    fn update_frame_builder_config(&self) {
        self.send_backend_message(
            BackendSceneBuilderRequest::SetFrameBuilderConfig(
                self.frame_config.clone()
            )
        );
    }

    fn prepare_for_frames(&mut self) {
        self.gpu_cache.prepare_for_frames();
    }

    fn bookkeep_after_frames(&mut self) {
        self.gpu_cache.bookkeep_after_frames();
    }

    fn requires_frame_build(&mut self) -> bool {
        self.gpu_cache.requires_frame_build()
    }

    fn prepare_transactions(
        &mut self,
        txns: Vec<Box<TransactionMsg>>,
        frame_counter: &mut u32,
        profile_counters: &mut BackendProfileCounters,
    ) {
        let mut use_scene_builder = txns.iter()
            .any(|transaction_msg| transaction_msg.use_scene_builder_thread);
        let use_high_priority = txns.iter()
            .any(|transaction_msg| !transaction_msg.low_priority);

        use_scene_builder = use_scene_builder || txns.iter().any(|txn| {
            !txn.scene_ops.is_empty()
                || !txn.blob_requests.is_empty()
                || txn.blob_rasterizer.is_some()
        });

        if !use_scene_builder {
            self.prepare_for_frames();
            self.maybe_force_nop_documents(
                frame_counter,
                profile_counters,
                |document_id| txns.iter().any(|txn| txn.document_id == document_id));

            let mut built_frame = false;
            for mut txn in txns {
                built_frame |= self.update_document(
                    txn.document_id,
                    txn.resource_updates.take(),
                    txn.frame_ops.take(),
                    txn.notifications.take(),
                    txn.generate_frame,
                    txn.invalidate_rendered_frame,
                    frame_counter,
                    profile_counters,
                    false
                );
            }
            if built_frame {
                #[cfg(feature = "capture")]
                self.save_capture_sequence();
            }
            self.bookkeep_after_frames();
            return;
        }

        let tx = if use_high_priority {
            &self.scene_tx
        } else {
            &self.low_priority_scene_tx
        };

        tx.send(SceneBuilderRequest::Transactions(txns)).unwrap();
    }

    /// In certain cases, resources shared by multiple documents have to run
    /// maintenance operations, like cleaning up unused cache items. In those
    /// cases, we are forced to build frames for all documents, however we
    /// may not have a transaction ready for every document - this method
    /// calls update_document with the details of a fake, nop transaction just
    /// to force a frame build.
    fn maybe_force_nop_documents<F>(&mut self,
                                    frame_counter: &mut u32,
                                    profile_counters: &mut BackendProfileCounters,
                                    document_already_present: F) where
        F: Fn(DocumentId) -> bool {
        if self.requires_frame_build() {
            let nop_documents : Vec<DocumentId> = self.documents.keys()
                .cloned()
                .filter(|key| !document_already_present(*key))
                .collect();
            #[allow(unused_variables)]
            let mut built_frame = false;
            for &document_id in &nop_documents {
                built_frame |= self.update_document(
                    document_id,
                    Vec::default(),
                    Vec::default(),
                    Vec::default(),
                    false,
                    false,
                    frame_counter,
                    profile_counters,
                    false);
            }
            #[cfg(feature = "capture")]
            match built_frame {
                true => self.save_capture_sequence(),
                _ => {},
            }
        }
    }

    fn update_document(
        &mut self,
        document_id: DocumentId,
        resource_updates: Vec<ResourceUpdate>,
        mut frame_ops: Vec<FrameMsg>,
        mut notifications: Vec<NotificationRequest>,
        mut render_frame: bool,
        invalidate_rendered_frame: bool,
        frame_counter: &mut u32,
        profile_counters: &mut BackendProfileCounters,
        has_built_scene: bool,
    ) -> bool {
        let requested_frame = render_frame;

        let requires_frame_build = self.requires_frame_build();
        let doc = self.documents.get_mut(&document_id).unwrap();
        // If we have a sampler, get more frame ops from it and add them
        // to the transaction. This is a hook to allow the WR user code to
        // fiddle with things after a potentially long scene build, but just
        // before rendering. This is useful for rendering with the latest
        // async transforms.
        if requested_frame || has_built_scene {
            if let Some(ref sampler) = self.sampler {
                frame_ops.append(&mut sampler.sample(document_id,
                                                     &doc.scene.pipeline_epochs));
            }
        }

        doc.has_built_scene |= has_built_scene;

        // TODO: this scroll variable doesn't necessarily mean we scrolled. It is only used
        // for something wrench specific and we should remove it.
        let mut scroll = false;
        for frame_msg in frame_ops {
            let _timer = profile_counters.total_time.timer();
            let op = doc.process_frame_msg(frame_msg);
            scroll |= op.scroll;
        }

        for update in &resource_updates {
            if let ResourceUpdate::UpdateImage(..) = update {
                doc.frame_is_valid = false;
            }
        }

        self.resource_cache.post_scene_building_update(
            resource_updates,
            &mut profile_counters.resources,
        );

        if doc.dynamic_properties.flush_pending_updates() {
            doc.frame_is_valid = false;
            doc.hit_tester_is_valid = false;
        }

        if !doc.can_render() {
            // TODO: this happens if we are building the first scene asynchronously and
            // scroll at the same time. we should keep track of the fact that we skipped
            // composition here and do it as soon as we receive the scene.
            render_frame = false;
        }

        // Avoid re-building the frame if the current built frame is still valid.
        // However, if the resource_cache requires a frame build, _always_ do that, unless
        // doc.can_render() is false, as in that case a frame build can't happen anyway.
        // We want to ensure we do this because even if the doc doesn't have pixels it
        // can still try to access stale texture cache items.
        let build_frame = (render_frame && !doc.frame_is_valid && doc.has_pixels()) ||
            (requires_frame_build && doc.can_render());

        // Request composite is true when we want to composite frame even when
        // there is no frame update. This happens when video frame is updated under
        // external image with NativeTexture or when platform requested to composite frame.
        if invalidate_rendered_frame {
            doc.rendered_frame_is_valid = false;
            if let CompositorKind::Draw { max_partial_present_rects, .. } = doc.scene.config.compositor_kind {

              // When partial present is enabled, we need to force redraw.
              if max_partial_present_rects > 0 {
                  let msg = ResultMsg::ForceRedraw;
                  self.result_tx.send(msg).unwrap();
              }
            }
        }

        let mut frame_build_time = None;
        if build_frame {
            profile_scope!("generate frame");

            *frame_counter += 1;

            // borrow ck hack for profile_counters
            let (pending_update, rendered_document) = {
                let _timer = profile_counters.total_time.timer();
                let frame_build_start_time = precise_time_ns();

                let rendered_document = doc.build_frame(
                    &mut self.resource_cache,
                    &mut self.gpu_cache,
                    &mut profile_counters.resources,
                    self.debug_flags,
                    &mut self.tile_cache_logger,
                );

                debug!("generated frame for document {:?} with {} passes",
                    document_id, rendered_document.frame.passes.len());

                let msg = ResultMsg::UpdateGpuCache(self.gpu_cache.extract_updates());
                self.result_tx.send(msg).unwrap();

                frame_build_time = Some(precise_time_ns() - frame_build_start_time);

                let pending_update = self.resource_cache.pending_updates();
                (pending_update, rendered_document)
            };

            // Build a small struct that represents the state of the tiles to be composited.
            let composite_descriptor = rendered_document
                .frame
                .composite_state
                .descriptor
                .clone();

            // If there are texture cache updates to apply, or if the produced
            // frame is not a no-op, or the compositor state has changed,
            // then we cannot skip compositing this frame.
            if !pending_update.is_nop() ||
               !rendered_document.frame.is_nop() ||
               composite_descriptor != doc.prev_composite_descriptor {
                doc.rendered_frame_is_valid = false;
            }
            doc.prev_composite_descriptor = composite_descriptor;

            #[cfg(feature = "capture")]
            match self.capture_config {
                Some(ref mut config) => {
                    // FIXME(aosmond): document splitting causes multiple prepare frames
                    config.prepare_frame();

                    if config.bits.contains(CaptureBits::FRAME) {
                        let file_name = format!("frame-{}-{}", document_id.namespace_id.0, document_id.id);
                        config.serialize_for_frame(&rendered_document.frame, file_name);
                    }

                    let data_stores_name = format!("data-stores-{}-{}", document_id.namespace_id.0, document_id.id);
                    config.serialize_for_frame(&doc.data_stores, data_stores_name);

                    let properties_name = format!("properties-{}-{}", document_id.namespace_id.0, document_id.id);
                    config.serialize_for_frame(&doc.dynamic_properties, properties_name);
                },
                None => {},
            }

            let msg = ResultMsg::PublishPipelineInfo(doc.updated_pipeline_info());
            self.result_tx.send(msg).unwrap();

            // Publish the frame
            let msg = ResultMsg::PublishDocument(
                document_id,
                rendered_document,
                pending_update,
                profile_counters.clone()
            );
            self.result_tx.send(msg).unwrap();
            profile_counters.reset();
        } else if requested_frame {
            // WR-internal optimization to avoid doing a bunch of render work if
            // there's no pixels. We still want to pretend to render and request
            // a render to make sure that the callbacks (particularly the
            // new_frame_ready callback below) has the right flags.
            let msg = ResultMsg::PublishPipelineInfo(doc.updated_pipeline_info());
            self.result_tx.send(msg).unwrap();
        }

        drain_filter(
            &mut notifications,
            |n| { n.when() == Checkpoint::FrameBuilt },
            |n| { n.notify(); },
        );

        if !notifications.is_empty() {
            self.result_tx.send(ResultMsg::AppendNotificationRequests(notifications)).unwrap();
        }

        // Always forward the transaction to the renderer if a frame was requested,
        // otherwise gecko can get into a state where it waits (forever) for the
        // transaction to complete before sending new work.
        if requested_frame {
            // If rendered frame is already valid, there is no need to render frame.
            if doc.rendered_frame_is_valid {
                render_frame = false;
            } else if render_frame {
                doc.rendered_frame_is_valid = true;
            }
            self.notifier.new_frame_ready(document_id, scroll, render_frame, frame_build_time);
        }

        if !doc.hit_tester_is_valid {
            doc.rebuild_hit_tester();
        }

        build_frame
    }

    fn send_backend_message(&self, msg: BackendSceneBuilderRequest) {
        self.backend_scene_tx.send(msg).unwrap();
        self.low_priority_scene_tx.send(SceneBuilderRequest::BackendMessage).unwrap();
    }

    #[cfg(not(feature = "debugger"))]
    fn get_spatial_tree_for_debugger(&self) -> String {
        String::new()
    }

    #[cfg(feature = "debugger")]
    fn get_spatial_tree_for_debugger(&self) -> String {
        use crate::print_tree::PrintableTree;

        let mut debug_root = debug_server::SpatialTreeList::new();

        for (_, doc) in &self.documents {
            let debug_node = debug_server::TreeNode::new("document spatial tree");
            let mut builder = debug_server::TreeNodeBuilder::new(debug_node);

            doc.scene.spatial_tree.print_with(&mut builder);

            debug_root.add(builder.build());
        }

        serde_json::to_string(&debug_root).unwrap()
    }

    fn report_memory(&mut self, tx: Sender<Box<MemoryReport>>) {
        let mut report = Box::new(MemoryReport::default());
        let ops = self.size_of_ops.as_mut().unwrap();
        let op = ops.size_of_op;
        report.gpu_cache_metadata = self.gpu_cache.size_of(ops);
        for doc in self.documents.values() {
            report.clip_stores += doc.scene.clip_store.size_of(ops);
            report.hit_testers += match &doc.hit_tester {
                Some(hit_tester) => hit_tester.size_of(ops),
                None => 0,
            };

            doc.data_stores.report_memory(ops, &mut report)
        }

        (*report) += self.resource_cache.report_memory(op);

        // Send a message to report memory on the scene-builder thread, which
        // will add its report to this one and send the result back to the original
        // thread waiting on the request.
        self.send_backend_message(
            BackendSceneBuilderRequest::ReportMemory(report, tx)
        );
    }

    #[cfg(feature = "capture")]
    fn save_capture_sequence(&mut self) {
        if let Some(ref mut config) = self.capture_config {
            let deferred = self.resource_cache.save_capture_sequence(config);

            let backend = PlainRenderBackend {
                default_device_pixel_ratio: self.default_device_pixel_ratio,
                frame_config: self.frame_config.clone(),
                resource_sequence_id: config.resource_id,
                documents: self.documents
                    .iter()
                    .map(|(id, doc)| (*id, doc.view))
                    .collect(),
            };
            config.serialize_for_frame(&backend, "backend");

            if !deferred.is_empty() {
                let msg = ResultMsg::DebugOutput(DebugOutput::SaveCapture(config.clone(), deferred));
                self.result_tx.send(msg).unwrap();
            }
        }
    }
}

impl RenderBackend {
    #[cfg(feature = "capture")]
    // Note: the mutable `self` is only needed here for resolving blob images
    fn save_capture(
        &mut self,
        root: PathBuf,
        bits: CaptureBits,
        profile_counters: &mut BackendProfileCounters,
    ) -> DebugOutput {
        use std::fs;
        use crate::render_task_graph::dump_render_tasks_as_svg;

        debug!("capture: saving {:?}", root);
        if !root.is_dir() {
            if let Err(e) = fs::create_dir_all(&root) {
                panic!("Unable to create capture dir: {:?}", e);
            }
        }
        let config = CaptureConfig::new(root, bits);

        if config.bits.contains(CaptureBits::FRAME) {
            self.prepare_for_frames();
        }

        for (&id, doc) in &mut self.documents {
            debug!("\tdocument {:?}", id);
            if config.bits.contains(CaptureBits::FRAME) {
                let rendered_document = doc.build_frame(
                    &mut self.resource_cache,
                    &mut self.gpu_cache,
                    &mut profile_counters.resources,
                    self.debug_flags,
                    &mut self.tile_cache_logger,
                );
                // After we rendered the frames, there are pending updates to both
                // GPU cache and resources. Instead of serializing them, we are going to make sure
                // they are applied on the `Renderer` side.
                let msg_update_gpu_cache = ResultMsg::UpdateGpuCache(self.gpu_cache.extract_updates());
                self.result_tx.send(msg_update_gpu_cache).unwrap();
                //TODO: write down doc's pipeline info?
                // it has `pipeline_epoch_map`,
                // which may capture necessary details for some cases.
                let file_name = format!("frame-{}-{}", id.namespace_id.0, id.id);
                config.serialize_for_frame(&rendered_document.frame, file_name);
                let file_name = format!("spatial-{}-{}", id.namespace_id.0, id.id);
                config.serialize_tree_for_frame(&doc.scene.spatial_tree, file_name);
                let file_name = format!("built-primitives-{}-{}", id.namespace_id.0, id.id);
                config.serialize_for_frame(&doc.scene.prim_store, file_name);
                let file_name = format!("built-clips-{}-{}", id.namespace_id.0, id.id);
                config.serialize_for_frame(&doc.scene.clip_store, file_name);
                let file_name = format!("scratch-{}-{}", id.namespace_id.0, id.id);
                config.serialize_for_frame(&doc.scratch, file_name);
                let file_name = format!("render-tasks-{}-{}.svg", id.namespace_id.0, id.id);
                let mut svg_file = fs::File::create(&config.file_path_for_frame(file_name, "svg"))
                    .expect("Failed to open the SVG file.");
                dump_render_tasks_as_svg(
                    &rendered_document.frame.render_tasks,
                    &rendered_document.frame.passes,
                    &mut svg_file
                ).unwrap();
            }

            let data_stores_name = format!("data-stores-{}-{}", id.namespace_id.0, id.id);
            config.serialize_for_frame(&doc.data_stores, data_stores_name);

            let properties_name = format!("properties-{}-{}", id.namespace_id.0, id.id);
            config.serialize_for_frame(&doc.dynamic_properties, properties_name);
        }

        if config.bits.contains(CaptureBits::FRAME) {
            // TODO: there is no guarantee that we won't hit this case, but we want to
            // report it here if we do. If we don't, it will simply crash in
            // Renderer::render_impl and give us less information about the source.
            assert!(!self.requires_frame_build(), "Caches were cleared during a capture.");
            self.bookkeep_after_frames();
        }

        debug!("\tscene builder");
        self.send_backend_message(
            BackendSceneBuilderRequest::SaveScene(config.clone())
        );

        debug!("\tresource cache");
        let (resources, deferred) = self.resource_cache.save_capture(&config.root);

        if config.bits.contains(CaptureBits::TILE_CACHE) {
            debug!("\ttile cache");
            self.tile_cache_logger.save_capture(&config.root);
        }

        info!("\tbackend");
        let backend = PlainRenderBackend {
            default_device_pixel_ratio: self.default_device_pixel_ratio,
            frame_config: self.frame_config.clone(),
            resource_sequence_id: 0,
            documents: self.documents
                .iter()
                .map(|(id, doc)| (*id, doc.view))
                .collect(),
        };

        config.serialize_for_frame(&backend, "backend");
        config.serialize_for_frame(&resources, "plain-resources");

        if config.bits.contains(CaptureBits::FRAME) {
            let msg_update_resources = ResultMsg::UpdateResources {
                resource_updates: self.resource_cache.pending_updates(),
                memory_pressure: false,
            };
            self.result_tx.send(msg_update_resources).unwrap();
            // Save the texture/glyph/image caches.
            info!("\tresource cache");
            let caches = self.resource_cache.save_caches(&config.root);
            config.serialize_for_resource(&caches, "resource_cache");
            info!("\tgpu cache");
            config.serialize_for_resource(&self.gpu_cache, "gpu_cache");
        }

        DebugOutput::SaveCapture(config, deferred)
    }

    #[cfg(feature = "capture")]
    fn start_capture_sequence(
        &mut self,
        root: PathBuf,
        bits: CaptureBits,
    ) {
        self.send_backend_message(
            BackendSceneBuilderRequest::StartCaptureSequence(CaptureConfig::new(root, bits))
        );
    }

    #[cfg(feature = "capture")]
    fn stop_capture_sequence(
        &mut self,
    ) {
        self.send_backend_message(
            BackendSceneBuilderRequest::StopCaptureSequence
        );
    }

    #[cfg(feature = "replay")]
    fn load_capture(
        &mut self,
        mut config: CaptureConfig,
        profile_counters: &mut BackendProfileCounters,
    ) {
        debug!("capture: loading {:?}", config.frame_root());
        let backend = config.deserialize_for_frame::<PlainRenderBackend, _>("backend")
            .expect("Unable to open backend.ron");

        // If this is a capture sequence, then the ID will be non-zero, and won't
        // match what is loaded, but for still captures, the ID will be zero.
        let first_load = backend.resource_sequence_id == 0;
        if self.loaded_resource_sequence_id != backend.resource_sequence_id || first_load {
            // FIXME(aosmond): We clear the documents because when we update the
            // resource cache, we actually wipe and reload, because we don't
            // know what is the same and what has changed. If we were to keep as
            // much of the resource cache state as possible, we could avoid
            // flushing the document state (which has its own dependecies on the
            // cache).
            //
            // FIXME(aosmond): If we try to load the next capture in the
            // sequence too quickly, we may lose resources we depend on in the
            // current frame. This can cause panics. Ideally we would not
            // advance to the next frame until the FrameRendered event for all
            // of the pipelines.
            self.documents.clear();

            config.resource_id = backend.resource_sequence_id;
            self.loaded_resource_sequence_id = backend.resource_sequence_id;

            let plain_resources = config.deserialize_for_resource::<PlainResources, _>("plain-resources")
                .expect("Unable to open plain-resources.ron");
            let caches_maybe = config.deserialize_for_resource::<PlainCacheOwn, _>("resource_cache");

            // Note: it would be great to have `RenderBackend` to be split
            // rather explicitly on what's used before and after scene building
            // so that, for example, we never miss anything in the code below:

            let plain_externals = self.resource_cache.load_capture(
                plain_resources,
                caches_maybe,
                &config,
            );

            let msg_load = ResultMsg::DebugOutput(
                DebugOutput::LoadCapture(config.clone(), plain_externals)
            );
            self.result_tx.send(msg_load).unwrap();

            self.gpu_cache = match config.deserialize_for_resource::<GpuCache, _>("gpu_cache") {
                Some(gpu_cache) => gpu_cache,
                None => GpuCache::new(),
            };
        }

        self.default_device_pixel_ratio = backend.default_device_pixel_ratio;
        self.frame_config = backend.frame_config;

        let mut scenes_to_build = Vec::new();

        for (id, view) in backend.documents {
            debug!("\tdocument {:?}", id);
            let scene_name = format!("scene-{}-{}", id.namespace_id.0, id.id);
            let scene = config.deserialize_for_scene::<Scene, _>(&scene_name)
                .expect(&format!("Unable to open {}.ron", scene_name));

            let interners_name = format!("interners-{}-{}", id.namespace_id.0, id.id);
            let interners = config.deserialize_for_scene::<Interners, _>(&interners_name)
                .expect(&format!("Unable to open {}.ron", interners_name));

            let data_stores_name = format!("data-stores-{}-{}", id.namespace_id.0, id.id);
            let data_stores = config.deserialize_for_frame::<DataStores, _>(&data_stores_name)
                .expect(&format!("Unable to open {}.ron", data_stores_name));

            let properties_name = format!("properties-{}-{}", id.namespace_id.0, id.id);
            let properties = config.deserialize_for_frame::<SceneProperties, _>(&properties_name)
                .expect(&format!("Unable to open {}.ron", properties_name));

            // Update the document if it still exists, rather than replace it entirely.
            // This allows us to preserve state information such as the frame stamp,
            // which is necessary for cache sanity.
            match self.documents.entry(id) {
                Occupied(entry) => {
                    let doc = entry.into_mut();
                    doc.view = view;
                    doc.loaded_scene = scene.clone();
                    doc.data_stores = data_stores;
                    doc.dynamic_properties = properties;
                    doc.frame_is_valid = false;
                    doc.rendered_frame_is_valid = false;
                    doc.has_built_scene = false;
                    doc.hit_tester_is_valid = false;
                }
                Vacant(entry) => {
                    let doc = Document {
                        id,
                        scene: BuiltScene::empty(),
                        removed_pipelines: Vec::new(),
                        view,
                        stamp: FrameStamp::first(id),
                        frame_builder: FrameBuilder::new(),
                        dynamic_properties: properties,
                        hit_tester: None,
                        shared_hit_tester: Arc::new(SharedHitTester::new()),
                        frame_is_valid: false,
                        hit_tester_is_valid: false,
                        rendered_frame_is_valid: false,
                        has_built_scene: false,
                        data_stores,
                        scratch: PrimitiveScratchBuffer::new(),
                        render_task_counters: RenderTaskGraphCounters::new(),
                        loaded_scene: scene.clone(),
                        prev_composite_descriptor: CompositeDescriptor::empty(),
                    };
                    entry.insert(doc);
                }
            };

            let frame_name = format!("frame-{}-{}", id.namespace_id.0, id.id);
            let frame = config.deserialize_for_frame::<Frame, _>(frame_name);
            let build_frame = match frame {
                Some(frame) => {
                    info!("\tloaded a built frame with {} passes", frame.passes.len());

                    let msg_update = ResultMsg::UpdateGpuCache(self.gpu_cache.extract_updates());
                    self.result_tx.send(msg_update).unwrap();

                    let msg_publish = ResultMsg::PublishDocument(
                        id,
                        RenderedDocument { frame, is_new_scene: true },
                        self.resource_cache.pending_updates(),
                        profile_counters.clone(),
                    );
                    self.result_tx.send(msg_publish).unwrap();
                    profile_counters.reset();

                    self.notifier.new_frame_ready(id, false, true, None);

                    // We deserialized the state of the frame so we don't want to build
                    // it (but we do want to update the scene builder's state)
                    false
                }
                None => true,
            };

            scenes_to_build.push(LoadScene {
                document_id: id,
                scene,
                view: view.scene.clone(),
                config: self.frame_config.clone(),
                font_instances: self.resource_cache.get_font_instances(),
                build_frame,
                interners,
            });
        }

        if !scenes_to_build.is_empty() {
            self.send_backend_message(
                BackendSceneBuilderRequest::LoadScenes(scenes_to_build)
            );
        }
    }
}
