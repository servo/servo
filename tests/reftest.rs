// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(fs_walk)]
#![feature(path_ext)]
#![feature(slice_patterns)]
#![feature(test)]

#[macro_use] extern crate bitflags;
extern crate png;
extern crate test;
extern crate url;
extern crate util;

use std::env;
use std::ffi::OsStr;
use std::fs::{PathExt, File, walk_dir};
use std::io::{self, Read, Result, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::process::{Command};
use std::thread::sleep_ms;
use test::run_tests_console;
use test::{AutoColor, DynTestName, DynTestFn, TestDesc, TestOpts, TestDescAndFn, ShouldPanic};
use url::Url;

bitflags!(
    flags RenderMode: u32 {
        const CPU_RENDERING  = 0x00000001,
        const GPU_RENDERING  = 0x00000010,
        const LINUX_TARGET   = 0x00000100,
        const MACOS_TARGET   = 0x00001000,
        const ANDROID_TARGET = 0x00010000
    }
);


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut parts = args[1..].split(|e| &**e == "--");

    let harness_args = parts.next().unwrap();  // .split() is never empty
    let servo_args = parts.next().unwrap_or(&[]);

    let (render_mode_string, base_path, testnames) = match harness_args {
        [ref render_mode_string, ref base_path, testnames..] =>
            (render_mode_string, base_path, testnames),
        _ => panic!("USAGE: cpu|gpu base_path [testname ...]"),
    };

    let mut render_mode = match &**render_mode_string {
        "cpu" => CPU_RENDERING,
        "gpu" => GPU_RENDERING,
        _ => panic!("First argument must specify cpu or gpu as rendering mode")
    };
    if cfg!(target_os = "linux") {
        render_mode.insert(LINUX_TARGET);
    }
    if cfg!(target_os = "macos") {
        render_mode.insert(MACOS_TARGET);
    }
    if cfg!(target_os = "android") {
        render_mode.insert(ANDROID_TARGET);
    }

    let mut all_tests = vec!();
    println!("Scanning {} for manifests\n", base_path);

    for file in walk_dir(base_path).unwrap() {
        let file = file.unwrap().path();
        let maybe_extension = file.extension();
        match maybe_extension {
            Some(extension) => {
                if extension == OsStr::new("list") && file.is_file() {
                    let len = all_tests.len();
                    let mut tests = parse_lists(&file, testnames, servo_args, render_mode, len);
                    println!("\t{} [{} tests]", file.display(), tests.len());
                    all_tests.append(&mut tests);
                }
            }
            _ => {}
        }
    }

    let test_opts = TestOpts {
        filter: None,
        run_ignored: false,
        logfile: None,
        run_tests: true,
        bench_benchmarks: false,
        nocapture: false,
        color: AutoColor,
    };

    match run(test_opts,
              all_tests,
              servo_args.iter().cloned().collect()) {
        Ok(false) => process::exit(1), // tests failed
        Err(_) => process::exit(2),    // I/O-related failure
        _ => (),
    }
}

fn run(test_opts: TestOpts, all_tests: Vec<TestDescAndFn>,
       servo_args: Vec<String>) -> io::Result<bool> {
    // Verify that we're passing in valid servo arguments. Otherwise, servo
    // will exit before we've run any tests, and it will appear to us as if
    // all the tests are failing.
    let mut command = Command::new(&servo_path());
    command
        .args(&servo_args)
        .arg("-z")
        .arg("about:blank");

    let mut child = match command.spawn() {
        Ok(p) => p,
        Err(e) => panic!("failed to execute process: {}", e),
    };

    // Wait for the shell to launch or to fail
    sleep_ms(1000);
    child.kill().unwrap();
    let output = try!(child.wait_with_output());

    let stderr = String::from_utf8(output.stderr).unwrap();

    if stderr.contains("Unrecognized") {
        println!("Servo: {}", stderr);
        return Ok(false);
    }

    run_tests_console(&test_opts, all_tests)
}

#[derive(PartialEq)]
enum ReftestKind {
    Same,
    Different,
}

struct Reftest {
    name: String,
    kind: ReftestKind,
    files: [PathBuf; 2],
    id: usize,
    servo_args: Vec<String>,
    render_mode: RenderMode,
    is_flaky: bool,
    prefs: Vec<String>,
    fragment_identifier: Option<String>,
    resolution: Option<String>,
    pixel_ratio: Option<f32>,
}

struct TestLine<'a> {
    conditions: &'a str,
    kind: &'a str,
    file_left: &'a str,
    file_right: &'a str,
}

fn parse_lists(file: &Path,
               filters: &[String],
               servo_args: &[String],
               render_mode: RenderMode,
               id_offset: usize)
               -> Vec<TestDescAndFn> {
    let mut tests = Vec::new();
    let contents = {
        let mut f = File::open(file).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();
        contents
    };

    for line in contents.lines() {
        // ignore comments or empty lines
        if line.starts_with("#") || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(' ').filter(|p| !p.is_empty()).collect();

        let test_line = match parts.len() {
            3 => TestLine {
                conditions: "",
                kind: parts[0],
                file_left: parts[1],
                file_right: parts[2],
            },
            4 => TestLine {
                conditions: parts[0],
                kind: parts[1],
                file_left: parts[2],
                file_right: parts[3],
            },
            _ => panic!("reftest line: '{}' doesn't match '[CONDITIONS] KIND LEFT RIGHT'", line),
        };

        let kind = match test_line.kind {
            "==" => ReftestKind::Same,
            "!=" => ReftestKind::Different,
            part => panic!("reftest line: '{}' has invalid kind '{}'", line, part)
        };

        let base = env::current_dir().unwrap().join(file.parent().unwrap());

        let file_left =  base.join(test_line.file_left);
        let file_right = base.join(test_line.file_right);

        let conditions_list = test_line.conditions.split(',');
        let mut flakiness = RenderMode::empty();
        let mut prefs = vec![];
        let mut fragment_identifier = None;
        let mut resolution = None;
        let mut pixel_ratio = None;
        for condition in conditions_list {
            match condition {
                "flaky_cpu" => flakiness.insert(CPU_RENDERING),
                "flaky_gpu" => flakiness.insert(GPU_RENDERING),
                "flaky_linux" => flakiness.insert(LINUX_TARGET),
                "flaky_macos" => flakiness.insert(MACOS_TARGET),
                _ => ()
            }
            if condition.starts_with("prefs:\"") {
                if let Some(joined) = condition.split("\"").nth(1) {
                    prefs.extend(joined.split(",").map(str::to_owned));
                }
            }
            if condition.starts_with("fragment=") {
                fragment_identifier = Some(condition["fragment=".len()..].to_string());
            }
            if condition.starts_with("resolution=") {
                resolution = Some(condition["resolution=".len() ..].to_string());
            }
            if condition.starts_with("device-pixel-ratio=") {
                pixel_ratio = Some(condition["device-pixel-ratio=".len() ..].to_string()
                                   .parse().expect("Invalid device-pixel-ratio"));
            }
        }

        let reftest = Reftest {
            name: format!("{} {} {}", test_line.file_left, test_line.kind, test_line.file_right),
            kind: kind,
            files: [file_left, file_right],
            id: id_offset + tests.len(),
            render_mode: render_mode,
            servo_args: servo_args.to_vec(),
            is_flaky: render_mode.intersects(flakiness),
            prefs: prefs,
            fragment_identifier: fragment_identifier,
            resolution: resolution,
            pixel_ratio: pixel_ratio,
        };

        if filters.is_empty() || filters.iter().any(|pattern| reftest.name.contains(pattern)) {
            tests.push(make_test(reftest));
        }
    }
    tests
}

fn make_test(reftest: Reftest) -> TestDescAndFn {
    let name = reftest.name.clone();
    TestDescAndFn {
        desc: TestDesc {
            name: DynTestName(name),
            ignore: false,
            should_panic: ShouldPanic::No,
        },
        testfn: DynTestFn(Box::new(move || {
            check_reftest(reftest);
        })),
    }
}

fn capture(reftest: &Reftest, side: usize) -> (u32, u32, Vec<u8>) {
    let png_filename = format!("/tmp/servo-reftest-{:06}-{}.png", reftest.id, side);
    let mut command = Command::new(&servo_path());
    command
        .args(&reftest.servo_args[..])
        .arg("--user-stylesheet").arg(util::resource_files::resources_dir_path().join("ahem.css"))
        // Allows pixel perfect rendering of Ahem font and the HTML canvas for reftests.
        .arg("-Z")
        .arg("disable-text-aa,disable-canvas-aa")
        .args(&["-f", "-o"])
        .arg(&png_filename)
        .arg(&{
            let mut url = Url::from_file_path(&*reftest.files[side]).unwrap();
            url.fragment = reftest.fragment_identifier.clone();
            url.to_string()
        });
    // CPU rendering is the default
    if reftest.render_mode.contains(CPU_RENDERING) {
        command.arg("-c");
    }
    if reftest.render_mode.contains(GPU_RENDERING) {
        command.arg("-g");
    }
    for pref in &reftest.prefs {
        command.arg("--pref");
        command.arg(pref);
    }
    if let Some(ref resolution) = reftest.resolution {
        command.arg("--resolution");
        command.arg(resolution);
    }
    if let Some(pixel_ratio) = reftest.pixel_ratio {
        command.arg("--device-pixel-ratio");
        command.arg(pixel_ratio.to_string());
    }
    let (exit_status, stderr, stdout) = match command.output() {
        Ok(output) => (output.status, output.stderr, output.stdout),
        Err(e) => panic!("failed to execute process: {}", e),
    };

    if !stdout.is_empty() {
        let stdout_filename = format!("/tmp/servo-reftest-{:06}-{}-stdout.txt", reftest.id, side);
        let mut stdout_file = File::create(stdout_filename).unwrap();
        stdout_file.write_all(&stdout[..]).unwrap();
    }

    if !stderr.is_empty() {
        let stderr_filename = format!("/tmp/servo-reftest-{:06}-{}-stderr.txt", reftest.id, side);
        let mut stderr_file = File::create(stderr_filename).unwrap();
        stderr_file.write_all(&stderr[..]).unwrap();
    }

    assert!(exit_status.success());

    let image = png::load_png(&png_filename).unwrap();
    let rgba8_bytes = match image.pixels {
        png::PixelsByColorType::RGBA8(pixels) => pixels,
        _ => panic!(),
    };
    (image.width, image.height, rgba8_bytes)
}

fn servo_path() -> PathBuf {
    let current_exe = env::current_exe().ok().expect("Could not locate current executable");
    current_exe.parent().unwrap().join("servo")
}

fn check_reftest(reftest: Reftest) {
    let (left_width, left_height, left_bytes) = capture(&reftest, 0);
    let (right_width, right_height, right_bytes) = capture(&reftest, 1);

    // TODO(gw): This is a workaround for https://github.com/servo/servo/issues/7730
    if !reftest.is_flaky {
        assert_eq!(left_width, right_width);
        assert_eq!(left_height, right_height);

        let left_all_white = left_bytes.iter().all(|&p| p == 255);
        let right_all_white = right_bytes.iter().all(|&p| p == 255);

        if left_all_white && right_all_white {
            panic!("Both renderings are empty")
        }
    }

    let pixels = left_bytes.iter().zip(right_bytes.iter()).map(|(&a, &b)| {
        if a == b {
            // White for correct
            0xFF
        } else {
            // "1100" in the RGBA channel with an error for an incorrect value
            // This results in some number of C0 and FFs, which is much more
            // readable (and distinguishable) than the previous difference-wise
            // scaling but does not require reconstructing the actual RGBA pixel.
            0xC0
        }
    }).collect::<Vec<u8>>();

    if pixels.iter().any(|&a| a < 255) {
        let output = format!("/tmp/servo-reftest-{:06}-diff.png", reftest.id);

        let mut img = png::Image {
            width: left_width,
            height: left_height,
            pixels: png::PixelsByColorType::RGBA8(pixels),
        };
        let res = png::store_png(&mut img, &output);
        assert!(res.is_ok());

        match (reftest.kind, reftest.is_flaky) {
            (ReftestKind::Same, true) => println!("flaky test - rendering difference: {}", output),
            (ReftestKind::Same, false) => panic!("rendering difference: {}", output),
            (ReftestKind::Different, _) => {}   // Result was different and that's what was expected
        }
    } else {
        assert!(reftest.is_flaky || reftest.kind == ReftestKind::Same);
    }
}
