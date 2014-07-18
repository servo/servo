/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout task. Performs layout on the DOM, builds display lists and sends them to be
/// rendered.

use css::matching::{ApplicableDeclarations, ApplicableDeclarationsCache, MatchMethods};
use css::matching::{StyleSharingCandidateCache};
use css::select::new_stylist;
use css::node_style::StyledNode;
use construct::{FlowConstructionResult, NoConstructionResult};
use context::LayoutContext;
use flow::{Flow, ImmutableFlowUtils, MutableFlowUtils, MutableOwnedFlowUtils};
use flow::{PreorderFlowTraversal, PostorderFlowTraversal};
use flow;
use flow_ref::FlowRef;
use incremental::RestyleDamage;
use parallel::UnsafeFlow;
use parallel;
use util::{LayoutDataAccess, LayoutDataWrapper, OpaqueNodeMethods, ToGfxColor};
use wrapper::{LayoutNode, TLayoutNode, ThreadSafeLayoutNode};

use collections::dlist::DList;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::display_list::{ClipDisplayItemClass, ContentStackingLevel, DisplayItem};
use gfx::display_list::{DisplayItemIterator, DisplayList, OpaqueNode};
use gfx::font_context::FontContext;
use gfx::render_task::{RenderMsg, RenderChan, RenderLayer};
use gfx::{render_task, color};
use script::dom::bindings::js::JS;
use script::dom::event::ReflowEvent;
use script::dom::node::{ElementNodeTypeId, LayoutDataRef, Node};
use script::dom::element::{HTMLBodyElementTypeId, HTMLHtmlElementTypeId};
use script::layout_interface::{AddStylesheetMsg, ContentBoxQuery};
use script::layout_interface::{ContentBoxesQuery, ContentBoxesResponse, ExitNowMsg, LayoutQuery};
use script::layout_interface::{HitTestQuery, ContentBoxResponse, HitTestResponse, MouseOverQuery, MouseOverResponse};
use script::layout_interface::{ContentChangedDocumentDamage, LayoutChan, Msg, PrepareToExitMsg};
use script::layout_interface::{QueryMsg, ReapLayoutDataMsg, Reflow, UntrustedNodeAddress};
use script::layout_interface::{ReflowForDisplay, ReflowMsg};
use script::script_task::{ReflowCompleteMsg, ScriptChan, SendEventMsg};
use servo_msg::compositor_msg::Scrollable;
use servo_msg::constellation_msg::{ConstellationChan, PipelineId, Failure, FailureMsg};
use servo_net::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use gfx::font_cache_task::{FontCacheTask};
use servo_net::local_image_cache::{ImageResponder, LocalImageCache};
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::opts::Opts;
use servo_util::smallvec::{SmallVec, SmallVec1};
use servo_util::time::{TimeProfilerChan, profile};
use servo_util::time;
use servo_util::task::send_on_failure;
use servo_util::workqueue::WorkQueue;
use std::comm::{channel, Sender, Receiver};
use std::mem;
use std::ptr;
use std::task::TaskBuilder;
use style::{AuthorOrigin, Stylesheet, Stylist};
use sync::{Arc, Mutex};
use url::Url;

/// Information needed by the layout task.
pub struct LayoutTask {
    /// The ID of the pipeline that we belong to.
    pub id: PipelineId,

    /// The port on which we receive messages.
    pub port: Receiver<Msg>,

    //// The channel to send messages to ourself.
    pub chan: LayoutChan,

    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the script task.
    pub script_chan: ScriptChan,

    /// The channel on which messages can be sent to the painting task.
    pub render_chan: RenderChan,

    /// The channel on which messages can be sent to the image cache.
    pub image_cache_task: ImageCacheTask,

    /// Public interface to the font cache task.
    pub font_cache_task: FontCacheTask,

    /// The local image cache.
    pub local_image_cache: Arc<Mutex<LocalImageCache>>,

    /// The size of the viewport.
    pub screen_size: Size2D<Au>,

    /// A cached display list.
    pub display_list: Option<Arc<DisplayList>>,

    pub stylist: Box<Stylist>,

    /// The workers that we use for parallel operation.
    pub parallel_traversal: Option<WorkQueue<*mut LayoutContext,UnsafeFlow>>,

    /// The channel on which messages can be sent to the time profiler.
    pub time_profiler_chan: TimeProfilerChan,

    /// The command-line options.
    pub opts: Opts,

    /// The dirty rect. Used during display list construction.
    pub dirty: Rect<Au>,
}

/// The damage computation traversal.
#[deriving(Clone)]
struct ComputeDamageTraversal;

impl PostorderFlowTraversal for ComputeDamageTraversal {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        let mut damage = flow::base(flow).restyle_damage;
        for child in flow::child_iter(flow) {
            damage.insert(flow::base(child).restyle_damage.propagate_up())
        }
        flow::mut_base(flow).restyle_damage = damage;
        true
    }
}

/// Propagates restyle damage up and down the tree as appropriate.
///
/// FIXME(pcwalton): Merge this with flow tree building and/or other traversals.
struct PropagateDamageTraversal {
    all_style_damage: bool,
}

impl PreorderFlowTraversal for PropagateDamageTraversal {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        if self.all_style_damage {
            flow::mut_base(flow).restyle_damage.insert(RestyleDamage::all())
        }
        debug!("restyle damage = {:?}", flow::base(flow).restyle_damage);

        let prop = flow::base(flow).restyle_damage.propagate_down();
        if !prop.is_empty() {
            for kid_ctx in flow::child_iter(flow) {
                flow::mut_base(kid_ctx).restyle_damage.insert(prop)
            }
        }
        true
    }
}

/// The flow tree verification traversal. This is only on in debug builds.
#[cfg(debug)]
struct FlowTreeVerificationTraversal;

#[cfg(debug)]
impl PreorderFlowTraversal for FlowTreeVerificationTraversal {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        let base = flow::base(flow);
        if !base.flags.is_leaf() && !base.flags.is_nonleaf() {
            println("flow tree verification failed: flow wasn't a leaf or a nonleaf!");
            flow.dump();
            fail!("flow tree verification failed")
        }
        true
    }
}

/// The bubble-inline-sizes traversal, the first part of layout computation. This computes preferred
/// and intrinsic inline-sizes and bubbles them up the tree.
pub struct BubbleISizesTraversal<'a> {
    pub layout_context: &'a mut LayoutContext,
}

impl<'a> PostorderFlowTraversal for BubbleISizesTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        flow.bubble_inline_sizes(self.layout_context);
        true
    }

    // FIXME: We can't prune until we start reusing flows
    /*
    #[inline]
    fn should_prune(&mut self, flow: &mut Flow) -> bool {
        flow::mut_base(flow).restyle_damage.lacks(BubbleISizes)
    }
    */
}

/// The assign-inline-sizes traversal. In Gecko this corresponds to `Reflow`.
pub struct AssignISizesTraversal<'a> {
    pub layout_context: &'a mut LayoutContext,
}

impl<'a> PreorderFlowTraversal for AssignISizesTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        flow.assign_inline_sizes(self.layout_context);
        true
    }
}

/// The assign-block-sizes-and-store-overflow traversal, the last (and most expensive) part of layout
/// computation. Determines the final block-sizes for all layout objects, computes positions, and
/// computes overflow regions. In Gecko this corresponds to `FinishAndStoreOverflow`.
pub struct AssignBSizesAndStoreOverflowTraversal<'a> {
    pub layout_context: &'a mut LayoutContext,
}

impl<'a> PostorderFlowTraversal for AssignBSizesAndStoreOverflowTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) -> bool {
        flow.assign_block_size(self.layout_context);
        // Skip store-overflow for absolutely positioned flows. That will be
        // done in a separate traversal.
        if !flow.is_store_overflow_delayed() {
            flow.store_overflow(self.layout_context);
        }
        true
    }

    #[inline]
    fn should_process(&mut self, flow: &mut Flow) -> bool {
        !flow::base(flow).flags.impacted_by_floats()
    }
}

/// The display list construction traversal.
pub struct BuildDisplayListTraversal<'a> {
    layout_context: &'a LayoutContext,
}

impl<'a> BuildDisplayListTraversal<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow) {
        flow.compute_absolute_position();

        for kid in flow::mut_base(flow).child_iter() {
            if !kid.is_absolutely_positioned() {
                self.process(kid)
            }
        }

        for absolute_descendant_link in flow::mut_base(flow).abs_descendants.iter() {
            self.process(absolute_descendant_link)
        }

        flow.build_display_list(self.layout_context)
    }
}

struct LayoutImageResponder {
    id: PipelineId,
    script_chan: ScriptChan,
}

impl ImageResponder for LayoutImageResponder {
    fn respond(&self) -> proc(ImageResponseMsg):Send {
        let id = self.id.clone();
        let script_chan = self.script_chan.clone();
        let f: proc(ImageResponseMsg):Send = proc(_) {
            let ScriptChan(chan) = script_chan;
            drop(chan.send_opt(SendEventMsg(id.clone(), ReflowEvent)))
        };
        f
    }
}

impl LayoutTask {
    /// Spawns a new layout task.
    pub fn create(id: PipelineId,
                  port: Receiver<Msg>,
                  chan: LayoutChan,
                  constellation_chan: ConstellationChan,
                  failure_msg: Failure,
                  script_chan: ScriptChan,
                  render_chan: RenderChan,
                  img_cache_task: ImageCacheTask,
                  font_cache_task: FontCacheTask,
                  opts: Opts,
                  time_profiler_chan: TimeProfilerChan,
                  shutdown_chan: Sender<()>) {
        let mut builder = TaskBuilder::new().named("LayoutTask");
        let ConstellationChan(con_chan) = constellation_chan.clone();
        send_on_failure(&mut builder, FailureMsg(failure_msg), con_chan);
        builder.spawn(proc() {
            { // Ensures layout task is destroyed before we send shutdown message
                let mut layout = LayoutTask::new(id,
                                                 port,
                                                 chan,
                                                 constellation_chan,
                                                 script_chan,
                                                 render_chan,
                                                 img_cache_task,
                                                 font_cache_task,
                                                 &opts,
                                                 time_profiler_chan);
                layout.start();
            }
            shutdown_chan.send(());
        });
    }

    /// Creates a new `LayoutTask` structure.
    fn new(id: PipelineId,
           port: Receiver<Msg>,
           chan: LayoutChan,
           constellation_chan: ConstellationChan,
           script_chan: ScriptChan,
           render_chan: RenderChan,
           image_cache_task: ImageCacheTask,
           font_cache_task: FontCacheTask,
           opts: &Opts,
           time_profiler_chan: TimeProfilerChan)
           -> LayoutTask {
        let local_image_cache = Arc::new(Mutex::new(LocalImageCache::new(image_cache_task.clone())));
        let screen_size = Size2D(Au(0), Au(0));
        let parallel_traversal = if opts.layout_threads != 1 {
            Some(WorkQueue::new("LayoutWorker", opts.layout_threads, ptr::mut_null()))
        } else {
            None
        };

        LayoutTask {
            id: id,
            port: port,
            chan: chan,
            constellation_chan: constellation_chan,
            script_chan: script_chan,
            render_chan: render_chan,
            image_cache_task: image_cache_task.clone(),
            font_cache_task: font_cache_task,
            local_image_cache: local_image_cache,
            screen_size: screen_size,

            display_list: None,
            stylist: box new_stylist(),
            parallel_traversal: parallel_traversal,
            time_profiler_chan: time_profiler_chan,
            opts: opts.clone(),
            dirty: Rect::zero(),
        }
    }

    /// Starts listening on the port.
    fn start(&mut self) {
        while self.handle_request() {
            // Loop indefinitely.
        }
    }

    // Create a layout context for use in building display lists, hit testing, &c.
    fn build_layout_context(&self, reflow_root: &LayoutNode, url: &Url) -> LayoutContext {
        LayoutContext {
            image_cache: self.local_image_cache.clone(),
            screen_size: self.screen_size.clone(),
            constellation_chan: self.constellation_chan.clone(),
            layout_chan: self.chan.clone(),
            font_cache_task: self.font_cache_task.clone(),
            stylist: &*self.stylist,
            url: (*url).clone(),
            reflow_root: OpaqueNodeMethods::from_layout_node(reflow_root),
            opts: self.opts.clone(),
            dirty: Rect::zero(),
        }
    }

    /// Receives and dispatches messages from the port.
    fn handle_request(&mut self) -> bool {
        match self.port.recv() {
            AddStylesheetMsg(sheet) => self.handle_add_stylesheet(sheet),
            ReflowMsg(data) => {
                profile(time::LayoutPerformCategory, self.time_profiler_chan.clone(), || {
                    self.handle_reflow(data);
                });
            }
            QueryMsg(query) => {
                let mut query = Some(query);
                profile(time::LayoutQueryCategory, self.time_profiler_chan.clone(), || {
                    self.handle_query(query.take_unwrap());
                });
            }
            ReapLayoutDataMsg(dead_layout_data) => {
                unsafe {
                    self.handle_reap_layout_data(dead_layout_data)
                }
            }
            PrepareToExitMsg(response_chan) => {
                debug!("layout: PrepareToExitMsg received");
                self.prepare_to_exit(response_chan);
                return false
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
    fn prepare_to_exit(&mut self, response_chan: Sender<()>) {
        response_chan.send(());
        loop {
            match self.port.recv() {
                ReapLayoutDataMsg(dead_layout_data) => {
                    unsafe {
                        self.handle_reap_layout_data(dead_layout_data)
                    }
                }
                ExitNowMsg => {
                    debug!("layout task is exiting...");
                    self.exit_now();
                    break
                }
                _ => {
                    fail!("layout: message that wasn't `ExitNowMsg` received after \
                           `PrepareToExitMsg`")
                }
            }
        }
    }

    /// Shuts down the layout task now. If there are any DOM nodes left, layout will now (safely)
    /// crash.
    fn exit_now(&mut self) {
        let (response_chan, response_port) = channel();

        match self.parallel_traversal {
            None => {}
            Some(ref mut traversal) => traversal.shutdown(),
        }

        self.render_chan.send(render_task::ExitMsg(Some(response_chan)));
        response_port.recv()
    }

    fn handle_add_stylesheet(&mut self, sheet: Stylesheet) {
        self.stylist.add_stylesheet(sheet, AuthorOrigin)
    }

    /// Retrieves the flow tree root from the root node.
    fn get_layout_root(&self, node: LayoutNode) -> FlowRef {
        let mut layout_data_ref = node.mutate_layout_data();
        let result = match &mut *layout_data_ref {
            &Some(ref mut layout_data) => {
                mem::replace(&mut layout_data.data.flow_construction_result, NoConstructionResult)
            }
            &None => fail!("no layout data for root node"),
        };
        let mut flow = match result {
            FlowConstructionResult(mut flow, abs_descendants) => {
                // Note: Assuming that the root has display 'static' (as per
                // CSS Section 9.3.1). Otherwise, if it were absolutely
                // positioned, it would return a reference to itself in
                // `abs_descendants` and would lead to a circular reference.
                // Set Root as CB for any remaining absolute descendants.
                flow.set_abs_descendants(abs_descendants);
                flow
            }
            _ => fail!("Flow construction didn't result in a flow at the root of the tree!"),
        };
        flow.get_mut().mark_as_root();
        flow
    }

    /// Performs layout constraint solving.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints(&mut self,
                         layout_root: &mut Flow,
                         layout_context: &mut LayoutContext) {
        if layout_context.opts.bubble_inline_sizes_separately {
            let mut traversal = BubbleISizesTraversal {
                layout_context: layout_context,
            };
            layout_root.traverse_postorder(&mut traversal);
        }

        // FIXME(kmc): We want to prune nodes without the Reflow restyle damage
        // bit, but FloatContext values can't be reused, so we need to
        // recompute them every time.
        // NOTE: this currently computes borders, so any pruning should separate that operation
        // out.
        {
            let mut traversal = AssignISizesTraversal {
                layout_context: layout_context,
            };
            layout_root.traverse_preorder(&mut traversal);
        }

        // FIXME(pcwalton): Prune this pass as well.
        {
            let mut traversal = AssignBSizesAndStoreOverflowTraversal {
                layout_context: layout_context,
            };
            layout_root.traverse_postorder(&mut traversal);
        }
    }

    /// Performs layout constraint solving in parallel.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints_parallel(&mut self,
                                  layout_root: &mut FlowRef,
                                  layout_context: &mut LayoutContext) {
        if layout_context.opts.bubble_inline_sizes_separately {
            let mut traversal = BubbleISizesTraversal {
                layout_context: layout_context,
            };
            layout_root.get_mut().traverse_postorder(&mut traversal);
        }

        match self.parallel_traversal {
            None => fail!("solve_contraints_parallel() called with no parallel traversal ready"),
            Some(ref mut traversal) => {
                // NOTE: this currently computes borders, so any pruning should separate that
                // operation out.
                parallel::traverse_flow_tree_preorder(layout_root,
                                                      self.time_profiler_chan.clone(),
                                                      layout_context,
                                                      traversal);
            }
        }
    }

    /// Verifies that every node was either marked as a leaf or as a nonleaf in the flow tree.
    /// This is only on in debug builds.
    #[inline(never)]
    #[cfg(debug)]
    fn verify_flow_tree(&mut self, layout_root: &mut FlowRef) {
        let mut traversal = FlowTreeVerificationTraversal;
        layout_root.traverse_preorder(&mut traversal);
    }

    #[cfg(not(debug))]
    fn verify_flow_tree(&mut self, _: &mut FlowRef) {
    }

    /// The high-level routine that performs layout tasks.
    fn handle_reflow(&mut self, data: &Reflow) {
        // FIXME: Isolate this transmutation into a "bridge" module.
        let node: &mut LayoutNode = unsafe {
            let mut node: JS<Node> = JS::from_trusted_node_address(data.document_root);
            mem::transmute(&mut node)
        };

        debug!("layout: received layout request for: {:s}", data.url.to_str());
        debug!("layout: damage is {:?}", data.damage);
        debug!("layout: parsed Node tree");
        debug!("{:?}", node.dump());

        {
            // Reset the image cache.
            let mut local_image_cache = self.local_image_cache.lock();
            local_image_cache.next_round(self.make_on_image_available_cb());
        }

        // true => Do the reflow with full style damage, because content
        // changed or the window was resized.
        let mut all_style_damage = match data.damage.level {
            ContentChangedDocumentDamage => true,
            _ => false
        };

        // TODO: Calculate the "actual viewport":
        // http://www.w3.org/TR/css-device-adapt/#actual-viewport
        let viewport_size = data.window_size.initial_viewport;

        let current_screen_size = Size2D(Au::from_frac32_px(viewport_size.width.get()),
                                         Au::from_frac32_px(viewport_size.height.get()));
        if self.screen_size != current_screen_size {
            all_style_damage = true
        }
        self.screen_size = current_screen_size;

        // Create a layout context for use throughout the following passes.
        let mut layout_ctx = self.build_layout_context(node, &data.url);

        // Create a font context, if this is sequential.
        //
        // FIXME(pcwalton): This is a pretty bogus thing to do. Essentially this is a workaround
        // for libgreen having slow TLS.
        let mut font_context_opt = if self.parallel_traversal.is_none() {
            Some(box FontContext::new(layout_ctx.font_cache_task.clone()))
        } else {
            None
        };

        let mut layout_root = profile(time::LayoutStyleRecalcCategory,
                                      self.time_profiler_chan.clone(),
                                      || {
            // Perform CSS selector matching and flow construction.
            match self.parallel_traversal {
                None => {
                    let mut applicable_declarations = ApplicableDeclarations::new();
                    let mut applicable_declarations_cache = ApplicableDeclarationsCache::new();
                    let mut style_sharing_candidate_cache = StyleSharingCandidateCache::new();
                    drop(node.recalc_style_for_subtree(self.stylist,
                                                       &mut layout_ctx,
                                                       font_context_opt.take_unwrap(),
                                                       &mut applicable_declarations,
                                                       &mut applicable_declarations_cache,
                                                       &mut style_sharing_candidate_cache,
                                                       None))
                }
                Some(ref mut traversal) => {
                    parallel::recalc_style_for_subtree(node, &mut layout_ctx, traversal)
                }
            }

            self.get_layout_root((*node).clone())
        });

        // Verification of the flow tree, which ensures that all nodes were either marked as leaves
        // or as non-leaves. This becomes a no-op in release builds. (It is inconsequential to
        // memory safety but is a useful debugging tool.)
        self.verify_flow_tree(&mut layout_root);

        // Propagate damage.
        profile(time::LayoutDamagePropagateCategory, self.time_profiler_chan.clone(), || {
            layout_root.get_mut().traverse_preorder(&mut PropagateDamageTraversal {
                all_style_damage: all_style_damage
            });
            layout_root.get_mut().traverse_postorder(&mut ComputeDamageTraversal.clone());
        });

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        profile(time::LayoutMainCategory, self.time_profiler_chan.clone(), || {
            match self.parallel_traversal {
                None => {
                    // Sequential mode.
                    self.solve_constraints(layout_root.get_mut(), &mut layout_ctx)
                }
                Some(_) => {
                    // Parallel mode.
                    self.solve_constraints_parallel(&mut layout_root, &mut layout_ctx)
                }
            }
        });

        // Build the display list if necessary, and send it to the renderer.
        if data.goal == ReflowForDisplay {
            let writing_mode = flow::base(layout_root.get()).writing_mode;
            profile(time::LayoutDispListBuildCategory, self.time_profiler_chan.clone(), || {
                // FIXME(#2795): Get the real container size
                let container_size = Size2D::zero();
                layout_ctx.dirty = flow::base(layout_root.get()).position.to_physical(
                    writing_mode, container_size);

                match self.parallel_traversal {
                    None => {
                        let mut traversal = BuildDisplayListTraversal {
                            layout_context: &layout_ctx,
                        };
                        traversal.process(layout_root.get_mut());
                    }
                    Some(ref mut traversal) => {
                        parallel::build_display_list_for_subtree(&mut layout_root,
                                                                 self.time_profiler_chan.clone(),
                                                                 &mut layout_ctx,
                                                                 traversal);
                    }
                }

                let root_display_list =
                    mem::replace(&mut flow::mut_base(layout_root.get_mut()).display_list,
                                 DisplayList::new());
                let display_list = Arc::new(root_display_list.flatten(ContentStackingLevel));

                // FIXME(pcwalton): This is really ugly and can't handle overflow: scroll. Refactor
                // it with extreme prejudice.
                let mut color = color::rgba(255.0, 255.0, 255.0, 255.0);
                for child in node.traverse_preorder() {
                    if child.type_id() == Some(ElementNodeTypeId(HTMLHtmlElementTypeId)) ||
                            child.type_id() == Some(ElementNodeTypeId(HTMLBodyElementTypeId)) {
                        let element_bg_color = {
                            let thread_safe_child = ThreadSafeLayoutNode::new(&child);
                            thread_safe_child.style()
                                             .resolve_color(thread_safe_child.style()
                                                                             .get_background()
                                                                             .background_color)
                                             .to_gfx_color()
                        };
                        match element_bg_color {
                            color::rgba(0., 0., 0., 0.) => {}
                            _ => {
                                color = element_bg_color;
                                break;
                           }
                        }
                    }
                }

                let root_size = {
                    let root_flow = flow::base(layout_root.get());
                    root_flow.position.size.to_physical(root_flow.writing_mode)
                };
                let root_size = Size2D(root_size.width.to_nearest_px() as uint,
                                       root_size.height.to_nearest_px() as uint);
                let render_layer = RenderLayer {
                    id: layout_root.get().layer_id(0),
                    display_list: display_list.clone(),
                    position: Rect(Point2D(0u, 0u), root_size),
                    background_color: color,
                    scroll_policy: Scrollable,
                };

                self.display_list = Some(display_list.clone());

                // TODO(pcwalton): Eventually, when we have incremental reflow, this will have to
                // be smarter in order to handle retained layer contents properly from reflow to
                // reflow.
                let mut layers = SmallVec1::new();
                layers.push(render_layer);
                for layer in mem::replace(&mut flow::mut_base(layout_root.get_mut()).layers,
                                          DList::new()).move_iter() {
                    layers.push(layer)
                }

                debug!("Layout done!");

                self.render_chan.send(RenderMsg(layers));
            });
        }

        // Tell script that we're done.
        //
        // FIXME(pcwalton): This should probably be *one* channel, but we can't fix this without
        // either select or a filtered recv() that only looks for messages of a given type.
        data.script_join_chan.send(());
        let ScriptChan(ref chan) = data.script_chan;
        chan.send(ReflowCompleteMsg(self.id, data.id));
    }

    /// Handles a query from the script task. This is the main routine that DOM functions like
    /// `getClientRects()` or `getBoundingClientRect()` ultimately invoke.
    fn handle_query(&self, query: LayoutQuery) {
        match query {
            // The neat thing here is that in order to answer the following two queries we only
            // need to compare nodes for equality. Thus we can safely work only with `OpaqueNode`.
            ContentBoxQuery(node, reply_chan) => {
                let node: OpaqueNode = OpaqueNodeMethods::from_script_node(node);
                fn union_boxes_for_node(accumulator: &mut Option<Rect<Au>>,
                                        mut iter: DisplayItemIterator,
                                        node: OpaqueNode) {
                    for item in iter {
                        union_boxes_for_node(accumulator, item.children(), node);
                        if item.base().node == node {
                            match *accumulator {
                                None => *accumulator = Some(item.base().bounds),
                                Some(ref mut acc) => *acc = acc.union(&item.base().bounds),
                            }
                        }
                    }
                }

                let mut rect = None;
                match self.display_list {
                    None => fail!("no display list!"),
                    Some(ref display_list) => {
                        union_boxes_for_node(&mut rect, display_list.iter(), node)
                    }
                }
                reply_chan.send(ContentBoxResponse(rect.unwrap_or(Rect::zero())))
            }
            ContentBoxesQuery(node, reply_chan) => {
                let node: OpaqueNode = OpaqueNodeMethods::from_script_node(node);

                fn add_boxes_for_node(accumulator: &mut Vec<Rect<Au>>,
                                      mut iter: DisplayItemIterator,
                                      node: OpaqueNode) {
                    for item in iter {
                        add_boxes_for_node(accumulator, item.children(), node);
                        if item.base().node == node {
                            accumulator.push(item.base().bounds)
                        }
                    }
                }

                let mut boxes = vec!();
                match self.display_list {
                    None => fail!("no display list!"),
                    Some(ref display_list) => {
                        add_boxes_for_node(&mut boxes, display_list.iter(), node)
                    }
                }
                reply_chan.send(ContentBoxesResponse(boxes))
            }
            HitTestQuery(_, point, reply_chan) => {
                fn hit_test<'a,I:Iterator<&'a DisplayItem>>(x: Au, y: Au, mut iterator: I)
                            -> Option<HitTestResponse> {
                    for item in iterator {
                        match *item {
                            ClipDisplayItemClass(ref cc) => {
                                if geometry::rect_contains_point(cc.base.bounds, Point2D(x, y)) {
                                    let ret = hit_test(x, y, cc.children.list.iter().rev());
                                    if !ret.is_none() {
                                        return ret
                                    }
                                }
                                continue
                            }
                            _ => {}
                        }

                        let bounds = item.bounds();

                        // TODO(tikue): This check should really be performed by a method of
                        // DisplayItem.
                        if x < bounds.origin.x + bounds.size.width &&
                                bounds.origin.x <= x &&
                                y < bounds.origin.y + bounds.size.height &&
                                bounds.origin.y <= y {
                            return Some(HitTestResponse(item.base()
                                                            .node
                                                            .to_untrusted_node_address()))
                        }
                    }
                    let ret: Option<HitTestResponse> = None;
                    ret
                }
                let (x, y) = (Au::from_frac_px(point.x as f64),
                              Au::from_frac_px(point.y as f64));
                let resp = match self.display_list {
                    None => fail!("no display list!"),
                    Some(ref display_list) => hit_test(x, y, display_list.list.iter().rev()),
                };
                if resp.is_some() {
                    reply_chan.send(Ok(resp.unwrap()));
                    return
                }
                reply_chan.send(Err(()));

            }
            MouseOverQuery(_, point, reply_chan) => {
                fn mouse_over_test<'a,
                                   I:Iterator<&'a DisplayItem>>(
                                   x: Au,
                                   y: Au,
                                   mut iterator: I,
                                   result: &mut Vec<UntrustedNodeAddress>) {
                    for item in iterator {
                        match *item {
                            ClipDisplayItemClass(ref cc) => {
                                mouse_over_test(x, y, cc.children.list.iter().rev(), result);
                            }
                            _ => {
                                let bounds = item.bounds();

                                // TODO(tikue): This check should really be performed by a method
                                // of DisplayItem.
                                if x < bounds.origin.x + bounds.size.width &&
                                        bounds.origin.x <= x &&
                                        y < bounds.origin.y + bounds.size.height &&
                                        bounds.origin.y <= y {
                                    result.push(item.base()
                                                    .node
                                                    .to_untrusted_node_address());
                                }
                            }
                        }
                    }
                }

                let mut mouse_over_list: Vec<UntrustedNodeAddress> = vec!();
                let (x, y) = (Au::from_frac_px(point.x as f64), Au::from_frac_px(point.y as f64));
                match self.display_list {
                    None => fail!("no display list!"),
                    Some(ref display_list) => {
                        mouse_over_test(x,
                                        y,
                                        display_list.list.iter().rev(),
                                        &mut mouse_over_list);
                    }
                };

                if mouse_over_list.is_empty() {
                    reply_chan.send(Err(()));
                } else {
                    reply_chan.send(Ok(MouseOverResponse(mouse_over_list)));
                }
            }
        }
    }

    // When images can't be loaded in time to display they trigger
    // this callback in some task somewhere. This will send a message
    // to the script task, and ultimately cause the image to be
    // re-requested. We probably don't need to go all the way back to
    // the script task for this.
    fn make_on_image_available_cb(&self) -> Box<ImageResponder+Send> {
        // This has a crazy signature because the image cache needs to
        // make multiple copies of the callback, and the dom event
        // channel is not a copyable type, so this is actually a
        // little factory to produce callbacks
        box LayoutImageResponder {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
        } as Box<ImageResponder+Send>
    }

    /// Handles a message to destroy layout data. Layout data must be destroyed on *this* task
    /// because it contains local managed pointers.
    unsafe fn handle_reap_layout_data(&self, layout_data: LayoutDataRef) {
        let mut layout_data_ref = layout_data.borrow_mut();
        let _: Option<LayoutDataWrapper> = mem::transmute(
            mem::replace(&mut *layout_data_ref, None));
    }
}
