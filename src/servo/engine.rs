import gfx::renderer;

enum msg {
    load_url(str),
    exit(comm::chan<()>)
}

fn engine<S: renderer::sink send>(sink: S) -> comm::chan<msg> {

    task::spawn_listener::<msg> {|self_ch|
        // The renderer
        let renderer = renderer::renderer(sink);

        // The layout task
        let layout = layout::layout::layout(renderer);

        // The content task
        let content = content::content(layout);

        loop {
            alt self_ch.recv() {
              load_url(url) { content.send(content::parse(url)) }
              exit(sender) {
                content.send(content::exit);
                layout.send(layout::layout::exit);
                listen {|resp_ch|
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
