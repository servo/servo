/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout task. Performs layout on the DOM, builds display lists and sends them to be
/// rendered.

use css::matching::MatchMethods;
use css::select::new_css_select_ctx;
use layout::aux::{LayoutData, LayoutAuxMethods};
use layout::box::RenderBox;
use layout::box_builder::LayoutTreeBuilder;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, FlowDisplayListBuilderMethods};
use layout::flow::FlowContext;
use util::task::spawn_listener;

use core::cast::transmute;
use core::cell::Cell;
use core::comm::{Chan, Port, SharedChan};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::display_list::DisplayList;
use gfx::font_context::FontContext;
use gfx::geometry::Au;
use gfx::opts::Opts;
use gfx::render_layers::RenderLayer;
use gfx::render_task::{RenderMsg, RenderTask};
use newcss::select::SelectCtx;
use newcss::stylesheet::Stylesheet;
use newcss::types::OriginAuthor;
use script::dom::event::ReflowEvent;
use script::dom::node::{AbstractNode, LayoutView};
use script::layout_interface::{AddStylesheetMsg, ContentBoxQuery};
use script::layout_interface::{HitTestQuery, ContentBoxResponse, HitTestResponse};
use script::layout_interface::{ContentBoxesQuery, ContentBoxesResponse, ExitMsg, LayoutQuery};
use script::layout_interface::{LayoutResponse, LayoutTask, MatchSelectorsDocumentDamage, Msg};
use script::layout_interface::{QueryMsg, Reflow, ReflowDocumentDamage, ReflowForDisplay};
use script::layout_interface::{ReflowMsg};
use script::script_task::{ReflowCompleteMsg, ScriptMsg, SendEventMsg};
use servo_net::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use servo_net::local_image_cache::LocalImageCache;
use servo_util::tree::{TreeNodeRef, TreeUtils};
use servo_util::time::{ProfilerChan, profile, time};
use servo_util::time;
use std::net::url::Url;

pub fn create_layout_task(render_task: RenderTask,
                          img_cache_task: ImageCacheTask,
                          opts: Opts,
                          profiler_chan: ProfilerChan)
                          -> LayoutTask {
    let chan = do spawn_listener::<Msg> |from_script| {
        let mut layout = Layout::new(render_task.clone(),
                                     img_cache_task.clone(),
                                     from_script,
                                     &opts,
                                     profiler_chan.clone());
        layout.start();
    };

    LayoutTask {
        chan: SharedChan::new(chan),
    }
}

struct Layout {
    render_task: RenderTask,
    image_cache_task: ImageCacheTask,
    local_image_cache: @mut LocalImageCache,
    from_script: Port<Msg>,
    font_ctx: @mut FontContext,
    doc_url: Option<Url>,
    screen_size: Option<Size2D<Au>>,

    /// This is used to root reader data.
    layout_refs: ~[@mut LayoutData],

    css_select_ctx: @mut SelectCtx,
    profiler_chan: ProfilerChan,
}

impl Layout {
    fn new(render_task: RenderTask, 
           image_cache_task: ImageCacheTask,
           from_script: Port<Msg>,
           opts: &Opts,
           profiler_chan: ProfilerChan)
           -> Layout {
        let fctx = @mut FontContext::new(opts.render_backend, true, profiler_chan.clone());

        Layout {
            render_task: render_task,
            image_cache_task: image_cache_task.clone(),
            local_image_cache: @mut LocalImageCache(image_cache_task),
            from_script: from_script,
            font_ctx: fctx,
            doc_url: None,
            screen_size: None,
            
            layout_refs: ~[],
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
        match self.from_script.recv() {
            AddStylesheetMsg(sheet) => self.handle_add_stylesheet(sheet),
            ReflowMsg(data) => {
                let data = Cell(data);

                do profile(time::LayoutPerformCategory, self.profiler_chan.clone()) {
                    self.handle_reflow(data.take());
                }
            }
            QueryMsg(query, chan) => {
                let chan = Cell(chan);
                do profile(time::LayoutQueryCategory, self.profiler_chan.clone()) {
                    self.handle_query(query, chan.take())
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
        let sheet = Cell(sheet);
        self.css_select_ctx.append_sheet(sheet.take(), OriginAuthor);
    }

    /// The high-level routine that performs layout tasks.
    fn handle_reflow(&mut self, data: &Reflow) {
        // FIXME: Isolate this transmutation into a "bridge" module.
        let node: &AbstractNode<LayoutView> = unsafe {
            transmute(&data.document_root)
        };

        // FIXME: Bad copy!
        let doc_url = copy data.url;
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
        self.screen_size = Some(screen_size);

        // Create a layout context for use throughout the following passes.
        let mut layout_ctx = self.build_layout_context();

        // Initialize layout data for each node.
        //
        // FIXME: This is inefficient. We don't need an entire traversal to do this!
        do profile(time::LayoutAuxInitCategory, self.profiler_chan.clone()) {
            node.initialize_style_for_subtree(&mut self.layout_refs);
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
        let layout_root: FlowContext = do profile(time::LayoutTreeBuilderCategory,
                                                  self.profiler_chan.clone()) {
            let mut builder = LayoutTreeBuilder::new();
            let layout_root: FlowContext = match builder.construct_trees(&layout_ctx, *node) {
                Ok(root) => root,
                Err(*) => fail!(~"Root flow should always exist")
            };

            debug!("layout: constructed Flow tree");
            debug!("%?", layout_root.dump());

            layout_root
        };

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        do profile(time::LayoutMainCategory, self.profiler_chan.clone()) {
            for layout_root.traverse_postorder |flow| {
                flow.bubble_widths(&mut layout_ctx);
            };
            for layout_root.traverse_preorder |flow| {
                flow.assign_widths(&mut layout_ctx);
            };
            for layout_root.traverse_postorder |flow| {
                flow.assign_height(&mut layout_ctx);
            };
        }

        // Build the display list if necessary, and send it to the renderer.
        if data.goal == ReflowForDisplay {
            do profile(time::LayoutDispListBuildCategory, self.profiler_chan.clone()) {
                let builder = DisplayListBuilder {
                    ctx: &layout_ctx,
                };

                let display_list = @Cell(DisplayList::new());

                // TODO: Set options on the builder before building.
                // TODO: Be smarter about what needs painting.
                layout_root.build_display_list(&builder, &layout_root.position(), display_list);

                let root_size = do layout_root.with_base |base| {
                    base.position.size
                };

                let render_layer = RenderLayer {
                    display_list: display_list.take(),
                    size: Size2D(root_size.width.to_px() as uint, root_size.height.to_px() as uint)
                };

                self.render_task.channel.send(RenderMsg(render_layer));
            } // time(layout: display list building)
        }

        debug!("%?", layout_root.dump());

        // Tell script that we're done.
        //
        // FIXME(pcwalton): This should probably be *one* channel, but we can't fix this without
        // either select or a filtered recv() that only looks for messages of a given type.
        data.script_join_chan.send(());
        data.script_chan.send(ReflowCompleteMsg);
    }

    /// Handles a query from the script task. This is the main routine that DOM functions like
    /// `getClientRects()` or `getBoundingClientRect()` ultimately invoke.
    fn handle_query(&self, query: LayoutQuery, reply_chan: Chan<Result<LayoutResponse,()>>) {
        match query {
            ContentBoxQuery(node) => {
                // FIXME: Isolate this transmutation into a single "bridge" module.
                let node: AbstractNode<LayoutView> = unsafe {
                    transmute(node)
                };

                let response = match node.layout_data().flow {
                    None => {
                        error!("no flow present");
                        Err(())
                    }
                    Some(flow) => {
                        let start_val: Option<Rect<Au>> = None;
                        let rect = do flow.foldl_boxes_for_node(node, start_val) |acc, box| {
                            match acc {
                                Some(acc) => Some(acc.union(&box.content_box())),
                                None => Some(box.content_box())
                            }
                        };
                        
                        match rect {
                            None => {
                                error!("no boxes for node");
                                Err(())
                            }
                            Some(rect) => Ok(ContentBoxResponse(rect))
                        }
                    }
                };

                reply_chan.send(response)
            }
            ContentBoxesQuery(node) => {
                // FIXME: Isolate this transmutation into a single "bridge" module.
                let node: AbstractNode<LayoutView> = unsafe {
                    transmute(node)
                };

                let response = match node.layout_data().flow {
                    None => Err(()),
                    Some(flow) => {
                        let mut boxes = ~[];
                        for flow.iter_boxes_for_node(node) |box| {
                            boxes.push(box.content_box());
                        }

                        Ok(ContentBoxesResponse(boxes))
                    }
                };

                reply_chan.send(response)
            }
            HitTestQuery(node, point) => {
                // FIXME: Isolate this transmutation into a single "bridge" module.
                let node: AbstractNode<LayoutView> = unsafe {
                    transmute(node)
                };
                let mut flow_node: AbstractNode<LayoutView> = node;
                for node.traverse_preorder |node| {
                    if node.layout_data().flow.is_some() {
                        flow_node = node;
                        break;
                    }
                };

                let response = match flow_node.layout_data().flow {
                    None => {
                        debug!("HitTestQuery: flow is None");
                        Err(())
                    }
                    Some(flow) => {
                        let layout_ctx = self.build_layout_context();
                        let builder = DisplayListBuilder {
                            ctx: &layout_ctx,
                        };
                        let display_list: @Cell<DisplayList<RenderBox>> =
                            @Cell(DisplayList::new());
                        flow.build_display_list(&builder,
                                                &flow.position(),
                                                display_list);
                        let (x, y) = (Au::from_frac_px(point.x as float),
                                      Au::from_frac_px(point.y as float));
                        let mut resp = Err(());
                        let display_list = &display_list.take().list;
                        // iterate in reverse to ensure we have the most recently painted render box
                        for display_list.each_reverse |display_item| {
                            let bounds = display_item.bounds();
                            // TODO this check should really be performed by a method of DisplayItem
                            if x <= bounds.origin.x + bounds.size.width &&
                               bounds.origin.x <= x &&
                               y < bounds.origin.y + bounds.size.height &&
                               bounds.origin.y <  y {
                                resp = Ok(HitTestResponse(display_item.base().extra.node()));
                                break;
                            }
                        }
                        resp
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
    fn make_on_image_available_cb(&self, script_chan: SharedChan<ScriptMsg>)
                                  -> @fn() -> ~fn(ImageResponseMsg) {
        // This has a crazy signature because the image cache needs to
        // make multiple copies of the callback, and the dom event
        // channel is not a copyable type, so this is actually a
        // little factory to produce callbacks
        let f: @fn() -> ~fn(ImageResponseMsg) = || {
            let script_chan = script_chan.clone();
            let f: ~fn(ImageResponseMsg) = |_| {
                script_chan.send(SendEventMsg(ReflowEvent))
            };
            f
        };
        return f;
    }
}

