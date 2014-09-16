// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(unused_imports, unused_variable)]

extern crate getopts;
extern crate regex;
extern crate test;

use test::{AutoColor, TestOpts, run_tests_console, TestDesc, TestDescAndFn, DynTestFn, DynTestName};
use getopts::{getopts, reqopt};
use std::{os, str};
use std::io::fs;
use std::io::Reader;
use std::io::process::{Command, Ignored, CreatePipe, InheritFd, ExitStatus};
use regex::Regex;

#[deriving(Clone)]
struct Config {
    source_dir: String,
    filter: Option<Regex>
}

fn main() {
    let args = os::args();
    let config = parse_config(args.into_iter().collect());
    let opts = test_options(config.clone());
    let tests = find_tests(config);
    match run_tests_console(&opts, tests) {
        Ok(false) => os::set_exit_status(1), // tests failed
        Err(_) => os::set_exit_status(2),    // I/O-related failure
        _ => (),
    }
}

fn parse_config(args: Vec<String>) -> Config {
    let args = args.tail();
    let opts = vec!(reqopt("s", "source-dir", "source-dir", "source-dir"));
    let matches = match getopts(args, opts.as_slice()) {
      Ok(m) => m,
      Err(f) => fail!(format!("{}", f))
    };

    Config {
        source_dir: matches.opt_str("source-dir").unwrap(),
        filter: matches.free.as_slice().head().map(|s| Regex::new(s.as_slice()).unwrap())
    }
}

fn test_options(config: Config) -> TestOpts {
    TestOpts {
        filter: config.filter,
        run_ignored: false,
        run_tests: true,
        run_benchmarks: false,
        ratchet_metrics: None,
        ratchet_noise_percent: None,
        save_metrics: None,
        test_shard: None,
        logfile: None,
        nocapture: false,
        color: AutoColor
    }
}

fn find_tests(config: Config) -> Vec<TestDescAndFn> {
    let files_res = fs::readdir(&Path::new(config.source_dir));
    let mut files = match files_res {
        Ok(files) => files,
        _ => fail!("Error reading directory."),
    };
    files.retain(|file| file.extension_str() == Some("html") );
    return files.iter().map(|file| make_test(format!("{}", file.display()))).collect();
}

fn make_test(file: String) -> TestDescAndFn {
    TestDescAndFn {
        desc: TestDesc {
            name: DynTestName(file.clone()),
            ignore: false,
            should_fail: false
        },
        testfn: DynTestFn(proc() { run_test(file) })
    }
}

fn run_test(file: String) {
    let path = os::make_absolute(&Path::new(file));
    // FIXME (#1094): not the right way to transform a path
    let infile = format!("file://{}", path.display());
    let stdout = CreatePipe(false, true);
    let stderr = InheritFd(2);
    let args = ["-z", "-f", infile.as_slice()];

    let mut prc = match Command::new("target/servo")
        .args(args)
        .stdin(Ignored)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
    {
        Ok(p) => p,
        _ => fail!("Unable to spawn process."),
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
            fail!(line.to_string());
        }
    }

    let retval = prc.wait();
    if retval != Ok(ExitStatus(0)) {
        fail!("Servo exited with non-zero status {}", retval);
    }
}
