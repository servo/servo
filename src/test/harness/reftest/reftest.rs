// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern mod std;
extern mod extra;

use std::cell::Cell;
use std::rt::io;
use std::rt::io::file;
use std::rt::io::Reader;
use std::os;
use std::run;
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
        let contents = match file::open(&file_path, io::Open, io::Read) {
            Some(mut f) => str::from_utf8(f.read_to_end()),
            None => fail!("Could not open file")
        };

        for line in contents.line_iter() {
            let parts: ~[&str] = line.split_iter(' ').filter(|p| !p.is_empty()).collect();

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
    let reftest = Cell::new(reftest);
    TestDescAndFn {
        desc: TestDesc {
            name: DynTestName(name),
            ignore: false,
            should_fail: false,
        },
        testfn: DynTestFn(|| {
            check_reftest(reftest.take());
        }),
    }
}

fn check_reftest(reftest: Reftest) {
    let left_filename = format!("/tmp/servo-reftest-{:06u}-left.png", reftest.id);
    let right_filename = format!("/tmp/servo-reftest-{:06u}-right.png", reftest.id);

    let args = ~[~"-o", left_filename.clone(), reftest.left.clone()];
    let mut process = run::Process::new("./servo", args, run::ProcessOptions::new());
    let _retval = process.finish();
    // assert!(retval == 0);

    let args = ~[~"-o", right_filename.clone(), reftest.right.clone()];
    let mut process = run::Process::new("./servo", args, run::ProcessOptions::new());
    let _retval = process.finish();
    // assert!(retval == 0);

    // check the pngs are bit equal
    let args = ~[left_filename.clone(), right_filename.clone()];
    let mut process = run::Process::new("cmp", args, run::ProcessOptions::new());
    let retval = process.finish();

    match reftest.kind {
        Same => assert!(retval == 0),
        Different => assert!(retval != 0),
    }
}
