/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, LazyLock};

use app_units::Au;
use base::Epoch;
use base::id::{PipelineId, WebViewId};
use compositing_traits::CrossProcessCompositorApi;
use compositing_traits::display_list::ScrollType;
use embedder_traits::{Theme, UntrustedNodeAddress, ViewportDetails};
use euclid::default::{Point2D as UntypedPoint2D, Rect as UntypedRect};
use euclid::{Point2D, Scale, Size2D};
use fnv::FnvHashMap;
use fonts::{FontContext, FontContextWebFontMethods};
use fonts_traits::StylesheetWebFontLoadFinishedCallback;
use fxhash::FxHashMap;
use ipc_channel::ipc::IpcSender;
use log::{debug, error, warn};
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
use webrender_api::units::{DevicePixel, DevicePoint, LayoutSize, LayoutVector2D};
use webrender_api::{ExternalScrollId, HitTestFlags};

use crate::context::{CachedImageOrError, LayoutContext};
use crate::display_list::{DisplayListBuilder, StackingContextTree};
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

    /// Whether or not user agent stylesheets have been added to the Stylist or not.
    have_added_user_agent_stylesheets: bool,

    /// Is this the first reflow in this LayoutThread?
    have_ever_generated_display_list: Cell<bool>,

    /// Whether a new display list is necessary due to changes to layout or stacking
    /// contexts. This is set to true every time layout changes, even when a display list
    /// isn't requested for this layout, such as for layout queries. The next time a
    /// layout requests a display list, it is produced unconditionally, even when the
    /// layout trees remain the same.
    need_new_display_list: Cell<bool>,

    /// Whether or not the existing stacking context tree is dirty and needs to be
    /// rebuilt. This happens after a relayout or overflow update. The reason that we
    /// don't simply clear the stacking context tree when it becomes dirty is that we need
    /// to preserve scroll offsets from the old tree to the new one.
    need_new_stacking_context_tree: Cell<bool>,

    /// The box tree.
    box_tree: RefCell<Option<Arc<BoxTree>>>,

    /// The fragment tree.
    fragment_tree: RefCell<Option<Rc<FragmentTree>>>,

    /// The [`StackingContextTree`] cached from previous layouts.
    stacking_context_tree: RefCell<Option<StackingContextTree>>,

    /// A counter for epoch messages
    epoch: Cell<Epoch>,

    // A cache that maps image resources specified in CSS (e.g as the `url()` value
    // for `background-image` or `content` properties) to either the final resolved
    // image data, or an error if the image cache failed to load/decode the image.
    resolved_images_cache: Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), CachedImageOrError>>>,

    /// The executors for paint worklets.
    registered_painters: RegisteredPaintersImpl,

    /// Cross-process access to the Compositor API.
    compositor_api: CrossProcessCompositorApi,

    /// Debug options, copied from configuration to this `LayoutThread` in order
    /// to avoid having to constantly access the thread-safe global options.
    debug: DebugOptions,

    /// Tracks the node that was highlighted by the devtools during the last reflow.
    ///
    /// If this changed, then we need to create a new display list.
    previously_highlighted_dom_node: Cell<Option<OpaqueNode>>,
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

    fn set_scroll_offsets_from_renderer(
        &mut self,
        scroll_states: &HashMap<ExternalScrollId, LayoutVector2D>,
    ) {
        let mut stacking_context_tree = self.stacking_context_tree.borrow_mut();
        let Some(stacking_context_tree) = stacking_context_tree.as_mut() else {
            warn!("Received scroll offsets before finishing layout.");
            return;
        };

        stacking_context_tree
            .compositor_info
            .scroll_tree
            .set_all_scroll_offsets(scroll_states);
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
            config.theme.into(),
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
            have_added_user_agent_stylesheets: false,
            have_ever_generated_display_list: Cell::new(false),
            need_new_display_list: Cell::new(false),
            need_new_stacking_context_tree: Cell::new(false),
            box_tree: Default::default(),
            fragment_tree: Default::default(),
            stacking_context_tree: Default::default(),
            // Epoch starts at 1 because of the initial display list for epoch 0 that we send to WR
            epoch: Cell::new(Epoch(1)),
            compositor_api: config.compositor_api,
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            resolved_images_cache: Default::default(),
            debug: opts::get().debug.clone(),
            previously_highlighted_dom_node: Cell::new(None),
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

        let viewport_changed = self.viewport_did_change(reflow_request.viewport_details);
        if self.update_device_if_necessary(&reflow_request, viewport_changed, &guards) {
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

        if self.previously_highlighted_dom_node.get() != reflow_request.highlighted_dom_node {
            // Need to manually force layout to build a new display list regardless of whether the box tree
            // changed or not.
            self.need_new_display_list.set(true);
        }

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
            resolved_images_cache: self.resolved_images_cache.clone(),
            pending_images: Mutex::default(),
            pending_rasterization_images: Mutex::default(),
            node_image_animation_map: Arc::new(RwLock::new(std::mem::take(
                &mut reflow_request.node_to_image_animation_map,
            ))),
            iframe_sizes: Mutex::default(),
            use_rayon: rayon_pool.is_some(),
            highlighted_dom_node: reflow_request.highlighted_dom_node,
        };

        let damage = self.restyle_and_build_trees(
            &reflow_request,
            root_element,
            rayon_pool,
            &mut layout_context,
            viewport_changed,
        );
        self.calculate_overflow(damage);
        self.build_stacking_context_tree(&reflow_request, damage);
        let built_display_list =
            self.build_display_list(&reflow_request, damage, &mut layout_context);

        if let ReflowGoal::UpdateScrollNode(external_scroll_id, offset) = reflow_request.reflow_goal
        {
            self.set_scroll_offset_from_script(external_scroll_id, offset);
        }

        let pending_images = std::mem::take(&mut *layout_context.pending_images.lock());
        let pending_rasterization_images =
            std::mem::take(&mut *layout_context.pending_rasterization_images.lock());
        let iframe_sizes = std::mem::take(&mut *layout_context.iframe_sizes.lock());
        let node_to_image_animation_map =
            std::mem::take(&mut *layout_context.node_image_animation_map.write());

        Some(ReflowResult {
            built_display_list,
            pending_images,
            pending_rasterization_images,
            iframe_sizes,
            node_to_image_animation_map,
        })
    }

    fn update_device_if_necessary(
        &mut self,
        reflow_request: &ReflowRequest,
        viewport_changed: bool,
        guards: &StylesheetGuards,
    ) -> bool {
        let had_used_viewport_units = self.stylist.device().used_viewport_units();
        let theme_changed = self.theme_did_change(reflow_request.theme);
        if !viewport_changed && !theme_changed {
            return false;
        }
        self.update_device(
            reflow_request.viewport_details,
            reflow_request.theme,
            guards,
        );
        (viewport_changed && had_used_viewport_units) || theme_changed
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
        if !self.have_added_user_agent_stylesheets {
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
            self.have_added_user_agent_stylesheets = true;
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
        viewport_changed: bool,
    ) -> RestyleDamage {
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
            return RestyleDamage::empty();
        }

        let dirty_root: ServoLayoutNode =
            driver::traverse_dom(&recalc_style_traversal, token, rayon_pool).as_node();

        let root_node = root_element.as_node();
        let mut damage =
            compute_damage_and_repair_style(layout_context.shared_context(), root_node);
        if viewport_changed {
            damage = RestyleDamage::REBUILD_BOX;
        } else if !damage.contains(RestyleDamage::REBUILD_BOX) {
            layout_context.style_context.stylist.rule_tree().maybe_gc();
            return damage;
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

        let viewport_size = self.stylist.device().au_viewport_size();
        let run_layout = || {
            box_tree
                .as_ref()
                .unwrap()
                .layout(recalc_style_traversal.context(), viewport_size)
        };
        let fragment_tree = Rc::new(if let Some(pool) = rayon_pool {
            pool.install(run_layout)
        } else {
            run_layout()
        });

        *self.fragment_tree.borrow_mut() = Some(fragment_tree);

        // Changes to layout require us to generate a new stacking context tree and display
        // list the next time one is requested.
        self.need_new_display_list.set(true);
        self.need_new_stacking_context_tree.set(true);

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
        damage
    }

    fn calculate_overflow(&self, damage: RestyleDamage) {
        if !damage.contains(RestyleDamage::RECALCULATE_OVERFLOW) {
            return;
        }

        if let Some(fragment_tree) = &*self.fragment_tree.borrow() {
            fragment_tree.calculate_scrollable_overflow();
            if self.debug.dump_flow_tree {
                fragment_tree.print();
            }
        }

        // Changes to overflow require us to generate a new stacking context tree and
        // display list the next time one is requested.
        self.need_new_display_list.set(true);
        self.need_new_stacking_context_tree.set(true);
    }

    fn build_stacking_context_tree(&self, reflow_request: &ReflowRequest, damage: RestyleDamage) {
        if !reflow_request.reflow_goal.needs_display_list() &&
            !reflow_request.reflow_goal.needs_display()
        {
            return;
        }
        let Some(fragment_tree) = &*self.fragment_tree.borrow() else {
            return;
        };
        if !damage.contains(RestyleDamage::REBUILD_STACKING_CONTEXT) &&
            !self.need_new_stacking_context_tree.get()
        {
            return;
        }

        let viewport_size = self.stylist.device().au_viewport_size();
        let viewport_size = LayoutSize::new(
            viewport_size.width.to_f32_px(),
            viewport_size.height.to_f32_px(),
        );

        let mut stacking_context_tree = self.stacking_context_tree.borrow_mut();
        let old_scroll_offsets = stacking_context_tree
            .as_ref()
            .map(|tree| tree.compositor_info.scroll_tree.scroll_offsets());

        // Build the StackingContextTree. This turns the `FragmentTree` into a
        // tree of fragments in CSS painting order and also creates all
        // applicable spatial and clip nodes.
        let mut new_stacking_context_tree = StackingContextTree::new(
            fragment_tree,
            viewport_size,
            self.id.into(),
            !self.have_ever_generated_display_list.get(),
            &self.debug,
        );

        // When a new StackingContextTree is built, it contains a freshly built
        // ScrollTree. We want to preserve any existing scroll offsets in that tree,
        // adjusted by any new scroll constraints.
        if let Some(old_scroll_offsets) = old_scroll_offsets {
            new_stacking_context_tree
                .compositor_info
                .scroll_tree
                .set_all_scroll_offsets(&old_scroll_offsets);
        }

        *stacking_context_tree = Some(new_stacking_context_tree);

        // Force display list generation as layout has changed.
        self.need_new_display_list.set(true);

        // The stacking context tree is up-to-date again.
        self.need_new_stacking_context_tree.set(false);
    }

    /// Build the display list for the current layout and send it to the renderer. If no display
    /// list is built, returns false.
    fn build_display_list(
        &self,
        reflow_request: &ReflowRequest,
        damage: RestyleDamage,
        layout_context: &mut LayoutContext<'_>,
    ) -> bool {
        if !reflow_request.reflow_goal.needs_display() {
            return false;
        }
        let Some(fragment_tree) = &*self.fragment_tree.borrow() else {
            return false;
        };
        let mut stacking_context_tree = self.stacking_context_tree.borrow_mut();
        let Some(stacking_context_tree) = stacking_context_tree.as_mut() else {
            return false;
        };

        // It's not enough to simply check `damage` here as not all reflow requests
        // require display lists. If a non-display-list-generating reflow updated layout
        // in a previous refow, we cannot skip display list generation here the next time
        // a display list is requested.
        if !self.need_new_display_list.get() && !damage.contains(RestyleDamage::REPAINT) {
            return false;
        }

        let mut epoch = self.epoch.get();
        epoch.next();
        self.epoch.set(epoch);
        stacking_context_tree.compositor_info.epoch = epoch.into();

        let built_display_list = DisplayListBuilder::build(
            layout_context,
            stacking_context_tree,
            fragment_tree,
            &self.debug,
        );
        self.compositor_api.send_display_list(
            self.webview_id,
            &stacking_context_tree.compositor_info,
            built_display_list,
        );

        let (keys, instance_keys) = self
            .font_context
            .collect_unused_webrender_resources(false /* all */);
        self.compositor_api
            .remove_unused_font_resources(keys, instance_keys);

        self.have_ever_generated_display_list.set(true);
        self.need_new_display_list.set(false);
        self.previously_highlighted_dom_node
            .set(reflow_request.highlighted_dom_node);
        true
    }

    fn set_scroll_offset_from_script(
        &self,
        external_scroll_id: ExternalScrollId,
        offset: LayoutVector2D,
    ) {
        let mut stacking_context_tree = self.stacking_context_tree.borrow_mut();
        let Some(stacking_context_tree) = stacking_context_tree.as_mut() else {
            return;
        };

        if let Some(offset) = stacking_context_tree
            .compositor_info
            .scroll_tree
            .set_scroll_offset_for_node_with_external_scroll_id(
                external_scroll_id,
                offset,
                ScrollType::Script,
            )
        {
            self.compositor_api.send_scroll_node(
                self.webview_id,
                self.id.into(),
                offset,
                external_scroll_id,
            );
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
            incremental: if self.have_ever_generated_display_list.get() {
                TimerMetadataReflowType::Incremental
            } else {
                TimerMetadataReflowType::FirstReflow
            },
        })
    }

    fn viewport_did_change(&mut self, viewport_details: ViewportDetails) -> bool {
        let new_pixel_ratio = viewport_details.hidpi_scale_factor.get();
        let new_viewport_size = Size2D::new(
            Au::from_f32_px(viewport_details.size.width),
            Au::from_f32_px(viewport_details.size.height),
        );

        let device = self.stylist.device();
        let size_did_change = device.au_viewport_size() != new_viewport_size;
        let pixel_ratio_did_change = device.device_pixel_ratio().get() != new_pixel_ratio;

        size_did_change || pixel_ratio_did_change
    }

    fn theme_did_change(&self, theme: Theme) -> bool {
        let theme: PrefersColorScheme = theme.into();
        theme != self.device().color_scheme()
    }

    /// Update layout given a new viewport. Returns true if the viewport changed or false if it didn't.
    fn update_device(
        &mut self,
        viewport_details: ViewportDetails,
        theme: Theme,
        guards: &StylesheetGuards,
    ) {
        let device = Device::new(
            MediaType::screen(),
            self.stylist.quirks_mode(),
            viewport_details.size,
            Scale::new(viewport_details.hidpi_scale_factor.get()),
            Box::new(LayoutFontMetricsProvider(self.font_context.clone())),
            self.stylist.device().default_computed_values().to_arc(),
            theme.into(),
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
