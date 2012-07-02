import gfx::renderer::{Renderer, Sink};
import task::spawn_listener;
import comm::chan;
import layout::layout_task;
import layout_task::Layout;
import content::{Content, ExecuteMsg, ParseMsg};

type Engine = chan<Msg>;

enum Msg {
    LoadURLMsg(~str),
    ExitMsg(chan<()>)
}

fn Engine<S: Sink send copy>(sink: S) -> Engine {
    spawn_listener::<Msg>(|request| {
        // The renderer
        let renderer = Renderer(sink);

        // The layout task
        let layout = Layout(renderer);

        // The content task
        let content = Content(layout);

        loop {
            alt request.recv() {
              LoadURLMsg(url) {
                let url = copy url;
                if (*url).ends_with(".js") {
                    content.send(ExecuteMsg(url))
                } else {
                    content.send(ParseMsg(url))
                }
              }

              ExitMsg(sender) {
                content.send(content::ExitMsg);
                layout.send(layout_task::ExitMsg);
                listen(|response_channel| {
                    renderer.send(renderer::ExitMsg(response_channel));
                    response_channel.recv();
                });
                sender.send(());
                break;
              }
            }
        }
    })
}
