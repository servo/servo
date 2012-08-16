import comm::*;
import gfx::renderer;
import platform::osmain;
import osmain::{OSMain, AddKeyHandler};
import opts::{Opts, Screen, Png};
import engine::{Engine, LoadURLMsg};

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
    let engine = Engine(osmain);
    let engine_chan = engine.start();

    for urls.each |filename| {
        let url = make_url(filename, none);
        #debug["master: Sending url `%s`", url_to_str(url)];
        engine_chan.send(LoadURLMsg(url));
        #debug["master: Waiting for keypress"];

        match keypress_from_osmain.try_recv() {
          some(*) => { }
          none => { #error("keypress stream closed unexpectedly") }
        };
    }

    // Shut everything down
    #debug["master: Shut down"];
    let (exit_chan, exit_response_from_engine) = pipes::stream();
    engine_chan.send(engine::ExitMsg(exit_chan));
    exit_response_from_engine.recv();

    osmain.send(osmain::Exit);
}

fn run_pipeline_png(-url: ~str, outfile: ~str) {

    // Use a PNG encoder as the graphics sink
    import gfx::pngsink;
    import pngsink::PngSink;
    import result::{ok, err};
    import io::{Writer, buffered_file_writer};

    listen(|pngdata_from_sink| {
        let sink = PngSink(pngdata_from_sink);
        let engine = Engine(sink);
        let engine_chan = engine.start();
        engine_chan.send(LoadURLMsg(make_url(url, none)));

        match buffered_file_writer(outfile) {
          ok(writer) => writer.write(pngdata_from_sink.recv()),
          err(e) => fail e
        }

        let (exit_chan, exit_response_from_engine) = pipes::stream();
        engine_chan.send(engine::ExitMsg(exit_chan));
        exit_response_from_engine.recv();
        sink.send(pngsink::Exit);
    })
}
