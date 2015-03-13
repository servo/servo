// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(collections, core, env, io, os, path, rustc_private, std_misc, test)]

extern crate getopts;
extern crate test;

use test::{AutoColor, TestOpts, run_tests_console, TestDesc, TestDescAndFn, DynTestFn, DynTestName};
use test::ShouldFail;
use getopts::{getopts, reqopt};
use std::{str, env};
use std::old_io::fs;
use std::old_io::Reader;
use std::old_io::process::{Command, Ignored, CreatePipe, InheritFd, ExitStatus};
use std::old_path::Path;
use std::thunk::Thunk;

#[derive(Clone)]
struct Config {
    source_dir: String,
    filter: Option<String>
}

fn main() {
    let args = env::args();
    let config = parse_config(args.map(|x| x.into_string().unwrap()).collect());
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
      Err(f) => panic!(format!("{}", f))
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
    let files_res = fs::readdir(&Path::new(config.source_dir));
    let mut files = match files_res {
        Ok(files) => files,
        _ => panic!("Error reading directory."),
    };
    files.retain(|file| file.extension_str() == Some("html") );
    return files.iter().map(|file| make_test(format!("{}", file.display()))).collect();
}

fn make_test(file: String) -> TestDescAndFn {
    TestDescAndFn {
        desc: TestDesc {
            name: DynTestName(file.clone()),
            ignore: false,
            should_fail: ShouldFail::No,
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
    let mut prc = match Command::new(prc_arg)
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
