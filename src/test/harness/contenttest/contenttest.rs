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

use extra::test::{TestOpts, run_tests_console, TestDesc, TestDescAndFn, DynTestFn, DynTestName};
use extra::getopts::{getopts, reqopt, opt_str, fail_str};
use std::{os, run, io, str};
use std::cell::Cell;
use std::os::list_dir_path;

#[deriving(Clone)]
struct Config {
    source_dir: ~str,
    filter: Option<~str>
}

fn main() {
    let args = os::args();
    let config = parse_config(args);
    let opts = test_options(config.clone());
    let tests = find_tests(config);
    run_tests_console(&opts, tests);
}

fn parse_config(args: ~[~str]) -> Config {
    let args = args.tail();
    let opts = ~[reqopt("source-dir")];
    let matches = match getopts(args, opts) {
      Ok(m) => m,
      Err(f) => fail!(fail_str(f))
    };

    Config {
        source_dir: opt_str(&matches, "source-dir"),
        filter: if matches.free.is_empty() {
            None
        } else {
            Some((*matches.free.head()).clone())
        }
    }
}

fn test_options(config: Config) -> TestOpts {
    TestOpts {
        filter: config.filter,
        run_ignored: false,
        run_tests: true,
        run_benchmarks: false,
        save_results: None,
        compare_results: None,
        logfile: None
    }
}

fn find_tests(config: Config) -> ~[TestDescAndFn] {
    let mut files = list_dir_path(&Path(config.source_dir));
    files.retain( |file| file.to_str().ends_with(".html") );
    return files.map(|file| make_test((*file).to_str()) );
}

fn make_test(file: ~str) -> TestDescAndFn {
    let f = Cell::new(file.clone());
    TestDescAndFn {
        desc: TestDesc {
            name: DynTestName(file),
            ignore: false,
            should_fail: false
        },
        testfn: DynTestFn(|| { run_test(f.take()) })
    }
}

fn run_test(file: ~str) {
    let infile = ~"file://" + os::make_absolute(&Path(file)).to_str();
    let res = run::process_output("./servo", [infile]);
    let out = str::from_bytes(res.output);
    io::print(out);
    let lines: ~[&str] = out.split_iter('\n').collect();
    for lines.iter().advance |&line| {
        if line.contains("TEST-UNEXPECTED-FAIL") {
            fail!(line);
        }
    }
}
