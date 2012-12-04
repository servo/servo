use content::content_task::{ContentTask, ExecuteMsg, ParseMsg, ExitMsg};
use content::content_task;
use dom::event::Event;
use layout::layout_task;
use layout_task::LayoutTask;
use resource::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use resource::resource_task::ResourceTask;
use resource::resource_task;

use core::pipes::{Port, Chan};
use core::task::spawn_listener;
use gfx::compositor::Compositor;
use gfx::opts::Opts;
use gfx::render_task::RenderTask;
use gfx::render_task;
use std::cell::Cell;
use std::net::url::Url;

pub type EngineTask = comm::Chan<Msg>;

pub enum Msg {
    LoadURLMsg(Url),
    ExitMsg(Chan<()>)
}

pub struct Engine<C:Compositor Send Copy> {
    request_port: comm::Port<Msg>,
    compositor: C,
    render_task: RenderTask,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    layout_task: LayoutTask,
    content_task: ContentTask
}

pub fn Engine<C:Compositor Send Copy>(compositor: C,
                                  opts: &Opts,
                                  dom_event_port: pipes::Port<Event>,
                                  dom_event_chan: pipes::SharedChan<Event>,
                                  resource_task: ResourceTask,
                                  image_cache_task: ImageCacheTask) -> EngineTask {

    let dom_event_port = Cell(move dom_event_port);
    let dom_event_chan = Cell(move dom_event_chan);

    let opts = Cell(copy *opts);
    do spawn_listener::<Msg> |request, move dom_event_port, move dom_event_chan,
                              move image_cache_task, move opts| {
        let render_task = RenderTask(compositor, opts.with_ref(|o| copy *o));
        let layout_task = LayoutTask(render_task, image_cache_task.clone(), opts.take());
        let content_task = ContentTask(layout_task,
                                       dom_event_port.take(), dom_event_chan.take(),
                                       resource_task, image_cache_task.clone());

        Engine {
            request_port: request,
            compositor: compositor,
            render_task: render_task,
            resource_task: resource_task,
            image_cache_task: image_cache_task.clone(),
            layout_task: move layout_task,
            content_task: move content_task
        }.run();
    }
}

impl<C: Compositor Copy Send> Engine<C> {
    fn run() {
        while self.handle_request(self.request_port.recv()) {
            // Go on...
        }
    }

    fn handle_request(request: Msg) -> bool {
        match move request {
          LoadURLMsg(move url) => {
            if url.path.ends_with(".js") {
                self.content_task.send(ExecuteMsg(move url))
            } else {
                self.content_task.send(ParseMsg(move url))
            }
            return true;
          }

          ExitMsg(move sender) => {
            self.content_task.send(content_task::ExitMsg);
            self.layout_task.send(layout_task::ExitMsg);
            
            let (response_chan, response_port) = pipes::stream();

            self.render_task.send(render_task::ExitMsg(move response_chan));
            response_port.recv();

            self.image_cache_task.exit();
            self.resource_task.send(resource_task::Exit);

            sender.send(());
            return false;
          }
        }
    }
}

