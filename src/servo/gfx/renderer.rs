import platform::osmain;
import geom::*;
import comm::*;
import layout::display_list::*;
import azure::*;
import azure::bindgen::*;

enum msg {
    render(display_list),
    exit(comm::chan<()>)
}

fn renderer(osmain: chan<osmain::msg>) -> chan<msg> {
    task::spawn_listener::<msg> {|po|
        listen {|draw_target_ch|
            #debug("renderer: beginning rendering loop");
            osmain.send(osmain::begin_drawing(draw_target_ch));
                
            loop {
                alt po.recv() {
                  render(display_list) {
                    #debug("renderer: got render request");
                    let draw_target = draw_target_ch.recv();
                    #debug("renderer: rendering");
                    draw_display_list(draw_target, display_list);
                    #debug("renderer: returning surface");
                    osmain.send(osmain::draw(draw_target_ch, draw_target));
                  }
                  exit(response_ch) {
                    response_ch.send(());
                    break;
                  }
                }
            }
        }
    }
}

fn draw_display_list(
    draw_target: AzDrawTargetRef,
    display_list: display_list
) {
    clear(draw_target);

    for display_list.each {|item|
        let (r, g, b) = alt check item.item_type {
          solid_color(r, g, b) { (r, g, b) }
        };
        let bounds = (*item).bounds;

        let to_float = fn@(u: u8) -> float {
            (u as float) / 255f
        };

        let red_color = {
            r: to_float(r) as AzFloat,
            g: to_float(g) as AzFloat,
            b: to_float(b) as AzFloat,
            a: 1f as AzFloat
        };
        let red_pattern = AzCreateColorPattern(ptr::addr_of(red_color));

        let red_rect = {
            x: au_to_int(bounds.origin.x) as AzFloat,
            y: au_to_int(bounds.origin.y) as AzFloat,
            width: au_to_int(bounds.size.width) as AzFloat,
            height: au_to_int(bounds.size.height) as AzFloat
        };
        AzDrawTargetFillRect(
            draw_target,
            ptr::addr_of(red_rect),
            unsafe { unsafe::reinterpret_cast(red_pattern) }
        );

        AzReleaseColorPattern(red_pattern);
    }
}

fn clear(draw_target: AzDrawTargetRef) {

    let black_color = {
        r: 0f as AzFloat,
        g: 0f as AzFloat,
        b: 0f as AzFloat,
        a: 1f as AzFloat
    };
    let black_pattern = AzCreateColorPattern(ptr::addr_of(black_color));

    let black_rect = {
        x: 0 as AzFloat,
        y: 0 as AzFloat,
        width: 800 as AzFloat,
        height: 600 as AzFloat,
    };

    AzDrawTargetFillRect(
        draw_target,
        ptr::addr_of(black_rect),
        unsafe { unsafe::reinterpret_cast(black_pattern) }
    );

    AzReleaseColorPattern(black_pattern);
}