import gfx::renderer::{Renderer, Sink};
import task::spawn_listener;
import comm::chan;
import layout::layout_task;
import layout_task::Layout;
import content::{Content, ExecuteMsg, ParseMsg, ExitMsg, create_content};

class Engine<S:Sink send copy> {
    let sink: S;

    let renderer: Renderer;
    let layout: Layout;
    let content: chan<content::ControlMsg>;

    new(sink: S) {
        self.sink = sink;

        let renderer = Renderer(sink);
        let layout = Layout(renderer);
        let content = create_content(layout, sink);

        self.renderer = renderer;
        self.layout = layout;
        self.content = content;
    }

    fn start() -> chan<Msg> {
        do spawn_listener::<Msg> |request| {
            while self.handle_request(request.recv()) {
                // Go on...
            }
        }
    }

    fn handle_request(request: Msg) -> bool {
        alt request {
          LoadURLMsg(url) {
            let url = copy url;
            if (*url).ends_with(".js") {
                self.content.send(ExecuteMsg(url))
            } else {
                self.content.send(ParseMsg(url))
            }
            ret true;
          }

          ExitMsg(sender) {
            self.content.send(content::ExitMsg);
            self.layout.send(layout_task::ExitMsg);
            do listen |response_channel| {
                self.renderer.send(renderer::ExitMsg(response_channel));
                response_channel.recv();
            }
            sender.send(());
            ret false;
          }
        }
    }
}

enum Msg {
    LoadURLMsg(~str),
    ExitMsg(chan<()>)
}

