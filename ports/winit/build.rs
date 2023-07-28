/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(target_os = "macos")]
extern crate cc;

#[cfg(windows)]
extern crate winres;

use vergen::EmitBuilder;

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../resources/servo.ico");
        res.set_manifest_file("platform/windows/servo.exe.manifest");
        res.compile().unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("platform/macos/count_threads.c")
            .compile("count_threads");
    }

    if let Err(error) = EmitBuilder::builder()
        .fail_on_error()
        .git_sha(true /* short */)
        .emit()
    {
        println!(
            "cargo:warning=Could not generate git version information: {:?}",
            error
        );
        println!("cargo:rustc-env=VERGEN_GIT_SHA=nogit");
    }
}
