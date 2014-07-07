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
extern crate regex;

use std::io;
use std::io::{File, Reader, Command};
use std::io::process::ExitStatus;
use std::os;
use test::{AutoColor, DynTestName, DynTestFn, TestDesc, TestOpts, TestDescAndFn};
use test::run_tests_console;
use regex::Regex;

fn main() {
    let args = os::args();
    let mut parts = args.tail().split(|e| "--" == e.as_slice());

    let harness_args = parts.next().unwrap();  // .split() is never empty
    let servo_args = parts.next().unwrap_or(&[]);

    let (manifest, testname) = match harness_args {
      [] => fail!("error: at least one reftest list must be given"),
      [ref manifest] => (manifest, None),
      [ref manifest, ref testname, ..] => (manifest, Some(Regex::new(testname.as_slice()).unwrap())),
    };

    let tests = parse_lists(manifest, servo_args);
    let test_opts = TestOpts {
        filter: testname,
        run_ignored: false,
        logfile: None,
        run_tests: true,
        run_benchmarks: false,
        ratchet_noise_percent: None,
        ratchet_metrics: None,
        save_metrics: None,
        test_shard: None,
        nocapture: false,
        color: AutoColor
    };

    match run_tests_console(&test_opts, tests) {
        Ok(false) => os::set_exit_status(1), // tests failed
        Err(_) => os::set_exit_status(2),    // I/O-related failure
        _ => (),
    }
}

#[deriving(PartialEq)]
enum ReftestKind {
    Same,
    Different,
}

struct Reftest {
    name: String,
    kind: ReftestKind,
    files: [String, ..2],
    id: uint,
    servo_args: Vec<String>,
}

fn parse_lists(file: &String, servo_args: &[String]) -> Vec<TestDescAndFn> {
    let mut tests = Vec::new();
    let mut next_id = 0;
    let file_path = Path::new(file.clone());
    let contents = match File::open_mode(&file_path, io::Open, io::Read)
       .and_then(|mut f| {
             f.read_to_str()
             }) {
          Ok(s) => s,
             _ => fail!("Could not read file"),
       };

    for line in contents.as_slice().lines() {
       // ignore comments or empty lines
       if line.starts_with("#") || line.is_empty() {
          continue;
       }

       let parts: Vec<&str> = line.split(' ').filter(|p| !p.is_empty()).collect();

       if parts.len() != 3 {
          fail!("reftest line: '{:s}' doesn't match 'KIND LEFT RIGHT'", line);
       }

       let kind = match parts.get(0) {
          & "==" => Same,
             & "!=" => Different,
             &part => fail!("reftest line: '{:s}' has invalid kind '{:s}'",
                   line, part)
       };
       let src_path = file_path.dir_path();
       let src_dir = src_path.display().to_str();
       let file_left =  src_dir.clone().append("/").append(*parts.get(1));
       let file_right = src_dir.append("/").append(*parts.get(2));

       let reftest = Reftest {
name: parts.get(1).to_string().append(" / ").append(*parts.get(2)),
         kind: kind,
         files: [file_left, file_right],
         id: next_id,
         servo_args: servo_args.iter().map(|x| x.clone()).collect(),
       };

       next_id += 1;

       tests.push(make_test(reftest));
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
    args.push_all_move(vec!("-f".to_string(), "-o".to_string(), filename.clone(), reftest.files[side].clone()));

    let retval = match Command::new("./servo").args(args.as_slice()).status() {
        Ok(status) => status,
        Err(e) => fail!("failed to execute process: {}", e),
    };
    assert!(retval == ExitStatus(0));

    png::load_png(&from_str::<Path>(filename.as_slice()).unwrap()).unwrap()
}

fn check_reftest(reftest: Reftest) {
    let left  = capture(&reftest, 0);
    let right = capture(&reftest, 1);

    let pixels: Vec<u8> = left.pixels.iter().zip(right.pixels.iter()).map(|(&a, &b)| {
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
        let output = from_str::<Path>(output_str.as_slice()).unwrap();

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
