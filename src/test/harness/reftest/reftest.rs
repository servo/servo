// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern mod extra;
extern mod png;
extern mod std;

use std::io;
use std::io::{File, Reader};
use std::io::process::ExitStatus;
use std::os;
use std::run::{Process, ProcessOptions};
use std::str;
use extra::test::{DynTestName, DynTestFn, TestDesc, TestOpts, TestDescAndFn};
use extra::test::run_tests_console;

fn main() {
    let args = os::args();
    if args.len() < 2 {
        println("error: at least one reftest list must be given");
        os::set_exit_status(1);
        return;
    }

    let tests = parse_lists(args.tail());
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

    if !run_tests_console(&test_opts, tests) {
        os::set_exit_status(1);
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
    left: ~str,
    right: ~str,
    id: uint,
}

fn parse_lists(filenames: &[~str]) -> ~[TestDescAndFn] {
    let mut tests: ~[TestDescAndFn] = ~[];
    let mut next_id = 0;
    for file in filenames.iter() {
        let file_path = Path::new(file.clone());
        let contents = match File::open_mode(&file_path, io::Open, io::Read) {
            Some(mut f) => str::from_utf8_owned(f.read_to_end()),
            None => fail!("Could not open file")
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
                left: file_left,
                right: file_right,
                id: next_id,
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

fn check_reftest(reftest: Reftest) {
    let left_filename = format!("/tmp/servo-reftest-{:06u}-left.png", reftest.id);
    let right_filename = format!("/tmp/servo-reftest-{:06u}-right.png", reftest.id);

    let args = ~[~"-f", ~"-o", left_filename.clone(), reftest.left.clone()];
    let mut process = Process::new("./servo", args, ProcessOptions::new()).unwrap();
    let retval = process.finish();
    assert!(retval == ExitStatus(0));

    let args = ~[~"-f", ~"-o", right_filename.clone(), reftest.right.clone()];
    let mut process = Process::new("./servo", args, ProcessOptions::new()).unwrap();
    let retval = process.finish();
    assert!(retval == ExitStatus(0));

    // check the pngs are bit equal
    let left = png::load_png(&from_str::<Path>(left_filename).unwrap()).unwrap();
    let right = png::load_png(&from_str::<Path>(right_filename).unwrap()).unwrap();

    let pixels: ~[u8] = left.pixels.iter().zip(right.pixels.iter()).map(|(&a, &b)| {
            if (a as i8 - b as i8 == 0) {
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
        let output = from_str::<Path>(format!("/tmp/servo-reftest-{:06u}-diff.png", reftest.id)).unwrap();

        let img = png::Image {
            width: left.width,
            height: left.height,
            color_type: png::RGBA8,
            pixels: pixels,
        };
        let res = png::store_png(&img, &output);
        assert!(res.is_ok());

        assert!(reftest.kind == Different);
    } else {
        assert!(reftest.kind == Same);
    }
}
