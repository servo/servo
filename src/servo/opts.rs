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

#[warn(no_non_implicitly_copyable_typarams)]
fn from_cmdline_args(args: ~[~str]) -> Opts {
    import std::getopts;

    let args = args.tail();

    let opts = ~[
        getopts::optopt(~"o")
    ];

    let opt_match = alt getopts::getopts(args, opts) {
      result::ok(m) { copy m }
      result::err(f) { fail getopts::fail_str(f) }
    };

    let urls = if opt_match.free.is_empty() {
        fail ~"servo asks that you provide 1 or more URLs"
    } else {
        copy opt_match.free
    };

    let render_mode = alt getopts::opt_maybe_str(opt_match, ~"o") {
      some(output_file) { Png(copy output_file) }
      none { Screen }
    };

    {
        urls: urls,
        render_mode: render_mode
    }
}
