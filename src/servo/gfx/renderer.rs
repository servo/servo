import platform::osmain;
import geom::*;
import comm::*;
import image::base::image;
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

fn renderer<S: sink send copy>(sink: S) -> chan<msg> {
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

impl to_float for u8 {
    fn to_float() -> float {
        (self as float) / 255f
    }
}

fn draw_solid_color(draw_target: AzDrawTargetRef, item: dl::display_item,
                    r: u8, g: u8, b: u8) {
    let bounds = (*item).bounds;

    let red_color = {
        r: r.to_float() as AzFloat,
        g: g.to_float() as AzFloat,
        b: b.to_float() as AzFloat,
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

fn draw_image(draw_target: AzDrawTargetRef, item: dl::display_item,
              -image: ~image) {
    // FIXME: This is hideously inefficient.

    let bounds = (*item).bounds;

    if (image.depth < 3u) {
        #debug("TODO: can't draw images with depth less than 3 yet");
        ret;
    }

    let stride = image.width * image.depth;
    uint::range(0u, image.height) {
        |y|
        uint::range(0u, image.width) {
            |x|
            let color = {
                r: image.data[y * stride + x * image.depth].to_float()
                    as AzFloat,
                g: image.data[y * stride + x * image.depth + 1u].to_float()
                    as AzFloat,
                b: image.data[y * stride + x * image.depth + 2u].to_float()
                    as AzFloat,
                a: 1f as AzFloat
            };
            let pattern = AzCreateColorPattern(ptr::addr_of(color));

            let pixel_rect = {
                x: (au_to_px(bounds.origin.x) + (x as int)) as AzFloat,
                y: (au_to_px(bounds.origin.y) + (y as int)) as AzFloat,
                width: 1f as AzFloat,
                height: 1f as AzFloat
            };
            AzDrawTargetFillRect(
                draw_target,
                ptr::addr_of(pixel_rect),
                unsafe { unsafe::reinterpret_cast(pattern) }
            );

            AzReleaseColorPattern(pattern);
        }
    }
}

fn draw_display_list(
    draw_target: AzDrawTargetRef,
    display_list: dl::display_list
) {
    clear(draw_target);

    for display_list.each {|item|
        #debug["drawing %?", item];

        alt item.item_type {
            dl::display_item_solid_color(r, g, b) {
                draw_solid_color(draw_target, item, r, g, b);
            }
            dl::display_item_image(image) {
                draw_image(draw_target, item, image);
            }
            dl::padding(*) {
                fail "should never see padding";
            }
        }
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
