export EngineTask, EngineTask_, EngineProto;

use gfx::compositor::Compositor;
use gfx::render_task;
use render_task::RenderTask;
use pipes::{spawn_service, select};
use layout::layout_task;
use layout_task::LayoutTask;
use content::content_task;
use content_task::{ContentTask};
use resource::resource_task;
use resource::resource_task::{ResourceTask};
use std::net::url::Url;
use resource::image_cache_task;
use image_cache_task::{ImageCacheTask, ImageCacheTaskClient};

use pipes::{Port, Chan};

fn macros() {
    include!("macros.rs");
}

type EngineTask = EngineProto::client::Running;

fn EngineTask<C: Compositor Send Copy>(+compositor: C) -> EngineTask {
    let resource_task = ResourceTask();
    let image_cache_task = ImageCacheTask(resource_task);
    EngineTask_(compositor, resource_task, image_cache_task)
}

fn EngineTask_<C: Compositor Send Copy>(
    +compositor: C,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask
) -> EngineTask {
    do spawn_service(EngineProto::init) |request, move compositor| {

        let render_task = RenderTask(compositor);
        let layout_task = LayoutTask(render_task, image_cache_task);
        let content_task = ContentTask(layout_task, compositor, resource_task);

        Engine {
            compositor: compositor,
            render_task: render_task,
            resource_task: resource_task,
            image_cache_task: image_cache_task,
            layout_task: layout_task,
            content_task: content_task,
        }.run(request);
    }
}


struct Engine<C:Compositor> {
    compositor: C,
    render_task: RenderTask,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    layout_task: LayoutTask,
    content_task: ContentTask,
}

impl<C: Compositor> Engine<C> {
    fn run(+request: EngineProto::server::Running) {
        use EngineProto::*;
        let mut request = request;

        loop {
            select!(
                request => {
                    LoadURL(url) -> next {
                        // TODO: change copy to move once we have match move
                        let url = move_ref!(url);
                        if url.path.ends_with(".js") {
                            self.content_task.send(content_task::ExecuteMsg(url))
                        } else {
                            self.content_task.send(content_task::ParseMsg(url))
                        }
                        request = next;
                    },

                    Exit -> channel {
                        self.content_task.send(content_task::ExitMsg);
                        self.layout_task.send(layout_task::ExitMsg);

                        let (response_chan, response_port) = pipes::stream();
                        self.render_task.send(render_task::ExitMsg(response_chan));
                        response_port.recv();

                        self.image_cache_task.exit();
                        self.resource_task.send(resource_task::Exit);

                        server::Exited(channel);
                        break
                    }
                }
            )
        }
    }
}

proto! EngineProto(
    Running:send {
        LoadURL(Url) -> Running,
        Exit -> Exiting
    }

    Exiting:recv {
        Exited -> !
    }
)

