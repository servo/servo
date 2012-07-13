import comm::*;
import gfx::renderer;
import platform::osmain;
import osmain::{OSMain, AddKeyHandler};
import opts::{Opts, Screen, Png};
import engine::{Engine, LoadURLMsg};

fn main(args: ~[str]) {
    run(opts::from_cmdline_args(args))
}

#[warn(no_non_implicitly_copyable_typarams)]
fn run(opts: Opts) {
    alt opts.render_mode {
      Screen {
        run_pipeline_screen(opts.urls)
      }
      Png(outfile) {
        assert opts.urls.is_not_empty();
        if opts.urls.len() > 1u {
            fail "servo asks that you stick to a single URL in PNG output mode"
        }
        run_pipeline_png(opts.urls.head(), outfile)
      }
    }
}

fn run_pipeline_screen(urls: ~[str]) {

    // The platform event handler thread
    let osmain = OSMain();

    // Create a serve instance
    let engine = Engine(osmain);
    let engine_chan = engine.start();

    // Send each file to render then wait for keypress
    listen(|keypress_from_osmain| {
        osmain.send(AddKeyHandler(keypress_from_osmain));

        for urls.each |filename| {
            #debug["master: Sending filename `%s`", filename];
            engine_chan.send(LoadURLMsg(copy filename));
            #debug["master: Waiting for keypress"];
            keypress_from_osmain.recv();
        }
    });

    // Shut everything down
    #debug["master: Shut down"];
    listen(|exit_response_from_engine| {
        engine_chan.send(engine::ExitMsg(exit_response_from_engine));
        exit_response_from_engine.recv();
    });
    osmain.send(osmain::Exit);
}

fn run_pipeline_png(-url: str, outfile: str) {

    // Use a PNG encoder as the graphics sink
    import gfx::pngsink;
    import pngsink::PngSink;
    import result::{ok, err};
    import io::{writer, buffered_file_writer};

    listen(|pngdata_from_sink| {
        let sink = PngSink(pngdata_from_sink);
        let engine = Engine(sink);
        let engine_chan = engine.start();
        let url = copy url;
        engine_chan.send(LoadURLMsg(url));
        alt buffered_file_writer(outfile) {
          ok(writer) {
            writer.write(pngdata_from_sink.recv())
          }
          err(e) { fail e }
        }
        listen(|exit_response_from_engine| {
            engine_chan.send(engine::ExitMsg(exit_response_from_engine));
            exit_response_from_engine.recv();
        });
        sink.send(pngsink::Exit);
    })
}
