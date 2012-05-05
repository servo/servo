import comm::*;
import parser::html;
import parser::html::methods;
import result::extensions;
import gfx::renderer;
import platform::osmain;

fn main(args: [str]) {
    run(opts::from_cmdline_args(args))
}

fn run(opts: opts::opts) {

    // The platform event handler thread
    let osmain = osmain::osmain();

    // The renderer
    let renderer = {
        // Use the platform thread as the renderer sink
        import osmain::gfxsink;
        renderer::renderer(osmain)
    };

    // The layout task
    let layout = layout::layout::layout(renderer);

    // The content task
    let content = content::content(layout);

    // Send each file to render then wait for keypress
    for opts.urls.each { |filename|
        #debug["master: Sending filename `%s`", filename];
        content.send(content::parse(filename));
        #debug["master: Waiting for keypress"];
        listen {|key_ch|
            osmain.send(platform::osmain::add_key_handler(key_ch));
            key_ch.recv();
        }
    }

    // Shut everything down
    #debug["master: Shut down"];
    content.send(content::exit);
    layout.send(layout::layout::exit);

    listen {|wait_ch|
        renderer.send(gfx::renderer::exit(wait_ch));
        wait_ch.recv();
    }
    osmain.send(platform::osmain::exit);
}
