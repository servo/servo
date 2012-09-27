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
use layout::box::RenderBox;
use layout::box_builder::LayoutTreeBuilder;
use layout::context::LayoutContext;
use opt = core::option;
use render_task::RenderTask;
use resource::image_cache_task::ImageCacheTask;
use servo_text::font_cache::FontCache;
use std::arc::ARC;
use std::net::url::Url;

use layout::traverse::*;
use comm::*;
use task::*;

type LayoutTask = Chan<Msg>;

enum LayoutQuery {
    ContentBox(Node)
}

type LayoutQueryResponse = Result<LayoutQueryResponse_, ()>;

enum LayoutQueryResponse_ {
    ContentSize(Size2D<int>)
}

enum Msg {
    BuildMsg(Node, ARC<Stylesheet>, Url, Chan<Event>),
    PingMsg(Chan<content_task::PingMsg>),
    QueryMsg(LayoutQuery, Chan<LayoutQueryResponse>),
    ExitMsg
}

fn LayoutTask(render_task: RenderTask,
              img_cache_task: ImageCacheTask) -> LayoutTask {
    do spawn_listener::<Msg>|from_content| {
        Layout(render_task, img_cache_task, from_content).start();
    }
}

struct Layout {
    render_task: RenderTask,
    image_cache_task: ImageCacheTask,
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
        image_cache_task: image_cache_task,
        from_content: from_content,
        font_cache: FontCache(),
        layout_refs: DVec()
    }
}

impl Layout {

    fn handle_query(query: LayoutQuery, 
                    reply_chan: comm::Chan<LayoutQueryResponse>) {
        match query {
            ContentBox(node) => {
                // TODO: extract me to a method when I get sibling arms
                let response = match node.aux(|a| a).flow {
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

    fn start() {
        while self.handle_request(self.from_content) {
            // loop indefinitely
        }
    }
        
    fn handle_request(request: comm::Port<Msg>) -> bool {
        match request.recv() {
            PingMsg(ping_channel) => ping_channel.send(content_task::PongMsg),
            QueryMsg(query, chan) => self.handle_query(query, chan),
            ExitMsg => {
                debug!("layout: ExitMsg received");
                return false
            },
            BuildMsg(node, styles, doc_url, to_content) => {
                debug!("layout: received layout request for: %s", doc_url.to_str());
                debug!("layout: parsed Node tree");
                node.dump();

                let layout_ctx = LayoutContext {
                    image_cache: self.image_cache_task,
                    font_cache: self.font_cache,
                    doc_url: doc_url,
                    reflow_cb: || to_content.send(ReflowEvent),
                    // TODO: obtain screen size from a real data source
                    screen_size: Rect(Point2D(au(0), au(0)), Size2D(au::from_px(800), au::from_px(600)))
                };

                do util::time::time(~"layout") {
                    // TODO: this is dumb. we don't need 3 separate traversals.
                    node.initialize_style_for_subtree(&layout_ctx, &self.layout_refs);
                    node.recompute_style_for_subtree(&layout_ctx, styles);
                    /* resolve styles (convert relative values) down the node tree */
                    apply_style(&layout_ctx, node, layout_ctx.reflow_cb);
                    
                    let builder = LayoutTreeBuilder();
                    let layout_root: @FlowContext = match builder.construct_trees(&layout_ctx, node) {
                        Ok(root) => root,
                        Err(*) => fail ~"Root flow should always exist"
                    };

                    debug!("layout: constructed Flow tree");
                    layout_root.dump();

                    /* perform layout passes over the flow tree */
                    do layout_root.traverse_postorder |f| { f.bubble_widths(&layout_ctx) }
                    do layout_root.traverse_preorder  |f| { f.assign_widths(&layout_ctx) }
                    do layout_root.traverse_postorder |f| { f.assign_height(&layout_ctx) }

                    let dlist = DVec();
                    let builder = dl::DisplayListBuilder {
                        ctx: &layout_ctx,
                    };
                    // TODO: set options on the builder before building
                    // TODO: be smarter about what needs painting
                    layout_root.build_display_list(&builder, &copy layout_root.d().position, &dlist);
                    self.render_task.send(render_task::RenderMsg(dlist));
                } // time(layout)
            } // BuildMsg
        } // match

        true
    }
}

