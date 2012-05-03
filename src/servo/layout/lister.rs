#[doc = "

Builds display lists on request and passes them to the renderer

"];

import task::*;
import comm::*;
import gfx::renderer;

enum msg {
    build,
    exit
}

fn lister(renderer: chan<renderer::msg>) -> chan<msg> {

    spawn_listener {|po|
        let mut x1 = 100;
        let mut y1 = 100;
        let mut w1 = 200;
        let mut h1 = 200;
        let mut x2 = 200;
        let mut y2 = 200;
        let mut w2 = 300;
        let mut h2 = 300;

        while !peek(po) {
            let model = {
                x1: x1, y1: y1, w1: w1, h1: h1,
                x2: x2, y2: y2, w2: w2, h2: h2
            };
            send(renderer, gfx::renderer::draw(model));

            std::timer::sleep(100u);

            x1 += 1;
            y1 += 1;
            x2 -= 1;
            y2 -= 1;
            if x1 > 800 { x1 = 0 }
            if y1 > 600 { y1 = 0 }
            if x2 < 0 { x2 = 800 }
            if y2 < 0 { y2 = 600 }
        }
    }

}
