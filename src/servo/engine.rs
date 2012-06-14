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
                let url <- url;
                if (*url).ends_with(".js") {
                    content.send(content::execute(url))
                } else {
                    content.send(content::parse(url))
                }
              }

              ExitMsg(sender) {
                content.send(content::exit);
                layout.send(layout::layout_task::exit);
                listen {
                    |resp_ch|
                    renderer.send(renderer::exit(resp_ch));
                    resp_ch.recv();
                }
                sender.send(());
                break;
              }
            }
        }
    }
}
