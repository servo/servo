#[doc = "

Configuration options for a single run of the servo application. Created
from command line arguments.

"];

type Opts = {
    urls: ~[~str],
    render_mode: RenderMode
};

enum RenderMode {
    Screen,
    Png(~str)
}

#[allow(non_implicitly_copyable_typarams)]
fn from_cmdline_args(args: ~[~str]) -> Opts {
    import std::getopts;

    let args = args.tail();

    let opts = ~[
        getopts::optopt(~"o")
    ];

    let opt_match = match getopts::getopts(args, opts) {
      result::Ok(m) => { copy m }
      result::Err(f) => { fail getopts::fail_str(f) }
    };

    let urls = if opt_match.free.is_empty() {
        fail ~"servo asks that you provide 1 or more URLs"
    } else {
        copy opt_match.free
    };

    let render_mode = match getopts::opt_maybe_str(opt_match, ~"o") {
      Some(output_file) => { Png(copy output_file) }
      None => { Screen }
    };

    {
        urls: urls,
        render_mode: render_mode
    }
}
