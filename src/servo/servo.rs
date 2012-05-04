import comm::*;
import parser::html;
import parser::html::methods;
import result::extensions;

fn main(args: [str]) {
    // The platform event handler thread
    let osmain = platform::osmain::osmain();

    // The renderer
    let renderer = gfx::renderer::renderer(osmain);

    // The layout task
    let layout = layout::layout::layout(renderer);

    // The content task
    let content = content::content(layout);

    // Send each file to render then wait for keypress
    for args.tail().each { |filename|
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
