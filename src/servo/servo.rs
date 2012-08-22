import comm::*;
import option::swap_unwrap;
import platform::osmain;
import osmain::{OSMain, AddKeyHandler};
import opts::{Opts, Screen, Png};
import engine::{EngineTask, EngineProto};

import url_to_str = std::net::url::to_str;
import util::url::make_url;

import pipes::{port, chan};

fn main(args: ~[~str]) {
    run(opts::from_cmdline_args(args))
}

#[allow(non_implicitly_copyable_typarams)]
fn run(opts: Opts) {
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

fn run_pipeline_screen(urls: ~[~str]) {

    // The platform event handler thread
    let osmain = OSMain();

    // Send each file to render then wait for keypress
    let (keypress_to_engine, keypress_from_osmain) = pipes::stream();
    osmain.send(AddKeyHandler(keypress_to_engine));

    // Create a serve instance
    let mut engine_task = some(EngineTask(osmain));

    for urls.each |filename| {
        let url = make_url(filename, none);
        #debug["master: Sending url `%s`", url_to_str(url)];
        engine_task =
            some(EngineProto::client::LoadURL(swap_unwrap(&mut engine_task),
                                              url));
        #debug["master: Waiting for keypress"];

        match keypress_from_osmain.try_recv() {
          some(*) => { }
          none => { #error("keypress stream closed unexpectedly") }
        };
    }

    // Shut everything down
    #debug["master: Shut down"];
    let engine_task = EngineProto::client::Exit(option::unwrap(engine_task));
    pipes::recv(engine_task);

    osmain.send(osmain::Exit);
}

fn run_pipeline_png(-url: ~str, outfile: ~str) {

    // Use a PNG encoder as the graphics compositor
    import gfx::png_compositor;
    import png_compositor::PngCompositor;
    import result::{ok, err};
    import io::{Writer, buffered_file_writer};
    import resource::resource_task::ResourceTask;
    import resource::image_cache_task::SyncImageCacheTask;
    import engine::EngineTask_;

    listen(|pngdata_from_compositor| {
        let compositor = PngCompositor(pngdata_from_compositor);
        let resource_task = ResourceTask();
        // For the PNG pipeline we are using a synchronous image cache
        // so that all requests will be fullfilled before the first
        // render
        let image_cache_task = SyncImageCacheTask(resource_task);
        let engine_task = EngineTask_(compositor, resource_task, image_cache_task);
        let engine_task = EngineProto::client::LoadURL(engine_task, make_url(url, none));

        match buffered_file_writer(outfile) {
          ok(writer) => writer.write(pngdata_from_compositor.recv()),
          err(e) => fail e
        }

        let engine_task = EngineProto::client::Exit(engine_task);
        pipes::recv(engine_task);
        compositor.send(png_compositor::Exit);
    })
}
