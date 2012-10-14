use dom::event::Event;
use gfx::render_task::LayerBuffer;

/**
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display
*/
trait Compositor {
    fn begin_drawing(next_dt: pipes::Chan<LayerBuffer>);
    fn draw(next_dt: pipes::Chan<LayerBuffer>, +draw_me: LayerBuffer);
}

