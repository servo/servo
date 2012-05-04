import comm::*;
import parser::html;
import parser::html::methods;
import result::extensions;

fn parse(filename: str) {
    let file_data = io::read_whole_file(filename).get();
    let reader = io::bytes_reader(file_data);
    let parser = html::parser(reader);
    loop {
        let t = parser.parse();
        log(error, #fmt("token: %?", t));
        if t == html::to_eof { break; }
    }
}

fn main(args: [str]) {
    if args.len() >= 2u {
        let p = html::spawn_parser_task(args[1]);
        loop {
            let token = p.recv();
            io::println(#fmt("token: %?", token));
            if token == html::to_eof { break; }
        }
    }

    // The platform event handler thread
    let osmain = platform::osmain::osmain();

    // The renderer
    let renderer = gfx::renderer::renderer(osmain);

    // The layout task
    let layout = layout::layout::layout(renderer);

    // The content task
    let content = content::content(layout);

    // Wait for keypress
    listen {|key_ch|
        osmain.send(platform::osmain::add_key_handler(key_ch));

        key_ch.recv();
    }

    // Shut everything down
    content.send(content::exit);
    layout.send(layout::layout::exit);

    listen {|wait_ch|
        renderer.send(gfx::renderer::exit(wait_ch));
        wait_ch.recv();
    }
    osmain.send(platform::osmain::exit);
}
