/**
    The layout task. Performs layout on the DOM, builds display lists and sends them to be
    rendered.
*/

use content::content_task;
use css::select::new_css_select_ctx;
use dom::event::{Event, ReflowEvent};
use dom::node::{Node, LayoutData};
use layout::box::RenderBox;
use layout::box_builder::LayoutTreeBuilder;
use layout::context::LayoutContext;
use layout::display_list_builder::DisplayListBuilder;
use layout::traverse::*;
use resource::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use resource::local_image_cache::LocalImageCache;
use util::time::time;
use util::task::spawn_listener;

use core::oldcomm::*;  // FIXME: Bad! Pipe-ify me.
use core::dvec::DVec;
use core::mutable::Mut;
use core::task::*;
use core::util::replace;
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
use opt = core::option;
use std::arc::ARC;
use std::cell::Cell;
use std::net::url::Url;

pub type LayoutTask = oldcomm::Chan<Msg>;

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
    QueryMsg(LayoutQuery, oldcomm::Chan<LayoutQueryResponse>),
    ExitMsg
}

// Dirty bits for layout.
pub enum Damage {
    NoDamage,               // Document is clean; do nothing.
    ReflowDamage,           // Reflow; don't perform CSS selector matching.
    MatchSelectorsDamage,   // Perform CSS selector matching and reflow.
}

impl Damage {
    fn add(&mut self, new_damage: Damage) {
        match (*self, new_damage) {
            (NoDamage, _) => *self = new_damage,
            (ReflowDamage, NoDamage) => *self = ReflowDamage,
            (ReflowDamage, new_damage) => *self = new_damage,
            (MatchSelectorsDamage, _) => *self = MatchSelectorsDamage
        }
    }
}

pub struct BuildData {
    node: Node,
    url: Url,
    dom_event_chan: pipes::SharedChan<Event>,
    window_size: Size2D<uint>,
    content_join_chan: pipes::Chan<()>,
    damage: Damage,
}

pub fn LayoutTask(render_task: RenderTask,
              img_cache_task: ImageCacheTask,
              opts: Opts) -> LayoutTask {
    do spawn_listener::<Msg> |from_content, move img_cache_task, move opts| {
        Layout(render_task, img_cache_task.clone(), from_content, &opts).start();
    }
}

struct Layout {
    render_task: RenderTask,
    image_cache_task: ImageCacheTask,
    local_image_cache: @LocalImageCache,
    from_content: oldcomm::Port<Msg>,

    font_ctx: @FontContext,
    // This is used to root auxilliary RCU reader data
    layout_refs: DVec<@LayoutData>,
    css_select_ctx: Mut<SelectCtx>,
}

fn Layout(render_task: RenderTask, 
          image_cache_task: ImageCacheTask,
          from_content: oldcomm::Port<Msg>,
          opts: &Opts) -> Layout {

    let fctx = @FontContext::new(opts.render_backend, true);

    Layout {
        render_task: render_task,
        image_cache_task: image_cache_task.clone(),
        local_image_cache: @LocalImageCache(move image_cache_task),
        from_content: from_content,
        font_ctx: fctx,
        layout_refs: DVec(),
        css_select_ctx: Mut(new_css_select_ctx())
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
            ctx.append_sheet(sheet.take(), OriginAuthor);
        }
    }

    fn handle_build(data: BuildData) {
        let node = &data.node;
        // FIXME: Bad copy
        let doc_url = copy data.url;
        // FIXME: Bad clone
        let dom_event_chan = data.dom_event_chan.clone();

        debug!("layout: received layout request for: %s", doc_url.to_str());
        debug!("layout: damage is %?", data.damage);
        debug!("layout: parsed Node tree");
        debug!("%?", node.dump());

        // Reset the image cache
        self.local_image_cache.next_round(self.make_on_image_available_cb(move dom_event_chan));

        let screen_size = Size2D(Au::from_px(data.window_size.width as int),
                                 Au::from_px(data.window_size.height as int));

        let layout_ctx = LayoutContext {
            image_cache: self.local_image_cache,
            font_ctx: self.font_ctx,
            doc_url: move doc_url,
            screen_size: Rect(Point2D(Au(0), Au(0)), screen_size)
        };

        do time("layout: aux initialization") {
            // TODO: this is dumb. we don't need an entire traversal to do this
            node.initialize_style_for_subtree(&self.layout_refs);
        }

        // Perform CSS selector matching if necessary.
        match data.damage {
            NoDamage | ReflowDamage => {}
            MatchSelectorsDamage => {
                do time("layout: selector matching") {
                    do self.css_select_ctx.borrow_imm |ctx| {
                        node.restyle_subtree(ctx);
                    }
                }
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
            let builder = DisplayListBuilder {
                ctx: &layout_ctx,
            };

            let display_list = Mut(DisplayList::new());
            
            // TODO: set options on the builder before building
            // TODO: be smarter about what needs painting
            layout_root.build_display_list(&builder, &copy layout_root.d().position,
                                           &display_list);

            let render_layer = RenderLayer {
                display_list: display_list.unwrap(),
                size: Size2D(screen_size.width.to_px() as uint,
                             screen_size.height.to_px() as uint)
            };

            self.render_task.send(RenderMsg(move render_layer));
        } // time(layout: display list building)

        // Tell content we're done
        data.content_join_chan.send(());
    }


    fn handle_query(query: LayoutQuery, 
                    reply_chan: oldcomm::Chan<LayoutQueryResponse>) {
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
                                    let size = Size2D(rect.size.width.to_px(),
                                                      rect.size.height.to_px());
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

