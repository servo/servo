#[doc = "
    The layout task. Performs layout on the DOM, builds display lists and sends them to be
    rendered.
"];

import std::arc::ARC;
import display_list_builder::build_display_list;
import dom::base::Node;
import css::values::Stylesheet;
import gfx::geometry::px_to_au;
import gfx::render_task;
import render_task::RenderTask;
import layout::base::Box;
import resource::image_cache_task::ImageCacheTask;
import std::net::url::Url;
import css::resolve::apply::apply_style;
import dom::event::{Event, ReflowEvent};
import content::content_task;

import task::*;
import comm::*;

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
                #debug("layout: ExitMsg received");
                break;
              }
              BuildMsg(node, styles, doc_url, event_chan) => {
                #debug("layout: received layout request for:");
                node.dump();

                do util::time::time(~"layout") {
                    layout_data_refs += node.initialize_style_for_subtree();
                    node.recompute_style_for_subtree(styles);

                    let root_box: @Box;
                    match node.construct_boxes() {
                        None => fail ~"Root node should always exist; did it get 'display: none' somehow?",
                        Some(root) => root_box = root
                    }
                    
                    root_box.dump();

                    let reflow: fn~() = || event_chan.send(ReflowEvent);

                    apply_style(root_box, &doc_url, image_cache_task, reflow);

                    root_box.reflow_subtree(px_to_au(800));

                    let dlist = build_display_list(root_box);
                    render_task.send(render_task::RenderMsg(dlist));
                }
              }
            }
        }
    }
}
