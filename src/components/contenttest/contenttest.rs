// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern mod std;

use std::test::{TestOpts, run_tests_console, TestDesc};
use std::getopts::{getopts, reqopt, opt_str, fail_str};
use os::list_dir_path;

struct Config {
    source_dir: ~str,
    filter: Option<~str>
}

fn main() {
    let args = os::args();
    let config = parse_config(args);
    let opts = test_options(config);
    let tests = find_tests(config);
    run_tests_console(&opts, tests);
}

fn parse_config(args: ~[~str]) -> Config {
    let args = args.tail();
    let opts = ~[reqopt(~"source-dir")];
    let matches = match getopts(args, opts) {
      Ok(m) => m,
      Err(f) => fail fail_str(f)
    };

    Config {
        source_dir: opt_str(matches, ~"source-dir"),
        filter: if matches.free.is_empty() {
            None
        } else {
            Some(matches.free.head())
        }
    }
}

fn test_options(config: Config) -> TestOpts {
    {
        filter: config.filter,
        run_ignored: false,
        logfile: None
    }
}

fn find_tests(config: Config) -> ~[TestDesc] {
    let all_files = list_dir_path(&Path(config.source_dir));
    let html_files = all_files.filter( |file| file.to_str().ends_with(".html") );
    return html_files.map(|file| make_test(config, (*file).to_str()) );
}

fn make_test(config: Config, file: ~str) -> TestDesc {
    {
        name: file,
        testfn: fn~() { run_test(config, file) },
        ignore: false,
        should_fail: false
    }
}

fn run_test(config: Config, file: ~str) {
    let infile = ~"file://" + os::make_absolute(&Path(file)).to_str();
    let res = run::program_output("./servo", ~[infile]);
    io::print(res.out);
    do str::split_char_each(res.out, '\n') |line| {
        if line.contains("TEST-UNEXPECTED-FAIL") {
            fail str::from_slice(line);
        }
        true
    }
}

fn render_servo(config: Config, file: ~str) {
}
