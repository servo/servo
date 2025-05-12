/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::process;
use std::sync::{Arc, LazyLock};

use app_units::Au;
use base::Epoch;
use base::id::{PipelineId, WebViewId};
use compositing_traits::CrossProcessCompositorApi;
use constellation_traits::ScrollState;
use embedder_traits::{UntrustedNodeAddress, ViewportDetails};
use euclid::default::{Point2D as UntypedPoint2D, Rect as UntypedRect, Size2D as UntypedSize2D};
use euclid::{Point2D, Scale, Size2D, Vector2D};
use fnv::FnvHashMap;
use fonts::{FontContext, FontContextWebFontMethods};
use fonts_traits::StylesheetWebFontLoadFinishedCallback;
use fxhash::FxHashMap;
use ipc_channel::ipc::IpcSender;
use log::{debug, error};
use malloc_size_of::{MallocConditionalSizeOf, MallocSizeOf, MallocSizeOfOps};
use net_traits::image_cache::{ImageCache, UsePlaceholder};
use parking_lot::{Mutex, RwLock};
use profile_traits::mem::{Report, ReportKind};
use profile_traits::time::{
    self as profile_time, TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType,
};
use profile_traits::{path, time_profile};
use rayon::ThreadPool;
use script::layout_dom::{ServoLayoutDocument, ServoLayoutElement, ServoLayoutNode};
use script_layout_interface::{
    Layout, LayoutConfig, LayoutFactory, NodesFromPointQueryType, OffsetParentResponse, ReflowGoal,
    ReflowRequest, ReflowResult, TrustedNodeAddress,
};
use script_traits::{DrawAPaintImageResult, PaintWorkletError, Painter, ScriptThreadMessage};
use servo_arc::Arc as ServoArc;
use servo_config::opts::{self, DebugOptions};
use servo_config::pref;
use servo_url::ServoUrl;
use style::animation::DocumentAnimationSet;
use style::context::{
    QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters, SharedStyleContext,
};
use style::dom::{OpaqueNode, ShowSubtreeDataAndPrimaryValues, TElement, TNode};
use style::error_reporting::RustLogReporter;
use style::font_metrics::FontMetrics;
use style::global_style_data::GLOBAL_STYLE_DATA;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::media_queries::{Device, MediaList, MediaType};
use style::properties::style_structs::Font;
use style::properties::{ComputedValues, PropertyId};
use style::queries::values::PrefersColorScheme;
use style::selector_parser::{PseudoElement, RestyleDamage, SnapshotMap};
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
use style::values::specified::font::{KeywordInfo, QueryFontMetricsFlags};
use style::{Zero, driver};
use style_traits::{CSSPixel, SpeculativePainter};
use stylo_atoms::Atom;
use url::Url;
use webrender_api::units::{DevicePixel, DevicePoint, LayoutPixel, LayoutPoint, LayoutSize};
use webrender_api::{ExternalScrollId, HitTestFlags};

use crate::context::LayoutContext;
use crate::display_list::{DisplayList, WebRenderImageInfo};
use crate::query::{
    get_the_text_steps, process_client_rect_request, process_content_box_request,
    process_content_boxes_request, process_node_scroll_area_request, process_offset_parent_query,
    process_resolved_font_style_query, process_resolved_style_request, process_text_index_request,
};
use crate::traversal::{RecalcStyle, compute_damage_and_repair_style};
use crate::{BoxTree, FragmentTree};

// This mutex is necessary due to syncronisation issues between two different types of thread-local storage
// which manifest themselves when the layout thread tries to layout iframes in parallel with the main page
//
// See: https://github.com/servo/servo/pull/29792
// And: https://gist.github.com/mukilan/ed57eb61b83237a05fbf6360ec5e33b0
static STYLE_THREAD_POOL: Mutex<&style::global_style_data::STYLE_THREAD_POOL> =
    Mutex::new(&style::global_style_data::STYLE_THREAD_POOL);

/// A CSS file to style the user agent stylesheet.
static USER_AGENT_CSS: &[u8] = include_bytes!("./stylesheets/user-agent.css");

/// A CSS file to style the Servo browser.
static SERVO_CSS: &[u8] = include_bytes!("./stylesheets/servo.css");

/// A CSS file to style the presentational hints.
static PRESENTATIONAL_HINTS_CSS: &[u8] = include_bytes!("./stylesheets/presentational-hints.css");

/// A CSS file to style the quirks mode.
static QUIRKS_MODE_CSS: &[u8] = include_bytes!("./stylesheets/quirks-mode.css");

/// Information needed by layout.
pub struct LayoutThread {
    /// The ID of the pipeline that we belong to.
    id: PipelineId,

    /// The webview that contains the pipeline we belong to.
    webview_id: WebViewId,

    /// The URL of the pipeline that we belong to.
    url: ServoUrl,

    /// Performs CSS selector matching and style resolution.
    stylist: Stylist,

    /// Is the current reflow of an iframe, as opposed to a root window?
    is_iframe: bool,

    /// The channel on which messages can be sent to the script thread.
    script_chan: IpcSender<ScriptThreadMessage>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// Reference to the script thread image cache.
    image_cache: Arc<dyn ImageCache>,

    /// A FontContext to be used during layout.
    font_context: Arc<FontContext>,

    /// Is this the first reflow in this LayoutThread?
    first_reflow: Cell<bool>,

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

    /// Cross-process access to the Compositor API.
    compositor_api: CrossProcessCompositorApi,

    /// Debug options, copied from configuration to this `LayoutThread` in order
    /// to avoid having to constantly access the thread-safe global options.
    debug: DebugOptions,
}

pub struct LayoutFactoryImpl();

impl LayoutFactory for LayoutFactoryImpl {
    fn create(&self, config: LayoutConfig) -> Box<dyn Layout> {
        Box::new(LayoutThread::new(config))
    }
}

impl Drop for LayoutThread {
    fn drop(&mut self) {
        let (keys, instance_keys) = self
            .font_context
            .collect_unused_webrender_resources(true /* all */);
        self.compositor_api
            .remove_unused_font_resources(keys, instance_keys)
    }
}

impl Layout for LayoutThread {
    fn device(&self) -> &Device {
        self.stylist.device()
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

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn remove_stylesheet(&mut self, stylesheet: ServoArc<Stylesheet>) {
        let guard = stylesheet.shared_lock.read();
        let stylesheet = DocumentStyleSheet(stylesheet.clone());
        self.stylist.remove_stylesheet(stylesheet.clone(), &guard);
        self.font_context
            .remove_all_web_fonts_from_stylesheet(&stylesheet);
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn query_content_box(&self, node: TrustedNodeAddress) -> Option<UntypedRect<Au>> {
        let node = unsafe { ServoLayoutNode::new(&node) };
        process_content_box_request(node)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn query_content_boxes(&self, node: TrustedNodeAddress) -> Vec<UntypedRect<Au>> {
        let node = unsafe { ServoLayoutNode::new(&node) };
        process_content_boxes_request(node)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn query_client_rect(&self, node: TrustedNodeAddress) -> UntypedRect<i32> {
        let node = unsafe { ServoLayoutNode::new(&node) };
        process_client_rect_request(node)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn query_element_inner_outer_text(
        &self,
        node: script_layout_interface::TrustedNodeAddress,
    ) -> String {
        let node = unsafe { ServoLayoutNode::new(&node) };
        get_the_text_steps(node)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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

        let client_point = DevicePoint::from_untyped(point);
        let results = self
            .compositor_api
            .hit_test(Some(self.id.into()), client_point, flags);

        results.iter().map(|result| result.node).collect()
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn query_offset_parent(&self, node: TrustedNodeAddress) -> OffsetParentResponse {
        let node = unsafe { ServoLayoutNode::new(&node) };
        process_offset_parent_query(node).unwrap_or_default()
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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

        process_resolved_style_request(&shared_style_context, node, &pseudo, &property_id)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn query_scrolling_area(&self, node: Option<TrustedNodeAddress>) -> UntypedRect<i32> {
        let node = node.map(|node| unsafe { ServoLayoutNode::new(&node) });
        process_node_scroll_area_request(node, self.fragment_tree.borrow().clone())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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

    fn collect_reports(&self, reports: &mut Vec<Report>, ops: &mut MallocSizeOfOps) {
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
            size: self.stylist.size_of(ops),
        });

        reports.push(Report {
            path: path![formatted_url, "layout-thread", "font-context"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: self.font_context.conditional_size_of(ops),
        });

        reports.push(Report {
            path: path![formatted_url, "layout-thread", "box-tree"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: self
                .box_tree
                .borrow()
                .as_ref()
                .map_or(0, |tree| tree.conditional_size_of(ops)),
        });

        reports.push(Report {
            path: path![formatted_url, "layout-thread", "fragment-tree"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: self
                .fragment_tree
                .borrow()
                .as_ref()
                .map(|tree| tree.conditional_size_of(ops))
                .unwrap_or_default(),
        });

        reports.push(self.image_cache.memory_report(formatted_url, ops));
    }

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.stylist.set_quirks_mode(quirks_mode);
    }

    fn reflow(&mut self, reflow_request: ReflowRequest) -> Option<ReflowResult> {
        time_profile!(
            profile_time::ProfilerCategory::LayoutPerform,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || self.handle_reflow(reflow_request),
        )
    }

    fn register_paint_worklet_modules(
        &mut self,
        _name: Atom,
        _properties: Vec<Atom>,
        _painter: Box<dyn Painter>,
    ) {
    }

    fn set_scroll_offsets(&mut self, scroll_states: &[ScrollState]) {
        *self.scroll_offsets.borrow_mut() = scroll_states
            .iter()
            .map(|scroll_state| (scroll_state.scroll_id, scroll_state.scroll_offset))
            .collect();
    }
}

impl LayoutThread {
    fn new(config: LayoutConfig) -> LayoutThread {
        // Let webrender know about this pipeline by sending an empty display list.
        config
            .compositor_api
            .send_initial_transaction(config.id.into());

        let mut font = Font::initial_values();
        let default_font_size = pref!(fonts_default_size);
        font.font_size = FontSize {
            computed_size: NonNegativeLength::new(default_font_size as f32),
            used_size: NonNegativeLength::new(default_font_size as f32),
            keyword_info: KeywordInfo::medium(),
        };

        // The device pixel ratio is incorrect (it does not have the hidpi value),
        // but it will be set correctly when the initial reflow takes place.
        let device = Device::new(
            MediaType::screen(),
            QuirksMode::NoQuirks,
            config.viewport_details.size,
            Scale::new(config.viewport_details.hidpi_scale_factor.get()),
            Box::new(LayoutFontMetricsProvider(config.font_context.clone())),
            ComputedValues::initial_values_with_font_override(font),
            // TODO: obtain preferred color scheme from embedder
            PrefersColorScheme::Light,
        );

        LayoutThread {
            id: config.id,
            webview_id: config.webview_id,
            url: config.url,
            is_iframe: config.is_iframe,
            script_chan: config.script_chan.clone(),
            time_profiler_chan: config.time_profiler_chan,
            registered_painters: RegisteredPaintersImpl(Default::default()),
            image_cache: config.image_cache,
            font_context: config.font_context,
            first_reflow: Cell::new(true),
            box_tree: Default::default(),
            fragment_tree: Default::default(),
            // Epoch starts at 1 because of the initial display list for epoch 0 that we send to WR
            epoch: Cell::new(Epoch(1)),
            viewport_size: Size2D::new(
                Au::from_f32_px(config.viewport_details.size.width),
                Au::from_f32_px(config.viewport_details.size.height),
            ),
            compositor_api: config.compositor_api,
            scroll_offsets: Default::default(),
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            webrender_image_cache: Default::default(),
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
                .send(ScriptThreadMessage::WebFontLoaded(pipeline_id, succeeded));
        };

        self.font_context.add_all_web_fonts_from_stylesheet(
            self.webview_id,
            stylesheet,
            guard,
            self.stylist.device(),
            Arc::new(web_font_finished_loading_callback) as StylesheetWebFontLoadFinishedCallback,
        );
    }

    /// The high-level routine that performs layout.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_reflow(&mut self, mut reflow_request: ReflowRequest) -> Option<ReflowResult> {
        let document = unsafe { ServoLayoutNode::new(&reflow_request.document) };
        let document = document.as_document().unwrap();
        let Some(root_element) = document.root_element() else {
            debug!("layout: No root node: bailing");
            return None;
        };

        let document_shared_lock = document.style_shared_lock();
        let author_guard = document_shared_lock.read();
        let ua_stylesheets = &*UA_STYLESHEETS;
        let ua_or_user_guard = ua_stylesheets.shared_lock.read();
        let rayon_pool = STYLE_THREAD_POOL.lock();
        let rayon_pool = rayon_pool.pool();
        let rayon_pool = rayon_pool.as_ref();
        let guards = StylesheetGuards {
            author: &author_guard,
            ua_or_user: &ua_or_user_guard,
        };

        if self.update_device_if_necessary(&reflow_request, &guards) {
            if let Some(mut data) = root_element.mutate_data() {
                data.hint.insert(RestyleHint::recascade_subtree());
            }
        }

        let mut snapshot_map = SnapshotMap::new();
        let _snapshot_setter = SnapshotSetter::new(&mut reflow_request, &mut snapshot_map);
        self.prepare_stylist_for_reflow(
            &reflow_request,
            document,
            root_element,
            &guards,
            ua_stylesheets,
            &snapshot_map,
        );

        let mut layout_context = LayoutContext {
            id: self.id,
            origin: reflow_request.origin.clone(),
            style_context: self.build_shared_style_context(
                guards,
                &snapshot_map,
                reflow_request.animation_timeline_value,
                &reflow_request.animations,
                match reflow_request.stylesheets_changed {
                    true => TraversalFlags::ForCSSRuleChanges,
                    false => TraversalFlags::empty(),
                },
            ),
            image_cache: self.image_cache.clone(),
            font_context: self.font_context.clone(),
            webrender_image_cache: self.webrender_image_cache.clone(),
            pending_images: Mutex::default(),
            node_image_animation_map: Arc::new(RwLock::new(std::mem::take(
                &mut reflow_request.node_to_image_animation_map,
            ))),
            iframe_sizes: Mutex::default(),
            use_rayon: rayon_pool.is_some(),
            highlighted_dom_node: reflow_request.highlighted_dom_node,
        };

        self.restyle_and_build_trees(
            &reflow_request,
            root_element,
            rayon_pool,
            &mut layout_context,
        );
        self.build_display_list(&reflow_request, &mut layout_context);
        self.first_reflow.set(false);

        if let ReflowGoal::UpdateScrollNode(scroll_state) = reflow_request.reflow_goal {
            self.update_scroll_node_state(&scroll_state);
        }

        let pending_images = std::mem::take(&mut *layout_context.pending_images.lock());
        let iframe_sizes = std::mem::take(&mut *layout_context.iframe_sizes.lock());
        let node_to_image_animation_map =
            std::mem::take(&mut *layout_context.node_image_animation_map.write());
        Some(ReflowResult {
            pending_images,
            iframe_sizes,
            node_to_image_animation_map,
        })
    }

    fn update_device_if_necessary(
        &mut self,
        reflow_request: &ReflowRequest,
        guards: &StylesheetGuards,
    ) -> bool {
        let had_used_viewport_units = self.stylist.device().used_viewport_units();
        let viewport_size_changed = self.viewport_did_change(reflow_request.viewport_details);
        let theme_changed = self.theme_did_change(reflow_request.theme);
        if !viewport_size_changed && !theme_changed {
            return false;
        }
        self.update_device(
            reflow_request.viewport_details,
            reflow_request.theme,
            guards,
        );
        (viewport_size_changed && had_used_viewport_units) || theme_changed
    }

    fn prepare_stylist_for_reflow<'dom>(
        &mut self,
        reflow_request: &ReflowRequest,
        document: ServoLayoutDocument<'dom>,
        root_element: ServoLayoutElement<'dom>,
        guards: &StylesheetGuards,
        ua_stylesheets: &UserAgentStylesheets,
        snapshot_map: &SnapshotMap,
    ) {
        if self.first_reflow.get() {
            for stylesheet in &ua_stylesheets.user_or_user_agent_stylesheets {
                self.stylist
                    .append_stylesheet(stylesheet.clone(), guards.ua_or_user);
                self.load_all_web_fonts_from_stylesheet_with_guard(stylesheet, guards.ua_or_user);
            }

            if self.stylist.quirks_mode() != QuirksMode::NoQuirks {
                self.stylist.append_stylesheet(
                    ua_stylesheets.quirks_mode_stylesheet.clone(),
                    guards.ua_or_user,
                );
                self.load_all_web_fonts_from_stylesheet_with_guard(
                    &ua_stylesheets.quirks_mode_stylesheet,
                    guards.ua_or_user,
                );
            }
        }

        if reflow_request.stylesheets_changed {
            self.stylist
                .force_stylesheet_origins_dirty(Origin::Author.into());
        }

        // Flush shadow roots stylesheets if dirty.
        document.flush_shadow_roots_stylesheets(&mut self.stylist, guards.author);

        self.stylist
            .flush(guards, Some(root_element), Some(snapshot_map));
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn restyle_and_build_trees(
        &self,
        reflow_request: &ReflowRequest,
        root_element: ServoLayoutElement<'_>,
        rayon_pool: Option<&ThreadPool>,
        layout_context: &mut LayoutContext<'_>,
    ) {
        let dirty_root = unsafe {
            ServoLayoutNode::new(&reflow_request.dirty_root.unwrap())
                .as_element()
                .unwrap()
        };

        let recalc_style_traversal = RecalcStyle::new(layout_context);
        let token = {
            let shared =
                DomTraversal::<ServoLayoutElement>::shared_context(&recalc_style_traversal);
            RecalcStyle::pre_traverse(dirty_root, shared)
        };

        if !token.should_traverse() {
            layout_context.style_context.stylist.rule_tree().maybe_gc();
            return;
        }

        let dirty_root: ServoLayoutNode =
            driver::traverse_dom(&recalc_style_traversal, token, rayon_pool).as_node();

        let root_node = root_element.as_node();
        let damage = compute_damage_and_repair_style(layout_context.shared_context(), root_node);
        if damage == RestyleDamage::REPAINT {
            layout_context.style_context.stylist.rule_tree().maybe_gc();
            return;
        }

        let mut box_tree = self.box_tree.borrow_mut();
        let box_tree = &mut *box_tree;
        let mut build_box_tree = || {
            if !BoxTree::update(recalc_style_traversal.context(), dirty_root) {
                *box_tree = Some(Arc::new(BoxTree::construct(
                    recalc_style_traversal.context(),
                    root_node,
                )));
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
                .layout(recalc_style_traversal.context(), viewport_size)
        };
        let fragment_tree = Arc::new(if let Some(pool) = rayon_pool {
            pool.install(run_layout)
        } else {
            run_layout()
        });

        *self.fragment_tree.borrow_mut() = Some(fragment_tree);

        if self.debug.dump_style_tree {
            println!(
                "{:?}",
                ShowSubtreeDataAndPrimaryValues(root_element.as_node())
            );
        }
        if self.debug.dump_rule_tree {
            recalc_style_traversal
                .context()
                .style_context
                .stylist
                .rule_tree()
                .dump_stdout(&layout_context.shared_context().guards);
        }

        // GC the rule tree if some heuristics are met.
        layout_context.style_context.stylist.rule_tree().maybe_gc();
    }

    fn build_display_list(
        &self,
        reflow_request: &ReflowRequest,
        layout_context: &mut LayoutContext<'_>,
    ) {
        let Some(fragment_tree) = &*self.fragment_tree.borrow() else {
            return;
        };
        if !reflow_request.reflow_goal.needs_display_list() {
            return;
        }

        let mut epoch = self.epoch.get();
        epoch.next();
        self.epoch.set(epoch);

        let viewport_size = LayoutSize::from_untyped(Size2D::new(
            self.viewport_size.width.to_f32_px(),
            self.viewport_size.height.to_f32_px(),
        ));
        let mut display_list = DisplayList::new(
            viewport_size,
            fragment_tree.scrollable_overflow(),
            self.id.into(),
            epoch.into(),
            fragment_tree.viewport_scroll_sensitivity,
            self.first_reflow.get(),
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
            display_list.build_stacking_context_tree(fragment_tree, &self.debug);

        // Build the rest of the display list which inclues all of the WebRender primitives.
        display_list.build(layout_context, fragment_tree, &root_stacking_context);

        if self.debug.dump_flow_tree {
            fragment_tree.print();
        }
        if self.debug.dump_stacking_context_tree {
            root_stacking_context.debug_print();
        }

        if reflow_request.reflow_goal.needs_display() {
            self.compositor_api.send_display_list(
                self.webview_id,
                display_list.compositor_info,
                display_list.wr.end().1,
            );

            let (keys, instance_keys) = self
                .font_context
                .collect_unused_webrender_resources(false /* all */);
            self.compositor_api
                .remove_unused_font_resources(keys, instance_keys)
        }
    }

    fn update_scroll_node_state(&self, state: &ScrollState) {
        self.scroll_offsets
            .borrow_mut()
            .insert(state.scroll_id, state.scroll_offset);
        let point = Point2D::new(-state.scroll_offset.x, -state.scroll_offset.y);
        self.compositor_api.send_scroll_node(
            self.webview_id,
            self.id.into(),
            LayoutPoint::from_untyped(point),
            state.scroll_id,
        );
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

    fn viewport_did_change(&mut self, viewport_details: ViewportDetails) -> bool {
        let new_pixel_ratio = viewport_details.hidpi_scale_factor.get();
        let new_viewport_size = Size2D::new(
            Au::from_f32_px(viewport_details.size.width),
            Au::from_f32_px(viewport_details.size.height),
        );

        // TODO: eliminate self.viewport_size in favour of using self.device.au_viewport_size()
        self.viewport_size = new_viewport_size;

        let device = self.stylist.device();
        let size_did_change = device.au_viewport_size() != new_viewport_size;
        let pixel_ratio_did_change = device.device_pixel_ratio().get() != new_pixel_ratio;

        size_did_change || pixel_ratio_did_change
    }

    fn theme_did_change(&self, theme: PrefersColorScheme) -> bool {
        theme != self.device().color_scheme()
    }

    /// Update layout given a new viewport. Returns true if the viewport changed or false if it didn't.
    fn update_device(
        &mut self,
        viewport_details: ViewportDetails,
        theme: PrefersColorScheme,
        guards: &StylesheetGuards,
    ) {
        let device = Device::new(
            MediaType::screen(),
            self.stylist.quirks_mode(),
            viewport_details.size,
            Scale::new(viewport_details.hidpi_scale_factor.get()),
            Box::new(LayoutFontMetricsProvider(self.font_context.clone())),
            self.stylist.device().default_computed_values().to_arc(),
            theme,
        );

        // Preserve any previously computed root font size.
        device.set_root_font_size(self.stylist.device().root_font_size().px());

        let sheet_origins_affected_by_device_change = self.stylist.set_device(device, guards);
        self.stylist
            .force_stylesheet_origins_dirty(sheet_origins_affected_by_device_change);
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
        parse_ua_stylesheet(shared_lock, "user-agent.css", USER_AGENT_CSS)?,
        parse_ua_stylesheet(shared_lock, "servo.css", SERVO_CSS)?,
        parse_ua_stylesheet(
            shared_lock,
            "presentational-hints.css",
            PRESENTATIONAL_HINTS_CSS,
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

    let quirks_mode_stylesheet =
        parse_ua_stylesheet(shared_lock, "quirks-mode.css", QUIRKS_MODE_CSS)?;

    Ok(UserAgentStylesheets {
        shared_lock: shared_lock.clone(),
        user_or_user_agent_stylesheets,
        quirks_mode_stylesheet,
    })
}

static UA_STYLESHEETS: LazyLock<UserAgentStylesheets> =
    LazyLock::new(|| match get_ua_stylesheets() {
        Ok(stylesheets) => stylesheets,
        Err(filename) => {
            error!("Failed to load UA stylesheet {}!", filename);
            process::exit(1);
        },
    });

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

struct LayoutFontMetricsProvider(Arc<FontContext>);

impl FontMetricsProvider for LayoutFontMetricsProvider {
    fn query_font_metrics(
        &self,
        _vertical: bool,
        font: &Font,
        base_size: CSSPixelLength,
        _flags: QueryFontMetricsFlags,
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
                    .find_by_codepoint(font_context, '0', None, None)?
                    .metrics
                    .zero_horizontal_advance
            })
            .map(CSSPixelLength::from);

        let ic_width = first_font_metrics
            .ic_horizontal_advance
            .or_else(|| {
                font_group
                    .write()
                    .find_by_codepoint(font_context, '\u{6C34}', None, None)?
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
            GenericFontFamily::Monospace => pref!(fonts_default_monospace_size),
            _ => pref!(fonts_default_size),
        } as f32)
        .max(Length::new(0.0))
    }
}

impl Debug for LayoutFontMetricsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LayoutFontMetricsProvider").finish()
    }
}

struct SnapshotSetter<'dom> {
    elements_with_snapshot: Vec<ServoLayoutElement<'dom>>,
}

impl SnapshotSetter<'_> {
    fn new(reflow_request: &mut ReflowRequest, snapshot_map: &mut SnapshotMap) -> Self {
        debug!(
            "Draining restyles: {}",
            reflow_request.pending_restyles.len()
        );
        let restyles = std::mem::take(&mut reflow_request.pending_restyles);

        let elements_with_snapshot: Vec<_> = restyles
            .iter()
            .filter(|r| r.1.snapshot.is_some())
            .map(|r| unsafe { ServoLayoutNode::new(&r.0).as_element().unwrap() })
            .collect();

        for (element, restyle) in restyles {
            let element = unsafe { ServoLayoutNode::new(&element).as_element().unwrap() };

            // If we haven't styled this node yet, we don't need to track a
            // restyle.
            let Some(mut style_data) = element.mutate_data() else {
                unsafe { element.unset_snapshot_flags() };
                continue;
            };

            debug!("Noting restyle for {:?}: {:?}", element, style_data);
            if let Some(s) = restyle.snapshot {
                unsafe { element.set_has_snapshot() };
                snapshot_map.insert(element.as_node().opaque(), s);
            }

            // Stash the data on the element for processing by the style system.
            style_data.hint.insert(restyle.hint);
            style_data.damage = restyle.damage;
        }
        Self {
            elements_with_snapshot,
        }
    }
}

impl Drop for SnapshotSetter<'_> {
    fn drop(&mut self) {
        for element in &self.elements_with_snapshot {
            unsafe { element.unset_snapshot_flags() }
        }
    }
}
