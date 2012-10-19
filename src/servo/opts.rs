//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

pub type Opts = {
    urls: ~[~str],
    render_mode: RenderMode
};

pub enum RenderMode {
    Screen,
    Png(~str)
}

#[allow(non_implicitly_copyable_typarams)]
pub fn from_cmdline_args(args: &[~str]) -> Opts {
    use std::getopts;

    let args = args.tail();

    let opts = ~[
        getopts::optopt(~"o")
    ];

    let opt_match = match getopts::getopts(args, opts) {
      result::Ok(m) => { copy m }
      result::Err(f) => { fail getopts::fail_str(copy f) }
    };

    let urls = if opt_match.free.is_empty() {
        fail ~"servo asks that you provide 1 or more URLs"
    } else {
        copy opt_match.free
    };

    let render_mode = match getopts::opt_maybe_str(move opt_match, ~"o") {
      Some(move output_file) => { Png(move output_file) }
      None => { Screen }
    };

    {
        urls: move urls,
        render_mode: move render_mode
    }
}
