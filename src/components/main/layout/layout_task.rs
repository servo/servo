/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout task. Performs layout on the DOM, builds display lists and sends them to be
/// rendered.

use css::matching::MatchMethods;
use css::select::new_stylist;
use css::node_style::StyledNode;
use layout::construct::{FlowConstructionResult, FlowConstructor, NoConstructionResult};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ToGfxColor};
use layout::extra::LayoutAuxMethods;
use layout::flow::{FlowContext, ImmutableFlowUtils, MutableFlowUtils, PreorderFlowTraversal};
use layout::flow::{PostorderFlowTraversal};
use layout::flow;
use layout::incremental::{RestyleDamage, BubbleWidths};
use layout::util::{LayoutData, LayoutDataAccess};

use extra::arc::{Arc, RWArc};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::display_list::{DisplayList,DisplayItem,ClipDisplayItemClass};
use gfx::font_context::FontContext;
use gfx::opts::Opts;
use gfx::render_task::{RenderMsg, RenderChan, RenderLayer};
use gfx::{render_task, color};
use script::dom::event::ReflowEvent;
use script::dom::node::{AbstractNode, LayoutDataRef, LayoutView, ElementNodeTypeId};
use script::dom::element::{HTMLBodyElementTypeId, HTMLHtmlElementTypeId};
use script::layout_interface::{AddStylesheetMsg, ContentBoxQuery};
use script::layout_interface::{ContentBoxesQuery, ContentBoxesResponse, ExitNowMsg, LayoutQuery};
use script::layout_interface::{HitTestQuery, ContentBoxResponse, HitTestResponse};
use script::layout_interface::{MatchSelectorsDocumentDamage, Msg, PrepareToExitMsg};
use script::layout_interface::{QueryMsg, ReapLayoutDataMsg, Reflow, ReflowDocumentDamage};
use script::layout_interface::{ReflowForDisplay, ReflowMsg};
use script::script_task::{ReflowCompleteMsg, ScriptChan, SendEventMsg};
use servo_msg::constellation_msg::{ConstellationChan, PipelineId};
use servo_net::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use servo_net::local_image_cache::{ImageResponder, LocalImageCache};
use servo_util::geometry::Au;
use servo_util::time::{ProfilerChan, profile};
use servo_util::time;
use servo_util::tree::TreeNodeRef;
use std::cast::transmute;
use std::cast;
use std::cell::Cell;
use std::comm::Port;
use std::task;
use std::util;
use style::AuthorOrigin;
use style::Stylesheet;
use style::Stylist;

/// Information needed by the layout task.
struct LayoutTask {
    /// The ID of the pipeline that we belong to.
    id: PipelineId,

    /// The port on which we receive messages.
    port: Port<Msg>,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the script task.
    script_chan: ScriptChan,

    /// The channel on which messages can be sent to the painting task.
    render_chan: RenderChan<AbstractNode<()>>,

    /// The channel on which messages can be sent to the image cache.
    image_cache_task: ImageCacheTask,

    /// The local image cache.
    local_image_cache: @mut LocalImageCache,

    /// The local font context.
    font_ctx: @mut FontContext,

    /// The size of the viewport.
    screen_size: Option<Size2D<Au>>,

    /// A cached display list.
    display_list: Option<Arc<DisplayList<AbstractNode<()>>>>,

    stylist: RWArc<Stylist>,

    /// The channel on which messages can be sent to the profiler.
    profiler_chan: ProfilerChan,
}

/// The damage computation traversal.
#[deriving(Clone)]
struct ComputeDamageTraversal;

impl PostorderFlowTraversal for ComputeDamageTraversal {
    #[inline]
    fn process(&mut self, flow: &mut FlowContext) -> bool {
        let mut damage = flow::base(flow).restyle_damage;
        for child in flow::child_iter(flow) {
            damage.union_in_place(flow::base(*child).restyle_damage)
        }
        flow::mut_base(flow).restyle_damage = damage;
        true
    }
}

/// Propagates restyle damage up and down the tree as appropriate.
///
/// FIXME(pcwalton): Merge this with flow tree building and/or other traversals.
struct PropagateDamageTraversal {
    resized: bool,
}

impl PreorderFlowTraversal for PropagateDamageTraversal {
    #[inline]
    fn process(&mut self, flow: &mut FlowContext) -> bool {
        // Also set any damage implied by resize.
        if self.resized {
            flow::mut_base(flow).restyle_damage.union_in_place(RestyleDamage::for_resize())
        }

        let prop = flow::base(flow).restyle_damage.propagate_down();
        if prop.is_nonempty() {
            for kid_ctx in flow::child_iter(flow) {
                flow::mut_base(*kid_ctx).restyle_damage.union_in_place(prop)
            }
        }
        true
    }
}

/// The bubble-widths traversal, the first part of layout computation. This computes preferred
/// and intrinsic widths and bubbles them up the tree.
struct BubbleWidthsTraversal<'self>(&'self mut LayoutContext);

impl<'self> PostorderFlowTraversal for BubbleWidthsTraversal<'self> {
    #[inline]
    fn process(&mut self, flow: &mut FlowContext) -> bool {
        flow.bubble_widths(**self);
        true
    }

    #[inline]
    fn should_prune(&mut self, flow: &mut FlowContext) -> bool {
        flow::mut_base(flow).restyle_damage.lacks(BubbleWidths)
    }
}

/// The assign-widths traversal. In Gecko this corresponds to `Reflow`.
struct AssignWidthsTraversal<'self>(&'self mut LayoutContext);

impl<'self> PreorderFlowTraversal for AssignWidthsTraversal<'self> {
    #[inline]
    fn process(&mut self, flow: &mut FlowContext) -> bool {
        flow.assign_widths(**self);
        true
    }
}

/// The assign-heights-and-store-overflow traversal, the last (and most expensive) part of layout
/// computation. Determines the final heights for all layout objects, computes positions, and
/// computes overflow regions. In Gecko this corresponds to `FinishAndStoreOverflow`.
struct AssignHeightsAndStoreOverflowTraversal<'self>(&'self mut LayoutContext);

impl<'self> PostorderFlowTraversal for AssignHeightsAndStoreOverflowTraversal<'self> {
    #[inline]
    fn process(&mut self, flow: &mut FlowContext) -> bool {
        flow.assign_height(**self);
        flow.store_overflow(**self);
        true
    }

    #[inline]
    fn should_process(&mut self, flow: &mut FlowContext) -> bool {
        !flow::base(flow).is_inorder
    }
}

struct LayoutImageResponder {
    id: PipelineId,
    script_chan: ScriptChan,
}

impl ImageResponder for LayoutImageResponder {
    fn respond(&self) -> ~fn(ImageResponseMsg) {
        let id = self.id.clone();
        let script_chan = self.script_chan.clone();
        let f: ~fn(ImageResponseMsg) = |_| {
            script_chan.send(SendEventMsg(id.clone(), ReflowEvent))
        };
        f
    }
}

impl LayoutTask {
    /// Spawns a new layout task.
    pub fn create(id: PipelineId,
                  port: Port<Msg>,
                  constellation_chan: ConstellationChan,
                  script_chan: ScriptChan,
                  render_chan: RenderChan<AbstractNode<()>>,
                  img_cache_task: ImageCacheTask,
                  opts: Opts,
                  profiler_chan: ProfilerChan) {
        spawn_with!(task::task(), [port, constellation_chan, script_chan,
                                   render_chan, img_cache_task, profiler_chan], {
            let mut layout = LayoutTask::new(id,
                                             port,
                                             constellation_chan,
                                             script_chan,
                                             render_chan,
                                             img_cache_task,
                                             &opts,
                                             profiler_chan);
            layout.start();
        });
    }

    /// Creates a new `LayoutTask` structure.
    fn new(id: PipelineId,
           port: Port<Msg>,
           constellation_chan: ConstellationChan,
           script_chan: ScriptChan,
           render_chan: RenderChan<AbstractNode<()>>, 
           image_cache_task: ImageCacheTask,
           opts: &Opts,
           profiler_chan: ProfilerChan)
           -> LayoutTask {
        let fctx = @mut FontContext::new(opts.render_backend, true, profiler_chan.clone());

        LayoutTask {
            id: id,
            port: port,
            constellation_chan: constellation_chan,
            script_chan: script_chan,
            render_chan: render_chan,
            image_cache_task: image_cache_task.clone(),
            local_image_cache: @mut LocalImageCache(image_cache_task),
            font_ctx: fctx,
            screen_size: None,

            display_list: None,

            stylist: RWArc::new(new_stylist()),
            profiler_chan: profiler_chan,
        }
    }

    /// Starts listening on the port.
    fn start(&mut self) {
        while self.handle_request() {
            // Loop indefinitely.
        }
    }

    // Create a layout context for use in building display lists, hit testing, &c.
    fn build_layout_context(&self) -> LayoutContext {
        let image_cache = self.local_image_cache;
        let font_ctx = self.font_ctx;
        let screen_size = self.screen_size.unwrap();

        LayoutContext {
            image_cache: image_cache,
            font_ctx: font_ctx,
            screen_size: Rect(Point2D(Au(0), Au(0)), screen_size),
        }
    }

    /// Receives and dispatches messages from the port.
    fn handle_request(&mut self) -> bool {
        match self.port.recv() {
            AddStylesheetMsg(sheet) => self.handle_add_stylesheet(sheet),
            ReflowMsg(data) => {
                let data = Cell::new(data);

                do profile(time::LayoutPerformCategory, self.profiler_chan.clone()) {
                    self.handle_reflow(data.take());
                }
            }
            QueryMsg(query) => {
                let query = Cell::new(query);
                do profile(time::LayoutQueryCategory, self.profiler_chan.clone()) {
                    self.handle_query(query.take());
                }
            }
            ReapLayoutDataMsg(dead_layout_data) => {
                unsafe {
                    self.handle_reap_layout_data(dead_layout_data)
                }
            }
            PrepareToExitMsg(response_chan) => {
                self.prepare_to_exit(response_chan)
            }
            ExitNowMsg => {
                debug!("layout: ExitNowMsg received");
                self.exit_now();
                return false
            }
        }

        true
    }

    /// Enters a quiescent state in which no new messages except for `ReapLayoutDataMsg` will be
    /// processed until an `ExitNowMsg` is received. A pong is immediately sent on the given
    /// response channel.
    fn prepare_to_exit(&mut self, response_chan: Chan<()>) {
        response_chan.send(());
        match self.port.recv() {
            ReapLayoutDataMsg(dead_layout_data) => {
                unsafe {
                    self.handle_reap_layout_data(dead_layout_data)
                }
            }
            ExitNowMsg => self.exit_now(),
            _ => {
                fail!("layout: message that wasn't `ExitNowMsg` received after `PrepareToExitMsg`")
            }
        }
    }

    /// Shuts down the layout task now. If there are any DOM nodes left, layout will now (safely)
    /// crash.
    fn exit_now(&mut self) {
        let (response_port, response_chan) = stream();
        self.render_chan.send(render_task::ExitMsg(response_chan));
        response_port.recv()
    }

    fn handle_add_stylesheet(&mut self, sheet: Stylesheet) {
        let sheet = Cell::new(sheet);
        do self.stylist.write |stylist| {
            stylist.add_stylesheet(sheet.take(), AuthorOrigin)
        }
    }

    /// Builds the flow tree.
    ///
    /// This corresponds to the various `nsCSSFrameConstructor` methods in Gecko or
    /// `createRendererIfNeeded` in WebKit. Note, however that in WebKit `createRendererIfNeeded`
    /// is intertwined with selector matching, making it difficult to compare directly. It is
    /// marked `#[inline(never)]` to aid benchmarking in sampling profilers.
    #[inline(never)]
    fn construct_flow_tree(&self, layout_context: &LayoutContext, node: AbstractNode<LayoutView>)
                           -> ~FlowContext: {
        node.traverse_postorder(&FlowConstructor::init(layout_context));

        let result = match *node.mutate_layout_data().ptr {
            Some(ref mut layout_data) => {
                util::replace(&mut layout_data.flow_construction_result, NoConstructionResult)
            }
            None => fail!("no layout data for root node"),
        };
        let mut flow = match result {
            FlowConstructionResult(flow) => flow,
            _ => fail!("Flow construction didn't result in a flow at the root of the tree!"),
        };
        flow.mark_as_root();
        flow
    }

    /// Performs layout constraint solving.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints(&mut self,
                         layout_root: &mut FlowContext,
                         layout_context: &mut LayoutContext) {
        let _ = layout_root.traverse_postorder(&mut BubbleWidthsTraversal(layout_context));

        // FIXME(kmc): We want to do
        //     for flow in layout_root.traverse_preorder_prune(|f|
        //          f.restyle_damage().lacks(Reflow)) 
        // but FloatContext values can't be reused, so we need to recompute them every time.
        // NOTE: this currently computes borders, so any pruning should separate that operation out.
        let _ = layout_root.traverse_preorder(&mut AssignWidthsTraversal(layout_context));

        // For now, this is an inorder traversal
        // FIXME: prune this traversal as well
        let _ = layout_root.traverse_postorder(&mut
            AssignHeightsAndStoreOverflowTraversal(layout_context));
    }

    /// The high-level routine that performs layout tasks.
    fn handle_reflow(&mut self, data: &Reflow) {
        // FIXME: Isolate this transmutation into a "bridge" module.
        let node: &AbstractNode<LayoutView> = unsafe {
            transmute(&data.document_root)
        };

        debug!("layout: received layout request for: {:s}", data.url.to_str());
        debug!("layout: damage is {:?}", data.damage);
        debug!("layout: parsed Node tree");
        debug!("{:?}", node.dump());

        // Reset the image cache.
        self.local_image_cache.next_round(self.make_on_image_available_cb());

        let screen_size = Size2D(Au::from_px(data.window_size.width as int),
                                 Au::from_px(data.window_size.height as int));
        let resized = self.screen_size != Some(screen_size);
        debug!("resized: {}", resized);
        self.screen_size = Some(screen_size);

        // Create a layout context for use throughout the following passes.
        let mut layout_ctx = self.build_layout_context();

        // Initialize layout data for each node.
        //
        // FIXME: This is inefficient. We don't need an entire traversal to do this!
        do profile(time::LayoutAuxInitCategory, self.profiler_chan.clone()) {
            node.initialize_style_for_subtree();
        }

        // Perform CSS selector matching if necessary.
        match data.damage.level {
            ReflowDocumentDamage => {}
            MatchSelectorsDocumentDamage => {
                do profile(time::LayoutSelectorMatchCategory, self.profiler_chan.clone()) {
                    node.match_subtree(self.stylist.clone());
                    node.cascade_subtree(None);
                }
            }
        }

        // Construct the flow tree.
        let mut layout_root = profile(time::LayoutTreeBuilderCategory,
                                      self.profiler_chan.clone(),
                                      || self.construct_flow_tree(&layout_ctx, *node));

        // Propagate damage.
        layout_root.traverse_preorder(&mut PropagateDamageTraversal {
            resized: resized,
        });
        layout_root.traverse_postorder(&mut ComputeDamageTraversal.clone());

        debug!("layout: constructed Flow tree");
        debug!("{:?}", layout_root.dump());

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        do profile(time::LayoutMainCategory, self.profiler_chan.clone()) {
            self.solve_constraints(layout_root, &mut layout_ctx)
        }

        // Build the display list if necessary, and send it to the renderer.
        if data.goal == ReflowForDisplay {
            do profile(time::LayoutDispListBuildCategory, self.profiler_chan.clone()) {
                let root_size = flow::base(layout_root).position.size;
                let display_list= ~Cell::new(DisplayList::<AbstractNode<()>>::new());
                let dirty = flow::base(layout_root).position.clone();
                layout_root.build_display_list(
                    &DisplayListBuilder {
                        ctx: &layout_ctx,
                    },
                    &dirty,
                    display_list);

                let display_list = Arc::new(display_list.take());

                for i in range(0,display_list.get().list.len()) {
                    self.display_item_bound_to_node(&display_list.get().list[i]);
                }

                    let mut color = color::rgba(255.0, 255.0, 255.0, 255.0);

                    for child in node.traverse_preorder() {
                      if child.type_id() == ElementNodeTypeId(HTMLHtmlElementTypeId) || 
                         child.type_id() == ElementNodeTypeId(HTMLBodyElementTypeId) {
                             let element_bg_color = child.style().resolve_color(
                                 child.style().Background.background_color
                             ).to_gfx_color();
                             match element_bg_color {
                                 color::rgba(0., 0., 0., 0.) => {}
                                 _ => {
                                   color = element_bg_color;
                                   break;
                               }
                             }
                      }
                    }

                let render_layer = RenderLayer {
                    display_list: display_list.clone(),
                    size: Size2D(root_size.width.to_nearest_px() as uint,
                                 root_size.height.to_nearest_px() as uint),
                    color: color
                };

                self.display_list = Some(display_list.clone());

                self.render_chan.send(RenderMsg(render_layer));
            } // time(layout: display list building)
        }

        // Tell script that we're done.
        //
        // FIXME(pcwalton): This should probably be *one* channel, but we can't fix this without
        // either select or a filtered recv() that only looks for messages of a given type.
        data.script_join_chan.send(());
        data.script_chan.send(ReflowCompleteMsg(self.id, data.id));
    }

    fn display_item_bound_to_node(&mut self,item: &DisplayItem<AbstractNode<()>>) {
        let node: AbstractNode<LayoutView> = unsafe {
            transmute(item.base().extra)
        };

        match *node.mutate_layout_data().ptr {
            Some(ref mut layout_data) => {
                let boxes = &mut layout_data.boxes;
                if boxes.display_bound_list.is_none() || boxes.display_bound.is_none() {
                    boxes.display_bound_list = Some(~[]);
                    boxes.display_bound = Some(Au::zero_rect());
                }

                match boxes.display_bound_list {
                    Some(ref mut list) => list.push(item.base().bounds),
                    None => {}
                }
                match boxes.display_bound {
                    Some(ref mut bounds) => *bounds = bounds.union(&item.base().bounds),
                    None => {}
                }
            }
            None => fail!("no layout data"),
        }

        match *item {
            ClipDisplayItemClass(ref cc) => {
                for item in cc.child_list.iter() {
                    self.display_item_bound_to_node(item);
                }
            }
            _ => {}
        }
    }

    /// Handles a query from the script task. This is the main routine that DOM functions like
    /// `getClientRects()` or `getBoundingClientRect()` ultimately invoke.
    fn handle_query(&self, query: LayoutQuery) {
        match query {
            ContentBoxQuery(node, reply_chan) => {
                // FIXME: Isolate this transmutation into a single "bridge" module.
                let node: AbstractNode<LayoutView> = unsafe {
                    transmute(node)
                };

                fn box_for_node(node: AbstractNode<LayoutView>) -> Option<Rect<Au>> {
                    // FIXME(pcwalton): Why are we cloning the display list here?!
                    let layout_data = node.borrow_layout_data();
                    let boxes = &layout_data.ptr.as_ref().unwrap().boxes;
                    match boxes.display_bound {
                        Some(_) => boxes.display_bound.clone(),
                        _ => {
                            let mut acc: Option<Rect<Au>> = None;
                            for child in node.children() {
                                let rect = box_for_node(child);
                                match rect {
                                    None => continue,
                                    Some(rect) => acc = match acc {
                                        Some(acc) =>  Some(acc.union(&rect)),
                                        None => Some(rect)
                                    }
                                }
                            }
                            acc
                        }
                    }
                }

                let rect = box_for_node(node).unwrap_or(Rect(Point2D(Au(0), Au(0)),
                                                             Size2D(Au(0), Au(0))));
                reply_chan.send(ContentBoxResponse(rect))
            }
            ContentBoxesQuery(node, reply_chan) => {
                // FIXME: Isolate this transmutation into a single "bridge" module.
                let node: AbstractNode<LayoutView> = unsafe {
                    transmute(node)
                };

                fn boxes_for_node(node: AbstractNode<LayoutView>, mut box_accumulator: ~[Rect<Au>])
                                  -> ~[Rect<Au>] {
                    let layout_data = node.borrow_layout_data();
                    let boxes = &layout_data.ptr.as_ref().unwrap().boxes;
                    match boxes.display_bound_list {
                        Some(ref display_bound_list) => {
                            for item in display_bound_list.iter() {
                                box_accumulator.push(*item);
                            }
                        }
                        _ => {
                            for child in node.children() {
                                box_accumulator = boxes_for_node(child, box_accumulator);
                            }
                        }
                    }
                    box_accumulator
                }

                let mut boxes = ~[];
                boxes = boxes_for_node(node, boxes);
                reply_chan.send(ContentBoxesResponse(boxes))
            }
            HitTestQuery(_, point, reply_chan) => {
                fn hit_test(x:Au, y:Au, list: &[DisplayItem<AbstractNode<()>>]) -> Option<HitTestResponse> {

                    for item in list.rev_iter() {
                        match *item {
                            ClipDisplayItemClass(ref cc) => {
                                let ret = hit_test(x, y, cc.child_list);
                                if !ret.is_none() {
                                    return ret;
                                }
                            }
                            _ => {}
                        }
                    }

                    for item in list.rev_iter() {
                        match *item {
                            ClipDisplayItemClass(_) => continue,
                            _ => {}
                        }
                        let bounds = item.bounds();
                        // TODO this check should really be performed by a method of DisplayItem
                        if x < bounds.origin.x + bounds.size.width &&
                            bounds.origin.x <= x &&
                            y < bounds.origin.y + bounds.size.height &&
                            bounds.origin.y <= y {
                            let node: AbstractNode<LayoutView> = unsafe {
                                transmute(item.base().extra)
                            };
                            let resp = Some(HitTestResponse(node));
                            return resp;
                        }
                    }

                    let ret: Option<HitTestResponse> = None;
                    ret
                }
                let response = {
                    match self.display_list {
                        Some(ref list) => {
                            let display_list = list.get();
                            let (x, y) = (Au::from_frac_px(point.x as f64),
                                          Au::from_frac_px(point.y as f64));
                            let resp = hit_test(x,y,display_list.list);
                            if resp.is_none() {
                                Err(())
                            } else {
                                Ok(resp.unwrap())
                            }
                        }
                        None => {
                            error!("Can't hit test: no display list");
                            Err(())
                        },
                    }
                };

                reply_chan.send(response)
            }
        }
    }

    // When images can't be loaded in time to display they trigger
    // this callback in some task somewhere. This will send a message
    // to the script task, and ultimately cause the image to be
    // re-requested. We probably don't need to go all the way back to
    // the script task for this.
    fn make_on_image_available_cb(&self) -> @ImageResponder {
        // This has a crazy signature because the image cache needs to
        // make multiple copies of the callback, and the dom event
        // channel is not a copyable type, so this is actually a
        // little factory to produce callbacks
        @LayoutImageResponder {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
        } as @ImageResponder
    }

    /// Handles a message to destroy layout data. Layout data must be destroyed on *this* task
    /// because it contains local managed pointers.
    unsafe fn handle_reap_layout_data(&self, layout_data: LayoutDataRef) {
        let ptr: &mut Option<~LayoutData> = cast::transmute(layout_data.borrow_unchecked());
        *ptr = None
    }
}

