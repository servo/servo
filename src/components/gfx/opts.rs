/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use azure::azure_hl::{BackendType, CairoBackend, CoreGraphicsBackend};
use azure::azure_hl::{CoreGraphicsAcceleratedBackend, Direct2DBackend, SkiaBackend};

pub struct Opts {
    urls: ~[~str],
    render_backend: BackendType,
    n_render_threads: uint,
    tile_size: uint,
}

#[allow(non_implicitly_copyable_typarams)]
pub fn from_cmdline_args(args: &[~str]) -> Opts {
    use std::getopts;

    let args = args.tail();

    let opts = ~[
        getopts::optopt(~"o"),  // output file
        getopts::optopt(~"r"),  // rendering backend
        getopts::optopt(~"s"),  // size of tiles
        getopts::optopt(~"t"),  // threads to render with
    ];

    let opt_match = match getopts::getopts(args, opts) {
      result::Ok(m) => { copy m }
      result::Err(f) => { fail!(getopts::fail_str(copy f)) }
    };

    let urls = if opt_match.free.is_empty() {
        fail!(~"servo asks that you provide 1 or more URLs")
    } else {
        copy opt_match.free
    };

    let render_backend = match getopts::opt_maybe_str(&opt_match, ~"r") {
        Some(backend_str) => {
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
                fail!(~"unknown backend type")
            }
        }
        None => SkiaBackend
    };

    let tile_size: uint = match getopts::opt_maybe_str(&opt_match, ~"s") {
        Some(tile_size_str) => uint::from_str(tile_size_str).get(),
        None => 512,
    };

    let n_render_threads: uint = match getopts::opt_maybe_str(&opt_match, ~"t") {
        Some(n_render_threads_str) => uint::from_str(n_render_threads_str).get(),
        None => 1,      // FIXME: Number of cores.
    };

    Opts {
        urls: urls,
        render_backend: render_backend,
        n_render_threads: n_render_threads,
        tile_size: tile_size,
    }
}
