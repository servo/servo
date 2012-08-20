#[doc = "
    The layout task. Performs layout on the DOM, builds display lists and sends them to be
    rendered.
"];

import std::arc::arc;
import display_list_builder::build_display_list;
import dom::base::Node;
import dom::style::Stylesheet;
import gfx::geometry::px_to_au;
import gfx::render_task;
import render_task::RenderTask;
import resource::image_cache_task::ImageCacheTask;
import std::net::url::url;
import style::apply::apply_style;
import dom::event::{Event, ReflowEvent};

import task::*;
import comm::*;

type LayoutTask = Chan<Msg>;

enum Msg {
    BuildMsg(Node, arc<Stylesheet>, url, Chan<Event>),
    PingMsg(Chan<content::PingMsg>),
    ExitMsg
}

fn LayoutTask(render_task: RenderTask, image_cache_task: ImageCacheTask) -> LayoutTask {
    do spawn_listener::<Msg>|request| {
        loop {
            match request.recv() {
              PingMsg(ping_channel) => ping_channel.send(content::PongMsg),
              ExitMsg => {
                #debug("layout: ExitMsg received");
                break;
              }
              BuildMsg(node, styles, doc_url, event_chan) => {
                #debug("layout: received layout request for:");
                node.dump();

                do util::time::time(~"layout") {
                    node.initialize_style_for_subtree();
                    node.recompute_style_for_subtree(styles);

                    let this_box = node.construct_boxes();
                    this_box.dump();

                    let reflow: fn~() = || event_chan.send(ReflowEvent);

                    apply_style(this_box, &doc_url, image_cache_task, reflow);

                    this_box.reflow_subtree(px_to_au(800));

                    let dlist = build_display_list(this_box);
                    render_task.send(render_task::RenderMsg(dlist));
                }
              }
            }
        }
    }
}
