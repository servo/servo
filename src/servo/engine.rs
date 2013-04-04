use content::content_task::{ContentTask, ExecuteMsg, ParseMsg};
use content::content_task;
use dom::event::Event;
use layout::layout_task;
use layout::layout_task::LayoutTask;
use resource::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use resource::resource_task::ResourceTask;
use resource::resource_task;
use util::task::spawn_listener;

use core::cell::Cell;
use core::comm::{Port, Chan};
use gfx::compositor::Compositor;
use gfx::opts::Opts;
use gfx::render_task::RenderTask;
use gfx::render_task;
use std::net::url::Url;

pub type EngineTask = Chan<Msg>;

pub enum Msg {
    LoadURLMsg(Url),
    ExitMsg(Chan<()>)
}

pub struct Engine<C> {
    request_port: Port<Msg>,
    compositor: C,
    render_task: RenderTask,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    layout_task: LayoutTask,
    content_task: ContentTask
}

pub fn Engine<C:Compositor + Owned + Clone>(compositor: C,
                                            opts: &Opts,
                                            dom_event_port: comm::Port<Event>,
                                            dom_event_chan: comm::SharedChan<Event>,
                                            resource_task: ResourceTask,
                                            image_cache_task: ImageCacheTask)
                                         -> EngineTask {
    let dom_event_port = Cell(dom_event_port);
    let dom_event_chan = Cell(dom_event_chan);

    let opts = Cell(copy *opts);
    do spawn_listener::<Msg> |request| {
        let render_task = RenderTask(compositor.clone(), opts.with_ref(|o| copy *o));
        let layout_task = LayoutTask(render_task.clone(), image_cache_task.clone(), opts.take());
        let content_task = ContentTask(layout_task.clone(),
                                       dom_event_port.take(),
                                       dom_event_chan.take(),
                                       resource_task.clone(),
                                       image_cache_task.clone());

        Engine {
            request_port: request,
            compositor: compositor.clone(),
            render_task: render_task,
            resource_task: resource_task.clone(),
            image_cache_task: image_cache_task.clone(),
            layout_task: layout_task,
            content_task: content_task
        }.run();
    }
}

impl<C:Compositor + Owned + Clone> Engine<C> {
    fn run(&self) {
        while self.handle_request(self.request_port.recv()) {
            // Go on...
        }
    }

    fn handle_request(&self, request: Msg) -> bool {
        match request {
          LoadURLMsg(url) => {
            if url.path.ends_with(".js") {
                self.content_task.send(ExecuteMsg(url))
            } else {
                self.content_task.send(ParseMsg(url))
            }
            return true;
          }

          ExitMsg(sender) => {
            self.content_task.send(content_task::ExitMsg);
            self.layout_task.send(layout_task::ExitMsg);
            
            let (response_port, response_chan) = comm::stream();

            self.render_task.send(render_task::ExitMsg(response_chan));
            response_port.recv();

            self.image_cache_task.exit();
            self.resource_task.send(resource_task::Exit);

            sender.send(());
            return false;
          }
        }
    }
}

