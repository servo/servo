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
use std::io;
use std::os;
use std::run;
use extra::digest::{Digest, DigestUtil};
use extra::sha1::Sha1;
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
        save_results: None,
        compare_results: None,
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
}

fn parse_lists(filenames: &[~str]) -> ~[TestDescAndFn] {
    let mut tests: ~[TestDescAndFn] = ~[];
    for filenames.iter().advance |file| {
        let file_path = Path(*file);
        let contents = match io::read_whole_file_str(&file_path) {
            Ok(x) => x,
            Err(s) => fail!(s)
        };

        for contents.line_iter().advance |line| {
            let parts: ~[&str] = line.split_iter(' ').filter(|p| !p.is_empty()).collect();

            if parts.len() != 3 {
                fail!(fmt!("reftest line: '%s' doesn't match 'KIND LEFT RIGHT'", line));
            }

            let kind = match parts[0] {
                "==" => Same,
                "!=" => Different,
                _ => fail!(fmt!("reftest line: '%s' has invalid kind '%s'",
                                line, parts[0]))
            };
            let src_dir = file_path.dirname();
            let file_left = src_dir + "/" + parts[1];
            let file_right = src_dir + "/" + parts[2];
            
            let reftest = Reftest {
                name: parts[1] + " / " + parts[2],
                kind: kind,
                left: file_left,
                right: file_right,
            };

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
    let id = gen_id(&reftest);
    let left_filename = fmt!("/tmp/%s-left.png", id);
    let right_filename = fmt!("/tmp/%s-right.png", id);
    let left_path = Path(left_filename);
    let right_path = Path(right_filename);

    let options = run::ProcessOptions::new();
    let args = ~[~"-o", left_filename.clone(), reftest.left.clone()];
    let mut process = run::Process::new("./servo", args, options);
    let _retval = process.finish();
    // assert!(retval == 0);

    let args = ~[~"-o", right_filename.clone(), reftest.right.clone()];
    let mut process = run::Process::new("./servo", args, options);
    let _retval = process.finish();
    // assert!(retval == 0);

    // check the pngs are bit equal
    let left_sha = calc_hash(&left_path);
    os::remove_file(&left_path);

    let right_sha = calc_hash(&right_path);
    os::remove_file(&right_path);

    assert!(left_sha.is_some());
    assert!(right_sha.is_some());
    match reftest.kind {
        Same => assert!(left_sha == right_sha),
        Different => assert!(left_sha != right_sha),
    }
}

fn gen_id(reftest: &Reftest) -> ~str {
    let mut sha = Sha1::new();
    match reftest.kind {
        Same => sha.input_str("=="),
        Different => sha.input_str("!="),
    }
    sha.input_str(reftest.left);
    sha.input_str(reftest.right);
    sha.result_str()
}

fn calc_hash(path: &Path) -> Option<~str> {
    match io::file_reader(path) {
        Err(*) => None,
        Ok(reader) => {
            let mut sha = Sha1::new();
            loop {
                let bytes = reader.read_bytes(4096);
                sha.input(bytes);
                if bytes.len() < 4096 { break; }
            }
            Some(sha.result_str())
        }
    }
}