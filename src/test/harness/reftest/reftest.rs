// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate png;
extern crate std;
extern crate test;

use std::io;
use std::io::{File, Reader, Process};
use std::io::process::ExitStatus;
use std::os;
use std::str;
use test::{DynTestName, DynTestFn, TestDesc, TestOpts, TestDescAndFn};
use test::run_tests_console;

fn main() {
    let args = os::args();
    let mut parts = args.tail().split(|e| "--" == e.as_slice());

    let files = parts.next().unwrap();  // .split() is never empty
    let servo_args = parts.next().unwrap_or(&[]);

    if files.len() == 0 {
        fail!("error: at least one reftest list must be given");
    }

    let tests = parse_lists(files, servo_args);
    let test_opts = TestOpts {
        filter: None,
        run_ignored: false,
        logfile: None,
        run_tests: true,
        run_benchmarks: false,
        ratchet_noise_percent: None,
        ratchet_metrics: None,
        save_metrics: None,
        test_shard: None,
    };

    match run_tests_console(&test_opts, tests) {
        Ok(false) => os::set_exit_status(1), // tests failed
        Err(_) => os::set_exit_status(2),    // I/O-related failure
        _ => (),
    }
}

#[deriving(Eq)]
enum ReftestKind {
    Same,
    Different,
}

struct Reftest {
    name: ~str,
    kind: ReftestKind,
    files: [~str, ..2],
    id: uint,
    servo_args: ~[~str],
}

fn parse_lists(filenames: &[~str], servo_args: &[~str]) -> Vec<TestDescAndFn> {
    let mut tests = Vec::new();
    let mut next_id = 0;
    for file in filenames.iter() {
        let file_path = Path::new(file.clone());
        let contents = match File::open_mode(&file_path, io::Open, io::Read)
            .and_then(|mut f| {
                f.read_to_str()
            }) {
                Ok(s) => s,
                _ => fail!("Could not read file"),
            };

        for line in contents.lines() {
            // ignore comments
            if line.starts_with("#") {
                continue;
            }

            let parts: ~[&str] = line.split(' ').filter(|p| !p.is_empty()).collect();

            if parts.len() != 3 {
                fail!("reftest line: '{:s}' doesn't match 'KIND LEFT RIGHT'", line);
            }

            let kind = match parts[0] {
                "==" => Same,
                "!=" => Different,
                _ => fail!("reftest line: '{:s}' has invalid kind '{:s}'",
                           line, parts[0])
            };
            let src_path = file_path.dir_path();
            let src_dir = src_path.display().to_str();
            let file_left =  src_dir + "/" + parts[1];
            let file_right = src_dir + "/" + parts[2];

            let reftest = Reftest {
                name: parts[1] + " / " + parts[2],
                kind: kind,
                files: [file_left, file_right],
                id: next_id,
                servo_args: servo_args.to_owned(),
            };

            next_id += 1;

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
            should_fail: false,
        },
        testfn: DynTestFn(proc() {
            check_reftest(reftest);
        }),
    }
}

fn capture(reftest: &Reftest, side: uint) -> png::Image {
    let filename = format!("/tmp/servo-reftest-{:06u}-{:u}.png", reftest.id, side);
    let mut args = reftest.servo_args.clone();
    args.push_all_move(~["-f".to_owned(), "-o".to_owned(), filename.clone(), reftest.files[side].clone()]);

    let retval = match Process::status("./servo", args) {
        Ok(status) => status,
        Err(e) => fail!("failed to execute process: {}", e),
    };
    assert!(retval == ExitStatus(0));

    png::load_png(&from_str::<Path>(filename).unwrap()).unwrap()
}

fn check_reftest(reftest: Reftest) {
    let left  = capture(&reftest, 0);
    let right = capture(&reftest, 1);

    let pixels: ~[u8] = left.pixels.iter().zip(right.pixels.iter()).map(|(&a, &b)| {
            if a as i8 - b as i8 == 0 {
                // White for correct
                0xFF
            } else {
                // "1100" in the RGBA channel with an error for an incorrect value
                // This results in some number of C0 and FFs, which is much more
                // readable (and distinguishable) than the previous difference-wise
                // scaling but does not require reconstructing the actual RGBA pixel.
                0xC0
            }
        }).collect();

    if pixels.iter().any(|&a| a < 255) {
        let output_str = format!("/tmp/servo-reftest-{:06u}-diff.png", reftest.id);
        let output = from_str::<Path>(output_str).unwrap();

        let img = png::Image {
            width: left.width,
            height: left.height,
            color_type: png::RGBA8,
            pixels: pixels,
        };
        let res = png::store_png(&img, &output);
        assert!(res.is_ok());

        assert!(reftest.kind == Different, "rendering difference: {}", output_str);
    } else {
        assert!(reftest.kind == Same);
    }
}
