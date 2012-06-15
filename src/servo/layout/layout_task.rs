#[doc = "
    The layout task. Performs layout on the DOM, builds display lists and sends them to be
    rendered.
"];

import display_list_builder::build_display_list;
import dom::base::{Node};
import dom::style::stylesheet;
import gfx::geometry::px_to_au;
import base::{NodeMethods, layout_methods};
import layout::style::style::style_methods;
import box_builder::box_builder_methods;
import layout::style::apply::ApplyStyleBoxMethods;

import task::*;
import comm::*;

enum Msg {
    BuildMsg(Node, stylesheet),
    PingMsg(chan<content::PingMsg>),
    ExitMsg
}

fn layout(to_renderer: chan<renderer::Msg>) -> chan<Msg> {
    spawn_listener::<Msg> { |po|
        loop {
            alt po.recv() {
                PingMsg(ping_channel) {
                    ping_channel.send(content::PongMsg);
                }
                ExitMsg {
                    break;
                }
                BuildMsg(node, styles) {
                    #debug("layout: received layout request for:");
                    node.dump();

                    node.initialize_style_for_subtree();
                    node.recompute_style_for_subtree(styles);

                    let this_box = node.construct_boxes();
                    this_box.dump();

                    this_box.apply_style_for_subtree();
                    this_box.reflow(px_to_au(800));

                    let dlist = build_display_list(this_box);
                    to_renderer.send(renderer::RenderMsg(dlist));
                }
            }
        }
    }
}
