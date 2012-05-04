import geom::*;
import comm::*;
import layout::display_list::*;

enum msg {
    draw(display_list),
    exit(comm::chan<()>)
}

fn renderer(osmain: chan<osmain::msg>) -> chan<msg> {
    task::spawn_listener::<msg> {|po|
        listen {|draw_target_ch|
            osmain.send(osmain::get_draw_target(draw_target_ch));
            let draw_target = draw_target_ch.recv();

            let mut exit_confirm_ch = none;
            loop {
                alt po.recv() {
                  draw(display_list) {

                    draw_display_list(draw_target, display_list);

                    listen {|draw_ch|
                        osmain.send(osmain::draw(draw_ch));
                        draw_ch.recv();
                    }
                  }
                  exit(response_ch) {
                    exit_confirm_ch = some(response_ch);
                    break;
                  }
                }
            }

            assert exit_confirm_ch.is_some();
            exit_confirm_ch.get().send(());
        }
    }
}

fn draw_display_list(
    draw_target: AzDrawTargetRef,
    display_list: display_list
) {
    clear(draw_target);

    for display_list.each {|item|
        let bounds = (*item).bounds;

        let red_color = {
            r: 1f as AzFloat,
            g: 0f as AzFloat,
            b: 0f as AzFloat,
            a: 0.5f as AzFloat
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

}