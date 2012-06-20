#[doc = "

Configuration options for a single run of the servo application. Created
from command line arguments.

"];

type opts = {
    urls: [str],
    render_mode: render_mode
};

enum render_mode {
    screen,
    png(str)
}

#[warn(no_non_implicitly_copyable_typarams)]
fn from_cmdline_args(args: [str]) -> opts {
    import std::getopts;

    let args = args.tail();

    let opts = [
        getopts::optopt("o")
    ];

    let match = alt getopts::getopts(args, opts) {
      result::ok(m) { let m <- m; m }
      result::err(f) { fail getopts::fail_str(f) }
    };

    let urls = if match.free.is_empty() {
        fail "servo asks that you provide 1 or more URLs"
    } else {
        copy match.free
    };

    let render_mode = alt getopts::opt_maybe_str(match, "o") {
      some(output_file) { let output_file <- output_file; png(output_file) }
      none { screen }
    };

    {
        urls: urls,
        render_mode: render_mode
    }
}
