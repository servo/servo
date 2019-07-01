/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

fn main() {
    let azure = std::env::var_os("CARGO_FEATURE_CANVAS2D_AZURE").is_some();
    let raqote = std::env::var_os("CARGO_FEATURE_CANVAS2D_RAQOTE").is_some();

    if !(azure || raqote) {
        error("Must enable one of the `canvas2d-azure` or `canvas2d-raqote` features.")
    }
    if azure && raqote {
        error("Must not enable both of the `canvas2d-azure` and `canvas2d-raqote` features.")
    }
}

fn error(message: &str) {
    print!("\n\n    Error: {}\n\n", message);
    std::process::exit(1)
}
