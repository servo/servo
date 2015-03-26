// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(collections)]
#![feature(core)]
#![feature(exit_status)]
#![feature(old_io)]
#![feature(path)]
#![feature(rustc_private)]
#![feature(std_misc)]
#![feature(test)]

extern crate getopts;
extern crate test;

use test::{AutoColor, TestOpts, run_tests_console, TestDesc, TestDescAndFn, DynTestFn, DynTestName};
use test::ShouldPanic;
use getopts::{getopts, reqopt};
use std::{str, env};
use std::ffi::OsStr;
use std::fs::read_dir;
use std::old_io::Reader;
use std::old_io::process::{Command, Ignored, CreatePipe, InheritFd, ExitStatus};
use std::thunk::Thunk;

#[derive(Clone)]
struct Config {
    source_dir: String,
    filter: Option<String>
}

fn main() {
    let args = env::args();
    let config = parse_config(args.collect());
    let opts = test_options(config.clone());
    let tests = find_tests(config);
    match run_tests_console(&opts, tests) {
        Ok(false) => env::set_exit_status(1), // tests failed
        Err(_) => env::set_exit_status(2),    // I/O-related failure
        _ => (),
    }
}

fn parse_config(args: Vec<String>) -> Config {
    let args = args.tail();
    let opts = vec!(reqopt("s", "source-dir", "source-dir", "source-dir"));
    let matches = match getopts(args, opts.as_slice()) {
      Ok(m) => m,
      Err(f) => panic!(f.to_string())
    };

    Config {
        source_dir: matches.opt_str("source-dir").unwrap(),
        filter: matches.free.first().map(|s| s.clone())
    }
}

fn test_options(config: Config) -> TestOpts {
    TestOpts {
        filter: config.filter,
        run_ignored: false,
        run_tests: true,
        run_benchmarks: false,
        logfile: None,
        nocapture: false,
        color: AutoColor,
    }
}

fn find_tests(config: Config) -> Vec<TestDescAndFn> {
    read_dir(&config.source_dir)
        .ok()
        .expect("Error reading directory.")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|file| file.extension().map_or(false, |e| e == OsStr::from_str("html")))
        .map(|file| make_test(file.display().to_string()))
        .collect()
}

fn make_test(file: String) -> TestDescAndFn {
    TestDescAndFn {
        desc: TestDesc {
            name: DynTestName(file.clone()),
            ignore: false,
            should_panic: ShouldPanic::No,
        },
        testfn: DynTestFn(Thunk::new(move || { run_test(file) }))
    }
}

fn run_test(file: String) {
    let path = env::current_dir().unwrap().join(&file);
    // FIXME (#1094): not the right way to transform a path
    let infile = format!("file://{}", path.display());
    let stdout = CreatePipe(false, true);
    let stderr = InheritFd(2);
    let args = ["-z", "-f", infile.as_slice()];

    let mut prc_arg = env::current_exe().unwrap();
    let prc_arg = match prc_arg.pop() {
        true => prc_arg.join("servo"),
        _ => panic!("could not pop directory"),
    };
    let mut prc = match Command::new(prc_arg.to_str().unwrap())
        .args(args.as_slice())
        .stdin(Ignored)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
    {
        Ok(p) => p,
        _ => panic!("Unable to spawn process."),
    };
    let mut output = Vec::new();
    loop {
        let byte = prc.stdout.as_mut().unwrap().read_byte();
        match byte {
            Ok(byte) => {
                print!("{}", byte as char);
                output.push(byte);
            }
            _ => break
        }
    }

    let out = str::from_utf8(output.as_slice());
    let lines: Vec<&str> = out.unwrap().split('\n').collect();
    for &line in lines.iter() {
        if line.contains("TEST-UNEXPECTED-FAIL") {
            panic!(line.to_string());
        }
    }

    let retval = prc.wait();
    if retval != Ok(ExitStatus(0)) {
        panic!("Servo exited with non-zero status {:?}", retval);
    }
}
