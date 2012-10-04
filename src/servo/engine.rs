use gfx::compositor::Compositor;
use mod gfx::render_task;
use gfx::render_task::{Renderer, RenderTask};
use task::spawn_listener;
use layout::layout_task;
use layout_task::LayoutTask;
use mod content::content_task;
use content::content_task::{ContentTask, ExecuteMsg, ParseMsg, ExitMsg};
use resource::resource_task;
use resource::resource_task::ResourceTask;
use std::net::url::Url;
use resource::image_cache_task;
use image_cache_task::{ImageCacheTask, image_cache_task, ImageCacheTaskClient};
use pipes::{Port, Chan};

pub struct Engine<C:Compositor Send Copy> {
    compositor: C,
    render_task: Renderer,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    layout_task: LayoutTask,
    content_task: ContentTask
}

pub fn Engine<C:Compositor Send Copy>(compositor: C,
                                      resource_task: ResourceTask,
                                      image_cache_task: ImageCacheTask) -> Engine<C> {
    let render_task = RenderTask(compositor);
    let layout_task = LayoutTask(render_task, image_cache_task);
    let content_task = ContentTask(layout_task, compositor, resource_task, image_cache_task);

    Engine {
        compositor: compositor,
        render_task: render_task,
        resource_task: resource_task,
        image_cache_task: image_cache_task,
        layout_task: layout_task,
        content_task: content_task
    }
}

impl<C: Compositor Copy Send> Engine<C> {
    fn start() -> comm::Chan<Msg> {
        do spawn_listener::<Msg> |request| {
            while self.handle_request(request.recv()) {
                // Go on...
            }
        }
    }

    fn handle_request(request: Msg) -> bool {
        match request {
          LoadURLMsg(url) => {
            // TODO: change copy to move once we have match move
            let url = copy url;
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
            
            let (response_chan, response_port) = pipes::stream();

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

pub enum Msg {
    LoadURLMsg(Url),
    ExitMsg(Chan<()>)
}

