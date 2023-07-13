/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{AsyncBlobImageRasterizer, BlobImageResult};
use api::{DocumentId, PipelineId, ExternalEvent, BlobImageRequest};
use api::{NotificationRequest, Checkpoint, IdNamespace, QualitySettings};
use api::{PrimitiveKeyKind, SharedFontInstanceMap};
use api::{GlyphDimensionRequest, GlyphIndexRequest};
use api::channel::{unbounded_channel, single_msg_channel, Receiver, Sender};
use api::units::*;
use crate::render_api::{ApiMsg, FrameMsg, SceneMsg, ResourceUpdate, TransactionMsg, MemoryReport};
#[cfg(feature = "capture")]
use crate::capture::CaptureConfig;
use crate::frame_builder::FrameBuilderConfig;
use crate::scene_building::SceneBuilder;
use crate::clip::{ClipIntern, PolygonIntern};
use crate::filterdata::FilterDataIntern;
use crate::intern::{Internable, Interner, UpdateList};
use crate::internal_types::{FastHashMap, FastHashSet};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use crate::prim_store::backdrop::Backdrop;
use crate::prim_store::borders::{ImageBorder, NormalBorderPrim};
use crate::prim_store::gradient::{LinearGradient, RadialGradient, ConicGradient};
use crate::prim_store::image::{Image, YuvImage};
use crate::prim_store::line_dec::LineDecoration;
use crate::prim_store::picture::Picture;
use crate::prim_store::text_run::TextRun;
use crate::profiler::{self, TransactionProfile};
use crate::render_backend::SceneView;
use crate::renderer::{FullFrameStats, PipelineInfo, SceneBuilderHooks};
use crate::scene::{Scene, BuiltScene, SceneStats};
use std::iter;
use time::precise_time_ns;
use crate::util::drain_filter;
use std::thread;
use std::time::Duration;

fn rasterize_blobs(txn: &mut TransactionMsg, is_low_priority: bool) {
    profile_scope!("rasterize_blobs");

    if let Some(ref mut rasterizer) = txn.blob_rasterizer {
        let mut rasterized_blobs = rasterizer.rasterize(&txn.blob_requests, is_low_priority);
        // try using the existing allocation if our current list is empty
        if txn.rasterized_blobs.is_empty() {
            txn.rasterized_blobs = rasterized_blobs;
        } else {
            txn.rasterized_blobs.append(&mut rasterized_blobs);
        }
    }
}

/// Represent the remaining work associated to a transaction after the scene building
/// phase as well as the result of scene building itself if applicable.
pub struct BuiltTransaction {
    pub document_id: DocumentId,
    pub built_scene: Option<BuiltScene>,
    pub view: SceneView,
    pub resource_updates: Vec<ResourceUpdate>,
    pub rasterized_blobs: Vec<(BlobImageRequest, BlobImageResult)>,
    pub blob_rasterizer: Option<Box<dyn AsyncBlobImageRasterizer>>,
    pub frame_ops: Vec<FrameMsg>,
    pub removed_pipelines: Vec<(PipelineId, DocumentId)>,
    pub notifications: Vec<NotificationRequest>,
    pub interner_updates: Option<InternerUpdates>,
    pub render_frame: bool,
    pub invalidate_rendered_frame: bool,
    pub discard_frame_state_for_pipelines: Vec<PipelineId>,
    pub profile: TransactionProfile,
    pub frame_stats: FullFrameStats,
}

#[cfg(feature = "replay")]
pub struct LoadScene {
    pub document_id: DocumentId,
    pub scene: Scene,
    pub font_instances: SharedFontInstanceMap,
    pub view: SceneView,
    pub config: FrameBuilderConfig,
    pub build_frame: bool,
    pub interners: Interners,
}

/// Message to the scene builder thread.
pub enum SceneBuilderRequest {
    Transactions(Vec<Box<TransactionMsg>>),
    AddDocument(DocumentId, DeviceIntSize),
    DeleteDocument(DocumentId),
    GetGlyphDimensions(GlyphDimensionRequest),
    GetGlyphIndices(GlyphIndexRequest),
    ClearNamespace(IdNamespace),
    SimulateLongSceneBuild(u32),
    ExternalEvent(ExternalEvent),
    WakeUp,
    StopRenderBackend,
    ShutDown(Option<Sender<()>>),
    Flush(Sender<()>),
    SetFrameBuilderConfig(FrameBuilderConfig),
    ReportMemory(Box<MemoryReport>, Sender<Box<MemoryReport>>),
    #[cfg(feature = "capture")]
    SaveScene(CaptureConfig),
    #[cfg(feature = "replay")]
    LoadScenes(Vec<LoadScene>),
    #[cfg(feature = "capture")]
    StartCaptureSequence(CaptureConfig),
    #[cfg(feature = "capture")]
    StopCaptureSequence,
}

// Message from scene builder to render backend.
pub enum SceneBuilderResult {
    Transactions(Vec<Box<BuiltTransaction>>, Option<Sender<SceneSwapResult>>),
    ExternalEvent(ExternalEvent),
    FlushComplete(Sender<()>),
    DeleteDocument(DocumentId),
    ClearNamespace(IdNamespace),
    GetGlyphDimensions(GlyphDimensionRequest),
    GetGlyphIndices(GlyphIndexRequest),
    StopRenderBackend,
    ShutDown(Option<Sender<()>>),

    #[cfg(feature = "capture")]
    /// The same as `Transactions`, but also supplies a `CaptureConfig` that the
    /// render backend should use for sequence capture, until the next
    /// `CapturedTransactions` or `StopCaptureSequence` result.
    CapturedTransactions(Vec<Box<BuiltTransaction>>, CaptureConfig, Option<Sender<SceneSwapResult>>),

    #[cfg(feature = "capture")]
    /// The scene builder has stopped sequence capture, so the render backend
    /// should do the same.
    StopCaptureSequence,
}

// Message from render backend to scene builder to indicate the
// scene swap was completed. We need a separate channel for this
// so that they don't get mixed with SceneBuilderRequest messages.
pub enum SceneSwapResult {
    Complete(Sender<()>),
    Aborted,
}

macro_rules! declare_interners {
    ( $( $name:ident : $ty:ident, )+ ) => {
        /// This struct contains all items that can be shared between
        /// display lists. We want to intern and share the same clips,
        /// primitives and other things between display lists so that:
        /// - GPU cache handles remain valid, reducing GPU cache updates.
        /// - Comparison of primitives and pictures between two
        ///   display lists is (a) fast (b) done during scene building.
        #[cfg_attr(feature = "capture", derive(Serialize))]
        #[cfg_attr(feature = "replay", derive(Deserialize))]
        #[derive(Default)]
        pub struct Interners {
            $(
                pub $name: Interner<$ty>,
            )+
        }

        $(
            impl AsMut<Interner<$ty>> for Interners {
                fn as_mut(&mut self) -> &mut Interner<$ty> {
                    &mut self.$name
                }
            }
        )+

        pub struct InternerUpdates {
            $(
                pub $name: UpdateList<<$ty as Internable>::Key>,
            )+
        }

        impl Interners {
            /// Reports CPU heap memory used by the interners.
            fn report_memory(
                &self,
                ops: &mut MallocSizeOfOps,
                r: &mut MemoryReport,
            ) {
                $(
                    r.interning.interners.$name += self.$name.size_of(ops);
                )+
            }

            fn end_frame_and_get_pending_updates(&mut self) -> InternerUpdates {
                InternerUpdates {
                    $(
                        $name: self.$name.end_frame_and_get_pending_updates(),
                    )+
                }
            }
        }
    }
}

crate::enumerate_interners!(declare_interners);

// A document in the scene builder contains the current scene,
// as well as a persistent clip interner. This allows clips
// to be de-duplicated, and persisted in the GPU cache between
// display lists.
struct Document {
    scene: Scene,
    interners: Interners,
    stats: SceneStats,
    view: SceneView,
}

impl Document {
    fn new(device_rect: DeviceIntRect, device_pixel_ratio: f32) -> Self {
        Document {
            scene: Scene::new(),
            interners: Interners::default(),
            stats: SceneStats::empty(),
            view: SceneView {
                device_rect,
                device_pixel_ratio,
                page_zoom_factor: 1.0,
                quality_settings: QualitySettings::default(),
            },
        }
    }
}

pub struct SceneBuilderThread {
    documents: FastHashMap<DocumentId, Document>,
    rx: Receiver<SceneBuilderRequest>,
    tx: Sender<ApiMsg>,
    config: FrameBuilderConfig,
    default_device_pixel_ratio: f32,
    font_instances: SharedFontInstanceMap,
    size_of_ops: Option<MallocSizeOfOps>,
    hooks: Option<Box<dyn SceneBuilderHooks + Send>>,
    simulate_slow_ms: u32,
    removed_pipelines: FastHashSet<PipelineId>,
    #[cfg(feature = "capture")]
    capture_config: Option<CaptureConfig>,
}

pub struct SceneBuilderThreadChannels {
    rx: Receiver<SceneBuilderRequest>,
    tx: Sender<ApiMsg>,
}

impl SceneBuilderThreadChannels {
    pub fn new(
        tx: Sender<ApiMsg>
    ) -> (Self, Sender<SceneBuilderRequest>) {
        let (in_tx, in_rx) = unbounded_channel();
        (
            Self {
                rx: in_rx,
                tx,
            },
            in_tx,
        )
    }
}

impl SceneBuilderThread {
    pub fn new(
        config: FrameBuilderConfig,
        default_device_pixel_ratio: f32,
        font_instances: SharedFontInstanceMap,
        size_of_ops: Option<MallocSizeOfOps>,
        hooks: Option<Box<dyn SceneBuilderHooks + Send>>,
        channels: SceneBuilderThreadChannels,
    ) -> Self {
        let SceneBuilderThreadChannels { rx, tx } = channels;

        Self {
            documents: Default::default(),
            rx,
            tx,
            config,
            default_device_pixel_ratio,
            font_instances,
            size_of_ops,
            hooks,
            simulate_slow_ms: 0,
            removed_pipelines: FastHashSet::default(),
            #[cfg(feature = "capture")]
            capture_config: None,
        }
    }

    /// Send a message to the render backend thread.
    ///
    /// We first put something in the result queue and then send a wake-up
    /// message to the api queue that the render backend is blocking on.
    pub fn send(&self, msg: SceneBuilderResult) {
        self.tx.send(ApiMsg::SceneBuilderResult(msg)).unwrap();
    }

    /// The scene builder thread's event loop.
    pub fn run(&mut self) {
        if let Some(ref hooks) = self.hooks {
            hooks.register();
        }

        loop {
            tracy_begin_frame!("scene_builder_thread");

            match self.rx.recv() {
                Ok(SceneBuilderRequest::WakeUp) => {}
                Ok(SceneBuilderRequest::Flush(tx)) => {
                    self.send(SceneBuilderResult::FlushComplete(tx));
                }
                Ok(SceneBuilderRequest::Transactions(txns)) => {
                    let built_txns : Vec<Box<BuiltTransaction>> = txns.into_iter()
                        .map(|txn| self.process_transaction(*txn))
                        .collect();
                    #[cfg(feature = "capture")]
                    match built_txns.iter().any(|txn| txn.built_scene.is_some()) {
                        true => self.save_capture_sequence(),
                        _ => {},
                    }
                    self.forward_built_transactions(built_txns);
                }
                Ok(SceneBuilderRequest::AddDocument(document_id, initial_size)) => {
                    let old = self.documents.insert(document_id, Document::new(
                        initial_size.into(),
                        self.default_device_pixel_ratio,
                    ));
                    debug_assert!(old.is_none());
                }
                Ok(SceneBuilderRequest::DeleteDocument(document_id)) => {
                    self.documents.remove(&document_id);
                    self.send(SceneBuilderResult::DeleteDocument(document_id));
                }
                Ok(SceneBuilderRequest::ClearNamespace(id)) => {
                    self.documents.retain(|doc_id, _doc| doc_id.namespace_id != id);
                    self.send(SceneBuilderResult::ClearNamespace(id));
                }
                Ok(SceneBuilderRequest::ExternalEvent(evt)) => {
                    self.send(SceneBuilderResult::ExternalEvent(evt));
                }
                Ok(SceneBuilderRequest::GetGlyphDimensions(request)) => {
                    self.send(SceneBuilderResult::GetGlyphDimensions(request));
                }
                Ok(SceneBuilderRequest::GetGlyphIndices(request)) => {
                    self.send(SceneBuilderResult::GetGlyphIndices(request));
                }
                Ok(SceneBuilderRequest::StopRenderBackend) => {
                    self.send(SceneBuilderResult::StopRenderBackend);
                }
                Ok(SceneBuilderRequest::ShutDown(sync)) => {
                    self.send(SceneBuilderResult::ShutDown(sync));
                    break;
                }
                Ok(SceneBuilderRequest::SimulateLongSceneBuild(time_ms)) => {
                    self.simulate_slow_ms = time_ms
                }
                Ok(SceneBuilderRequest::ReportMemory(mut report, tx)) => {
                    (*report) += self.report_memory();
                    tx.send(report).unwrap();
                }
                Ok(SceneBuilderRequest::SetFrameBuilderConfig(cfg)) => {
                    self.config = cfg;
                }
                #[cfg(feature = "replay")]
                Ok(SceneBuilderRequest::LoadScenes(msg)) => {
                    self.load_scenes(msg);
                }
                #[cfg(feature = "capture")]
                Ok(SceneBuilderRequest::SaveScene(config)) => {
                    self.save_scene(config);
                }
                #[cfg(feature = "capture")]
                Ok(SceneBuilderRequest::StartCaptureSequence(config)) => {
                    self.start_capture_sequence(config);
                }
                #[cfg(feature = "capture")]
                Ok(SceneBuilderRequest::StopCaptureSequence) => {
                    // FIXME(aosmond): clear config for frames and resource cache without scene
                    // rebuild?
                    self.capture_config = None;
                    self.send(SceneBuilderResult::StopCaptureSequence);
                }
                Err(_) => {
                    break;
                }
            }

            if let Some(ref hooks) = self.hooks {
                hooks.poke();
            }

            tracy_end_frame!("scene_builder_thread");
        }

        if let Some(ref hooks) = self.hooks {
            hooks.deregister();
        }
    }

    #[cfg(feature = "capture")]
    fn save_scene(&mut self, config: CaptureConfig) {
        for (id, doc) in &self.documents {
            let interners_name = format!("interners-{}-{}", id.namespace_id.0, id.id);
            config.serialize_for_scene(&doc.interners, interners_name);

            use crate::render_api::CaptureBits;
            if config.bits.contains(CaptureBits::SCENE) {
                let file_name = format!("scene-{}-{}", id.namespace_id.0, id.id);
                config.serialize_for_scene(&doc.scene, file_name);
            }
        }
    }

    #[cfg(feature = "replay")]
    fn load_scenes(&mut self, scenes: Vec<LoadScene>) {
        for mut item in scenes {
            self.config = item.config;

            let mut built_scene = None;
            let mut interner_updates = None;

            if item.scene.has_root_pipeline() {
                built_scene = Some(SceneBuilder::build(
                    &item.scene,
                    item.font_instances,
                    &item.view,
                    &self.config,
                    &mut item.interners,
                    &SceneStats::empty(),
                ));

                interner_updates = Some(
                    item.interners.end_frame_and_get_pending_updates()
                );
            }

            self.documents.insert(
                item.document_id,
                Document {
                    scene: item.scene,
                    interners: item.interners,
                    stats: SceneStats::empty(),
                    view: item.view.clone(),
                },
            );

            let txns = vec![Box::new(BuiltTransaction {
                document_id: item.document_id,
                render_frame: item.build_frame,
                invalidate_rendered_frame: false,
                built_scene,
                view: item.view,
                resource_updates: Vec::new(),
                rasterized_blobs: Vec::new(),
                blob_rasterizer: None,
                frame_ops: Vec::new(),
                removed_pipelines: Vec::new(),
                discard_frame_state_for_pipelines: Vec::new(),
                notifications: Vec::new(),
                interner_updates,
                profile: TransactionProfile::new(),
                frame_stats: FullFrameStats::default(),
            })];

            self.forward_built_transactions(txns);
        }
    }

    #[cfg(feature = "capture")]
    fn save_capture_sequence(
        &mut self,
    ) {
        if let Some(ref mut config) = self.capture_config {
            config.prepare_scene();
            for (id, doc) in &self.documents {
                let interners_name = format!("interners-{}-{}", id.namespace_id.0, id.id);
                config.serialize_for_scene(&doc.interners, interners_name);

                use crate::render_api::CaptureBits;
                if config.bits.contains(CaptureBits::SCENE) {
                    let file_name = format!("scene-{}-{}", id.namespace_id.0, id.id);
                    config.serialize_for_scene(&doc.scene, file_name);
                }
            }
        }
    }

    #[cfg(feature = "capture")]
    fn start_capture_sequence(
        &mut self,
        config: CaptureConfig,
    ) {
        self.capture_config = Some(config);
        self.save_capture_sequence();
    }

    /// Do the bulk of the work of the scene builder thread.
    fn process_transaction(&mut self, mut txn: TransactionMsg) -> Box<BuiltTransaction> {
        profile_scope!("process_transaction");

        if let Some(ref hooks) = self.hooks {
            hooks.pre_scene_build();
        }

        let doc = self.documents.get_mut(&txn.document_id).unwrap();
        let scene = &mut doc.scene;

        let mut profile = txn.profile.take();

        let scene_build_start = precise_time_ns();
        let mut discard_frame_state_for_pipelines = Vec::new();
        let mut removed_pipelines = Vec::new();
        let mut rebuild_scene = false;
        let mut frame_stats = FullFrameStats::default();

        for message in txn.scene_ops.drain(..) {
            match message {
                SceneMsg::UpdateEpoch(pipeline_id, epoch) => {
                    scene.update_epoch(pipeline_id, epoch);
                }
                SceneMsg::SetPageZoom(factor) => {
                    doc.view.page_zoom_factor = factor.get();
                }
                SceneMsg::SetQualitySettings { settings } => {
                    doc.view.quality_settings = settings;
                }
                SceneMsg::SetDocumentView { device_rect, device_pixel_ratio } => {
                    doc.view.device_rect = device_rect;
                    doc.view.device_pixel_ratio = device_pixel_ratio;
                }
                SceneMsg::SetDisplayList {
                    epoch,
                    pipeline_id,
                    background,
                    viewport_size,
                    display_list,
                    preserve_frame_state,
                } => {
                    let (builder_start_time_ns, builder_end_time_ns, send_time_ns) =
                      display_list.times();
                    let content_send_time = profiler::ns_to_ms(precise_time_ns() - send_time_ns);
                    let dl_build_time = profiler::ns_to_ms(builder_end_time_ns - builder_start_time_ns);
                    profile.set(profiler::CONTENT_SEND_TIME, content_send_time);
                    profile.set(profiler::DISPLAY_LIST_BUILD_TIME, dl_build_time);
                    profile.set(profiler::DISPLAY_LIST_MEM, profiler::bytes_to_mb(display_list.data().len()));

                    let (gecko_display_list_time, full_display_list) = display_list.gecko_display_list_stats();
                    frame_stats.full_display_list = full_display_list;
                    frame_stats.gecko_display_list_time = gecko_display_list_time;
                    frame_stats.wr_display_list_time += dl_build_time;

                    if self.removed_pipelines.contains(&pipeline_id) {
                        continue;
                    }

                    // Note: We could further reduce the amount of unnecessary scene
                    // building by keeping track of which pipelines are used by the
                    // scene (bug 1490751).
                    rebuild_scene = true;

                    scene.set_display_list(
                        pipeline_id,
                        epoch,
                        display_list,
                        background,
                        viewport_size,
                    );

                    if !preserve_frame_state {
                        discard_frame_state_for_pipelines.push(pipeline_id);
                    }
                }
                SceneMsg::SetRootPipeline(pipeline_id) => {
                    if scene.root_pipeline_id != Some(pipeline_id) {
                        rebuild_scene = true;
                        scene.set_root_pipeline_id(pipeline_id);
                    }
                }
                SceneMsg::RemovePipeline(pipeline_id) => {
                    scene.remove_pipeline(pipeline_id);
                    self.removed_pipelines.insert(pipeline_id);
                    removed_pipelines.push((pipeline_id, txn.document_id));
                }
            }
        }

        self.removed_pipelines.clear();

        let mut built_scene = None;
        let mut interner_updates = None;
        if scene.has_root_pipeline() && rebuild_scene {

            let built = SceneBuilder::build(
                &scene,
                self.font_instances.clone(),
                &doc.view,
                &self.config,
                &mut doc.interners,
                &doc.stats,
            );

            // Update the allocation stats for next scene
            doc.stats = built.get_stats();

            // Retrieve the list of updates from the clip interner.
            interner_updates = Some(
                doc.interners.end_frame_and_get_pending_updates()
            );

            built_scene = Some(built);
        }

        let scene_build_time_ms =
            profiler::ns_to_ms(precise_time_ns() - scene_build_start);
        profile.set(profiler::SCENE_BUILD_TIME, scene_build_time_ms);

        frame_stats.scene_build_time += scene_build_time_ms;

        if !txn.blob_requests.is_empty() {
            profile.start_time(profiler::BLOB_RASTERIZATION_TIME);

            let is_low_priority = false;
            rasterize_blobs(&mut txn, is_low_priority);

            profile.end_time(profiler::BLOB_RASTERIZATION_TIME);
        }

        drain_filter(
            &mut txn.notifications,
            |n| { n.when() == Checkpoint::SceneBuilt },
            |n| { n.notify(); },
        );

        if self.simulate_slow_ms > 0 {
            thread::sleep(Duration::from_millis(self.simulate_slow_ms as u64));
        }

        Box::new(BuiltTransaction {
            document_id: txn.document_id,
            render_frame: txn.generate_frame.as_bool(),
            invalidate_rendered_frame: txn.invalidate_rendered_frame,
            built_scene,
            view: doc.view,
            rasterized_blobs: txn.rasterized_blobs,
            resource_updates: txn.resource_updates,
            blob_rasterizer: txn.blob_rasterizer,
            frame_ops: txn.frame_ops,
            removed_pipelines,
            discard_frame_state_for_pipelines,
            notifications: txn.notifications,
            interner_updates,
            profile,
            frame_stats,
        })
    }

    /// Send the results of process_transaction back to the render backend.
    fn forward_built_transactions(&mut self, txns: Vec<Box<BuiltTransaction>>) {
        let (pipeline_info, result_tx, result_rx) = match self.hooks {
            Some(ref hooks) => {
                if txns.iter().any(|txn| txn.built_scene.is_some()) {
                    let info = PipelineInfo {
                        epochs: txns.iter()
                            .filter(|txn| txn.built_scene.is_some())
                            .map(|txn| {
                                txn.built_scene.as_ref().unwrap()
                                    .pipeline_epochs.iter()
                                    .zip(iter::repeat(txn.document_id))
                                    .map(|((&pipeline_id, &epoch), document_id)| ((pipeline_id, document_id), epoch))
                            }).flatten().collect(),
                        removed_pipelines: txns.iter()
                            .map(|txn| txn.removed_pipelines.clone())
                            .flatten().collect(),
                    };

                    let (tx, rx) = single_msg_channel();
                    let txn = txns.iter().find(|txn| txn.built_scene.is_some()).unwrap();
                    hooks.pre_scene_swap((txn.profile.get(profiler::SCENE_BUILD_TIME).unwrap() * 1000000.0) as u64);

                    (Some(info), Some(tx), Some(rx))
                } else {
                    (None, None, None)
                }
            }
            _ => (None, None, None)
        };

        let scene_swap_start_time = precise_time_ns();
        let document_ids = txns.iter().map(|txn| txn.document_id).collect();
        let have_resources_updates : Vec<DocumentId> = if pipeline_info.is_none() {
            txns.iter()
                .filter(|txn| !txn.resource_updates.is_empty() || txn.invalidate_rendered_frame)
                .map(|txn| txn.document_id)
                .collect()
        } else {
            Vec::new()
        };

        #[cfg(feature = "capture")]
        match self.capture_config {
            Some(ref config) => self.send(SceneBuilderResult::CapturedTransactions(txns, config.clone(), result_tx)),
            None => self.send(SceneBuilderResult::Transactions(txns, result_tx)),
        };

        #[cfg(not(feature = "capture"))]
        self.send(SceneBuilderResult::Transactions(txns, result_tx));

        if let Some(pipeline_info) = pipeline_info {
            // Block until the swap is done, then invoke the hook.
            let swap_result = result_rx.unwrap().recv();
            let scene_swap_time = precise_time_ns() - scene_swap_start_time;
            self.hooks.as_ref().unwrap().post_scene_swap(&document_ids,
                                                         pipeline_info, scene_swap_time);
            // Once the hook is done, allow the RB thread to resume
            if let Ok(SceneSwapResult::Complete(resume_tx)) = swap_result {
                resume_tx.send(()).ok();
            }
        } else if !have_resources_updates.is_empty() {
            if let Some(ref hooks) = self.hooks {
                hooks.post_resource_update(&have_resources_updates);
            }
        } else if let Some(ref hooks) = self.hooks {
            hooks.post_empty_scene_build();
        }
    }

    /// Reports CPU heap memory used by the SceneBuilder.
    fn report_memory(&mut self) -> MemoryReport {
        let ops = self.size_of_ops.as_mut().unwrap();
        let mut report = MemoryReport::default();
        for doc in self.documents.values() {
            doc.interners.report_memory(ops, &mut report);
            doc.scene.report_memory(ops, &mut report);
        }

        report
    }
}

/// A scene builder thread which executes expensive operations such as blob rasterization
/// with a lower priority than the normal scene builder thread.
///
/// After rasterizing blobs, the secene building request is forwarded to the normal scene
/// builder where the FrameBuilder is generated.
pub struct LowPrioritySceneBuilderThread {
    pub rx: Receiver<SceneBuilderRequest>,
    pub tx: Sender<SceneBuilderRequest>,
}

impl LowPrioritySceneBuilderThread {
    pub fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(SceneBuilderRequest::Transactions(mut txns)) => {
                    let txns : Vec<Box<TransactionMsg>> = txns.drain(..)
                        .map(|txn| self.process_transaction(txn))
                        .collect();
                    self.tx.send(SceneBuilderRequest::Transactions(txns)).unwrap();
                }
                Ok(SceneBuilderRequest::ShutDown(sync)) => {
                    self.tx.send(SceneBuilderRequest::ShutDown(sync)).unwrap();
                    break;
                }
                Ok(other) => {
                    self.tx.send(other).unwrap();
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    fn process_transaction(&mut self, mut txn: Box<TransactionMsg>) -> Box<TransactionMsg> {
        let is_low_priority = true;
        rasterize_blobs(&mut txn, is_low_priority);
        txn.blob_requests = Vec::new();

        txn
    }
}
