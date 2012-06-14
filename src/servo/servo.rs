import comm::*;
import parser::lexer;
import result::extensions;
import gfx::renderer;
import platform::osmain;

fn main(args: [str]) {
    run(opts::from_cmdline_args(args))
}

fn run(opts: opts::opts) {

    alt opts.render_mode {
      opts::screen {
        run_pipeline_screen(opts.urls)
      }
      opts::png(outfile) {
        assert opts.urls.is_not_empty();
        if opts.urls.len() > 1u {
            fail "servo asks that you stick to a single URL in PNG output mode"
        }
        run_pipeline_png(opts.urls.head(), outfile)
      }
    }
}

fn run_pipeline_screen(urls: [str]) {

    // Use the platform thread as the renderer sink
    import osmain::gfxsink;

    // The platform event handler thread
    let osmain = osmain::osmain();

    // Create a serve instance
    let engine = engine::engine(osmain);

    // Send each file to render then wait for keypress
    listen {|key_ch|
        osmain.send(platform::osmain::add_key_handler(key_ch));

        for urls.each { |filename|
            #debug["master: Sending filename `%s`", filename];
            engine.send(engine::LoadURLMsg(~copy filename));
            #debug["master: Waiting for keypress"];
            key_ch.recv();
        }
    }

    // Shut everything down
    #debug["master: Shut down"];
    listen {|resp_ch|
        engine.send(engine::ExitMsg(resp_ch));
        resp_ch.recv();
    }
    osmain.send(platform::osmain::exit);
}

fn run_pipeline_png(-url: str, outfile: str) {

    // Use a PNG encoder as the graphics sink
    import gfx::pngsink;
    import pngsink::pngsink;

    listen {|pngdata|
        let sink = pngsink::pngsink(pngdata);
        let engine = engine::engine(sink);
        let url <- url;
        engine.send(engine::LoadURLMsg(~url));
        alt io::buffered_file_writer(outfile) {
          result::ok(writer) {
            import io::writer;
            writer.write(pngdata.recv())
          }
          result::err(e) { fail e }
        }
        listen {|response_channel|
            engine.send(engine::ExitMsg(response_channel));
            response_channel.recv();
        }
        sink.send(pngsink::exit);
    }
}
