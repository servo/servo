#[doc = "
    The layout task. Performs layout on the DOM, builds display lists and sends them to be
    rendered.
"];

use au = gfx::geometry;
use au::au;
use content::content_task;
use core::dvec::DVec;
use css::resolve::apply::apply_style;
use css::values::Stylesheet;
use dl = gfx::display_list;
use dom::node::{Node, LayoutData};
use dom::event::{Event, ReflowEvent};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::render_task;
use layout::base::RenderBox;
use layout::box_builder::LayoutTreeBuilder;
use layout::context::LayoutContext;
use render_task::RenderTask;
use resource::image_cache_task::ImageCacheTask;
use std::arc::ARC;
use std::net::url::Url;
use servo_text::font_cache::FontCache;

use layout::traverse::*;
use comm::*;
use task::*;

type LayoutTask = Chan<Msg>;

enum Msg {
    BuildMsg(Node, ARC<Stylesheet>, Url, Chan<Event>),
    PingMsg(Chan<content_task::PingMsg>),
    ExitMsg
}

fn LayoutTask(render_task: RenderTask, image_cache_task: ImageCacheTask) -> LayoutTask {
    do spawn_listener::<Msg>|request| {

        // This just keeps our dom aux objects alive
        let layout_data_refs = DVec();
        let font_cache = FontCache();

        loop {
            match request.recv() {
                PingMsg(ping_channel) => ping_channel.send(content_task::PongMsg),
                ExitMsg => {
                    debug!("layout: ExitMsg received");
                    break;
                }
                BuildMsg(node, styles, doc_url, event_chan) => {
                    debug!("layout: received layout request for: %s", doc_url.to_str());
                    debug!("layout: parsed Node tree");
                    node.dump();

                    let layout_ctx = LayoutContext {
                        image_cache: image_cache_task,
                        font_cache: font_cache,
                        doc_url: doc_url,
                        reflow_cb: || event_chan.send(ReflowEvent),
                        // TODO: obtain screen size from a real data source
                        screen_size: Rect(Point2D(au(0), au(0)), Size2D(au::from_px(800), au::from_px(600)))
                    };

                    do util::time::time(~"layout") {
                        // TODO: this is dumb. we don't need 3 separate traversals.
                        node.initialize_style_for_subtree(&layout_ctx, &layout_data_refs);
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
                        layout_root.build_display_list(&builder, &copy layout_root.data.position, &dlist);
                        render_task.send(render_task::RenderMsg(dlist));
                    }
                }
            }
        }
    }
}
