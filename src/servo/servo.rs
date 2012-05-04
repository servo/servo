import comm::*;
import libc::c_double;
import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;
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
        parse(args[1]);
    }

    // The platform event handler thread
    let osmain = platform::osmain::osmain();

    // The compositor
    let renderer = gfx::renderer::renderer(osmain);

    // The layout task
    let layout = layout::layout::layout(renderer);

    // The keyboard handler
    let key_po = port();
    send(osmain, platform::osmain::add_key_handler(chan(key_po)));

    loop {
        send(layout, layout::layout::build);

        std::timer::sleep(200u);

        if peek(key_po) {
            comm::send(layout, layout::layout::exit);

            let draw_exit_confirm_po = comm::port();
            comm::send(renderer, gfx::renderer::exit(comm::chan(draw_exit_confirm_po)));

            comm::recv(draw_exit_confirm_po);
            comm::send(osmain, platform::osmain::exit);
            break;
        }
    }
}
