import libc::c_double;
import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;

// FIXME: Busy wait hack
fn sleep() {
    iter::repeat(100000u) {||
        task::yield();
    }
}

fn main() {
    // The platform event handler thread
    let osmain_ch = osmain::osmain();

    // The drawing task
    let draw_ch = gfx::compositor::compositor(osmain_ch);

    // The model
    let model_ch = task::spawn_listener {|po|
        let mut x1 = 100;
        let mut y1 = 100;
        let mut w1 = 200;
        let mut h1 = 200;
        let mut x2 = 200;
        let mut y2 = 200;
        let mut w2 = 300;
        let mut h2 = 300;

        while !comm::peek(po) {
            let model = {
                x1: x1, y1: y1, w1: w1, h1: h1,
                x2: x2, y2: y2, w2: w2, h2: h2
            };
            comm::send(draw_ch, gfx::compositor::draw(model));

            sleep();

            x1 += 1;
            y1 += 1;
            x2 -= 1;
            y2 -= 1;
            if x1 > 800 { x1 = 0 }
            if y1 > 600 { y1 = 0 }
            if x2 < 0 { x2 = 800 }
            if y2 < 0 { y2 = 600 }
        }
    };

    // The keyboard handler
    input::input(osmain_ch, draw_ch, model_ch);
}