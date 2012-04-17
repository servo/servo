type model = {
    x1: int, y1: int, w1: int, h1: int,
    x2: int, y2: int, w2: int, h2: int
};

enum msg {
    draw(model),
    exit(comm::chan<()>)
}

fn compositor(osmain_ch: comm::chan<osmain::msg>) -> comm::chan<msg> {
    task::spawn_listener {|po|
        let draw_target_po = comm::port();
        comm::send(osmain_ch, osmain::get_draw_target(comm::chan(draw_target_po)));
        let draw_target = comm::recv(draw_target_po);

        let black_color = {
            r: 0f as azure::AzFloat,
            g: 0f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 1f as azure::AzFloat
        };
        let black_pattern = AzCreateColorPattern(ptr::addr_of(black_color));

        let red_color = {
            r: 1f as azure::AzFloat,
            g: 0f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 0.5f as azure::AzFloat
        };
        let red_pattern = AzCreateColorPattern(ptr::addr_of(red_color));

        let green_color = {
            r: 0f as azure::AzFloat,
            g: 1f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 0.5f as azure::AzFloat
        };
        let green_pattern = AzCreateColorPattern(ptr::addr_of(green_color));

        let mut exit_confirm_ch = none;
        loop {
            alt comm::recv::<msg>(po) {
              draw(model) {
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

                let red_rect = {
                    x: model.x1 as azure::AzFloat,
                    y: model.y1 as azure::AzFloat,
                    width: model.w1 as azure::AzFloat,
                    height: model.h1 as azure::AzFloat
                };
                AzDrawTargetFillRect(
                    draw_target,
                    ptr::addr_of(red_rect),
                    unsafe { unsafe::reinterpret_cast(red_pattern) }
                );
                let green_rect = {
                    x: model.x2 as azure::AzFloat,
                    y: model.y2 as azure::AzFloat,
                    width: model.w2 as azure::AzFloat,
                    height: model.h2 as azure::AzFloat
                };
                AzDrawTargetFillRect(
                    draw_target,
                    ptr::addr_of(green_rect),
                    unsafe { unsafe::reinterpret_cast(green_pattern) }
                );
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

        AzReleaseColorPattern(red_pattern);
        AzReleaseColorPattern(green_pattern);

        assert exit_confirm_ch.is_some();
        comm::send(exit_confirm_ch.get(), ());
    }
}