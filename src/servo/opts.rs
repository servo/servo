//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use azure::azure_hl::{BackendType, CairoBackend, CoreGraphicsBackend};
use azure::azure_hl::{CoreGraphicsAcceleratedBackend, Direct2DBackend, SkiaBackend};

pub struct Opts {
    urls: ~[~str],
    render_mode: RenderMode,
    render_backend: BackendType,
    n_render_threads: uint,
}

pub enum RenderMode {
    Screen,
    Png(~str)
}

#[allow(non_implicitly_copyable_typarams)]
pub fn from_cmdline_args(args: &[~str]) -> Opts {
    use std::getopts;

    let args = args.tail();

    let opts = ~[
        getopts::optopt(~"o"),
        getopts::optopt(~"r"),
        getopts::optopt(~"t"),
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

    let render_mode = match getopts::opt_maybe_str(copy opt_match, ~"o") {
      Some(move output_file) => { Png(move output_file) }
      None => { Screen }
    };

    let render_backend = match getopts::opt_maybe_str(copy opt_match, ~"r") {
        Some(move backend_str) => {
            if backend_str == ~"direct2d" {
                Direct2DBackend
            } else if backend_str == ~"core-graphics" {
                CoreGraphicsBackend
            } else if backend_str == ~"core-graphics-accelerated" {
                CoreGraphicsAcceleratedBackend
            } else if backend_str == ~"cairo" {
                CairoBackend
            } else if backend_str == ~"skia" {
                SkiaBackend
            } else {
                fail ~"unknown backend type"
            }
        }
        None => CairoBackend
    };

    let n_render_threads: uint = match getopts::opt_maybe_str(move opt_match, ~"t") {
        Some(move n_render_threads_str) => from_str::from_str(n_render_threads_str).get(),
        None => 2,      // FIXME: Number of cores.
    };

    Opts {
        urls: move urls,
        render_mode: move render_mode,
        render_backend: move render_backend,
        n_render_threads: n_render_threads,
    }
}
