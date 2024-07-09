/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Work around https://github.com/rust-lang/rust/issues/62132
#![recursion_limit = "128"]

//! Layout. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use std::process;
use std::sync::{Arc, Mutex};

use app_units::Au;
use base::id::{BrowsingContextId, PipelineId};
use base::Epoch;
use embedder_traits::resources::{self, Resource};
use euclid::default::{Point2D as UntypedPoint2D, Rect as UntypedRect, Size2D as UntypedSize2D};
use euclid::{Point2D, Rect, Scale, Size2D};
use fnv::FnvHashMap;
use fonts::{
    get_and_reset_text_shaping_performance_counter, FontCacheThread, FontContext,
    FontContextWebFontMethods,
};
use fonts_traits::WebFontLoadFinishedCallback;
use fxhash::{FxHashMap, FxHashSet};
use histogram::Histogram;
use ipc_channel::ipc::IpcSender;
use layout::construct::ConstructionResult;
use layout::context::{LayoutContext, RegisteredPainter, RegisteredPainters};
use layout::display_list::items::{DisplayList, ScrollOffsetMap, WebRenderImageInfo};
use layout::display_list::{IndexableText, ToLayout};
use layout::flow::{Flow, FlowFlags, GetBaseFlow, ImmutableFlowUtils, MutableOwnedFlowUtils};
use layout::flow_ref::FlowRef;
use layout::incremental::{RelayoutMode, SpecialRestyleDamage};
use layout::query::{
    process_client_rect_query, process_content_box_request, process_content_boxes_request,
    process_element_inner_text_query, process_offset_parent_query,
    process_resolved_font_style_request, process_resolved_style_request,
    process_scrolling_area_request,
};
use layout::traversal::{
    construct_flows_at_ancestors, ComputeStackingRelativePositions, PreorderFlowTraversal,
    RecalcStyleAndConstructFlows,
};
use layout::wrapper::ThreadSafeLayoutNodeHelpers;
use layout::{layout_debug, layout_debug_scope, parallel, sequential};
use lazy_static::lazy_static;
use log::{debug, error, trace, warn};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory};
use net_traits::image_cache::{ImageCache, UsePlaceholder};
use net_traits::ResourceThreads;
use parking_lot::RwLock;
use profile_traits::mem::{Report, ReportKind};
use profile_traits::path;
use profile_traits::time::{
    self as profile_time, profile, TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType,
};
use script::layout_dom::{ServoLayoutDocument, ServoLayoutElement, ServoLayoutNode};
use script_layout_interface::wrapper_traits::LayoutNode;
use script_layout_interface::{
    Layout, LayoutConfig, LayoutFactory, NodesFromPointQueryType, OffsetParentResponse, Reflow,
    ReflowComplete, ReflowGoal, ScriptReflow, TrustedNodeAddress,
};
use script_traits::{
    ConstellationControlMsg, DrawAPaintImageResult, IFrameSizeMsg, LayoutMsg as ConstellationMsg,
    PaintWorkletError, Painter, ScrollState, UntrustedNodeAddress, WindowSizeData, WindowSizeType,
};
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_config::opts::{self, DebugOptions};
use servo_config::pref;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::animation::{AnimationSetKey, DocumentAnimationSet, ElementAnimationSet};
use style::context::{
    QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters, SharedStyleContext,
};
use style::dom::{OpaqueNode, ShowSubtree, ShowSubtreeDataAndPrimaryValues, TElement, TNode};
use style::driver;
use style::error_reporting::RustLogReporter;
use style::global_style_data::{GLOBAL_STYLE_DATA, STYLE_THREAD_POOL};
use style::invalidation::element::restyle_hints::RestyleHint;
use style::logical_geometry::LogicalPoint;
use style::media_queries::{Device, MediaList, MediaType};
use style::properties::style_structs::Font;
use style::properties::{ComputedValues, PropertyId};
use style::selector_parser::{PseudoElement, SnapshotMap};
use style::servo::media_queries::FontMetricsProvider;
use style::servo::restyle_damage::ServoRestyleDamage;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards};
use style::stylesheets::{
    DocumentStyleSheet, Origin, Stylesheet, StylesheetInDocument, UrlExtraData,
    UserAgentStylesheets,
};
use style::stylist::Stylist;
use style::traversal::DomTraversal;
use style::traversal_flags::TraversalFlags;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{FontSize, Length, NonNegativeLength};
use style::values::specified::font::KeywordInfo;
use style_traits::{CSSPixel, DevicePixel, SpeculativePainter};
use url::Url;
use webrender_api::{units, ColorF, HitTestFlags};
use webrender_traits::WebRenderScriptApi;

/// Information needed by layout.
pub struct LayoutThread {
    /// The ID of the pipeline that we belong to.
    id: PipelineId,

    /// The URL of the pipeline that we belong to.
    url: ServoUrl,

    /// Performs CSS selector matching and style resolution.
    stylist: Stylist,

    /// Is the current reflow of an iframe, as opposed to a root window?
    is_iframe: bool,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: IpcSender<ConstellationMsg>,

    /// The channel on which messages can be sent to the script thread.
    script_chan: IpcSender<ConstellationControlMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// Reference to the script thread image cache.
    image_cache: Arc<dyn ImageCache>,

    /// A FontContext tFontCacheThreadImplg layout.
    font_context: Arc<FontContext<FontCacheThread>>,

    /// Is this the first reflow iFontCacheThreadImplread?
    first_reflow: Cell<bool>,

    /// Flag to indicate whether to use parallel operations
    parallel_flag: bool,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    generation: Cell<u32>,

    /// The root of the flow tree.
    root_flow: RefCell<Option<FlowRef>>,

    /// A counter for epoch messages
    epoch: Cell<Epoch>,

    /// The size of the viewport. This may be different from the size of the screen due to viewport
    /// constraints.
    viewport_size: UntypedSize2D<Au>,

    /// The root stacking context.
    display_list: RefCell<Option<DisplayList>>,

    /// A map that stores all of the indexable text in this layout.
    indexable_text: RefCell<IndexableText>,

    /// Scroll offsets of scrolling regions.
    scroll_offsets: RefCell<ScrollOffsetMap>,

    webrender_image_cache: Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), WebRenderImageInfo>>>,

    /// The executors for paint worklets.
    registered_painters: RegisteredPaintersImpl,

    /// Webrender interface.
    webrender_api: WebRenderScriptApi,

    /// Paint time metrics.
    paint_time_metrics: PaintTimeMetrics,

    /// The time a layout query has waited before serviced by layout.
    layout_query_waiting_time: Histogram,

    /// The sizes of all iframes encountered during the last layout operation.
    last_iframe_sizes: RefCell<FnvHashMap<BrowsingContextId, Size2D<f32, CSSPixel>>>,

    /// Debug options, copied from configuration to this `LayoutThread` in order
    /// to avoid having to constantly access the thread-safe global options.
    debug: DebugOptions,

    /// True to turn off incremental layout.
    nonincremental_layout: bool,
}

pub struct LayoutFactoryImpl();

impl LayoutFactory for LayoutFactoryImpl {
    fn create(&self, config: LayoutConfig) -> Box<dyn Layout> {
        Box::new(LayoutThread::new(
            config.id,
            config.url,
            config.is_iframe,
            config.constellation_chan,
            config.script_chan,
            config.image_cache,
            config.resource_threads,
            config.font_cache_thread,
            config.time_profiler_chan,
            config.webrender_api_sender,
            config.paint_time_metrics,
            config.window_size,
        ))
    }
}

struct ScriptReflowResult {
    script_reflow: ScriptReflow,
    result: RefCell<Option<ReflowComplete>>,
}

impl Deref for ScriptReflowResult {
    type Target = ScriptReflow;
    fn deref(&self) -> &ScriptReflow {
        &self.script_reflow
    }
}

impl DerefMut for ScriptReflowResult {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.script_reflow
    }
}

impl ScriptReflowResult {
    fn new(script_reflow: ScriptReflow) -> ScriptReflowResult {
        ScriptReflowResult {
            script_reflow,
            result: RefCell::new(Some(Default::default())),
        }
    }
}

impl Drop for ScriptReflowResult {
    fn drop(&mut self) {
        self.script_reflow
            .script_join_chan
            .send(self.result.borrow_mut().take().unwrap())
            .unwrap();
    }
}

impl Drop for LayoutThread {
    fn drop(&mut self) {
        let (keys, instance_keys) = self
            .font_context
            .collect_unused_webrender_resources(true /* all */);
        self.webrender_api
            .remove_unused_font_resources(keys, instance_keys)
    }
}

impl Layout for LayoutThread {
    fn device(&self) -> &Device {
        self.stylist.device()
    }

    fn waiting_for_web_fonts_to_load(&self) -> bool {
        self.font_context.web_fonts_still_loading() != 0
    }

    fn current_epoch(&self) -> Epoch {
        self.epoch.get()
    }

    fn load_web_fonts_from_stylesheet(&self, stylesheet: ServoArc<Stylesheet>) {
        let guard = stylesheet.shared_lock.read();
        self.load_all_web_fonts_from_stylesheet_with_guard(
            &DocumentStyleSheet(stylesheet.clone()),
            &guard,
        );
    }

    fn add_stylesheet(
        &mut self,
        stylesheet: ServoArc<Stylesheet>,
        before_stylesheet: Option<ServoArc<Stylesheet>>,
    ) {
        let guard = stylesheet.shared_lock.read();
        let stylesheet = DocumentStyleSheet(stylesheet.clone());
        self.load_all_web_fonts_from_stylesheet_with_guard(&stylesheet, &guard);

        match before_stylesheet {
            Some(insertion_point) => self.stylist.insert_stylesheet_before(
                stylesheet,
                DocumentStyleSheet(insertion_point),
                &guard,
            ),
            None => self.stylist.append_stylesheet(stylesheet, &guard),
        }
    }

    fn remove_stylesheet(&mut self, stylesheet: ServoArc<Stylesheet>) {
        let guard = stylesheet.shared_lock.read();
        let stylesheet = DocumentStyleSheet(stylesheet.clone());
        self.stylist.remove_stylesheet(stylesheet.clone(), &guard);
        self.font_context
            .remove_all_web_fonts_from_stylesheet(&stylesheet);
    }

    fn query_content_box(&self, node: OpaqueNode) -> Option<UntypedRect<Au>> {
        let mut root_flow = self.root_flow_for_query()?;

        let root_flow_ref = FlowRef::deref_mut(&mut root_flow);
        process_content_box_request(node, root_flow_ref)
    }

    fn query_content_boxes(&self, node: OpaqueNode) -> Vec<UntypedRect<Au>> {
        let Some(mut root_flow) = self.root_flow_for_query() else {
            return vec![];
        };
        let root_flow_ref = FlowRef::deref_mut(&mut root_flow);
        process_content_boxes_request(node, root_flow_ref)
    }

    fn query_client_rect(&self, node: OpaqueNode) -> UntypedRect<i32> {
        let Some(mut root_flow) = self.root_flow_for_query() else {
            return UntypedRect::zero();
        };
        let root_flow_ref = FlowRef::deref_mut(&mut root_flow);
        process_client_rect_query(node, root_flow_ref)
    }

    fn query_element_inner_text(
        &self,
        node: script_layout_interface::TrustedNodeAddress,
    ) -> String {
        let node = unsafe { ServoLayoutNode::new(&node) };
        process_element_inner_text_query(node, &self.indexable_text.borrow())
    }

    fn query_inner_window_dimension(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> Option<Size2D<f32, CSSPixel>> {
        self.last_iframe_sizes
            .borrow()
            .get(&browsing_context_id)
            .cloned()
    }

    fn query_nodes_from_point(
        &self,
        point: UntypedPoint2D<f32>,
        query_type: NodesFromPointQueryType,
    ) -> Vec<UntrustedNodeAddress> {
        let mut flags = match query_type {
            NodesFromPointQueryType::Topmost => HitTestFlags::empty(),
            NodesFromPointQueryType::All => HitTestFlags::FIND_ALL,
        };

        // The point we get is not relative to the entire WebRender scene, but to this
        // particular pipeline, so we need to tell WebRender about that.
        flags.insert(HitTestFlags::POINT_RELATIVE_TO_PIPELINE_VIEWPORT);

        let client_point = units::DevicePoint::from_untyped(point);
        let results = self
            .webrender_api
            .hit_test(Some(self.id.into()), client_point, flags);

        results.iter().map(|result| result.node.into()).collect()
    }

    fn query_offset_parent(&self, node: OpaqueNode) -> OffsetParentResponse {
        let Some(mut root_flow) = self.root_flow_for_query() else {
            return OffsetParentResponse::default();
        };
        let root_flow_ref = FlowRef::deref_mut(&mut root_flow);
        process_offset_parent_query(node, root_flow_ref)
    }

    fn query_resolved_style(
        &self,
        node: TrustedNodeAddress,
        pseudo: Option<PseudoElement>,
        property_id: PropertyId,
        animations: DocumentAnimationSet,
        animation_timeline_value: f64,
    ) -> String {
        let node = unsafe { ServoLayoutNode::new(&node) };
        let document = node.owner_doc();
        let document_shared_lock = document.style_shared_lock();
        let guards = StylesheetGuards {
            author: &document_shared_lock.read(),
            ua_or_user: &UA_STYLESHEETS.shared_lock.read(),
        };
        let snapshot_map = SnapshotMap::new();

        let shared_style_context = self.build_shared_style_context(
            guards,
            &snapshot_map,
            animation_timeline_value,
            &animations,
            TraversalFlags::empty(),
        );

        let Some(mut root_flow) = self.root_flow_for_query() else {
            return String::new();
        };
        let root_flow_ref = FlowRef::deref_mut(&mut root_flow);

        process_resolved_style_request(
            &shared_style_context,
            node,
            &pseudo,
            &property_id,
            root_flow_ref,
        )
    }

    fn query_resolved_font_style(
        &self,
        node: TrustedNodeAddress,
        value: &str,
        animations: DocumentAnimationSet,
        animation_timeline_value: f64,
    ) -> Option<ServoArc<Font>> {
        let node = unsafe { ServoLayoutNode::new(&node) };
        let document = node.owner_doc();
        let document_shared_lock = document.style_shared_lock();
        let guards = StylesheetGuards {
            author: &document_shared_lock.read(),
            ua_or_user: &UA_STYLESHEETS.shared_lock.read(),
        };
        let snapshot_map = SnapshotMap::new();
        let shared_style_context = self.build_shared_style_context(
            guards,
            &snapshot_map,
            animation_timeline_value,
            &animations,
            TraversalFlags::empty(),
        );

        process_resolved_font_style_request(
            &shared_style_context,
            node,
            value,
            self.url.clone(),
            document_shared_lock,
        )
    }

    fn query_scrolling_area(&self, node: Option<OpaqueNode>) -> UntypedRect<i32> {
        let Some(mut root_flow) = self.root_flow_for_query() else {
            return UntypedRect::zero();
        };
        let root_flow_ref = FlowRef::deref_mut(&mut root_flow);
        process_scrolling_area_request(node, root_flow_ref)
    }

    fn query_text_indext(
        &self,
        node: OpaqueNode,
        point_in_node: UntypedPoint2D<f32>,
    ) -> Option<usize> {
        let point_in_node = Point2D::new(
            Au::from_f32_px(point_in_node.x),
            Au::from_f32_px(point_in_node.y),
        );
        self.indexable_text.borrow().text_index(node, point_in_node)
    }

    fn exit_now(&mut self) {
        // Drop the root flow explicitly to avoid holding style data, such as
        // rule nodes.  The `Stylist` checks when it is dropped that all rule
        // nodes have been GCed, so we want drop anyone who holds them first.
        let waiting_time_min = self.layout_query_waiting_time.minimum().unwrap_or(0);
        let waiting_time_max = self.layout_query_waiting_time.maximum().unwrap_or(0);
        let waiting_time_mean = self.layout_query_waiting_time.mean().unwrap_or(0);
        let waiting_time_stddev = self.layout_query_waiting_time.stddev().unwrap_or(0);
        debug!(
            "layout: query waiting time: min: {}, max: {}, mean: {}, standard_deviation: {}",
            waiting_time_min, waiting_time_max, waiting_time_mean, waiting_time_stddev
        );

        self.root_flow.borrow_mut().take();
    }

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.stylist.set_quirks_mode(quirks_mode);
    }

    fn register_paint_worklet_modules(
        &mut self,
        name: Atom,
        mut properties: Vec<Atom>,
        painter: Box<dyn Painter>,
    ) {
        debug!("Registering the painter");
        let properties = properties
            .drain(..)
            .filter_map(|name| {
                let id = PropertyId::parse_enabled_for_all_content(&name).ok()?;
                Some((name.clone(), id))
            })
            .filter(|(_, id)| !id.is_shorthand())
            .collect();
        let registered_painter = RegisteredPainterImpl {
            name: name.clone(),
            properties,
            painter,
        };
        self.registered_painters.0.insert(name, registered_painter);
    }

    fn collect_reports(&self, reports: &mut Vec<Report>) {
        // Servo uses vanilla jemalloc, which doesn't have a
        // malloc_enclosing_size_of function.
        let mut ops = MallocSizeOfOps::new(servo_allocator::usable_size, None, None);

        // TODO: Measure more than just display list, stylist, and font context.
        let display_list = self.display_list.borrow();
        let display_list_ref = display_list.as_ref();
        let formatted_url = &format!("url({})", self.url);
        reports.push(Report {
            path: path![formatted_url, "layout-thread", "display-list"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: display_list_ref.map_or(0, |sc| sc.size_of(&mut ops)),
        });

        reports.push(Report {
            path: path![formatted_url, "layout-thread", "stylist"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: self.stylist.size_of(&mut ops),
        });

        reports.push(Report {
            path: path![formatted_url, "layout-thread", "font-context"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: self.font_context.size_of(&mut ops),
        });
    }

    fn reflow(&mut self, script_reflow: script_layout_interface::ScriptReflow) {
        let mut result = ScriptReflowResult::new(script_reflow);
        profile(
            profile_time::ProfilerCategory::LayoutPerform,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || self.handle_reflow(&mut result),
        );
    }

    fn set_scroll_states(&mut self, scroll_states: &[ScrollState]) {
        *self.scroll_offsets.borrow_mut() = scroll_states
            .iter()
            .map(|scroll_state| (scroll_state.scroll_id, scroll_state.scroll_offset))
            .collect();
    }

    fn set_epoch_paint_time(&mut self, epoch: Epoch, paint_time: u64) {
        self.paint_time_metrics.maybe_set_metric(epoch, paint_time);
    }
}
impl LayoutThread {
    fn root_flow_for_query(&self) -> Option<FlowRef> {
        self.root_flow.borrow().clone()
    }

    fn new(
        id: PipelineId,
        url: ServoUrl,
        is_iframe: bool,
        constellation_chan: IpcSender<ConstellationMsg>,
        script_chan: IpcSender<ConstellationControlMsg>,
        image_cache: Arc<dyn ImageCache>,
        resource_threads: ResourceThreads,
        font_cache_thread: FontCacheThread,
        time_profiler_chan: profile_time::ProfilerChan,
        webrender_api: WebRenderScriptApi,
        paint_time_metrics: PaintTimeMetrics,
        window_size: WindowSizeData,
    ) -> LayoutThread {
        // Let webrender know about this pipeline by sending an empty display list.
        webrender_api.send_initial_transaction(id.into());

        let mut font = Font::initial_values();
        let default_font_size = pref!(fonts.default_size);
        font.font_size = FontSize {
            computed_size: NonNegativeLength::new(default_font_size as f32),
            used_size: NonNegativeLength::new(default_font_size as f32),
            keyword_info: KeywordInfo::medium(),
        };

        let font_context = Arc::new(FontContext::new(font_cache_thread, resource_threads));
        let device = Device::new(
            MediaType::screen(),
            QuirksMode::NoQuirks,
            window_size.initial_viewport,
            window_size.device_pixel_ratio,
            Box::new(LayoutFontMetricsProvider),
            ComputedValues::initial_values_with_font_override(font),
        );

        LayoutThread {
            id,
            url,
            is_iframe,
            script_chan,
            constellation_chan,
            time_profiler_chan,
            registered_painters: RegisteredPaintersImpl(Default::default()),
            image_cache,
            font_context,
            first_reflow: Cell::new(true),
            parallel_flag: true,
            generation: Cell::new(0),
            root_flow: RefCell::new(None),
            // Epoch starts at 1 because of the initial display list for epoch 0 that we send to WR
            epoch: Cell::new(Epoch(1)),
            viewport_size: Size2D::new(
                Au::from_f32_px(window_size.initial_viewport.width),
                Au::from_f32_px(window_size.initial_viewport.height),
            ),
            webrender_api,
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            display_list: Default::default(),
            indexable_text: Default::default(),
            scroll_offsets: Default::default(),
            webrender_image_cache: Arc::new(RwLock::new(FnvHashMap::default())),
            paint_time_metrics,
            layout_query_waiting_time: Histogram::new(),
            last_iframe_sizes: Default::default(),
            debug: opts::get().debug.clone(),
            nonincremental_layout: opts::get().nonincremental_layout,
        }
    }

    fn build_shared_style_context<'a>(
        &'a self,
        guards: StylesheetGuards<'a>,
        snapshot_map: &'a SnapshotMap,
        animation_timeline_value: f64,
        animations: &DocumentAnimationSet,
        traversal_flags: TraversalFlags,
    ) -> SharedStyleContext<'a> {
        SharedStyleContext {
            stylist: &self.stylist,
            options: GLOBAL_STYLE_DATA.options.clone(),
            guards,
            visited_styles_enabled: false,
            animations: animations.clone(),
            registered_speculative_painters: &self.registered_painters,
            current_time_for_animations: animation_timeline_value,
            traversal_flags,
            snapshot_map,
        }
    }

    // Create a layout context for use in building display lists, hit testing, &c.
    fn build_layout_context<'a>(
        &'a self,
        guards: StylesheetGuards<'a>,
        snapshot_map: &'a SnapshotMap,
        origin: ImmutableOrigin,
        animation_timeline_value: f64,
        animations: &DocumentAnimationSet,
        stylesheets_changed: bool,
    ) -> LayoutContext<'a> {
        let traversal_flags = match stylesheets_changed {
            true => TraversalFlags::ForCSSRuleChanges,
            false => TraversalFlags::empty(),
        };

        LayoutContext {
            id: self.id,
            origin,
            style_context: self.build_shared_style_context(
                guards,
                snapshot_map,
                animation_timeline_value,
                animations,
                traversal_flags,
            ),
            image_cache: self.image_cache.clone(),
            font_context: self.font_context.clone(),
            webrender_image_cache: self.webrender_image_cache.clone(),
            pending_images: Mutex::new(vec![]),
            registered_painters: &self.registered_painters,
        }
    }

    fn load_all_web_fonts_from_stylesheet_with_guard(
        &self,
        stylesheet: &DocumentStyleSheet,
        guard: &SharedRwLockReadGuard,
    ) {
        if !stylesheet.is_effective_for_device(self.stylist.device(), guard) {
            return;
        }

        let locked_script_channel = Mutex::new(self.script_chan.clone());
        let pipeline_id = self.id;
        let web_font_finished_loading_callback =
            move |succeeded: bool| {
                let _ = locked_script_channel.lock().unwrap().send(
                    ConstellationControlMsg::WebFontLoaded(pipeline_id, succeeded),
                );
            };

        // Find all font-face rules and notify the FontContext of them.
        // GWTODO: Need to handle unloading web fonts.
        let newly_loading_font_count = self.font_context.add_all_web_fonts_from_stylesheet(
            stylesheet,
            guard,
            self.stylist.device(),
            Arc::new(web_font_finished_loading_callback) as WebFontLoadFinishedCallback,
            self.debug.load_webfonts_synchronously,
        );

        if self.debug.load_webfonts_synchronously && newly_loading_font_count > 0 {
            // TODO: Handle failure in web font loading
            let _ = self
                .script_chan
                .send(ConstellationControlMsg::WebFontLoaded(self.id, true));
        }
    }

    fn try_get_layout_root<'dom>(&self, node: impl LayoutNode<'dom>) -> Option<FlowRef> {
        let result = node
            .to_threadsafe()
            .mutate_layout_data()?
            .flow_construction_result
            .get();

        let mut flow = match result {
            ConstructionResult::Flow(mut flow, abs_descendants) => {
                // Note: Assuming that the root has display 'static' (as per
                // CSS Section 9.3.1). If it was absolutely positioned,
                // it would return a reference to itself in `abs_descendants`
                // and would lead to a circular reference. Otherwise, we
                // set Root as CB and push remaining absolute descendants.
                if flow
                    .base()
                    .flags
                    .contains(FlowFlags::IS_ABSOLUTELY_POSITIONED)
                {
                    flow.set_absolute_descendants(abs_descendants);
                } else {
                    flow.push_absolute_descendants(abs_descendants);
                }

                flow
            },
            _ => return None,
        };

        FlowRef::deref_mut(&mut flow).mark_as_root();

        Some(flow)
    }

    /// Performs layout constraint solving.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints(layout_root: &mut dyn Flow, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints");
        sequential::reflow(layout_root, layout_context, RelayoutMode::Incremental);
    }

    /// Performs layout constraint solving in parallel.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints_parallel(
        traversal: &rayon::ThreadPool,
        layout_root: &mut dyn Flow,
        profiler_metadata: Option<TimerMetadata>,
        time_profiler_chan: profile_time::ProfilerChan,
        layout_context: &LayoutContext,
    ) {
        let _scope = layout_debug_scope!("solve_constraints_parallel");

        // NOTE: this currently computes borders, so any pruning should separate that
        // operation out.
        parallel::reflow(
            layout_root,
            profiler_metadata,
            time_profiler_chan,
            layout_context,
            traversal,
        );
    }

    /// Update the recorded iframe sizes of the contents of layout and when these sizes changes,
    /// send a message to the constellation informing it of the new sizes.
    fn update_iframe_sizes(
        &self,
        new_iframe_sizes: FnvHashMap<BrowsingContextId, Size2D<f32, CSSPixel>>,
    ) {
        let old_iframe_sizes =
            std::mem::replace(&mut *self.last_iframe_sizes.borrow_mut(), new_iframe_sizes);

        if self.last_iframe_sizes.borrow().is_empty() {
            return;
        }

        let size_messages: Vec<_> = self
            .last_iframe_sizes
            .borrow()
            .iter()
            .filter_map(|(browsing_context_id, size)| {
                match old_iframe_sizes.get(browsing_context_id) {
                    Some(old_size) if old_size != size => Some(IFrameSizeMsg {
                        browsing_context_id: *browsing_context_id,
                        size: *size,
                        type_: WindowSizeType::Resize,
                    }),
                    None => Some(IFrameSizeMsg {
                        browsing_context_id: *browsing_context_id,
                        size: *size,
                        type_: WindowSizeType::Initial,
                    }),
                    _ => None,
                }
            })
            .collect();

        if !size_messages.is_empty() {
            let msg = ConstellationMsg::IFrameSizes(size_messages);
            if let Err(e) = self.constellation_chan.send(msg) {
                warn!("Layout resize to constellation failed ({}).", e);
            }
        }
    }

    /// Computes the stacking-relative positions of all flows and, if the painting is dirty and the
    /// reflow type need it, builds the display list.
    fn compute_abs_pos_and_build_display_list(
        &self,
        data: &Reflow,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument>,
        layout_root: &mut dyn Flow,
        layout_context: &mut LayoutContext,
    ) {
        let writing_mode = layout_root.base().writing_mode;
        let (metadata, sender) = (self.profiler_metadata(), self.time_profiler_chan.clone());
        profile(
            profile_time::ProfilerCategory::LayoutDispListBuild,
            metadata.clone(),
            sender.clone(),
            || {
                layout_root.mut_base().stacking_relative_position =
                    LogicalPoint::zero(writing_mode)
                        .to_physical(writing_mode, self.viewport_size)
                        .to_vector();

                layout_root.mut_base().clip = data.page_clip_rect;

                let traversal = ComputeStackingRelativePositions { layout_context };
                traversal.traverse(layout_root);

                if (layout_root
                    .base()
                    .restyle_damage
                    .contains(ServoRestyleDamage::REPAINT) ||
                    self.display_list.borrow().is_none()) &&
                    reflow_goal.needs_display_list()
                {
                    let background_color = get_root_flow_background_color(layout_root);
                    let mut build_state = sequential::build_display_list_for_subtree(
                        layout_root,
                        layout_context,
                        background_color,
                        data.page_clip_rect.size,
                    );

                    debug!("Done building display list.");

                    let root_size = {
                        let root_flow = layout_root.base();
                        root_flow.overflow.scroll.size
                    };

                    let origin = Rect::new(Point2D::new(Au(0), Au(0)), root_size).to_layout();
                    build_state.root_stacking_context.bounds = origin;
                    build_state.root_stacking_context.overflow = origin;

                    // We will not use build_state.iframe_sizes again, so it's safe to move it.
                    let iframe_sizes = std::mem::take(&mut build_state.iframe_sizes);
                    self.update_iframe_sizes(iframe_sizes);

                    *self.indexable_text.borrow_mut() =
                        std::mem::take(&mut build_state.indexable_text);
                    *self.display_list.borrow_mut() = Some(build_state.to_display_list());
                }

                if !reflow_goal.needs_display() {
                    // Defer the paint step until the next ForDisplay.
                    //
                    // We need to tell the document about this so it doesn't
                    // incorrectly suppress reflows. See #13131.
                    document
                        .expect("No document in a non-display reflow?")
                        .needs_paint_from_layout();
                    return;
                }
                if let Some(document) = document {
                    document.will_paint();
                }

                let mut display_list = self.display_list.borrow_mut();
                let display_list = display_list.as_mut().unwrap();
                if self.debug.dump_display_list {
                    display_list.print();
                }
                if self.debug.dump_display_list_json {
                    println!("{}", serde_json::to_string_pretty(&display_list).unwrap());
                }

                debug!("Layout done!");

                let viewport_size = units::LayoutSize::new(
                    self.viewport_size.width.to_f32_px(),
                    self.viewport_size.height.to_f32_px(),
                );

                let mut epoch = self.epoch.get();
                epoch.next();
                self.epoch.set(epoch);

                // TODO: Avoid the temporary conversion and build webrender sc/dl directly!
                let (mut builder, compositor_info, is_contentful) =
                    display_list.convert_to_webrender(self.id, viewport_size, epoch.into());

                // Observe notifications about rendered frames if needed right before
                // sending the display list to WebRender in order to set time related
                // Progressive Web Metrics.
                self.paint_time_metrics
                    .maybe_observe_paint_time(self, epoch, is_contentful.0);

                self.webrender_api
                    .send_display_list(compositor_info, builder.end().1);

                let (keys, instance_keys) = self
                    .font_context
                    .collect_unused_webrender_resources(false /* all */);
                self.webrender_api
                    .remove_unused_font_resources(keys, instance_keys)
            },
        );
    }

    /// The high-level routine that performs layout.
    fn handle_reflow(&mut self, data: &mut ScriptReflowResult) {
        let document = unsafe { ServoLayoutNode::new(&data.document) };
        let document = document.as_document().unwrap();

        // Parallelize if there's more than 750 objects based on rzambre's suggestion
        // https://github.com/servo/servo/issues/10110
        self.parallel_flag = data.dom_count > 750;
        debug!("layout: received layout request for: {}", self.url);
        debug!("Number of objects in DOM: {}", data.dom_count);
        debug!("layout: parallel? {}", self.parallel_flag);

        // Record the time that layout query has been waited.
        let now = time::precise_time_ns();
        if let ReflowGoal::LayoutQuery(_, timestamp) = data.reflow_goal {
            self.layout_query_waiting_time
                .increment(now - timestamp)
                .expect("layout: wrong layout query timestamp");
        };

        let Some(root_element) = document.root_element() else {
            debug!("layout: No root node: bailing");
            return;
        };

        debug!(
            "layout: processing reflow request for: {:?} ({}) (query={:?})",
            root_element, self.url, data.reflow_goal
        );
        trace!("{:?}", ShowSubtree(root_element.as_node()));

        // Calculate the actual viewport as per DEVICE-ADAPT § 6
        // If the entire flow tree is invalid, then it will be reflowed anyhow.
        let document_shared_lock = document.style_shared_lock();
        let author_guard = document_shared_lock.read();

        let ua_stylesheets = &*UA_STYLESHEETS;
        let ua_or_user_guard = ua_stylesheets.shared_lock.read();
        let guards = StylesheetGuards {
            author: &author_guard,
            ua_or_user: &ua_or_user_guard,
        };

        let had_used_viewport_units = self.stylist.device().used_viewport_units();
        let viewport_size_changed = self.handle_viewport_change(data.window_size, &guards);
        if viewport_size_changed && had_used_viewport_units {
            if let Some(mut data) = root_element.mutate_data() {
                data.hint.insert(RestyleHint::recascade_subtree());
            }
        }

        if self.first_reflow.get() {
            debug!("First reflow, rebuilding user and UA rules");
            for stylesheet in &ua_stylesheets.user_or_user_agent_stylesheets {
                self.stylist
                    .append_stylesheet(stylesheet.clone(), &ua_or_user_guard);
                self.load_all_web_fonts_from_stylesheet_with_guard(stylesheet, &ua_or_user_guard);
            }

            if self.stylist.quirks_mode() != QuirksMode::NoQuirks {
                self.stylist.append_stylesheet(
                    ua_stylesheets.quirks_mode_stylesheet.clone(),
                    &ua_or_user_guard,
                );
                self.load_all_web_fonts_from_stylesheet_with_guard(
                    &ua_stylesheets.quirks_mode_stylesheet,
                    &ua_or_user_guard,
                );
            }
        }

        if data.stylesheets_changed {
            debug!("Doc sheets changed, flushing author sheets too");
            self.stylist
                .force_stylesheet_origins_dirty(Origin::Author.into());
        }

        if viewport_size_changed {
            if let Some(mut flow) = self.try_get_layout_root(root_element.as_node()) {
                LayoutThread::reflow_all_nodes(FlowRef::deref_mut(&mut flow));
            }
        }

        debug!(
            "Shadow roots in document {:?}",
            document.shadow_roots().len()
        );

        // Flush shadow roots stylesheets if dirty.
        document.flush_shadow_roots_stylesheets(&mut self.stylist, guards.author);

        let restyles = std::mem::take(&mut data.pending_restyles);
        debug!("Draining restyles: {}", restyles.len());

        let mut map = SnapshotMap::new();
        let elements_with_snapshot: Vec<_> = restyles
            .iter()
            .filter(|r| r.1.snapshot.is_some())
            .map(|r| unsafe { ServoLayoutNode::new(&r.0).as_element().unwrap() })
            .collect();

        for (el, restyle) in restyles {
            let el = unsafe { ServoLayoutNode::new(&el).as_element().unwrap() };

            // If we haven't styled this node yet, we don't need to track a
            // restyle.
            let mut style_data = match el.mutate_data() {
                Some(d) => d,
                None => {
                    unsafe { el.unset_snapshot_flags() };
                    continue;
                },
            };

            if let Some(s) = restyle.snapshot {
                unsafe { el.set_has_snapshot() };
                map.insert(el.as_node().opaque(), s);
            }

            // Stash the data on the element for processing by the style system.
            style_data.hint.insert(restyle.hint);
            style_data.damage = restyle.damage;
            debug!("Noting restyle for {:?}: {:?}", el, style_data);
        }

        self.stylist.flush(&guards, Some(root_element), Some(&map));

        // Create a layout context for use throughout the following passes.
        let mut layout_context = self.build_layout_context(
            guards.clone(),
            &map,
            data.origin.clone(),
            data.animation_timeline_value,
            &data.animations,
            data.stylesheets_changed,
        );

        let pool = STYLE_THREAD_POOL.lock().unwrap();
        let thread_pool = pool.pool();
        let (thread_pool, num_threads) = if self.parallel_flag {
            (thread_pool.as_ref(), pool.num_threads.unwrap_or(1))
        } else {
            (None, 1)
        };

        let dirty_root = unsafe {
            ServoLayoutNode::new(&data.dirty_root.unwrap())
                .as_element()
                .unwrap()
        };

        let traversal = RecalcStyleAndConstructFlows::new(layout_context);
        let token = {
            let shared =
                <RecalcStyleAndConstructFlows as DomTraversal<ServoLayoutElement>>::shared_context(
                    &traversal,
                );
            RecalcStyleAndConstructFlows::pre_traverse(dirty_root, shared)
        };

        if token.should_traverse() {
            // Recalculate CSS styles and rebuild flows and fragments.
            profile(
                profile_time::ProfilerCategory::LayoutStyleRecalc,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
                    // Perform CSS selector matching and flow construction.
                    let root = driver::traverse_dom::<
                        ServoLayoutElement,
                        RecalcStyleAndConstructFlows,
                    >(&traversal, token, thread_pool);
                    unsafe {
                        construct_flows_at_ancestors(traversal.context(), root.as_node());
                    }
                },
            );
            // TODO(pcwalton): Measure energy usage of text shaping, perhaps?
            let text_shaping_time = get_and_reset_text_shaping_performance_counter() / num_threads;
            profile_time::send_profile_data(
                profile_time::ProfilerCategory::LayoutTextShaping,
                self.profiler_metadata(),
                &self.time_profiler_chan,
                0,
                text_shaping_time as u64,
            );

            // Retrieve the (possibly rebuilt) root flow.
            *self.root_flow.borrow_mut() = self.try_get_layout_root(root_element.as_node());
        }

        for element in elements_with_snapshot {
            unsafe { element.unset_snapshot_flags() }
        }

        layout_context = traversal.destroy();

        if self.debug.dump_style_tree {
            println!(
                "{:?}",
                ShowSubtreeDataAndPrimaryValues(root_element.as_node())
            );
        }

        if self.debug.dump_rule_tree {
            layout_context
                .style_context
                .stylist
                .rule_tree()
                .dump_stdout(&guards);
        }

        // GC the rule tree if some heuristics are met.
        layout_context.style_context.stylist.rule_tree().maybe_gc();

        // Perform post-style recalculation layout passes.
        if let Some(mut root_flow) = self.root_flow.borrow().clone() {
            self.perform_post_style_recalc_layout_passes(
                &mut root_flow,
                &data.reflow_info,
                &data.reflow_goal,
                Some(&document),
                &mut layout_context,
                thread_pool,
            );
        }

        self.first_reflow.set(false);

        data.result.borrow_mut().as_mut().unwrap().pending_images =
            std::mem::take(&mut *layout_context.pending_images.lock().unwrap());

        if let ReflowGoal::UpdateScrollNode(scroll_state) = data.reflow_goal {
            self.update_scroll_node_state(&scroll_state);
        }
    }

    fn update_scroll_node_state(&self, state: &ScrollState) {
        self.scroll_offsets
            .borrow_mut()
            .insert(state.scroll_id, state.scroll_offset);

        let point = Point2D::new(-state.scroll_offset.x, -state.scroll_offset.y);
        self.webrender_api.send_scroll_node(
            self.id.into(),
            units::LayoutPoint::from_untyped(point),
            state.scroll_id,
        );
    }

    /// Cancel animations for any nodes which have been removed from flow tree.
    /// TODO(mrobinson): We should look into a way of doing this during flow tree construction.
    /// This also doesn't yet handles nodes that have been reparented.
    fn cancel_animations_for_nodes_not_in_flow_tree(
        animations: &mut FxHashMap<AnimationSetKey, ElementAnimationSet>,
        root_flow: &mut dyn Flow,
    ) {
        // Assume all nodes have been removed until proven otherwise.
        let mut invalid_nodes = animations.keys().cloned().collect();

        fn traverse_flow(flow: &mut dyn Flow, invalid_nodes: &mut FxHashSet<AnimationSetKey>) {
            flow.mutate_fragments(&mut |fragment| {
                // Ideally we'd only not cancel ::before and ::after animations if they
                // were actually in the tree. At this point layout has lost information
                // about whether or not they exist, but have had their fragments accumulated
                // together.
                invalid_nodes.remove(&AnimationSetKey::new_for_non_pseudo(fragment.node));
                invalid_nodes.remove(&AnimationSetKey::new_for_pseudo(
                    fragment.node,
                    PseudoElement::Before,
                ));
                invalid_nodes.remove(&AnimationSetKey::new_for_pseudo(
                    fragment.node,
                    PseudoElement::After,
                ));
            });
            for kid in flow.mut_base().children.iter_mut() {
                traverse_flow(kid, invalid_nodes)
            }
        }

        traverse_flow(root_flow, &mut invalid_nodes);

        // Cancel animations for any nodes that are no longer in the flow tree.
        for node in &invalid_nodes {
            if let Some(state) = animations.get_mut(node) {
                state.cancel_all_animations();
            }
        }
    }

    fn perform_post_style_recalc_layout_passes(
        &self,
        root_flow: &mut FlowRef,
        data: &Reflow,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument>,
        context: &mut LayoutContext,
        thread_pool: Option<&rayon::ThreadPool>,
    ) {
        Self::cancel_animations_for_nodes_not_in_flow_tree(
            &mut (context.style_context.animations.sets.write()),
            FlowRef::deref_mut(root_flow),
        );

        profile(
            profile_time::ProfilerCategory::LayoutRestyleDamagePropagation,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || {
                // Call `compute_layout_damage` even in non-incremental mode, because it sets flags
                // that are needed in both incremental and non-incremental traversals.
                let damage = FlowRef::deref_mut(root_flow).compute_layout_damage();

                if self.nonincremental_layout ||
                    damage.contains(SpecialRestyleDamage::REFLOW_ENTIRE_DOCUMENT)
                {
                    FlowRef::deref_mut(root_flow).reflow_entire_document()
                }
            },
        );

        if self.debug.trace_layout {
            layout_debug::begin_trace(root_flow.clone());
        }

        // Resolve generated content.
        profile(
            profile_time::ProfilerCategory::LayoutGeneratedContent,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || sequential::resolve_generated_content(FlowRef::deref_mut(root_flow), context),
        );

        // Guess float placement.
        profile(
            profile_time::ProfilerCategory::LayoutFloatPlacementSpeculation,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || sequential::guess_float_placement(FlowRef::deref_mut(root_flow)),
        );

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        if root_flow
            .base()
            .restyle_damage
            .intersects(ServoRestyleDamage::REFLOW | ServoRestyleDamage::REFLOW_OUT_OF_FLOW)
        {
            profile(
                profile_time::ProfilerCategory::LayoutMain,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
                    let profiler_metadata = self.profiler_metadata();

                    if let Some(pool) = thread_pool {
                        // Parallel mode.
                        LayoutThread::solve_constraints_parallel(
                            pool,
                            FlowRef::deref_mut(root_flow),
                            profiler_metadata,
                            self.time_profiler_chan.clone(),
                            &*context,
                        );
                    } else {
                        //Sequential mode
                        LayoutThread::solve_constraints(FlowRef::deref_mut(root_flow), context)
                    }
                },
            );
        }

        profile(
            profile_time::ProfilerCategory::LayoutStoreOverflow,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || {
                sequential::store_overflow(context, FlowRef::deref_mut(root_flow) as &mut dyn Flow);
            },
        );

        self.perform_post_main_layout_passes(data, root_flow, reflow_goal, document, context);
    }

    fn perform_post_main_layout_passes(
        &self,
        data: &Reflow,
        root_flow: &mut FlowRef,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument>,
        layout_context: &mut LayoutContext,
    ) {
        // Build the display list if necessary, and send it to the painter.
        self.compute_abs_pos_and_build_display_list(
            data,
            reflow_goal,
            document,
            FlowRef::deref_mut(root_flow),
            &mut *layout_context,
        );

        if self.debug.trace_layout {
            layout_debug::end_trace(self.generation.get());
        }

        if self.debug.dump_flow_tree {
            root_flow.print("Post layout flow tree".to_owned());
        }

        self.generation.set(self.generation.get() + 1);
    }

    fn reflow_all_nodes(flow: &mut dyn Flow) {
        debug!("reflowing all nodes!");
        flow.mut_base().restyle_damage.insert(
            ServoRestyleDamage::REPAINT |
                ServoRestyleDamage::STORE_OVERFLOW |
                ServoRestyleDamage::REFLOW |
                ServoRestyleDamage::REPOSITION,
        );

        for child in flow.mut_base().child_iter_mut() {
            LayoutThread::reflow_all_nodes(child);
        }
    }

    /// Returns profiling information which is passed to the time profiler.
    fn profiler_metadata(&self) -> Option<TimerMetadata> {
        Some(TimerMetadata {
            url: self.url.to_string(),
            iframe: if self.is_iframe {
                TimerMetadataFrameType::IFrame
            } else {
                TimerMetadataFrameType::RootWindow
            },
            incremental: if self.first_reflow.get() {
                TimerMetadataReflowType::FirstReflow
            } else {
                TimerMetadataReflowType::Incremental
            },
        })
    }

    /// Update layout given a new viewport. Returns true if the viewport changed or false if it didn't.
    fn handle_viewport_change(
        &mut self,
        window_size_data: WindowSizeData,
        guards: &StylesheetGuards,
    ) -> bool {
        // If the viewport size and device pixel ratio has not changed, do not make any changes.
        let au_viewport_size = Size2D::new(
            Au::from_f32_px(window_size_data.initial_viewport.width),
            Au::from_f32_px(window_size_data.initial_viewport.height),
        );

        if self.stylist.device().au_viewport_size() == au_viewport_size &&
            self.stylist.device().device_pixel_ratio() == window_size_data.device_pixel_ratio
        {
            return false;
        }

        let device = Device::new(
            MediaType::screen(),
            self.stylist.quirks_mode(),
            window_size_data.initial_viewport,
            window_size_data.device_pixel_ratio,
            Box::new(LayoutFontMetricsProvider),
            self.stylist.device().default_computed_values().to_arc(),
        );

        // Preserve any previously computed root font size.
        device.set_root_font_size(self.stylist.device().root_font_size().px());

        let sheet_origins_affected_by_device_change = self.stylist.set_device(device, guards);
        self.stylist
            .force_stylesheet_origins_dirty(sheet_origins_affected_by_device_change);

        self.viewport_size = au_viewport_size;
        true
    }
}

impl ProfilerMetadataFactory for LayoutThread {
    fn new_metadata(&self) -> Option<TimerMetadata> {
        self.profiler_metadata()
    }
}

// The default computed value for background-color is transparent (see
// http://dev.w3.org/csswg/css-backgrounds/#background-color). However, we
// need to propagate the background color from the root HTML/Body
// element (http://dev.w3.org/csswg/css-backgrounds/#special-backgrounds) if
// it is non-transparent. The phrase in the spec "If the canvas background
// is not opaque, what shows through is UA-dependent." is handled by rust-layers
// clearing the frame buffer to white. This ensures that setting a background
// color on an iframe element, while the iframe content itself has a default
// transparent background color is handled correctly.
fn get_root_flow_background_color(flow: &mut dyn Flow) -> ColorF {
    let transparent = ColorF {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    if !flow.is_block_like() {
        return transparent;
    }

    let block_flow = flow.as_mut_block();
    let kid = match block_flow.base.children.iter_mut().next() {
        None => return transparent,
        Some(kid) => kid,
    };
    if !kid.is_block_like() {
        return transparent;
    }

    let kid_block_flow = kid.as_block();
    let color = kid_block_flow.fragment.style.resolve_color(
        kid_block_flow
            .fragment
            .style
            .get_background()
            .background_color
            .clone(),
    );
    color.to_layout()
}

fn get_ua_stylesheets() -> Result<UserAgentStylesheets, &'static str> {
    fn parse_ua_stylesheet(
        shared_lock: &SharedRwLock,
        filename: &str,
        content: &[u8],
    ) -> Result<DocumentStyleSheet, &'static str> {
        let url = Url::parse(&format!("chrome://resources/{:?}", filename))
            .ok()
            .unwrap();
        Ok(DocumentStyleSheet(ServoArc::new(Stylesheet::from_bytes(
            content,
            url.into(),
            None,
            None,
            Origin::UserAgent,
            MediaList::empty(),
            shared_lock.clone(),
            None,
            None,
            QuirksMode::NoQuirks,
        ))))
    }

    let shared_lock = &GLOBAL_STYLE_DATA.shared_lock;

    // FIXME: presentational-hints.css should be at author origin with zero specificity.
    //        (Does it make a difference?)
    let mut user_or_user_agent_stylesheets = vec![
        parse_ua_stylesheet(
            shared_lock,
            "user-agent.css",
            &resources::read_bytes(Resource::UserAgentCSS),
        )?,
        parse_ua_stylesheet(
            shared_lock,
            "servo.css",
            &resources::read_bytes(Resource::ServoCSS),
        )?,
        parse_ua_stylesheet(
            shared_lock,
            "presentational-hints.css",
            &resources::read_bytes(Resource::PresentationalHintsCSS),
        )?,
    ];

    for (contents, url) in &opts::get().user_stylesheets {
        user_or_user_agent_stylesheets.push(DocumentStyleSheet(ServoArc::new(
            Stylesheet::from_bytes(
                contents,
                UrlExtraData(url.get_arc()),
                None,
                None,
                Origin::User,
                MediaList::empty(),
                shared_lock.clone(),
                None,
                Some(&RustLogReporter),
                QuirksMode::NoQuirks,
            ),
        )));
    }

    let quirks_mode_stylesheet = parse_ua_stylesheet(
        shared_lock,
        "quirks-mode.css",
        &resources::read_bytes(Resource::QuirksModeCSS),
    )?;

    Ok(UserAgentStylesheets {
        shared_lock: shared_lock.clone(),
        user_or_user_agent_stylesheets,
        quirks_mode_stylesheet,
    })
}

lazy_static! {
    static ref UA_STYLESHEETS: UserAgentStylesheets = {
        match get_ua_stylesheets() {
            Ok(stylesheets) => stylesheets,
            Err(filename) => {
                error!("Failed to load UA stylesheet {}!", filename);
                process::exit(1);
            },
        }
    };
}

struct RegisteredPainterImpl {
    painter: Box<dyn Painter>,
    name: Atom,
    // FIXME: Should be a PrecomputedHashMap.
    properties: FxHashMap<Atom, PropertyId>,
}

impl SpeculativePainter for RegisteredPainterImpl {
    fn speculatively_draw_a_paint_image(
        &self,
        properties: Vec<(Atom, String)>,
        arguments: Vec<String>,
    ) {
        self.painter
            .speculatively_draw_a_paint_image(properties, arguments);
    }
}

impl RegisteredSpeculativePainter for RegisteredPainterImpl {
    fn properties(&self) -> &FxHashMap<Atom, PropertyId> {
        &self.properties
    }
    fn name(&self) -> Atom {
        self.name.clone()
    }
}

impl Painter for RegisteredPainterImpl {
    fn draw_a_paint_image(
        &self,
        size: Size2D<f32, CSSPixel>,
        device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
        properties: Vec<(Atom, String)>,
        arguments: Vec<String>,
    ) -> Result<DrawAPaintImageResult, PaintWorkletError> {
        self.painter
            .draw_a_paint_image(size, device_pixel_ratio, properties, arguments)
    }
}

impl RegisteredPainter for RegisteredPainterImpl {}

struct RegisteredPaintersImpl(FnvHashMap<Atom, RegisteredPainterImpl>);

impl RegisteredSpeculativePainters for RegisteredPaintersImpl {
    fn get(&self, name: &Atom) -> Option<&dyn RegisteredSpeculativePainter> {
        self.0
            .get(name)
            .map(|painter| painter as &dyn RegisteredSpeculativePainter)
    }
}

impl RegisteredPainters for RegisteredPaintersImpl {
    fn get(&self, name: &Atom) -> Option<&dyn RegisteredPainter> {
        self.0
            .get(name)
            .map(|painter| painter as &dyn RegisteredPainter)
    }
}

#[derive(Debug)]
struct LayoutFontMetricsProvider;

impl FontMetricsProvider for LayoutFontMetricsProvider {
    fn query_font_metrics(
        &self,
        _vertical: bool,
        _font: &Font,
        _base_size: style::values::computed::CSSPixelLength,
        _in_media_query: bool,
        _retrieve_math_scales: bool,
    ) -> style::font_metrics::FontMetrics {
        Default::default()
    }

    fn base_size_for_generic(&self, generic: GenericFontFamily) -> Length {
        Length::new(match generic {
            GenericFontFamily::Monospace => pref!(fonts.default_monospace_size),
            _ => pref!(fonts.default_size),
        } as f32)
        .max(Length::new(0.0))
    }
}
