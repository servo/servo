import libc::c_double;
import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;

fn main() {
    // The platform event handler thread
    let osmain_ch = osmain::osmain();

    // The compositor
    let renderer = gfx::renderer::renderer(osmain_ch);

    // The display list builder
    let lister = layout::lister::lister(renderer);

    // The keyboard handler
    input::input(osmain_ch, renderer, lister);
}