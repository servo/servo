/**
    The layout task. Performs layout on the DOM, builds display lists and sends them to be
    rendered.
*/

use au = gfx::geometry;
use au::au;
use content::content_task;
use core::dvec::DVec;
use css::resolve::apply::apply_style;
use css::values::Stylesheet;
use dl = gfx::display_list;
use dom::event::{Event, ReflowEvent};
use dom::node::{Node, LayoutData};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::render_task;
use gfx::render_layers::RenderLayer;
use layout::box::RenderBox;
use layout::box_builder::LayoutTreeBuilder;
use layout::context::LayoutContext;
use opt = core::option;
use render_task::RenderTask;
use resource::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use resource::local_image_cache::LocalImageCache;
use servo_text::font_cache::FontCache;
use std::arc::ARC;
use std::net::url::Url;
use core::util::replace;

use layout::traverse::*;
use comm::*;
use task::*;

pub type LayoutTask = comm::Chan<Msg>;

pub enum LayoutQuery {
    ContentBox(Node)
}

pub type LayoutQueryResponse = Result<LayoutQueryResponse_, ()>;

enum LayoutQueryResponse_ {
    ContentSize(Size2D<int>)
}

pub enum Msg {
    BuildMsg(Node, ARC<Stylesheet>, Url, comm::Chan<Event>, Size2D<uint>, pipes::Chan<()>),
    QueryMsg(LayoutQuery, comm::Chan<LayoutQueryResponse>),
    ExitMsg
}

fn LayoutTask(render_task: RenderTask,
              img_cache_task: ImageCacheTask) -> LayoutTask {
    do spawn_listener::<Msg> |from_content| {
        Layout(render_task, img_cache_task.clone(), from_content).start();
    }
}

struct Layout {
    render_task: RenderTask,
    image_cache_task: ImageCacheTask,
    local_image_cache: @LocalImageCache,
    from_content: comm::Port<Msg>,

    font_cache: @FontCache,
    // This is used to root auxilliary RCU reader data
    layout_refs: DVec<@LayoutData>
}

fn Layout(render_task: RenderTask, 
         image_cache_task: ImageCacheTask,
         from_content: comm::Port<Msg>) -> Layout {

    Layout {
        render_task: render_task,
        image_cache_task: image_cache_task.clone(),
        local_image_cache: @LocalImageCache(move image_cache_task),
        from_content: from_content,
        font_cache: FontCache(),
        layout_refs: DVec()
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
            BuildMsg(move node, move styles, move doc_url,
                     move to_content, move window_size, move join_chan) => {
                self.handle_build(node, styles, doc_url, to_content, window_size, join_chan);
            }
            QueryMsg(query, chan) => self.handle_query(query, chan),
            ExitMsg => {
                debug!("layout: ExitMsg received");
                return false
            }
        }

        true
    }

    fn handle_build(node: Node, styles: ARC<Stylesheet>, doc_url: Url,
                    to_content: comm::Chan<Event>, window_size: Size2D<uint>,
                    join_chan: pipes::Chan<()>) {
        debug!("layout: received layout request for: %s", doc_url.to_str());
        debug!("layout: parsed Node tree");
        debug!("%?", node.dump());

        // Reset the image cache
        self.local_image_cache.next_round(self.make_on_image_available_cb(to_content));

        let screen_size = Size2D(au::from_px(window_size.width as int),
                                 au::from_px(window_size.height as int));

        let layout_ctx = LayoutContext {
            image_cache: self.local_image_cache,
            font_cache: self.font_cache,
            doc_url: doc_url,
            screen_size: Rect(Point2D(au(0), au(0)), screen_size)
        };

        let layout_root: @FlowContext = do util::time::time("layout: tree construction") {
            // TODO: this is dumb. we don't need 3 separate traversals.
            node.initialize_style_for_subtree(&layout_ctx, &self.layout_refs);
            node.recompute_style_for_subtree(&layout_ctx, &styles);
            /* resolve styles (convert relative values) down the node tree */
            apply_style(&layout_ctx, node);
            
            let builder = LayoutTreeBuilder();
            let layout_root: @FlowContext = match builder.construct_trees(&layout_ctx,
                                                                          node) {
                Ok(root) => root,
                Err(*) => fail ~"Root flow should always exist"
            };

            debug!("layout: constructed Flow tree");
            debug!("%?", layout_root.dump());

            layout_root
        };

        do util::time::time("layout: main layout") {
            /* perform layout passes over the flow tree */
            do layout_root.traverse_postorder |f| { f.bubble_widths(&layout_ctx) }
            do layout_root.traverse_preorder  |f| { f.assign_widths(&layout_ctx) }
            do layout_root.traverse_postorder |f| { f.assign_height(&layout_ctx) }
        }

        do util::time::time("layout: display list building") {
            let dlist = DVec();
            let builder = dl::DisplayListBuilder {
                ctx: &layout_ctx,
            };
            let render_layer = RenderLayer {
                display_list: move dlist,
                size: Size2D(au::to_px(screen_size.width) as uint,
                             au::to_px(screen_size.height) as uint)
            };

            // TODO: set options on the builder before building
            // TODO: be smarter about what needs painting
            layout_root.build_display_list(&builder, &copy layout_root.d().position,
                                           &render_layer.display_list);
            self.render_task.send(render_task::RenderMsg(move render_layer));
        } // time(layout: display list building)

        // Tell content we're done
        join_chan.send(());

    }


    fn handle_query(query: LayoutQuery, 
                    reply_chan: comm::Chan<LayoutQueryResponse>) {
        match query {
            ContentBox(node) => {
                // TODO: extract me to a method when I get sibling arms
                let response = match node.aux(|a| copy *a).flow {
                    None => Err(()),
                    Some(flow) => {
                        let start_val : Option<Rect<au>> = None;
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
                };

                reply_chan.send(response)
            }
        }
    }

    // When images can't be loaded in time to display they trigger
    // this callback in some task somewhere. This w
    fn make_on_image_available_cb(to_content: comm::Chan<Event>) -> ~fn(ImageResponseMsg) {
        let f: ~fn(ImageResponseMsg) = |_msg| {
            to_content.send(ReflowEvent)
        };
        return f;
    }
}

