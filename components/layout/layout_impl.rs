/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, LazyLock};

use app_units::Au;
use base::generic_channel::GenericSender;
use base::id::{PipelineId, WebViewId};
use bitflags::bitflags;
use compositing_traits::CrossProcessPaintApi;
use compositing_traits::display_list::ScrollType;
use cssparser::ParserInput;
use embedder_traits::{Theme, ViewportDetails};
use euclid::default::{Point2D as UntypedPoint2D, Rect as UntypedRect};
use euclid::{Point2D, Scale, Size2D};
use fonts::{FontContext, FontContextWebFontMethods, WebFontDocumentContext};
use fonts_traits::StylesheetWebFontLoadFinishedCallback;
use layout_api::wrapper_traits::LayoutNode;
use layout_api::{
    BoxAreaType, IFrameSizes, Layout, LayoutConfig, LayoutDamage, LayoutFactory,
    OffsetParentResponse, PhysicalSides, PropertyRegistration, QueryMsg, ReflowGoal,
    ReflowPhasesRun, ReflowRequest, ReflowRequestRestyle, ReflowResult, RegisterPropertyError,
    ScrollContainerQueryFlags, ScrollContainerResponse, TrustedNodeAddress,
};
use log::{debug, error, warn};
use malloc_size_of::{MallocConditionalSizeOf, MallocSizeOf, MallocSizeOfOps};
use net_traits::image_cache::ImageCache;
use parking_lot::{Mutex, RwLock};
use profile_traits::mem::{Report, ReportKind};
use profile_traits::time::{
    self as profile_time, TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType,
};
use profile_traits::{path, time_profile};
use rustc_hash::FxHashMap;
use script::layout_dom::{ServoLayoutDocument, ServoLayoutElement, ServoLayoutNode};
use script_traits::{DrawAPaintImageResult, PaintWorkletError, Painter, ScriptThreadMessage};
use servo_arc::Arc as ServoArc;
use servo_config::opts::{self, DiagnosticsLogging};
use servo_config::pref;
use servo_url::ServoUrl;
use style::animation::DocumentAnimationSet;
use style::context::{
    QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters, SharedStyleContext,
};
use style::custom_properties::{SpecifiedValue, parse_name};
use style::dom::{OpaqueNode, ShowSubtreeDataAndPrimaryValues, TElement, TNode};
use style::error_reporting::RustLogReporter;
use style::font_metrics::FontMetrics;
use style::global_style_data::GLOBAL_STYLE_DATA;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::media_queries::{Device, MediaList, MediaType};
use style::properties::style_structs::Font;
use style::properties::{ComputedValues, PropertyId};
use style::properties_and_values::registry::{
    PropertyRegistration as StyloPropertyRegistration, PropertyRegistrationData,
};
use style::properties_and_values::rule::{Inherits, PropertyRegistrationError, PropertyRuleName};
use style::properties_and_values::syntax::Descriptor;
use style::queries::values::PrefersColorScheme;
use style::selector_parser::{PseudoElement, RestyleDamage, SnapshotMap};
use style::servo::media_queries::FontMetricsProvider;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards};
use style::stylesheets::{
    CustomMediaMap, DocumentStyleSheet, Origin, Stylesheet, StylesheetInDocument, UrlExtraData,
    UserAgentStylesheets,
};
use style::stylist::Stylist;
use style::traversal::DomTraversal;
use style::traversal_flags::TraversalFlags;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{CSSPixelLength, FontSize, Length, NonNegativeLength};
use style::values::specified::font::{KeywordInfo, QueryFontMetricsFlags};
use style::values::{Parser, SourceLocation};
use style::{Zero, driver};
use style_traits::{CSSPixel, SpeculativePainter};
use stylo_atoms::Atom;
use url::Url;
use webrender_api::ExternalScrollId;
use webrender_api::units::{DevicePixel, LayoutVector2D};

use crate::context::{CachedImageOrError, ImageResolver, LayoutContext};
use crate::display_list::{
    DisplayListBuilder, HitTest, LargestContentfulPaintCandidateCollector, StackingContextTree,
};
use crate::query::{
    get_the_text_steps, process_box_area_request, process_box_areas_request,
    process_client_rect_request, process_current_css_zoom_query, process_node_scroll_area_request,
    process_offset_parent_query, process_padding_request, process_resolved_font_style_query,
    process_resolved_style_request, process_scroll_container_query, process_text_index_request,
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
    script_chan: GenericSender<ScriptThreadMessage>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// Reference to the script thread image cache.
    image_cache: Arc<dyn ImageCache>,

    /// A FontContext to be used during layout.
    font_context: Arc<FontContext>,

    /// Whether or not user agent stylesheets have been added to the Stylist or not.
    have_added_user_agent_stylesheets: bool,

    /// Whether or not this [`LayoutImpl`]'s [`Device`] has changed since the last restyle.
    /// If it has, a restyle is pending.
    device_has_changed: bool,

    /// Is this the first reflow in this LayoutThread?
    have_ever_generated_display_list: Cell<bool>,

    /// Whether a new overflow calculation needs to happen due to changes to the fragment
    /// tree. This is set to true every time a restyle requests overflow calculation.
    need_overflow_calculation: Cell<bool>,

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

    // A cache that maps image resources specified in CSS (e.g as the `url()` value
    // for `background-image` or `content` properties) to either the final resolved
    // image data, or an error if the image cache failed to load/decode the image.
    resolved_images_cache: Arc<RwLock<HashMap<ServoUrl, CachedImageOrError>>>,

    /// The executors for paint worklets.
    registered_painters: RegisteredPaintersImpl,

    /// Cross-process access to the `Paint` API.
    paint_api: CrossProcessPaintApi,

    /// Debug options, copied from configuration to this `LayoutThread` in order
    /// to avoid having to constantly access the thread-safe global options.
    debug: DiagnosticsLogging,

    /// Tracks the node that was highlighted by the devtools during the last reflow.
    ///
    /// If this changed, then we need to create a new display list.
    previously_highlighted_dom_node: Cell<Option<OpaqueNode>>,

    /// The collector for calculating Largest Contentful Paint
    lcp_candidate_collector: RefCell<Option<LargestContentfulPaintCandidateCollector>>,
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
        self.paint_api
            .remove_unused_font_resources(self.webview_id.into(), keys, instance_keys)
    }
}

impl Layout for LayoutThread {
    fn device(&self) -> &Device {
        self.stylist.device()
    }

    fn set_theme(&mut self, theme: Theme) -> bool {
        let theme: PrefersColorScheme = theme.into();
        let device = self.stylist.device_mut();
        if theme == device.color_scheme() {
            return false;
        }

        device.set_color_scheme(theme);
        self.device_has_changed = true;
        true
    }

    fn set_viewport_details(&mut self, viewport_details: ViewportDetails) -> bool {
        let device = self.stylist.device_mut();
        let device_pixel_ratio = Scale::new(viewport_details.hidpi_scale_factor.get());
        if device.viewport_size() == viewport_details.size &&
            device.device_pixel_ratio() == device_pixel_ratio
        {
            return false;
        }

        device.set_viewport_size(viewport_details.size);
        device.set_device_pixel_ratio(device_pixel_ratio);
        self.device_has_changed = true;
        true
    }

    fn load_web_fonts_from_stylesheet(
        &self,
        stylesheet: &ServoArc<Stylesheet>,
        document_context: &WebFontDocumentContext,
    ) {
        let guard = stylesheet.shared_lock.read();
        self.load_all_web_fonts_from_stylesheet_with_guard(
            &DocumentStyleSheet(stylesheet.clone()),
            &guard,
            document_context,
        );
    }

    #[servo_tracing::instrument(skip_all)]
    fn add_stylesheet(
        &mut self,
        stylesheet: ServoArc<Stylesheet>,
        before_stylesheet: Option<ServoArc<Stylesheet>>,
        document_context: &WebFontDocumentContext,
    ) {
        let guard = stylesheet.shared_lock.read();
        let stylesheet = DocumentStyleSheet(stylesheet.clone());
        self.load_all_web_fonts_from_stylesheet_with_guard(&stylesheet, &guard, document_context);

        match before_stylesheet {
            Some(insertion_point) => self.stylist.insert_stylesheet_before(
                stylesheet,
                DocumentStyleSheet(insertion_point),
                &guard,
            ),
            None => self.stylist.append_stylesheet(stylesheet, &guard),
        }
    }

    #[servo_tracing::instrument(skip_all)]
    fn remove_stylesheet(&mut self, stylesheet: ServoArc<Stylesheet>) {
        let guard = stylesheet.shared_lock.read();
        let stylesheet = DocumentStyleSheet(stylesheet.clone());
        self.stylist.remove_stylesheet(stylesheet.clone(), &guard);
        self.font_context
            .remove_all_web_fonts_from_stylesheet(&stylesheet);
    }

    /// Return the resolved values of this node's padding rect.
    #[servo_tracing::instrument(skip_all)]
    fn query_padding(&self, node: TrustedNodeAddress) -> Option<PhysicalSides> {
        // If we have not built a fragment tree yet, there is no way we have layout information for
        // this query, which can be run without forcing a layout (for IntersectionObserver).
        if self.fragment_tree.borrow().is_none() {
            return None;
        }

        let node = unsafe { ServoLayoutNode::new(&node) };
        process_padding_request(node.to_threadsafe())
    }

    /// Return the union of this node's areas in the coordinate space of the Document. This is used
    /// to implement `getBoundingClientRect()` and support many other API where the such query is
    /// required.
    ///
    /// Part of <https://drafts.csswg.org/cssom-view-1/#element-get-the-bounding-box>.
    #[servo_tracing::instrument(skip_all)]
    fn query_box_area(
        &self,
        node: TrustedNodeAddress,
        area: BoxAreaType,
        exclude_transform_and_inline: bool,
    ) -> Option<UntypedRect<Au>> {
        // If we have not built a fragment tree yet, there is no way we have layout information for
        // this query, which can be run without forcing a layout (for IntersectionObserver).
        if self.fragment_tree.borrow().is_none() {
            return None;
        }

        let node = unsafe { ServoLayoutNode::new(&node) };
        let stacking_context_tree = self.stacking_context_tree.borrow();
        let stacking_context_tree = stacking_context_tree
            .as_ref()
            .expect("Should always have a StackingContextTree for box area queries");
        process_box_area_request(
            stacking_context_tree,
            node.to_threadsafe(),
            area,
            exclude_transform_and_inline,
        )
    }

    /// Get a `Vec` of bounding boxes of this node's `Fragment`s specific area in the coordinate space of
    /// the Document. This is used to implement `getClientRects()`.
    ///
    /// See <https://drafts.csswg.org/cssom-view/#dom-element-getclientrects>.
    #[servo_tracing::instrument(skip_all)]
    fn query_box_areas(&self, node: TrustedNodeAddress, area: BoxAreaType) -> Vec<UntypedRect<Au>> {
        // If we have not built a fragment tree yet, there is no way we have layout information for
        // this query, which can be run without forcing a layout (for IntersectionObserver).
        if self.fragment_tree.borrow().is_none() {
            return vec![];
        }

        let node = unsafe { ServoLayoutNode::new(&node) };
        let stacking_context_tree = self.stacking_context_tree.borrow();
        let stacking_context_tree = stacking_context_tree
            .as_ref()
            .expect("Should always have a StackingContextTree for box area queries");
        process_box_areas_request(stacking_context_tree, node.to_threadsafe(), area)
    }

    #[servo_tracing::instrument(skip_all)]
    fn query_client_rect(&self, node: TrustedNodeAddress) -> UntypedRect<i32> {
        let node = unsafe { ServoLayoutNode::new(&node) };
        process_client_rect_request(node.to_threadsafe())
    }

    #[servo_tracing::instrument(skip_all)]
    fn query_current_css_zoom(&self, node: TrustedNodeAddress) -> f32 {
        let node = unsafe { ServoLayoutNode::new(&node) };
        process_current_css_zoom_query(node)
    }

    #[servo_tracing::instrument(skip_all)]
    fn query_element_inner_outer_text(&self, node: layout_api::TrustedNodeAddress) -> String {
        let node = unsafe { ServoLayoutNode::new(&node) };
        get_the_text_steps(node)
    }
    #[servo_tracing::instrument(skip_all)]
    fn query_offset_parent(&self, node: TrustedNodeAddress) -> OffsetParentResponse {
        let node = unsafe { ServoLayoutNode::new(&node) };
        let stacking_context_tree = self.stacking_context_tree.borrow();
        let stacking_context_tree = stacking_context_tree
            .as_ref()
            .expect("Should always have a StackingContextTree for offset parent queries");
        process_offset_parent_query(&stacking_context_tree.paint_info.scroll_tree, node)
            .unwrap_or_default()
    }

    #[servo_tracing::instrument(skip_all)]
    fn query_scroll_container(
        &self,
        node: Option<TrustedNodeAddress>,
        flags: ScrollContainerQueryFlags,
    ) -> Option<ScrollContainerResponse> {
        let node = unsafe { node.as_ref().map(|node| ServoLayoutNode::new(node)) };
        let viewport_overflow = self
            .box_tree
            .borrow()
            .as_ref()
            .expect("Should have a BoxTree for all scroll container queries.")
            .viewport_overflow;
        process_scroll_container_query(node, flags, viewport_overflow)
    }

    #[servo_tracing::instrument(skip_all)]
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

    #[servo_tracing::instrument(skip_all)]
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

    #[servo_tracing::instrument(skip_all)]
    fn query_scrolling_area(&self, node: Option<TrustedNodeAddress>) -> UntypedRect<i32> {
        let node = node.map(|node| unsafe { ServoLayoutNode::new(&node).to_threadsafe() });
        process_node_scroll_area_request(node, self.fragment_tree.borrow().clone())
    }

    #[servo_tracing::instrument(skip_all)]
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

    #[servo_tracing::instrument(skip_all)]
    fn query_elements_from_point(
        &self,
        point: webrender_api::units::LayoutPoint,
        flags: layout_api::ElementsFromPointFlags,
    ) -> Vec<layout_api::ElementsFromPointResult> {
        self.stacking_context_tree
            .borrow_mut()
            .as_mut()
            .map(|tree| HitTest::run(tree, point, flags))
            .unwrap_or_default()
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

        reports.push(Report {
            path: path![formatted_url, "layout-thread", "stacking-context-tree"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: self.stacking_context_tree.size_of(ops),
        });

        reports.extend(self.image_cache.memory_reports(formatted_url, ops));
    }

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.stylist.set_quirks_mode(quirks_mode);
    }

    fn reflow(&mut self, reflow_request: ReflowRequest) -> Option<ReflowResult> {
        time_profile!(
            profile_time::ProfilerCategory::Layout,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || self.handle_reflow(reflow_request),
        )
    }

    fn ensure_stacking_context_tree(&self, viewport_details: ViewportDetails) {
        if self.stacking_context_tree.borrow().is_some() &&
            !self.need_new_stacking_context_tree.get()
        {
            return;
        }
        self.build_stacking_context_tree(viewport_details);
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
        scroll_states: &FxHashMap<ExternalScrollId, LayoutVector2D>,
    ) {
        let mut stacking_context_tree = self.stacking_context_tree.borrow_mut();
        let Some(stacking_context_tree) = stacking_context_tree.as_mut() else {
            warn!("Received scroll offsets before finishing layout.");
            return;
        };

        stacking_context_tree
            .paint_info
            .scroll_tree
            .set_all_scroll_offsets(scroll_states);
    }

    fn scroll_offset(&self, id: ExternalScrollId) -> Option<LayoutVector2D> {
        self.stacking_context_tree
            .borrow_mut()
            .as_mut()
            .and_then(|tree| tree.paint_info.scroll_tree.scroll_offset(id))
    }

    fn needs_new_display_list(&self) -> bool {
        self.need_new_display_list.get()
    }

    fn set_needs_new_display_list(&self) {
        self.need_new_display_list.set(true);
    }

    /// <https://drafts.css-houdini.org/css-properties-values-api-1/#the-registerproperty-function>
    fn register_custom_property(
        &mut self,
        property_registration: PropertyRegistration,
    ) -> Result<(), RegisterPropertyError> {
        // Step 2. If name is not a custom property name string, throw a SyntaxError and exit this algorithm.
        // If property set already contains an entry with name as its property name
        // (compared codepoint-wise), throw an InvalidModificationError and exit this algorithm.
        let Ok(name) = parse_name(&property_registration.name) else {
            return Err(RegisterPropertyError::InvalidName);
        };
        let name = Atom::from(name);

        if self
            .stylist
            .custom_property_script_registry()
            .get(&name)
            .is_some()
        {
            return Err(RegisterPropertyError::AlreadyRegistered);
        }

        // Step 3. Attempt to consume a syntax definition from syntax. If it returns failure,
        // throw a SyntaxError. Otherwise, let syntax definition be the returned syntax definition.
        let syntax = Descriptor::from_str(&property_registration.syntax, false)
            .map_err(|_| RegisterPropertyError::InvalidSyntax)?;

        // Step 4 - Parse and validate initial value
        let initial_value = match property_registration.initial_value {
            Some(value) => {
                let mut input = ParserInput::new(&value);
                let parsed = Parser::new(&mut input)
                    .parse_entirely(|input| {
                        input.skip_whitespace();
                        SpecifiedValue::parse(input, &property_registration.url_data)
                            .map(servo_arc::Arc::new)
                    })
                    .ok();
                if parsed.is_none() {
                    return Err(RegisterPropertyError::InvalidInitialValue);
                }
                parsed
            },
            None => None,
        };

        StyloPropertyRegistration::validate_initial_value(&syntax, initial_value.as_deref())
            .map_err(|error| match error {
                PropertyRegistrationError::InitialValueNotComputationallyIndependent => {
                    RegisterPropertyError::InitialValueNotComputationallyIndependent
                },
                PropertyRegistrationError::InvalidInitialValue => {
                    RegisterPropertyError::InvalidInitialValue
                },
                PropertyRegistrationError::NoInitialValue => RegisterPropertyError::NoInitialValue,
            })?;

        // Step 5. Set inherit flag to the value of inherits.
        let inherits = if property_registration.inherits {
            Inherits::True
        } else {
            Inherits::False
        };

        // Step 6. Let registered property be a struct with a property name of name, a syntax of
        // syntax definition, an initial value of parsed initial value, and an inherit flag of inherit flag.
        // Append registered property to property set.
        let property_registration = StyloPropertyRegistration {
            name: PropertyRuleName(name),
            data: PropertyRegistrationData {
                syntax,
                initial_value,
                inherits,
            },
            url_data: property_registration.url_data,
            source_location: SourceLocation { line: 0, column: 0 },
        };

        self.stylist
            .custom_property_script_registry_mut()
            .register(property_registration);
        self.stylist.rebuild_initial_values_for_custom_properties();

        Ok(())
    }
}

impl LayoutThread {
    fn new(config: LayoutConfig) -> LayoutThread {
        // Let webrender know about this pipeline by sending an empty display list.
        config
            .paint_api
            .send_initial_transaction(config.webview_id, config.id.into());

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
            device_has_changed: false,
            need_overflow_calculation: Cell::new(false),
            need_new_display_list: Cell::new(false),
            need_new_stacking_context_tree: Cell::new(false),
            box_tree: Default::default(),
            fragment_tree: Default::default(),
            stacking_context_tree: Default::default(),
            paint_api: config.paint_api,
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            resolved_images_cache: Default::default(),
            debug: opts::get().debug.clone(),
            previously_highlighted_dom_node: Cell::new(None),
            lcp_candidate_collector: Default::default(),
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
        document_context: &WebFontDocumentContext,
    ) {
        let custom_media = &CustomMediaMap::default();
        if !stylesheet.is_effective_for_device(self.stylist.device(), custom_media, guard) {
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
            document_context,
        );
    }

    /// In some cases, if a restyle isn't necessary we can skip doing any work for layout
    /// entirely. This check allows us to return early from layout without doing any work
    /// at all.
    fn can_skip_reflow_request_entirely(&self, reflow_request: &ReflowRequest) -> bool {
        // If a restyle is necessary, restyle and reflow is a necessity.
        if reflow_request.restyle.is_some() {
            return false;
        }
        // We always need to at least build a fragment tree.
        if self.fragment_tree.borrow().is_none() {
            return false;
        }

        // If we have a fragment tree and it's up-to-date and this reflow
        // doesn't need more reflow results, we can skip the rest of layout.
        let necessary_phases = ReflowPhases::necessary(&reflow_request.reflow_goal);
        if necessary_phases.is_empty() {
            return true;
        }

        // If only the stacking context tree is required, and it's up-to-date,
        // layout is unnecessary, otherwise a layout is necessary.
        if necessary_phases == ReflowPhases::StackingContextTreeConstruction {
            return self.stacking_context_tree.borrow().is_some() &&
                !self.need_new_stacking_context_tree.get();
        }

        // Otherwise, the only interesting thing is whether the current display
        // list is up-to-date.
        assert_eq!(
            necessary_phases,
            ReflowPhases::StackingContextTreeConstruction | ReflowPhases::DisplayListConstruction
        );
        !self.need_new_display_list.get()
    }

    fn maybe_print_reflow_event(&self, reflow_request: &ReflowRequest) {
        if !self.debug.relayout_event {
            return;
        }

        println!(
            "**** Reflow({}) => {:?}, {:?}",
            self.id,
            reflow_request.reflow_goal,
            reflow_request
                .restyle
                .as_ref()
                .map(|restyle| restyle.reason)
                .unwrap_or_default()
        );
    }

    /// Checks whether we need to update the scroll node, and report whether the
    /// node is scrolled. We need to update the scroll node whenever it is requested.
    fn handle_update_scroll_node_request(&self, reflow_request: &ReflowRequest) -> bool {
        if let ReflowGoal::UpdateScrollNode(external_scroll_id, offset) = reflow_request.reflow_goal
        {
            self.set_scroll_offset_from_script(external_scroll_id, offset)
        } else {
            false
        }
    }

    /// The high-level routine that performs layout.
    #[servo_tracing::instrument(skip_all)]
    fn handle_reflow(&mut self, mut reflow_request: ReflowRequest) -> Option<ReflowResult> {
        self.maybe_print_reflow_event(&reflow_request);

        if self.can_skip_reflow_request_entirely(&reflow_request) {
            // We can skip layout, but we might need to update a scroll node.
            return self
                .handle_update_scroll_node_request(&reflow_request)
                .then(|| ReflowResult {
                    reflow_phases_run: ReflowPhasesRun::UpdatedScrollNodeOffset,
                    ..Default::default()
                });
        }

        let document = unsafe { ServoLayoutNode::new(&reflow_request.document) };
        let document = document.as_document().unwrap();
        let Some(root_element) = document.root_element() else {
            debug!("layout: No root node: bailing");
            return None;
        };

        let image_resolver = Arc::new(ImageResolver {
            origin: reflow_request.origin.clone(),
            image_cache: self.image_cache.clone(),
            resolved_images_cache: self.resolved_images_cache.clone(),
            pending_images: Mutex::default(),
            pending_rasterization_images: Mutex::default(),
            pending_svg_elements_for_serialization: Mutex::default(),
            animating_images: reflow_request.animating_images.clone(),
            animation_timeline_value: reflow_request.animation_timeline_value,
        });

        let (mut reflow_phases_run, iframe_sizes) = self.restyle_and_build_trees(
            &mut reflow_request,
            document,
            root_element,
            &image_resolver,
        );
        if self.calculate_overflow() {
            reflow_phases_run.insert(ReflowPhasesRun::CalculatedOverflow);
        }
        if self.build_stacking_context_tree_for_reflow(&reflow_request) {
            reflow_phases_run.insert(ReflowPhasesRun::BuiltStackingContextTree);
        }
        if self.build_display_list(&reflow_request, &image_resolver) {
            reflow_phases_run.insert(ReflowPhasesRun::BuiltDisplayList);
        }
        if self.handle_update_scroll_node_request(&reflow_request) {
            reflow_phases_run.insert(ReflowPhasesRun::UpdatedScrollNodeOffset);
        }

        let pending_images = std::mem::take(&mut *image_resolver.pending_images.lock());
        let pending_rasterization_images =
            std::mem::take(&mut *image_resolver.pending_rasterization_images.lock());
        let pending_svg_elements_for_serialization =
            std::mem::take(&mut *image_resolver.pending_svg_elements_for_serialization.lock());

        Some(ReflowResult {
            reflow_phases_run,
            pending_images,
            pending_rasterization_images,
            pending_svg_elements_for_serialization,
            iframe_sizes: Some(iframe_sizes),
        })
    }

    #[servo_tracing::instrument(skip_all)]
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
                self.load_all_web_fonts_from_stylesheet_with_guard(
                    stylesheet,
                    guards.ua_or_user,
                    &reflow_request.document_context,
                );
            }

            if self.stylist.quirks_mode() == QuirksMode::Quirks {
                self.stylist.append_stylesheet(
                    ua_stylesheets.quirks_mode_stylesheet.clone(),
                    guards.ua_or_user,
                );
                self.load_all_web_fonts_from_stylesheet_with_guard(
                    &ua_stylesheets.quirks_mode_stylesheet,
                    guards.ua_or_user,
                    &reflow_request.document_context,
                );
            }
            self.have_added_user_agent_stylesheets = true;
        }

        if reflow_request.stylesheets_changed() {
            self.stylist
                .force_stylesheet_origins_dirty(Origin::Author.into());
        }

        // Flush shadow roots stylesheets if dirty.
        document.flush_shadow_roots_stylesheets(&mut self.stylist, guards.author);

        self.stylist
            .flush(guards, Some(root_element), Some(snapshot_map));
    }

    #[servo_tracing::instrument(skip_all)]
    fn restyle_and_build_trees(
        &mut self,
        reflow_request: &mut ReflowRequest,
        document: ServoLayoutDocument<'_>,
        root_element: ServoLayoutElement<'_>,
        image_resolver: &Arc<ImageResolver>,
    ) -> (ReflowPhasesRun, IFrameSizes) {
        let mut snapshot_map = SnapshotMap::new();
        let _snapshot_setter = match reflow_request.restyle.as_mut() {
            Some(restyle) => SnapshotSetter::new(restyle, &mut snapshot_map),
            None => return Default::default(),
        };

        let document_shared_lock = document.style_shared_lock();
        let author_guard = document_shared_lock.read();
        let ua_stylesheets = &*UA_STYLESHEETS;
        let ua_or_user_guard = ua_stylesheets.shared_lock.read();
        let guards = StylesheetGuards {
            author: &author_guard,
            ua_or_user: &ua_or_user_guard,
        };

        let rayon_pool = STYLE_THREAD_POOL.lock();
        let rayon_pool = rayon_pool.pool();
        let rayon_pool = rayon_pool.as_ref();

        let device_has_changed = std::mem::replace(&mut self.device_has_changed, false);
        if device_has_changed {
            let sheet_origins_affected_by_device_change = self
                .stylist
                .media_features_change_changed_style(&guards, self.device());
            self.stylist
                .force_stylesheet_origins_dirty(sheet_origins_affected_by_device_change);

            if let Some(mut data) = root_element.mutate_data() {
                data.hint.insert(RestyleHint::recascade_subtree());
            }
        }

        self.prepare_stylist_for_reflow(
            reflow_request,
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

        let layout_context = LayoutContext {
            style_context: self.build_shared_style_context(
                guards,
                &snapshot_map,
                reflow_request.animation_timeline_value,
                &reflow_request.animations,
                match reflow_request.stylesheets_changed() {
                    true => TraversalFlags::ForCSSRuleChanges,
                    false => TraversalFlags::empty(),
                },
            ),
            font_context: self.font_context.clone(),
            iframe_sizes: Mutex::default(),
            use_rayon: rayon_pool.is_some(),
            image_resolver: image_resolver.clone(),
            painter_id: self.webview_id.into(),
        };

        let restyle = reflow_request
            .restyle
            .as_ref()
            .expect("Should not get here if there is not restyle.");

        let recalc_style_traversal;
        let dirty_root;
        {
            let _span = profile_traits::trace_span!("Styling").entered();

            let original_dirty_root = unsafe {
                ServoLayoutNode::new(&restyle.dirty_root.unwrap())
                    .as_element()
                    .unwrap()
            };

            recalc_style_traversal = RecalcStyle::new(&layout_context);
            let token = {
                let shared =
                    DomTraversal::<ServoLayoutElement>::shared_context(&recalc_style_traversal);
                RecalcStyle::pre_traverse(original_dirty_root, shared)
            };

            if !token.should_traverse() {
                layout_context.style_context.stylist.rule_tree().maybe_gc();
                return Default::default();
            }

            dirty_root = driver::traverse_dom(&recalc_style_traversal, token, rayon_pool).as_node();
        }

        let root_node = root_element.as_node();
        let damage_from_environment = if device_has_changed {
            RestyleDamage::RELAYOUT
        } else {
            Default::default()
        };

        let damage = compute_damage_and_repair_style(
            &layout_context.style_context,
            root_node.to_threadsafe(),
            damage_from_environment,
        );
        if damage.contains(RestyleDamage::RECALCULATE_OVERFLOW) {
            self.need_overflow_calculation.set(true);
        }
        if damage.contains(RestyleDamage::REBUILD_STACKING_CONTEXT) {
            self.need_new_stacking_context_tree.set(true);
        }
        if damage.contains(RestyleDamage::REPAINT) {
            self.need_new_display_list.set(true);
        }

        if !damage.contains(RestyleDamage::RELAYOUT) {
            layout_context.style_context.stylist.rule_tree().maybe_gc();
            return (ReflowPhasesRun::empty(), IFrameSizes::default());
        }

        let mut box_tree = self.box_tree.borrow_mut();
        let box_tree = &mut *box_tree;
        let layout_damage: LayoutDamage = damage.into();
        if box_tree.is_none() || layout_damage.has_box_damage() {
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
        }

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

        if self.debug.style_tree {
            println!(
                "{:?}",
                ShowSubtreeDataAndPrimaryValues(root_element.as_node())
            );
        }
        if self.debug.rule_tree {
            recalc_style_traversal
                .context()
                .style_context
                .stylist
                .rule_tree()
                .dump_stdout(&layout_context.style_context.guards);
        }

        // GC the rule tree if some heuristics are met.
        layout_context.style_context.stylist.rule_tree().maybe_gc();

        let mut iframe_sizes = layout_context.iframe_sizes.lock();
        (
            ReflowPhasesRun::RanLayout,
            std::mem::take(&mut *iframe_sizes),
        )
    }

    #[servo_tracing::instrument(name = "Overflow Calculation", skip_all)]
    fn calculate_overflow(&self) -> bool {
        if !self.need_overflow_calculation.get() {
            return false;
        }

        if let Some(fragment_tree) = &*self.fragment_tree.borrow() {
            fragment_tree.calculate_scrollable_overflow();
            if self.debug.flow_tree {
                fragment_tree.print();
            }
        }

        self.need_overflow_calculation.set(false);
        assert!(self.need_new_display_list.get());
        assert!(self.need_new_stacking_context_tree.get());

        true
    }

    fn build_stacking_context_tree_for_reflow(&self, reflow_request: &ReflowRequest) -> bool {
        if !ReflowPhases::necessary(&reflow_request.reflow_goal)
            .contains(ReflowPhases::StackingContextTreeConstruction)
        {
            return false;
        }
        if !self.need_new_stacking_context_tree.get() {
            return false;
        }

        self.build_stacking_context_tree(reflow_request.viewport_details)
    }

    #[servo_tracing::instrument(name = "Stacking Context Tree Construction", skip_all)]
    fn build_stacking_context_tree(&self, viewport_details: ViewportDetails) -> bool {
        let Some(fragment_tree) = &*self.fragment_tree.borrow() else {
            return false;
        };

        let mut stacking_context_tree = self.stacking_context_tree.borrow_mut();
        let old_scroll_offsets = stacking_context_tree
            .as_ref()
            .map(|tree| tree.paint_info.scroll_tree.scroll_offsets());

        // Build the StackingContextTree. This turns the `FragmentTree` into a
        // tree of fragments in CSS painting order and also creates all
        // applicable spatial and clip nodes.
        let mut new_stacking_context_tree = StackingContextTree::new(
            fragment_tree,
            viewport_details,
            self.id.into(),
            !self.have_ever_generated_display_list.get(),
            &self.debug,
        );

        // When a new StackingContextTree is built, it contains a freshly built
        // ScrollTree. We want to preserve any existing scroll offsets in that tree,
        // adjusted by any new scroll constraints.
        if let Some(old_scroll_offsets) = old_scroll_offsets {
            new_stacking_context_tree
                .paint_info
                .scroll_tree
                .set_all_scroll_offsets(&old_scroll_offsets);
        }

        if self.debug.scroll_tree {
            new_stacking_context_tree
                .paint_info
                .scroll_tree
                .debug_print();
        }

        *stacking_context_tree = Some(new_stacking_context_tree);

        // The stacking context tree is up-to-date again.
        self.need_new_stacking_context_tree.set(false);
        assert!(self.need_new_display_list.get());

        true
    }

    /// Build the display list for the current layout and send it to the renderer. If no display
    /// list is built, returns false.
    #[servo_tracing::instrument(name = "Display List Construction", skip_all)]
    fn build_display_list(
        &self,
        reflow_request: &ReflowRequest,
        image_resolver: &Arc<ImageResolver>,
    ) -> bool {
        if !ReflowPhases::necessary(&reflow_request.reflow_goal)
            .contains(ReflowPhases::DisplayListConstruction)
        {
            return false;
        }
        let Some(fragment_tree) = &*self.fragment_tree.borrow() else {
            return false;
        };
        let mut stacking_context_tree = self.stacking_context_tree.borrow_mut();
        let Some(stacking_context_tree) = stacking_context_tree.as_mut() else {
            return false;
        };

        // If a non-display-list-generating reflow updated layout in a previous refow, we
        // cannot skip display list generation here the next time a display list is
        // requested.
        if !self.need_new_display_list.get() {
            return false;
        }

        // TODO: Eventually this should be set when `paint_info` is created, but that requires
        // ensuring that the Epoch is passed to any method that can creates `StackingContextTree`.
        stacking_context_tree.paint_info.epoch = reflow_request.epoch;

        let mut lcp_candidate_collector = self.lcp_candidate_collector.borrow_mut();
        if pref!(largest_contentful_paint_enabled) {
            // This ensures that we only create the LCP collector once per layout thread.
            if lcp_candidate_collector.is_none() {
                *lcp_candidate_collector = Some(LargestContentfulPaintCandidateCollector::new(
                    stacking_context_tree
                        .paint_info
                        .viewport_details
                        .layout_size(),
                ));
            }
        } else {
            *lcp_candidate_collector = None;
        }

        let built_display_list = DisplayListBuilder::build(
            stacking_context_tree,
            fragment_tree,
            image_resolver.clone(),
            self.device().device_pixel_ratio(),
            reflow_request.highlighted_dom_node,
            &self.debug,
            lcp_candidate_collector.as_mut(),
        );
        self.paint_api.send_display_list(
            self.webview_id,
            &stacking_context_tree.paint_info,
            built_display_list,
        );
        if let Some(lcp_candidate_collector) = lcp_candidate_collector.as_mut() {
            if lcp_candidate_collector.did_lcp_candidate_update {
                if let Some(lcp_candidate) = lcp_candidate_collector.largest_contentful_paint() {
                    self.paint_api.send_lcp_candidate(
                        lcp_candidate,
                        self.webview_id,
                        self.id,
                        stacking_context_tree.paint_info.epoch,
                    );
                    lcp_candidate_collector.did_lcp_candidate_update = false;
                }
            }
        }

        let (keys, instance_keys) = self
            .font_context
            .collect_unused_webrender_resources(false /* all */);
        self.paint_api
            .remove_unused_font_resources(self.webview_id.into(), keys, instance_keys);

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
    ) -> bool {
        let mut stacking_context_tree = self.stacking_context_tree.borrow_mut();
        let Some(stacking_context_tree) = stacking_context_tree.as_mut() else {
            return false;
        };

        if let Some(offset) = stacking_context_tree
            .paint_info
            .scroll_tree
            .set_scroll_offset_for_node_with_external_scroll_id(
                external_scroll_id,
                offset,
                ScrollType::Script,
            )
        {
            self.paint_api.scroll_node_by_delta(
                self.webview_id,
                self.id.into(),
                offset,
                external_scroll_id,
            );
            true
        } else {
            false
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
            ServoArc::new(shared_lock.wrap(MediaList::empty())),
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
                ServoArc::new(shared_lock.wrap(MediaList::empty())),
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

struct RegisteredPaintersImpl(HashMap<Atom, RegisteredPainterImpl>);

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
                    .find_by_codepoint(font_context, '0', None, None, None)?
                    .metrics
                    .zero_horizontal_advance
            })
            .map(CSSPixelLength::from);

        let ic_width = first_font_metrics
            .ic_horizontal_advance
            .or_else(|| {
                font_group
                    .write()
                    .find_by_codepoint(font_context, '\u{6C34}', None, None, None)?
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
    fn new(restyle: &mut ReflowRequestRestyle, snapshot_map: &mut SnapshotMap) -> Self {
        debug!("Draining restyles: {}", restyle.pending_restyles.len());
        let restyles = std::mem::take(&mut restyle.pending_restyles);

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

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ReflowPhases: u8 {
        const StackingContextTreeConstruction = 1 << 0;
        const DisplayListConstruction = 1 << 1;
    }
}

impl ReflowPhases {
    /// Return the necessary phases of layout for the given [`ReflowGoal`]. Note that all
    /// [`ReflowGoals`] need the basic restyle + box tree layout + fragment tree layout,
    /// so [`ReflowPhases::empty()`] implies that.
    fn necessary(reflow_goal: &ReflowGoal) -> Self {
        match reflow_goal {
            ReflowGoal::LayoutQuery(query) => match query {
                QueryMsg::NodesFromPointQuery => {
                    Self::StackingContextTreeConstruction | Self::DisplayListConstruction
                },
                QueryMsg::BoxArea |
                QueryMsg::BoxAreas |
                QueryMsg::ElementsFromPoint |
                QueryMsg::OffsetParentQuery |
                QueryMsg::ResolvedStyleQuery |
                QueryMsg::ScrollingAreaOrOffsetQuery => Self::StackingContextTreeConstruction,
                QueryMsg::ClientRectQuery |
                QueryMsg::CurrentCSSZoomQuery |
                QueryMsg::ElementInnerOuterTextQuery |
                QueryMsg::InnerWindowDimensionsQuery |
                QueryMsg::PaddingQuery |
                QueryMsg::ResolvedFontStyleQuery |
                QueryMsg::ScrollParentQuery |
                QueryMsg::StyleQuery |
                QueryMsg::TextIndexQuery => Self::empty(),
            },
            ReflowGoal::UpdateScrollNode(..) | ReflowGoal::UpdateTheRendering => {
                Self::StackingContextTreeConstruction | Self::DisplayListConstruction
            },
        }
    }
}
