import geom::*;
import comm::*;
import layout::display_list::*;

enum msg {
    draw(display_list),
    exit(comm::chan<()>)
}

fn renderer(osmain_ch: comm::chan<osmain::msg>) -> comm::chan<msg> {
    task::spawn_listener {|po|
        let draw_target_po = comm::port();
        comm::send(osmain_ch, osmain::get_draw_target(comm::chan(draw_target_po)));
        let draw_target = comm::recv(draw_target_po);

        let mut exit_confirm_ch = none;
        loop {
            alt comm::recv::<msg>(po) {
              draw(display_list) {

                draw_display_list(draw_target, display_list);

                let draw_po = comm::port();
                comm::send(osmain_ch, osmain::draw(comm::chan(draw_po)));
                comm::recv(draw_po);
              }
              exit(response_ch) {
                exit_confirm_ch = some(response_ch);
                break;
              }
            }
        }

        assert exit_confirm_ch.is_some();
        comm::send(exit_confirm_ch.get(), ());
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
            r: 1f as azure::AzFloat,
            g: 0f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 0.5f as azure::AzFloat
        };
        let red_pattern = AzCreateColorPattern(ptr::addr_of(red_color));

        let red_rect = {
            x: au_to_int(bounds.origin.x) as azure::AzFloat,
            y: au_to_int(bounds.origin.y) as azure::AzFloat,
            width: au_to_int(bounds.size.width) as azure::AzFloat,
            height: au_to_int(bounds.size.height) as azure::AzFloat
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
        r: 0f as azure::AzFloat,
        g: 0f as azure::AzFloat,
        b: 0f as azure::AzFloat,
        a: 1f as azure::AzFloat
    };
    let black_pattern = AzCreateColorPattern(ptr::addr_of(black_color));

    let black_rect = {
        x: 0 as azure::AzFloat,
        y: 0 as azure::AzFloat,
        width: 800 as azure::AzFloat,
        height: 600 as azure::AzFloat,
    };

    AzDrawTargetFillRect(
        draw_target,
        ptr::addr_of(black_rect),
        unsafe { unsafe::reinterpret_cast(black_pattern) }
    );

}