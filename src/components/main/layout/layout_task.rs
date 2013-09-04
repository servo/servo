/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout task. Performs layout on the DOM, builds display lists and sends them to be
/// rendered.

use css::matching::MatchMethods;
use css::select::new_css_select_ctx;
use layout::aux::LayoutAuxMethods;
use layout::box_builder::LayoutTreeBuilder;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder};
use layout::flow::FlowContext;
use layout::incremental::{RestyleDamage, BubbleWidths};

use std::cast::transmute;
use std::cell::Cell;
use std::comm::{Port};
use extra::arc::Arc;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::display_list::DisplayList;
use gfx::font_context::FontContext;
use gfx::geometry::Au;
use gfx::opts::Opts;
use gfx::render_task::{RenderMsg, RenderChan, RenderLayer};
use newcss::select::SelectCtx;
use newcss::stylesheet::Stylesheet;
use newcss::types::OriginAuthor;
use script::dom::event::ReflowEvent;
use script::dom::node::{AbstractNode, LayoutView};
use script::layout_interface::{AddStylesheetMsg, ContentBoxQuery};
use script::layout_interface::{HitTestQuery, ContentBoxResponse, HitTestResponse};
use script::layout_interface::{ContentBoxesQuery, ContentBoxesResponse, ExitMsg, LayoutQuery};
use script::layout_interface::{MatchSelectorsDocumentDamage, Msg};
use script::layout_interface::{QueryMsg, Reflow, ReflowDocumentDamage};
use script::layout_interface::{ReflowForDisplay, ReflowMsg};
use script::script_task::{ReflowCompleteMsg, ScriptChan, SendEventMsg};
use servo_msg::constellation_msg::{ConstellationChan, PipelineId};
use servo_net::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use servo_net::local_image_cache::LocalImageCache;
use servo_util::tree::TreeNodeRef;
use servo_util::time::{ProfilerChan, profile};
use servo_util::time;
use servo_util::range::Range;
use extra::url::Url;

struct LayoutTask {
    id: PipelineId,
    port: Port<Msg>,
    constellation_chan: ConstellationChan,
    script_chan: ScriptChan,
    render_chan: RenderChan<AbstractNode<()>>,
    image_cache_task: ImageCacheTask,
    local_image_cache: @mut LocalImageCache,
    font_ctx: @mut FontContext,
    doc_url: Option<Url>,
    screen_size: Option<Size2D<Au>>,

    display_list: Option<Arc<DisplayList<AbstractNode<()>>>>,

    css_select_ctx: @mut SelectCtx,
    profiler_chan: ProfilerChan,
}

impl LayoutTask {
    pub fn create(id: PipelineId,
                  port: Port<Msg>,
                  constellation_chan: ConstellationChan,
                  script_chan: ScriptChan,
                  render_chan: RenderChan<AbstractNode<()>>,
                  img_cache_task: ImageCacheTask,
                  opts: Opts,
                  profiler_chan: ProfilerChan) {

        let port = Cell::new(port);
        let constellation_chan = Cell::new(constellation_chan);
        let script_chan = Cell::new(script_chan);
        let render_chan = Cell::new(render_chan);
        let img_cache_task = Cell::new(img_cache_task);
        let profiler_chan = Cell::new(profiler_chan);

        do spawn {
            let mut layout = LayoutTask::new(id,
                                             port.take(),
                                             constellation_chan.take(),
                                             script_chan.take(),
                                             render_chan.take(),
                                             img_cache_task.take(),
                                             &opts,
                                             profiler_chan.take());
            layout.start();
        };
    }

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
            doc_url: None,
            screen_size: None,

            display_list: None,
            
            css_select_ctx: @mut new_css_select_ctx(),
            profiler_chan: profiler_chan,
        }
    }

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
            ExitMsg => {
                debug!("layout: ExitMsg received");
                return false
            }
        }

        true
    }

    fn handle_add_stylesheet(&self, sheet: Stylesheet) {
        let sheet = Cell::new(sheet);
        self.css_select_ctx.append_sheet(sheet.take(), OriginAuthor);
    }

    /// The high-level routine that performs layout tasks.
    fn handle_reflow(&mut self, data: &Reflow) {
        // FIXME: Isolate this transmutation into a "bridge" module.
        let node: &AbstractNode<LayoutView> = unsafe {
            transmute(&data.document_root)
        };

        // FIXME: Bad copy!
        let doc_url = data.url.clone();
        let script_chan = data.script_chan.clone();

        debug!("layout: received layout request for: %s", doc_url.to_str());
        debug!("layout: damage is %?", data.damage);
        debug!("layout: parsed Node tree");
        debug!("%?", node.dump());
        // Reset the image cache.
        self.local_image_cache.next_round(self.make_on_image_available_cb(script_chan));

        self.doc_url = Some(doc_url);
        let screen_size = Size2D(Au::from_px(data.window_size.width as int),
                                 Au::from_px(data.window_size.height as int));
        let resized = self.screen_size != Some(screen_size);
        debug!("resized: %?", resized);
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
                    node.restyle_subtree(self.css_select_ctx);
                }
            }
        }

        // Construct the flow tree.
        let mut layout_root: FlowContext = do profile(time::LayoutTreeBuilderCategory,
                                                  self.profiler_chan.clone()) {
            let mut builder = LayoutTreeBuilder::new();
            let layout_root: FlowContext = match builder.construct_trees(&layout_ctx, *node) {
                Ok(root) => root,
                Err(*) => fail!(~"Root flow should always exist")
            };

            layout_root
        };

        // Propagate restyle damage up and down the tree, as appropriate.
        // FIXME: Merge this with flow tree building and/or the other traversals.
        do layout_root.each_preorder |flow| {
            // Also set any damage implied by resize.
            if resized {
                do flow.with_mut_base |base| {
                    base.restyle_damage.union_in_place(RestyleDamage::for_resize());
                }
            }

            let prop = flow.with_base(|base| base.restyle_damage.propagate_down());
            if prop.is_nonempty() {
                for kid_ctx in flow.child_iter() {
                    do kid_ctx.with_mut_base |kid| {
                        kid.restyle_damage.union_in_place(prop);
                    }
                }
            }
            true
        };

        do layout_root.each_postorder |flow| {
            let mut damage = do flow.with_base |base| {
                base.restyle_damage
            };
            for child in flow.child_iter() {
                do child.with_base |child_base| {
                    damage.union_in_place(child_base.restyle_damage);
                }
            }
            do flow.with_mut_base |base| {
                base.restyle_damage = damage;
            }
            true
        };

        debug!("layout: constructed Flow tree");
        debug!("%?", layout_root.dump());

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        do profile(time::LayoutMainCategory, self.profiler_chan.clone()) {
            do layout_root.each_postorder_prune(|f| f.restyle_damage().lacks(BubbleWidths)) |flow| {
                flow.bubble_widths(&mut layout_ctx);
                true
            };

            // FIXME: We want to do
            //     for flow in layout_root.traverse_preorder_prune(|f| f.restyle_damage().lacks(Reflow)) 
            // but FloatContext values can't be reused, so we need to recompute them every time.
            debug!("assigning widths");
            do layout_root.each_preorder |flow| {
                flow.assign_widths(&mut layout_ctx);
                true
            };

            // For now, this is an inorder traversal
            // FIXME: prune this traversal as well
            debug!("assigning height");
            do layout_root.each_bu_sub_inorder |flow| {
                flow.assign_height(&mut layout_ctx);
                true
            };
        }

        // Build the display list if necessary, and send it to the renderer.
        if data.goal == ReflowForDisplay {
            do profile(time::LayoutDispListBuildCategory, self.profiler_chan.clone()) {
                let builder = DisplayListBuilder {
                    ctx: &layout_ctx,
                };

                let display_list = ~Cell::new(DisplayList::<AbstractNode<()>>::new());

                // TODO: Set options on the builder before building.
                // TODO: Be smarter about what needs painting.
                let root_pos = &layout_root.position().clone();
                layout_root.each_preorder_prune(|flow| {  
                    flow.build_display_list(&builder, root_pos, display_list) 
                }, |_| { true } );

                let root_size = do layout_root.with_base |base| {
                    base.position.size
                };

                let display_list = Arc::new(display_list.take());

                for i in range(0,display_list.get().list.len()) {
                    let node: AbstractNode<LayoutView> = unsafe {
                        transmute(display_list.get().list[i].base().extra)
                    };

                    do node.write_layout_data |layout_data| {
                        layout_data.boxes.display_list = Some(display_list.clone());

                        if layout_data.boxes.range.is_none() {
                            debug!("Creating initial range for node");
                            layout_data.boxes.range = Some(Range::new(i,1));
                        } else {
                            debug!("Appending item to range");
                            unsafe {
                                let old_node: AbstractNode<()> = transmute(node);
                                assert!(old_node == display_list.get().list[i-1].base().extra,
                                "Non-contiguous arrangement of display items");
                            }

                            layout_data.boxes.range.unwrap().extend_by(1);
                        }
                    }
                }

                let render_layer = RenderLayer {
                    display_list: display_list.clone(),
                    size: Size2D(root_size.width.to_nearest_px() as uint,
                                 root_size.height.to_nearest_px() as uint)
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
        data.script_chan.send(ReflowCompleteMsg(self.id));
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
                    do node.read_layout_data |layout_data| {
                        match (layout_data.boxes.display_list.clone(), layout_data.boxes.range) {
                            (Some(display_list), Some(range)) => {
                                let mut rect: Option<Rect<Au>> = None;
                                for i in range.eachi() {
                                    rect = match rect {
                                        Some(acc) => {
                                            Some(acc.union(&display_list.get().list[i].bounds()))
                                        }
                                        None => Some(display_list.get().list[i].bounds())
                                    }
                                }
                                rect
                            }
                            _ => {
                                let mut acc: Option<Rect<Au>> = None;
                                for child in node.children() {
                                    let rect = box_for_node(child);
                                    match rect {
                                        None => loop,
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
                }

                let rect = box_for_node(node).unwrap_or_default(Rect(Point2D(Au(0), Au(0)),
                                                                     Size2D(Au(0), Au(0))));
                reply_chan.send(ContentBoxResponse(rect))
            }
            ContentBoxesQuery(node, reply_chan) => {
                // FIXME: Isolate this transmutation into a single "bridge" module.
                let node: AbstractNode<LayoutView> = unsafe {
                    transmute(node)
                };

                fn boxes_for_node(node: AbstractNode<LayoutView>,
                                  boxes: ~[Rect<Au>]) -> ~[Rect<Au>] {
                    let boxes = Cell::new(boxes);
                    do node.read_layout_data |layout_data| {
                        let mut boxes = boxes.take();
                        match (layout_data.boxes.display_list.clone(), layout_data.boxes.range) {
                            (Some(display_list), Some(range)) => {
                                for i in range.eachi() {
                                    boxes.push(display_list.get().list[i].bounds());
                                }
                            }
                            _ => {
                                for child in node.children() {
                                    boxes = boxes_for_node(child, boxes);
                                }
                            }
                        }
                        boxes
                    }
                }

                let mut boxes = ~[];
                boxes = boxes_for_node(node, boxes);
                reply_chan.send(ContentBoxesResponse(boxes))
            }
            HitTestQuery(_, point, reply_chan) => {
                let response = {
                    match self.display_list {
                        Some(ref list) => {
                            let display_list = list.get();
                            let (x, y) = (Au::from_frac_px(point.x as float),
                                    Au::from_frac_px(point.y as float));
                            let mut resp = Err(());
                            // iterate in reverse to ensure we have the most recently painted render box
                            for display_item in display_list.list.rev_iter() {
                                let bounds = display_item.bounds();
                                // TODO this check should really be performed by a method of DisplayItem
                                if x <= bounds.origin.x + bounds.size.width &&
                                    bounds.origin.x <= x &&
                                        y < bounds.origin.y + bounds.size.height &&
                                        bounds.origin.y <  y {
                                            let node: AbstractNode<LayoutView> = unsafe {
                                                transmute(display_item.base().extra)
                                            };
                                            resp = Ok(HitTestResponse(node));
                                            break;
                                        }
                            }
                            resp
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
    fn make_on_image_available_cb(&self, script_chan: ScriptChan)
                                  -> @fn() -> ~fn(ImageResponseMsg) {
        // This has a crazy signature because the image cache needs to
        // make multiple copies of the callback, and the dom event
        // channel is not a copyable type, so this is actually a
        // little factory to produce callbacks
        let id = self.id.clone();
        let f: @fn() -> ~fn(ImageResponseMsg) = || {
            let script_chan = script_chan.clone();
            let f: ~fn(ImageResponseMsg) = |_| {
                script_chan.send(SendEventMsg(id.clone(), ReflowEvent))
            };
            f
        };
        return f;
    }
}

