#[doc = "
    The layout task. Performs layout on the DOM, builds display lists and sends them to be
    rendered.
"];

use au = gfx::geometry;
use content::content_task;
use css::resolve::apply::apply_style;
use css::values::Stylesheet;
use display_list_builder::build_display_list;
use dom::base::Node;
use dom::event::{Event, ReflowEvent};
use gfx::render_task;
use layout::base::Box;
use layout::box_builder::LayoutTreeBuilder;
use render_task::RenderTask;
use resource::image_cache_task::ImageCacheTask;
use std::arc::ARC;
use std::net::url::Url;

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
        let mut layout_data_refs = ~[];

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

                do util::time::time(~"layout") {
                    layout_data_refs += node.initialize_style_for_subtree();
                    node.recompute_style_for_subtree(styles);

                    let root_box: @Box;
                    let builder = LayoutTreeBuilder();
                    match builder.construct_trees(node) {
                        Ok(root) => root_box = root,
                        Err(*) => fail ~"Root node should always exist"
                    }

                    debug!("layout: constructed Box tree");
                    root_box.dump();

                    debug!("layout: constructed Flow tree");
                    root_box.ctx.dump();

                    /* resolve styles (convert relative values) down the box tree */
                    let reflow_cb: fn~() = || event_chan.send(ReflowEvent);
                    apply_style(root_box, &doc_url, image_cache_task, reflow_cb);

                    /* perform layout passes over the flow tree */
                    let root_flow = root_box.ctx;
                    do root_flow.traverse_postorder |f| { f.bubble_widths() }
                    root_flow.data.position.origin = au::zero_point();
                    root_flow.data.position.size.width = au::from_px(800); // TODO: window/frame size
                    do root_flow.traverse_preorder |f| { f.assign_widths() }
                    do root_flow.traverse_postorder |f| { f.assign_height() }

                    let dlist = build_display_list(root_box);
                    render_task.send(render_task::RenderMsg(dlist));
                }
              }
            }
        }
    }
}
