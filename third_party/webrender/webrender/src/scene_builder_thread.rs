/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{AsyncBlobImageRasterizer, BlobImageRequest, BlobImageResult};
use api::{DocumentId, PipelineId, ApiMsg, FrameMsg, SceneMsg, ResourceUpdate, ExternalEvent};
use api::{NotificationRequest, Checkpoint, IdNamespace, QualitySettings, TransactionMsg};
use api::{ClipIntern, FilterDataIntern, MemoryReport, PrimitiveKeyKind, SharedFontInstanceMap};
use api::{DocumentLayer, GlyphDimensionRequest, GlyphIndexRequest};
use api::units::*;
#[cfg(feature = "capture")]
use crate::capture::CaptureConfig;
use crate::frame_builder::FrameBuilderConfig;
use crate::scene_building::SceneBuilder;
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
use crate::render_backend::SceneView;
use crate::renderer::{PipelineInfo, SceneBuilderHooks};
use crate::scene::{Scene, BuiltScene, SceneStats};
use std::iter;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::mem::replace;
use time::precise_time_ns;
use crate::util::drain_filter;
use std::thread;
use std::time::Duration;

#[cfg(feature = "debugger")]
use crate::debug_server;
#[cfg(feature = "debugger")]
use api::{BuiltDisplayListIter, DisplayItem};

/// Various timing information that will be turned into
/// TransactionProfileCounters later down the pipeline.
#[derive(Clone, Debug)]
pub struct TransactionTimings {
    pub builder_start_time_ns: u64,
    pub builder_end_time_ns: u64,
    pub send_time_ns: u64,
    pub scene_build_start_time_ns: u64,
    pub scene_build_end_time_ns: u64,
    pub blob_rasterization_end_time_ns: u64,
    pub display_list_len: usize,
}

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
    pub scene_build_start_time: u64,
    pub scene_build_end_time: u64,
    pub render_frame: bool,
    pub invalidate_rendered_frame: bool,
    pub discard_frame_state_for_pipelines: Vec<PipelineId>,
    pub timings: Option<TransactionTimings>,
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

// Message to the scene builder thread.
pub enum SceneBuilderRequest {
    Transactions(Vec<Box<TransactionMsg>>),
    ExternalEvent(ExternalEvent),
    AddDocument(DocumentId, DeviceIntSize, DocumentLayer),
    DeleteDocument(DocumentId),
    GetGlyphDimensions(GlyphDimensionRequest),
    GetGlyphIndices(GlyphIndexRequest),
    WakeUp,
    Stop,
    Flush(Sender<()>),
    ClearNamespace(IdNamespace),
    SimulateLongSceneBuild(u32),
    SimulateLongLowPrioritySceneBuild(u32),
    /// Enqueue this to inform the scene builder to pick one message from
    /// backend_rx.
    BackendMessage,
}

/// Message from render backend to scene builder.
pub enum BackendSceneBuilderRequest {
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
    DocumentsForDebugger
}

// Message from scene builder to render backend.
pub enum SceneBuilderResult {
    Transactions(Vec<Box<BuiltTransaction>>, Option<Sender<SceneSwapResult>>),
    #[cfg(feature = "capture")]
    CapturedTransactions(Vec<Box<BuiltTransaction>>, CaptureConfig, Option<Sender<SceneSwapResult>>),
    ExternalEvent(ExternalEvent),
    FlushComplete(Sender<()>),
    ClearNamespace(IdNamespace),
    GetGlyphDimensions(GlyphDimensionRequest),
    GetGlyphIndices(GlyphIndexRequest),
    Stopped,
    DocumentsForDebugger(String)
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

enumerate_interners!(declare_interners);

// A document in the scene builder contains the current scene,
// as well as a persistent clip interner. This allows clips
// to be de-duplicated, and persisted in the GPU cache between
// display lists.
struct Document {
    scene: Scene,
    interners: Interners,
    stats: SceneStats,
    view: SceneView,
    /// A set of pipelines that the caller has requested be
    /// made available as output textures.
    output_pipelines: FastHashSet<PipelineId>,
}

impl Document {
    fn new(device_rect: DeviceIntRect, layer: DocumentLayer, device_pixel_ratio: f32) -> Self {
        Document {
            scene: Scene::new(),
            interners: Interners::default(),
            stats: SceneStats::empty(),
            output_pipelines: FastHashSet::default(),
            view: SceneView {
                device_rect,
                layer,
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
    backend_rx: Receiver<BackendSceneBuilderRequest>,
    tx: Sender<SceneBuilderResult>,
    api_tx: Sender<ApiMsg>,
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
    backend_rx: Receiver<BackendSceneBuilderRequest>,
    tx: Sender<SceneBuilderResult>,
    api_tx: Sender<ApiMsg>,
}

impl SceneBuilderThreadChannels {
    pub fn new(
        api_tx: Sender<ApiMsg>
    ) -> (Self, Sender<SceneBuilderRequest>, Sender<BackendSceneBuilderRequest>, Receiver<SceneBuilderResult>) {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = channel();
        let (backend_tx, backend_rx) = channel();
        (
            Self {
                rx: in_rx,
                backend_rx,
                tx: out_tx,
                api_tx,
            },
            in_tx,
            backend_tx,
            out_rx,
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
        let SceneBuilderThreadChannels { rx, backend_rx, tx, api_tx } = channels;

        Self {
            documents: Default::default(),
            rx,
            backend_rx,
            tx,
            api_tx,
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
        self.tx.send(msg).unwrap();
        let _ = self.api_tx.send(ApiMsg::WakeUp);
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
                Ok(SceneBuilderRequest::Transactions(mut txns)) => {
                    let built_txns : Vec<Box<BuiltTransaction>> = txns.iter_mut()
                        .map(|txn| self.process_transaction(txn))
                        .collect();
                    #[cfg(feature = "capture")]
                    match built_txns.iter().any(|txn| txn.built_scene.is_some()) {
                        true => self.save_capture_sequence(),
                        _ => {},
                    }
                    self.forward_built_transactions(built_txns);
                }
                Ok(SceneBuilderRequest::AddDocument(document_id, initial_size, layer)) => {
                    let old = self.documents.insert(document_id, Document::new(
                        initial_size.into(),
                        layer,
                        self.default_device_pixel_ratio,
                    ));
                    debug_assert!(old.is_none());
                }
                Ok(SceneBuilderRequest::DeleteDocument(document_id)) => {
                    self.documents.remove(&document_id);
                }
                Ok(SceneBuilderRequest::ClearNamespace(id)) => {
                    self.documents.retain(|doc_id, _doc| doc_id.namespace_id != id);
                    self.send(SceneBuilderResult::ClearNamespace(id));
                }
                Ok(SceneBuilderRequest::ExternalEvent(evt)) => {
                    self.send(SceneBuilderResult::ExternalEvent(evt));
                }
                Ok(SceneBuilderRequest::GetGlyphDimensions(request)) => {
                    self.send(SceneBuilderResult::GetGlyphDimensions(request))
                }
                Ok(SceneBuilderRequest::GetGlyphIndices(request)) => {
                    self.send(SceneBuilderResult::GetGlyphIndices(request))
                }
                Ok(SceneBuilderRequest::Stop) => {
                    self.tx.send(SceneBuilderResult::Stopped).unwrap();
                    // We don't need to send a WakeUp to api_tx because we only
                    // get the Stop when the RenderBackend loop is exiting.
                    break;
                }
                Ok(SceneBuilderRequest::SimulateLongSceneBuild(time_ms)) => {
                    self.simulate_slow_ms = time_ms
                }
                Ok(SceneBuilderRequest::SimulateLongLowPrioritySceneBuild(_)) => {}
                Ok(SceneBuilderRequest::BackendMessage) => {
                    let msg = self.backend_rx.try_recv().unwrap();
                    match msg {
                        BackendSceneBuilderRequest::ReportMemory(mut report, tx) => {
                            (*report) += self.report_memory();
                            tx.send(report).unwrap();
                        }
                        BackendSceneBuilderRequest::SetFrameBuilderConfig(cfg) => {
                            self.config = cfg;
                        }
                        #[cfg(feature = "replay")]
                        BackendSceneBuilderRequest::LoadScenes(msg) => {
                            self.load_scenes(msg);
                        }
                        #[cfg(feature = "capture")]
                        BackendSceneBuilderRequest::SaveScene(config) => {
                            self.save_scene(config);
                        }
                        #[cfg(feature = "capture")]
                        BackendSceneBuilderRequest::StartCaptureSequence(config) => {
                            self.start_capture_sequence(config);
                        }
                        #[cfg(feature = "capture")]
                        BackendSceneBuilderRequest::StopCaptureSequence => {
                            // FIXME(aosmond): clear config for frames and resource cache without scene
                            // rebuild?
                            self.capture_config = None;
                        }
                        BackendSceneBuilderRequest::DocumentsForDebugger => {
                            let json = self.get_docs_for_debugger();
                            self.send(SceneBuilderResult::DocumentsForDebugger(json));
                        }
                    }
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

            if config.bits.contains(api::CaptureBits::SCENE) {
                let file_name = format!("scene-{}-{}", id.namespace_id.0, id.id);
                config.serialize_for_scene(&doc.scene, file_name);
            }
        }
    }

    #[cfg(feature = "replay")]
    fn load_scenes(&mut self, scenes: Vec<LoadScene>) {
        for mut item in scenes {
            self.config = item.config;

            let scene_build_start_time = precise_time_ns();

            let mut built_scene = None;
            let mut interner_updates = None;

            let output_pipelines = FastHashSet::default();

            if item.scene.has_root_pipeline() {
                built_scene = Some(SceneBuilder::build(
                    &item.scene,
                    item.font_instances,
                    &item.view,
                    &output_pipelines,
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
                    output_pipelines,
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
                scene_build_start_time,
                scene_build_end_time: precise_time_ns(),
                interner_updates,
                timings: None,
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

                if config.bits.contains(api::CaptureBits::SCENE) {
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

    #[cfg(feature = "debugger")]
    fn traverse_items<'a>(
        &self,
        traversal: &mut BuiltDisplayListIter<'a>,
        node: &mut debug_server::TreeNode,
    ) {
        loop {
            let subtraversal = {
                let item = match traversal.next() {
                    Some(item) => item,
                    None => break,
                };

                match *item.item() {
                    display_item @ DisplayItem::PushStackingContext(..) => {
                        let mut subtraversal = item.sub_iter();
                        let mut child_node =
                            debug_server::TreeNode::new(&display_item.debug_name().to_string());
                        self.traverse_items(&mut subtraversal, &mut child_node);
                        node.add_child(child_node);
                        Some(subtraversal)
                    }
                    DisplayItem::PopStackingContext => {
                        return;
                    }
                    display_item => {
                        node.add_item(&display_item.debug_name().to_string());
                        None
                    }
                }
            };

            // If flatten_item created a sub-traversal, we need `traversal` to have the
            // same state as the completed subtraversal, so we reinitialize it here.
            if let Some(subtraversal) = subtraversal {
                *traversal = subtraversal;
            }
        }
    }

    #[cfg(not(feature = "debugger"))]
    fn get_docs_for_debugger(&self) -> String {
        String::new()
    }

    #[cfg(feature = "debugger")]
    fn get_docs_for_debugger(&self) -> String {
        let mut docs = debug_server::DocumentList::new();

        for (_, doc) in &self.documents {
            let mut debug_doc = debug_server::TreeNode::new("document");

            for (_, pipeline) in &doc.scene.pipelines {
                let mut debug_dl = debug_server::TreeNode::new("display-list");
                self.traverse_items(&mut pipeline.display_list.iter(), &mut debug_dl);
                debug_doc.add_child(debug_dl);
            }

            docs.add(debug_doc);
        }

        serde_json::to_string(&docs).unwrap()
    }

    /// Do the bulk of the work of the scene builder thread.
    fn process_transaction(&mut self, txn: &mut TransactionMsg) -> Box<BuiltTransaction> {
        profile_scope!("process_transaction");

        if let Some(ref hooks) = self.hooks {
            hooks.pre_scene_build();
        }

        let scene_build_start_time = precise_time_ns();

        let doc = self.documents.get_mut(&txn.document_id).unwrap();
        let scene = &mut doc.scene;

        let mut timings = None;

        let mut discard_frame_state_for_pipelines = Vec::new();
        let mut removed_pipelines = Vec::new();
        let mut rebuild_scene = false;
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
                    content_size,
                    display_list,
                    preserve_frame_state,
                } => {
                    let display_list_len = display_list.data().len();

                    let (builder_start_time_ns, builder_end_time_ns, send_time_ns) =
                        display_list.times();

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
                        content_size,
                    );

                    timings = Some(TransactionTimings {
                        builder_start_time_ns,
                        builder_end_time_ns,
                        send_time_ns,
                        scene_build_start_time_ns: 0,
                        scene_build_end_time_ns: 0,
                        blob_rasterization_end_time_ns: 0,
                        display_list_len,
                    });

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
                SceneMsg::EnableFrameOutput(pipeline_id, enable) => {
                    if enable {
                        doc.output_pipelines.insert(pipeline_id);
                    } else {
                        doc.output_pipelines.remove(&pipeline_id);
                    }
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
                &doc.output_pipelines,
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

        let scene_build_end_time = precise_time_ns();

        let is_low_priority = false;
        rasterize_blobs(txn, is_low_priority);

        if let Some(timings) = timings.as_mut() {
            timings.blob_rasterization_end_time_ns = precise_time_ns();
            timings.scene_build_start_time_ns = scene_build_start_time;
            timings.scene_build_end_time_ns = scene_build_end_time;
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
            render_frame: txn.generate_frame,
            invalidate_rendered_frame: txn.invalidate_rendered_frame,
            built_scene,
            view: doc.view,
            rasterized_blobs: replace(&mut txn.rasterized_blobs, Vec::new()),
            resource_updates: replace(&mut txn.resource_updates, Vec::new()),
            blob_rasterizer: replace(&mut txn.blob_rasterizer, None),
            frame_ops: replace(&mut txn.frame_ops, Vec::new()),
            removed_pipelines,
            discard_frame_state_for_pipelines,
            notifications: replace(&mut txn.notifications, Vec::new()),
            interner_updates,
            scene_build_start_time,
            scene_build_end_time,
            timings,
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

                    let (tx, rx) = channel();
                    let txn = txns.iter().find(|txn| txn.built_scene.is_some()).unwrap();
                    hooks.pre_scene_swap(txn.scene_build_end_time - txn.scene_build_start_time);

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
            Some(ref config) => self.tx.send(SceneBuilderResult::CapturedTransactions(txns, config.clone(), result_tx)).unwrap(),
            None => self.tx.send(SceneBuilderResult::Transactions(txns, result_tx)).unwrap(),
        }

        #[cfg(not(feature = "capture"))]
        self.tx.send(SceneBuilderResult::Transactions(txns, result_tx)).unwrap();

        let _ = self.api_tx.send(ApiMsg::WakeUp);

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
    pub simulate_slow_ms: u32,
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
                Ok(SceneBuilderRequest::AddDocument(id, size, layer)) => {
                    self.tx.send(SceneBuilderRequest::AddDocument(id, size, layer)).unwrap();
                }
                Ok(SceneBuilderRequest::DeleteDocument(document_id)) => {
                    self.tx.send(SceneBuilderRequest::DeleteDocument(document_id)).unwrap();
                }
                Ok(SceneBuilderRequest::Stop) => {
                    self.tx.send(SceneBuilderRequest::Stop).unwrap();
                    break;
                }
                Ok(SceneBuilderRequest::SimulateLongLowPrioritySceneBuild(time_ms)) => {
                    self.simulate_slow_ms = time_ms;
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

        if self.simulate_slow_ms > 0 {
            thread::sleep(Duration::from_millis(self.simulate_slow_ms as u64));
        }

        txn
    }
}
