import gfx::renderer::{Renderer, Sink};
import task::spawn_listener;
import comm::chan;
import layout::layout_task;
import layout_task::Layout;
import content::{Content, ExecuteMsg, ParseMsg, ExitMsg, create_content};
import resource::resource_task;
import resource::resource_task::{ResourceTask};
import std::net::url::url;
import resource::image_cache_task;
import image_cache_task::{ImageCacheTask, image_cache_task, ImageCacheTaskClient};

import pipes::{port, chan};

class Engine<S:Sink send copy> {
    let sink: S;

    let renderer: Renderer;
    let resource_task: ResourceTask;
    let image_cache_task: ImageCacheTask;
    let layout: Layout;
    let content: comm::chan<content::ControlMsg>;

    new(+sink: S) {
        self.sink = sink;

        let renderer = Renderer(sink);
        let resource_task = ResourceTask();
        let image_cache_task = image_cache_task(resource_task);
        let layout = Layout(renderer, image_cache_task);
        let content = create_content(layout, sink, resource_task);

        self.renderer = renderer;
        self.resource_task = resource_task;
        self.image_cache_task = image_cache_task;
        self.layout = layout;
        self.content = content;
    }

    fn start() -> comm::chan<Msg> {
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
                self.content.send(ExecuteMsg(url))
            } else {
                self.content.send(ParseMsg(url))
            }
            return true;
          }

          ExitMsg(sender) => {
            self.content.send(content::ExitMsg);
            self.layout.send(layout_task::ExitMsg);
            
            let (response_chan, response_port) = pipes::stream();

            self.renderer.send(renderer::ExitMsg(response_chan));
            response_port.recv();

            self.image_cache_task.exit();
            self.resource_task.send(resource_task::Exit);

            sender.send(());
            return false;
          }
        }
    }
}

enum Msg {
    LoadURLMsg(url),
    ExitMsg(chan<()>)
}

