/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Work around https://github.com/rust-lang/rust/issues/62132
#![recursion_limit = "128"]

//! Layout. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::process;
use std::sync::Arc;

use app_units::Au;
use base::id::{BrowsingContextId, PipelineId};
use base::Epoch;
use embedder_traits::resources::{self, Resource};
use euclid::default::{Point2D as UntypedPoint2D, Rect as UntypedRect, Size2D as UntypedSize2D};
use euclid::{Point2D, Scale, Size2D, Vector2D};
use fnv::FnvHashMap;
use fonts::{FontCacheThread, FontContext, FontContextWebFontMethods};
use fonts_traits::WebFontLoadFinishedCallback;
use fxhash::FxHashMap;
use ipc_channel::ipc::IpcSender;
use layout::context::LayoutContext;
use layout::display_list::{DisplayList, WebRenderImageInfo};
use layout::query::{
    process_content_box_request, process_content_boxes_request, process_element_inner_text_query,
    process_node_geometry_request, process_node_scroll_area_request, process_offset_parent_query,
    process_resolved_font_style_query, process_resolved_style_request, process_text_index_request,
};
use layout::traversal::RecalcStyle;
use layout::{layout_debug, BoxTree, FragmentTree};
use lazy_static::lazy_static;
use log::{debug, error, warn};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory};
use net_traits::image_cache::{ImageCache, UsePlaceholder};
use net_traits::ResourceThreads;
use parking_lot::{Mutex, RwLock};
use profile_traits::mem::{Report, ReportKind};
use profile_traits::path;
use profile_traits::time::{
    self as profile_time, profile, TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType,
};
use script::layout_dom::{ServoLayoutDocument, ServoLayoutElement, ServoLayoutNode};
use script_layout_interface::{
    Layout, LayoutConfig, LayoutFactory, NodesFromPointQueryType, OffsetParentResponse,
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
use style::animation::DocumentAnimationSet;
use style::context::{
    QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters, SharedStyleContext,
};
use style::dom::{OpaqueNode, TElement, TNode};
use style::error_reporting::RustLogReporter;
use style::font_metrics::FontMetrics;
use style::global_style_data::{GLOBAL_STYLE_DATA, STYLE_THREAD_POOL};
use style::invalidation::element::restyle_hints::RestyleHint;
use style::media_queries::{Device, MediaList, MediaType};
use style::properties::style_structs::Font;
use style::properties::{ComputedValues, PropertyId};
use style::selector_parser::{PseudoElement, SnapshotMap};
use style::servo::media_queries::FontMetricsProvider;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards};
use style::stylesheets::{
    DocumentStyleSheet, Origin, Stylesheet, StylesheetInDocument, UrlExtraData,
    UserAgentStylesheets,
};
use style::stylist::Stylist;
use style::traversal::DomTraversal;
use style::traversal_flags::TraversalFlags;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{CSSPixelLength, FontSize, Length, NonNegativeLength};
use style::values::specified::font::KeywordInfo;
use style::{driver, Zero};
use style_traits::{CSSPixel, DevicePixel, SpeculativePainter};
use url::Url;
use webrender_api::units::LayoutPixel;
use webrender_api::{units, ExternalScrollId, HitTestFlags};
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

    /// A FontContext to be used during layout.
    font_context: Arc<FontContext<FontCacheThread>>,

    /// Is this the first reflow in this LayoutThread?
    first_reflow: Cell<bool>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    generation: Cell<u32>,

    /// The box tree.
    box_tree: RefCell<Option<Arc<BoxTree>>>,

    /// The fragment tree.
    fragment_tree: RefCell<Option<Arc<FragmentTree>>>,

    /// A counter for epoch messages
    epoch: Cell<Epoch>,

    /// The size of the viewport. This may be different from the size of the screen due to viewport
    /// constraints.
    viewport_size: UntypedSize2D<Au>,

    /// Scroll offsets of nodes that scroll.
    scroll_offsets: RefCell<HashMap<ExternalScrollId, Vector2D<f32, LayoutPixel>>>,

    webrender_image_cache: Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), WebRenderImageInfo>>>,

    /// The executors for paint worklets.
    registered_painters: RegisteredPaintersImpl,

    /// Webrender interface.
    webrender_api: WebRenderScriptApi,

    /// Paint time metrics.
    paint_time_metrics: PaintTimeMetrics,

    /// The sizes of all iframes encountered during the last layout operation.
    last_iframe_sizes: RefCell<FnvHashMap<BrowsingContextId, Size2D<f32, CSSPixel>>>,

    /// Debug options, copied from configuration to this `LayoutThread` in order
    /// to avoid having to constantly access the thread-safe global options.
    debug: DebugOptions,
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
        process_content_box_request(node, self.fragment_tree.borrow().clone())
    }

    fn query_content_boxes(&self, node: OpaqueNode) -> Vec<UntypedRect<Au>> {
        process_content_boxes_request(node, self.fragment_tree.borrow().clone())
    }

    fn query_client_rect(&self, node: OpaqueNode) -> UntypedRect<i32> {
        process_node_geometry_request(node, self.fragment_tree.borrow().clone())
    }

    fn query_element_inner_text(
        &self,
        node: script_layout_interface::TrustedNodeAddress,
    ) -> String {
        let node = unsafe { ServoLayoutNode::new(&node) };
        process_element_inner_text_query(node)
    }

    fn query_inner_window_dimension(
        &self,
        _context: BrowsingContextId,
    ) -> Option<Size2D<f32, CSSPixel>> {
        // TODO(jdm): port the iframe sizing code from layout2013's display
        //            builder in order to support query iframe sizing.
        None
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
        process_offset_parent_query(node, self.fragment_tree.borrow().clone())
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

        let fragment_tree = self.fragment_tree.borrow().clone();
        process_resolved_style_request(
            &shared_style_context,
            node,
            &pseudo,
            &property_id,
            fragment_tree,
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

        process_resolved_font_style_query(
            &shared_style_context,
            node,
            value,
            self.url.clone(),
            document_shared_lock,
        )
    }

    fn query_scrolling_area(&self, node: Option<OpaqueNode>) -> UntypedRect<i32> {
        process_node_scroll_area_request(node, self.fragment_tree.borrow().clone())
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
        process_text_index_request(node, point_in_node)
    }

    fn exit_now(&mut self) {}

    fn collect_reports(&self, reports: &mut Vec<Report>) {
        // Servo uses vanilla jemalloc, which doesn't have a
        // malloc_enclosing_size_of function.
        let mut ops = MallocSizeOfOps::new(servo_allocator::usable_size, None, None);

        // TODO: Measure more than just display list, stylist, and font context.
        let formatted_url = &format!("url({})", self.url);
        reports.push(Report {
            path: path![formatted_url, "layout-thread", "display-list"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: 0,
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

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.stylist.set_quirks_mode(quirks_mode);
    }

    fn reflow(&mut self, script_reflow: ScriptReflow) {
        let mut result = ScriptReflowResult::new(script_reflow);
        profile(
            profile_time::ProfilerCategory::LayoutPerform,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || self.handle_reflow(&mut result),
        );
    }

    fn register_paint_worklet_modules(
        &mut self,
        _name: Atom,
        _properties: Vec<Atom>,
        _painter: Box<dyn Painter>,
    ) {
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
        webrender_api_sender: WebRenderScriptApi,
        paint_time_metrics: PaintTimeMetrics,
        window_size: WindowSizeData,
    ) -> LayoutThread {
        // Let webrender know about this pipeline by sending an empty display list.
        webrender_api_sender.send_initial_transaction(id.into());

        let mut font = Font::initial_values();
        let default_font_size = pref!(fonts.default_size);
        font.font_size = FontSize {
            computed_size: NonNegativeLength::new(default_font_size as f32),
            used_size: NonNegativeLength::new(default_font_size as f32),
            keyword_info: KeywordInfo::medium(),
        };

        // The device pixel ratio is incorrect (it does not have the hidpi value),
        // but it will be set correctly when the initial reflow takes place.
        let font_context = Arc::new(FontContext::new(font_cache_thread, resource_threads));
        let device = Device::new(
            MediaType::screen(),
            QuirksMode::NoQuirks,
            window_size.initial_viewport,
            window_size.device_pixel_ratio,
            Box::new(LayoutFontMetricsProvider(font_context.clone())),
            ComputedValues::initial_values_with_font_override(font),
        );

        LayoutThread {
            id,
            url,
            is_iframe,
            constellation_chan,
            script_chan: script_chan.clone(),
            time_profiler_chan,
            registered_painters: RegisteredPaintersImpl(Default::default()),
            image_cache,
            font_context,
            first_reflow: Cell::new(true),
            generation: Cell::new(0),
            box_tree: Default::default(),
            fragment_tree: Default::default(),
            // Epoch starts at 1 because of the initial display list for epoch 0 that we send to WR
            epoch: Cell::new(Epoch(1)),
            viewport_size: Size2D::new(
                Au::from_f32_px(window_size.initial_viewport.width),
                Au::from_f32_px(window_size.initial_viewport.height),
            ),
            webrender_api: webrender_api_sender,
            scroll_offsets: Default::default(),
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            webrender_image_cache: Default::default(),
            paint_time_metrics,
            last_iframe_sizes: Default::default(),
            debug: opts::get().debug.clone(),
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
        use_rayon: bool,
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
            use_rayon,
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
        let web_font_finished_loading_callback = move |succeeded: bool| {
            let _ = locked_script_channel
                .lock()
                .send(ConstellationControlMsg::WebFontLoaded(
                    pipeline_id,
                    succeeded,
                ));
        };

        // Find all font-face rules and notify the font cache of them.
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

    /// The high-level routine that performs layout.
    fn handle_reflow(&mut self, data: &mut ScriptReflowResult) {
        let document = unsafe { ServoLayoutNode::new(&data.document) };
        let document = document.as_document().unwrap();
        let Some(root_element) = document.root_element() else {
            debug!("layout: No root node: bailing");
            return;
        };

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
            self.stylist
                .force_stylesheet_origins_dirty(Origin::Author.into());
        }

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

        let rayon_pool = STYLE_THREAD_POOL.lock().unwrap();
        let rayon_pool = rayon_pool.pool();
        let rayon_pool = rayon_pool.as_ref();

        // Create a layout context for use throughout the following passes.
        let mut layout_context = self.build_layout_context(
            guards.clone(),
            &map,
            data.origin.clone(),
            data.animation_timeline_value,
            &data.animations,
            data.stylesheets_changed,
            rayon_pool.is_some(),
        );

        let dirty_root = unsafe {
            ServoLayoutNode::new(&data.dirty_root.unwrap())
                .as_element()
                .unwrap()
        };

        let traversal = RecalcStyle::new(layout_context);
        let token = {
            let shared = DomTraversal::<ServoLayoutElement>::shared_context(&traversal);
            RecalcStyle::pre_traverse(dirty_root, shared)
        };

        if token.should_traverse() {
            let dirty_root: ServoLayoutNode =
                driver::traverse_dom(&traversal, token, rayon_pool).as_node();

            let root_node = root_element.as_node();
            let mut box_tree = self.box_tree.borrow_mut();
            let box_tree = &mut *box_tree;
            let mut build_box_tree = || {
                if !BoxTree::update(traversal.context(), dirty_root) {
                    *box_tree = Some(Arc::new(BoxTree::construct(traversal.context(), root_node)));
                }
            };
            if let Some(pool) = rayon_pool {
                pool.install(build_box_tree)
            } else {
                build_box_tree()
            };

            let viewport_size = Size2D::new(
                self.viewport_size.width.to_f32_px(),
                self.viewport_size.height.to_f32_px(),
            );
            let run_layout = || {
                box_tree
                    .as_ref()
                    .unwrap()
                    .layout(traversal.context(), viewport_size)
            };
            let fragment_tree = Arc::new(if let Some(pool) = rayon_pool {
                pool.install(run_layout)
            } else {
                run_layout()
            });
            *self.fragment_tree.borrow_mut() = Some(fragment_tree);
        }

        layout_context = traversal.destroy();

        for element in elements_with_snapshot {
            unsafe { element.unset_snapshot_flags() }
        }

        if self.debug.dump_style_tree {
            println!(
                "{:?}",
                style::dom::ShowSubtreeDataAndPrimaryValues(root_element.as_node())
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
        if let Some(root) = &*self.fragment_tree.borrow() {
            self.perform_post_style_recalc_layout_passes(
                root.clone(),
                &data.reflow_goal,
                Some(&document),
                &mut layout_context,
            );
        }

        self.first_reflow.set(false);

        data.result.borrow_mut().as_mut().unwrap().pending_images =
            std::mem::take(&mut *layout_context.pending_images.lock());
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

    fn perform_post_style_recalc_layout_passes(
        &self,
        fragment_tree: Arc<FragmentTree>,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument>,
        context: &mut LayoutContext,
    ) {
        Self::cancel_animations_for_nodes_not_in_fragment_tree(
            &context.style_context.animations,
            &fragment_tree,
        );

        if self.debug.trace_layout {
            if let Some(box_tree) = &*self.box_tree.borrow() {
                layout_debug::begin_trace(box_tree.clone(), fragment_tree.clone());
            }
        }

        if !reflow_goal.needs_display_list() {
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

        let mut epoch = self.epoch.get();
        epoch.next();
        self.epoch.set(epoch);

        let viewport_size = units::LayoutSize::from_untyped(Size2D::new(
            self.viewport_size.width.to_f32_px(),
            self.viewport_size.height.to_f32_px(),
        ));
        let mut display_list = DisplayList::new(
            viewport_size,
            fragment_tree.scrollable_overflow(),
            self.id.into(),
            epoch.into(),
            fragment_tree.root_scroll_sensitivity,
        );
        display_list.wr.begin();

        // `dump_serialized_display_list` doesn't actually print anything. It sets up
        // the display list for printing the serialized version when `finalize()` is called.
        // We need to call this before adding any display items so that they are printed
        // during `finalize()`.
        if self.debug.dump_display_list {
            display_list.wr.dump_serialized_display_list();
        }

        // Build the root stacking context. This turns the `FragmentTree` into a
        // tree of fragments in CSS painting order and also creates all
        // applicable spatial and clip nodes.
        let root_stacking_context =
            display_list.build_stacking_context_tree(&fragment_tree, &self.debug);

        // Build the rest of the display list which inclues all of the WebRender primitives.
        let (iframe_sizes, is_contentful) =
            display_list.build(context, &fragment_tree, &root_stacking_context);

        if self.debug.dump_flow_tree {
            fragment_tree.print();
        }
        if self.debug.dump_stacking_context_tree {
            root_stacking_context.debug_print();
        }
        debug!("Layout done!");

        // Observe notifications about rendered frames if needed right before
        // sending the display list to WebRender in order to set time related
        // Progressive Web Metrics.
        self.paint_time_metrics
            .maybe_observe_paint_time(self, epoch, is_contentful);

        if reflow_goal.needs_display() {
            self.webrender_api
                .send_display_list(display_list.compositor_info, display_list.wr.end().1);

            let (keys, instance_keys) = self
                .font_context
                .collect_unused_webrender_resources(false /* all */);
            self.webrender_api
                .remove_unused_font_resources(keys, instance_keys)
        }

        self.update_iframe_sizes(iframe_sizes);

        if self.debug.trace_layout {
            layout_debug::end_trace(self.generation.get());
        }

        self.generation.set(self.generation.get() + 1);
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

    /// Cancel animations for any nodes which have been removed from fragment tree.
    /// TODO(mrobinson): We should look into a way of doing this during flow tree construction.
    /// This also doesn't yet handles nodes that have been reparented.
    fn cancel_animations_for_nodes_not_in_fragment_tree(
        animations: &DocumentAnimationSet,
        root: &FragmentTree,
    ) {
        // Assume all nodes have been removed until proven otherwise.
        let mut animations = animations.sets.write();
        let mut invalid_nodes = animations.keys().cloned().collect();
        root.remove_nodes_in_fragment_tree_from_set(&mut invalid_nodes);

        // Cancel animations for any nodes that are no longer in the fragment tree.
        for node in &invalid_nodes {
            if let Some(state) = animations.get_mut(node) {
                state.cancel_all_animations();
            }
        }
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
            Box::new(LayoutFontMetricsProvider(self.font_context.clone())),
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

struct RegisteredPaintersImpl(FnvHashMap<Atom, RegisteredPainterImpl>);

impl RegisteredSpeculativePainters for RegisteredPaintersImpl {
    fn get(&self, name: &Atom) -> Option<&dyn RegisteredSpeculativePainter> {
        self.0
            .get(name)
            .map(|painter| painter as &dyn RegisteredSpeculativePainter)
    }
}

struct LayoutFontMetricsProvider(Arc<FontContext<FontCacheThread>>);

impl FontMetricsProvider for LayoutFontMetricsProvider {
    fn query_font_metrics(
        &self,
        _vertical: bool,
        font: &Font,
        base_size: CSSPixelLength,
        _in_media_query: bool,
        _retrieve_math_scales: bool,
    ) -> FontMetrics {
        let font_context = &self.0;
        let font_group = self
            .0
            .font_group_with_size(ServoArc::new(font.clone()), base_size.into());

        let Some(first_font_metrics) = font_group
            .write()
            .first(font_context)
            .map(|font| font.metrics.clone())
        else {
            return Default::default();
        };

        // Only use the x-height of this font if it is non-zero. Some fonts return
        // inaccurate metrics, which shouldn't be used.
        let x_height = Some(first_font_metrics.x_height)
            .filter(|x_height| !x_height.is_zero())
            .map(CSSPixelLength::from);

        let zero_advance_measure = first_font_metrics
            .zero_horizontal_advance
            .or_else(|| {
                font_group
                    .write()
                    .find_by_codepoint(font_context, '0', None)?
                    .metrics
                    .zero_horizontal_advance
            })
            .map(CSSPixelLength::from);

        let ic_width = first_font_metrics
            .ic_horizontal_advance
            .or_else(|| {
                font_group
                    .write()
                    .find_by_codepoint(font_context, '\u{6C34}', None)?
                    .metrics
                    .ic_horizontal_advance
            })
            .map(CSSPixelLength::from);

        FontMetrics {
            x_height,
            zero_advance_measure,
            cap_height: None,
            ic_width,
            ascent: first_font_metrics.ascent.into(),
            script_percent_scale_down: None,
            script_script_percent_scale_down: None,
        }
    }

    fn base_size_for_generic(&self, generic: GenericFontFamily) -> Length {
        Length::new(match generic {
            GenericFontFamily::Monospace => pref!(fonts.default_monospace_size),
            _ => pref!(fonts.default_size),
        } as f32)
        .max(Length::new(0.0))
    }
}

impl Debug for LayoutFontMetricsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LayoutFontMetricsProvider").finish()
    }
}
