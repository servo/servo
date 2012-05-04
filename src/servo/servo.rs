import comm::*;
import libc::c_double;
import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;

fn main() {
    // The platform event handler thread
    let osmain = osmain::osmain();

    // The compositor
    let renderer = gfx::renderer::renderer(osmain);

    // The display list builder
    let lister = layout::lister::lister(renderer);

    // The layout task
    let layout = layout::layout::layout(lister);

    // The keyboard handler
    let key_po = port();
    send(osmain, osmain::add_key_handler(chan(key_po)));

    loop {
        send(layout, layout::layout::do_layout);

        std::timer::sleep(10u);

        if peek(key_po) {
            comm::send(layout, layout::layout::exit);
            comm::send(lister, layout::lister::exit);

            let draw_exit_confirm_po = comm::port();
            comm::send(renderer, gfx::renderer::exit(comm::chan(draw_exit_confirm_po)));

            comm::recv(draw_exit_confirm_po);
            comm::send(osmain, osmain::exit);
            break;
        }
    }
}