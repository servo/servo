import gfx::renderer::{Renderer, Compositor};
import pipes::{spawn_service, select};
import layout::layout_task;
import layout_task::Layout;
import content::{Content, ExecuteMsg, ParseMsg, ExitMsg, create_content};
import resource::resource_task;
import resource::resource_task::{ResourceTask};
import std::net::url::url;
import resource::image_cache_task;
import image_cache_task::{ImageCacheTask, image_cache_task, ImageCacheTaskClient};

import pipes::{port, chan};

fn macros() {
    include!("macros.rs");
}

struct Engine<C:Compositor send copy> {
    let compositor: C;

    let renderer: Renderer;
    let resource_task: ResourceTask;
    let image_cache_task: ImageCacheTask;
    let layout: Layout;
    let content: comm::Chan<content::ControlMsg>;

    new(+compositor: C) {
        self.compositor = compositor;

        let renderer = Renderer(compositor);
        let resource_task = ResourceTask();
        let image_cache_task = image_cache_task(resource_task);
        let layout = Layout(renderer, image_cache_task);
        let content = create_content(layout, compositor, resource_task);

        self.renderer = renderer;
        self.resource_task = resource_task;
        self.image_cache_task = image_cache_task;
        self.layout = layout;
        self.content = content;
    }

    fn start() -> EngineProto::client::Running {
        do spawn_service(EngineProto::init) |request| {
            import EngineProto::*;
            let mut request = request;

            loop {
                select! {
                    request => {
                        LoadURL(url) -> next {
                            // TODO: change copy to move once we have match move
                            let url = move_ref!(url);
                            if url.path.ends_with(".js") {
                                self.content.send(ExecuteMsg(url))
                            } else {
                                self.content.send(ParseMsg(url))
                            }
                            request = next;
                        },
                        
                        Exit -> channel {
                            self.content.send(content::ExitMsg);
                            self.layout.send(layout_task::ExitMsg);
                            
                            let (response_chan, response_port) =
                                pipes::stream();
                            
                            self.renderer.send(
                                renderer::ExitMsg(response_chan));
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
