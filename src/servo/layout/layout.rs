#[doc = "

Builds display lists on request and passes them to the renderer

"];

import task::*;
import comm::*;
import display_list::*;
import gfx::geom;
import gfx::geom::*;
import gfx::renderer;

enum msg {
    build,
    exit
}

fn layout(renderer: chan<renderer::msg>) -> chan<msg> {

    spawn_listener::<msg> {|po|
        let mut x1 = 100;
        let mut y1 = 100;
        let mut w1 = 200;
        let mut h1 = 200;
        let mut x2 = 200;
        let mut y2 = 200;
        let mut w2 = 300;
        let mut h2 = 300;

        loop {
            alt recv(po) {
              build {
                let dlist = [
                    display_item({
                        item_type: solid_color,
                        bounds: geom::box(
                            int_to_au(x1),
                            int_to_au(y1),
                            int_to_au(w1),
                            int_to_au(h1))
                    }),
                    display_item({
                        item_type: solid_color,
                        bounds: geom::box(
                            int_to_au(x2),
                            int_to_au(y2),
                            int_to_au(w2),
                            int_to_au(h2))
                    })
                ];

                send(renderer, gfx::renderer::draw(dlist));

                x1 += 1;
                y1 += 1;
                x2 -= 1;
                y2 -= 1;
                if x1 > 800 { x1 = 0 }
                if y1 > 600 { y1 = 0 }
                if x2 < 0 { x2 = 800 }
                if y2 < 0 { y2 = 600 }
              }
              exit {
                break;
              }
            }
        }
    }

}
