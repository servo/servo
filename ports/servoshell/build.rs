/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::path::Path;

use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    // Cargo does not expose the profile name to crates or their build scripts,
    // but we can extract it from OUT_DIR and set a custom cfg() ourselves.
    let out = std::env::var("OUT_DIR")?;
    let out = Path::new(&out);
    let krate = out.parent().unwrap();
    let build = krate.parent().unwrap();
    let profile = build.parent().unwrap();
    if profile.file_name().unwrap() == "production" {
        println!("cargo:rustc-cfg=servo_production");
    } else {
        println!("cargo:rustc-cfg=servo_do_not_use_in_production");
    }

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

    // On MacOS, all dylib dependencies are shipped along with the binary
    // in the "/lib" directory. Setting the rpath here, allows the dynamic
    // linker to locate them. See `man dyld` for more info.
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/lib/");

    Ok(())
}
