import comm::chan;
import gfx::renderer;

enum Msg {
    LoadURLMsg(~str),
    ExitMsg(comm::chan<()>)
}

fn engine<S:renderer::sink send copy>(sink: S) -> chan<Msg> {
    task::spawn_listener::<Msg> {|self_ch|
        // The renderer
        let renderer = renderer::renderer(sink);

        // The layout task
        let layout = layout::layout_task::layout(renderer);

        // The content task
        let content = content::content(layout);

        loop {
            alt self_ch.recv() {
              LoadURLMsg(url) {
                let url = copy url;
                if (*url).ends_with(".js") {
                    content.send(content::ExecuteMsg(url))
                } else {
                    content.send(content::ParseMsg(url))
                }
              }

              ExitMsg(sender) {
                content.send(content::ExitMsg);
                layout.send(layout::layout_task::ExitMsg);
                listen {
                    |response_channel|
                    renderer.send(renderer::ExitMsg(response_channel));
                    response_channel.recv();
                }
                sender.send(());
                break;
              }
            }
        }
    }
}
