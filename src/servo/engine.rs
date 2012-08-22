export EngineTask, EngineTask_, EngineProto;

import gfx::compositor::Compositor;
import gfx::render_task;
import render_task::RenderTask;
import pipes::{spawn_service, select};
import layout::layout_task;
import layout_task::LayoutTask;
import content::content_task;
import content_task::{ContentTask};
import resource::resource_task;
import resource::resource_task::{ResourceTask};
import std::net::url::url;
import resource::image_cache_task;
import image_cache_task::{ImageCacheTask, ImageCacheTaskClient};

import pipes::{port, chan};

fn macros() {
    include!("macros.rs");
}

type EngineTask = EngineProto::client::Running;

fn EngineTask<C: Compositor send copy>(+compositor: C) -> EngineTask {
    let resource_task = ResourceTask();
    let image_cache_task = ImageCacheTask(resource_task);
    EngineTask_(compositor, resource_task, image_cache_task)
}

fn EngineTask_<C: Compositor send copy>(
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
    compositor: C;
    render_task: RenderTask;
    resource_task: ResourceTask;
    image_cache_task: ImageCacheTask;
    layout_task: LayoutTask;
    content_task: ContentTask;
}

impl<C: Compositor> Engine<C> {
    fn run(+request: EngineProto::server::Running) {
        import EngineProto::*;
        let mut request = request;

        loop {
            select! {
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
            }
        }
    }
}

proto! EngineProto {
    Running:send {
        LoadURL(url) -> Running,
        Exit -> Exiting
    }

    Exiting:recv {
        Exited -> !
    }
}
