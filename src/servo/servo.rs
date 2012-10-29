use comm::*;
use option::swap_unwrap;
use platform::osmain;
use osmain::{OSMain, AddKeyHandler};
use opts::{Opts, Screen, Png};
use engine::{Engine, ExitMsg, LoadURLMsg};
use resource::image_cache_task::ImageCacheTask;
use resource::resource_task::ResourceTask;

use util::url::make_url;

use pipes::{Port, Chan};

fn main() {
    let args = os::args();
    run(&opts::from_cmdline_args(args))
}

#[allow(non_implicitly_copyable_typarams)]
fn run(opts: &Opts) {
    match opts.render_mode {
      Screen => run_pipeline_screen(opts.urls),
      Png(outfile) => {
        assert opts.urls.is_not_empty();
        if opts.urls.len() > 1u {
            fail ~"servo asks that you stick to a single URL in PNG output mode"
        }
        run_pipeline_png(opts.urls.head(), outfile)
      }
    }
}

fn run_pipeline_screen(urls: &[~str]) {

    let (dom_event_chan, dom_event_port) = pipes::stream();
    let dom_event_chan = pipes::SharedChan(move dom_event_chan);

    // The platform event handler thread
    let osmain = OSMain(dom_event_chan.clone());

    // Send each file to render then wait for keypress
    let (keypress_to_engine, keypress_from_osmain) = pipes::stream();
    osmain.send(AddKeyHandler(move keypress_to_engine));

    // Create a servo instance
    let resource_task = ResourceTask();
    let image_cache_task = ImageCacheTask(copy resource_task);
    let engine_task = Engine(osmain, move dom_event_port, move dom_event_chan, move resource_task,
                             move image_cache_task);

    for urls.each |filename| {
        let url = make_url(copy *filename, None);
        #debug["master: Sending url `%s`", url.to_str()];
        engine_task.send(LoadURLMsg(move url));
        #debug["master: Waiting for keypress"];

        match keypress_from_osmain.try_recv() {
          Some(*) => { }
          None => { #error("keypress stream closed unexpectedly") }
        };
    }

    // Shut everything down
    #debug["master: Shut down"];
    let (exit_chan, exit_response_from_engine) = pipes::stream();
    engine_task.send(engine::ExitMsg(move exit_chan));
    exit_response_from_engine.recv();

    osmain.send(osmain::Exit);
}

fn run_pipeline_png(_url: ~str, _outfile: &str) {
    fail ~"PNG compositor is broken";
}

#[cfg(broken)]
fn run_pipeline_png(url: ~str, outfile: &str) {
    // Use a PNG encoder as the graphics compositor
    use gfx::png_compositor;
    use png_compositor::PngCompositor;
    use io::{Writer, buffered_file_writer};
    use resource::resource_task::ResourceTask;
    use resource::image_cache_task::SyncImageCacheTask;

    listen(|pngdata_from_compositor| {
        let (dom_event_chan, dom_event_port) = pipes::stream();
        let dom_event_chan = pipes::SharedChan(move dom_event_chan);

        let compositor = PngCompositor(pngdata_from_compositor);
        let resource_task = ResourceTask();
        // For the PNG pipeline we are using a synchronous image task so that all images will be
        // fulfilled before the first paint.
        let image_cache_task = SyncImageCacheTask(resource_task);
        let engine_task = Engine(copy compositor, move dom_event_port, move dom_event_chan,
                                 move resource_task, move image_cache_task);
        engine_task.send(LoadURLMsg(make_url(copy url, None)));

        match buffered_file_writer(&Path(outfile)) {
          Ok(writer) => writer.write(pngdata_from_compositor.recv()),
          Err(e) => fail e
        }

        let (exit_chan, exit_response_from_engine) = pipes::stream();
        engine_task.send(engine::ExitMsg(move exit_chan));
        exit_response_from_engine.recv();
        compositor.send(png_compositor::Exit);
    })
}
