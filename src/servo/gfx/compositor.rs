use dom::event::Event;
use azure::azure_hl::DrawTarget;

#[doc = "
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display
"]
trait Compositor {
    fn begin_drawing(+next_dt: pipes::Chan<DrawTarget>);
    fn draw(+next_dt: pipes::Chan<DrawTarget>, +draw_me: DrawTarget);
    fn add_event_listener(listener: comm::Chan<Event>);
}

