import platform::osmain;
import geom::*;
import comm::*;
import dl = layout::display_list;
import azure::*;
import azure::bindgen::*;

enum msg {
    render(dl::display_list),
    exit(comm::chan<()>)
}

#[doc = "
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display
"]
iface sink {
    fn begin_drawing(next_dt: chan<AzDrawTargetRef>);
    fn draw(next_dt: chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef);
}

fn renderer<S: sink send>(sink: S) -> chan<msg> {
    task::spawn_listener::<msg> {|po|
        listen {|draw_target_ch|
            #debug("renderer: beginning rendering loop");
            sink.begin_drawing(draw_target_ch);

            loop {
                alt po.recv() {
                  render(display_list) {
                    #debug("renderer: got render request");
                    let draw_target = draw_target_ch.recv();
                    #debug("renderer: rendering");
                    draw_display_list(draw_target, display_list);
                    #debug("renderer: returning surface");
                    sink.draw(draw_target_ch, draw_target);
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
    display_list: dl::display_list
) {
    clear(draw_target);

    for display_list.each {|item|
        let (r, g, b) = alt check item.item_type {
          dl::solid_color(r, g, b) { (r, g, b) }
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
            x: au_to_px(bounds.origin.x) as AzFloat,
            y: au_to_px(bounds.origin.y) as AzFloat,
            width: au_to_px(bounds.size.width) as AzFloat,
            height: au_to_px(bounds.size.height) as AzFloat
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