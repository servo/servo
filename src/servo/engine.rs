import gfx::renderer::{Renderer, Sink};
import task::spawn_listener;
import comm::chan;
import layout::layout_task;
import layout_task::Layout;
import content::{Content, ExecuteMsg, ParseMsg, ExitMsg, create_content};
import resource::resource_task;
import resource::resource_task::{ResourceTask};
import std::net::url::url;

import pipes::{port, chan};

class Engine<S:Sink send copy> {
    let sink: S;

    let renderer: Renderer;
    let layout: Layout;
    let resource_task: ResourceTask;
    let content: comm::chan<content::ControlMsg>;

    new(+sink: S) {
        self.sink = sink;

        let renderer = Renderer(sink);
        let layout = Layout(renderer);
        let resource_task = ResourceTask();
        let content = create_content(layout, sink, resource_task);

        self.renderer = renderer;
        self.layout = layout;
        self.resource_task = resource_task;
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
        alt request {
          LoadURLMsg(url) => {
            // TODO: change copy to move once we have alt move
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

