/**
    The layout task. Performs layout on the DOM, builds display lists and sends them to be
    rendered.
*/

use au = gfx::geometry;
use au::Au;
use content::content_task;
use core::dvec::DVec;
use newcss::Stylesheet;
use dl = gfx::display_list;
use dom::event::{Event, ReflowEvent};
use dom::node::{Node, LayoutData};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::display_list::DisplayList;
use gfx::render_task;
use gfx::render_layers::RenderLayer;
use layout::box::RenderBox;
use layout::box_builder::LayoutTreeBuilder;
use layout::context::LayoutContext;
use opt = core::option;
use render_task::RenderTask;
use resource::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use resource::local_image_cache::LocalImageCache;
use servo_text::font_context::FontContext;
use servo_text::font_cache::FontCache;
use servo_text::font_matcher::FontMatcher;
use std::arc::ARC;
use std::net::url::Url;
use core::util::replace;
use util::time::time;
use std::cell::Cell;
use layout::traverse::*;
use comm::*;
use task::*;
use core::mutable::Mut;
use newcss::SelectCtx;

pub type LayoutTask = comm::Chan<Msg>;

pub enum LayoutQuery {
    ContentBox(Node)
}

pub type LayoutQueryResponse = Result<LayoutQueryResponse_, ()>;

enum LayoutQueryResponse_ {
    ContentSize(Size2D<int>)
}

pub enum Msg {
    AddStylesheet(Stylesheet),
    BuildMsg(BuildData),
    QueryMsg(LayoutQuery, comm::Chan<LayoutQueryResponse>),
    ExitMsg
}

struct BuildData {
    node: Node,
    url: Url,
    dom_event_chan: pipes::SharedChan<Event>,
    window_size: Size2D<uint>,
    content_join_chan: pipes::Chan<()>
}

fn LayoutTask(render_task: RenderTask,
              img_cache_task: ImageCacheTask) -> LayoutTask {
    do spawn_listener::<Msg> |from_content, move img_cache_task| {
        Layout(render_task, img_cache_task.clone(), from_content).start();
    }
}

struct Layout {
    render_task: RenderTask,
    image_cache_task: ImageCacheTask,
    local_image_cache: @LocalImageCache,
    from_content: comm::Port<Msg>,

    font_cache: @FontCache,
    font_matcher: @FontMatcher,
    // This is used to root auxilliary RCU reader data
    layout_refs: DVec<@LayoutData>,
    css_select_ctx: Mut<SelectCtx>,
}

fn Layout(render_task: RenderTask, 
         image_cache_task: ImageCacheTask,
         from_content: comm::Port<Msg>) -> Layout {

    let fctx = @FontContext::new();

    Layout {
        render_task: render_task,
        image_cache_task: image_cache_task.clone(),
        local_image_cache: @LocalImageCache(move image_cache_task),
        from_content: from_content,
        font_matcher: @FontMatcher::new(fctx),
        font_cache: @FontCache::new(fctx),
        layout_refs: DVec(),
        css_select_ctx: Mut(SelectCtx::new())
    }
}

impl Layout {

    fn start() {
        while self.handle_request() {
            // loop indefinitely
        }
    }

    fn handle_request() -> bool {

        match self.from_content.recv() {
            AddStylesheet(move sheet) => {
                self.handle_add_stylesheet(move sheet);
            }
            BuildMsg(move data) => {
                let data = Cell(move data);

                do time("layout: performing layout") {
                    self.handle_build(data.take());
                }

            }
            QueryMsg(query, chan) => {
                do time("layout: querying layout") {
                    self.handle_query(query, chan)
                }
            }
            ExitMsg => {
                debug!("layout: ExitMsg received");
                return false
            }
        }

        true
    }

    fn handle_add_stylesheet(sheet: Stylesheet) {
        let sheet = Cell(move sheet);
        do self.css_select_ctx.borrow_mut |ctx| {
            ctx.append_sheet(sheet.take());
        }
    }

    fn handle_build(data: BuildData) {

        let node = &data.node;
        // FIXME: Bad copy
        let doc_url = copy data.url;
        // FIXME: Bad clone
        let dom_event_chan = data.dom_event_chan.clone();

        debug!("layout: received layout request for: %s", doc_url.to_str());
        debug!("layout: parsed Node tree");
        debug!("%?", node.dump());

        // Reset the image cache
        self.local_image_cache.next_round(self.make_on_image_available_cb(move dom_event_chan));

        let screen_size = Size2D(au::from_px(data.window_size.width as int),
                                 au::from_px(data.window_size.height as int));

        let layout_ctx = LayoutContext {
            image_cache: self.local_image_cache,
            font_cache: self.font_cache,
            doc_url: move doc_url,
            screen_size: Rect(Point2D(Au(0), Au(0)), screen_size)
        };

        do time("layout: aux initialization") {
            // TODO: this is dumb. we don't need an entire traversal to do this
            node.initialize_style_for_subtree(&self.layout_refs);
        }

        do time("layout: selector matching") {
            do self.css_select_ctx.borrow_imm |ctx| {
                node.restyle_subtree(ctx);
            }
        }

        let layout_root: @FlowContext = do time("layout: tree construction") {
            let builder = LayoutTreeBuilder::new();
            let layout_root: @FlowContext = match builder.construct_trees(&layout_ctx,
                                                                          *node) {
                Ok(root) => root,
                Err(*) => fail ~"Root flow should always exist"
            };

            debug!("layout: constructed Flow tree");
            debug!("%?", layout_root.dump());

            layout_root
        };

        do time("layout: main layout") {
            /* perform layout passes over the flow tree */
            do layout_root.traverse_postorder |f| { f.bubble_widths(&layout_ctx) }
            do layout_root.traverse_preorder  |f| { f.assign_widths(&layout_ctx) }
            do layout_root.traverse_postorder |f| { f.assign_height(&layout_ctx) }
        }

        do time("layout: display list building") {
            let builder = dl::DisplayListBuilder {
                ctx: &layout_ctx,
            };
            let mut render_layer = RenderLayer {
                display_list: DisplayList::new(),
                size: Size2D(au::to_px(screen_size.width) as uint,
                             au::to_px(screen_size.height) as uint)
            };

            // TODO: set options on the builder before building
            // TODO: be smarter about what needs painting
            layout_root.build_display_list(&builder, &copy layout_root.d().position,
                                           &mut render_layer.display_list);
            self.render_task.send(render_task::RenderMsg(move render_layer));
        } // time(layout: display list building)

        // Tell content we're done
        data.content_join_chan.send(());
    }


    fn handle_query(query: LayoutQuery, 
                    reply_chan: comm::Chan<LayoutQueryResponse>) {
        match query {
            ContentBox(node) => {
                let response = do node.aux |a| {
                    match a.flow {
                        None => Err(()),
                        Some(flow) => {
                            let start_val : Option<Rect<Au>> = None;
                            let rect = do flow.foldl_boxes_for_node(node, start_val) |acc, box| {
                                match acc {
                                    Some(acc) => Some(acc.union(&box.content_box())),
                                    None => Some(box.content_box())
                                }
                            };
                            
                            match rect {
                                None => Err(()),
                                Some(rect) => {
                                    let size = Size2D(au::to_px(rect.size.width),
                                                      au::to_px(rect.size.height));
                                    Ok(ContentSize(move size))
                                }
                            }
                        }
                    }
                };

                reply_chan.send(response)
            }
        }
    }

    // When images can't be loaded in time to display they trigger
    // this callback in some task somewhere. This will send a message
    // to the content task, and ultimately cause the image to be
    // re-requested. We probably don't need to go all the way back to
    // the content task for this.
    fn make_on_image_available_cb(dom_event_chan: pipes::SharedChan<Event>) -> @fn() -> ~fn(ImageResponseMsg) {
        // This has a crazy signature because the image cache needs to
        // make multiple copies of the callback, and the dom event
        // channel is not a copyable type, so this is actually a
        // little factory to produce callbacks
        let f: @fn() -> ~fn(ImageResponseMsg) = |move dom_event_chan| {
            let dom_event_chan = dom_event_chan.clone();
            let f: ~fn(ImageResponseMsg) = |_msg, move dom_event_chan| {
                dom_event_chan.send(ReflowEvent)
            };
            move f
        };
        return f;
    }
}

