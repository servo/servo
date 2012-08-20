import dom::event::Event;
import azure::azure_hl::DrawTarget;

#[doc = "
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display
"]
trait Compositor {
    fn begin_drawing(+next_dt: pipes::chan<DrawTarget>);
    fn draw(+next_dt: pipes::chan<DrawTarget>, +draw_me: DrawTarget);
    fn add_event_listener(listener: comm::Chan<Event>);
}

