// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate std;
extern crate extra;
extern crate getopts;
extern crate test;

use test::{TestOpts, run_tests_console, TestDesc, TestDescAndFn, DynTestFn, DynTestName};
use getopts::{getopts, reqopt};
use std::{os, str};
use std::io::fs;
use std::io::Reader;
use std::io::process::{Process, ProcessConfig, Ignored, CreatePipe, InheritFd, ExitStatus};

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
    match run_tests_console(&opts, tests) {
        Err(_) => os::set_exit_status(1),
        _ => (),
    }
}

fn parse_config(args: ~[~str]) -> Config {
    let args = args.tail();
    let opts = ~[reqopt("s", "source-dir", "source-dir", "source-dir")];
    let matches = match getopts(args, opts) {
      Ok(m) => m,
      Err(f) => fail!(f.to_err_msg())
    };

    Config {
        source_dir: matches.opt_str("source-dir").unwrap(),
        filter: if matches.free.is_empty() {
            None
        } else {
            Some(matches.free.head().unwrap().clone())
        }
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
        logfile: None
    }
}

fn find_tests(config: Config) -> ~[TestDescAndFn] {
    let mut files_res = fs::readdir(&Path::new(config.source_dir));
    let mut files = match files_res {
        Ok(files) => files,
        _ => fail!("Error reading directory."),
    };
    files.retain(|file| file.extension_str() == Some("html") );
    return files.map(|file| make_test(file.display().to_str()) );
}

fn make_test(file: ~str) -> TestDescAndFn {
    TestDescAndFn {
        desc: TestDesc {
            name: DynTestName(file.clone()),
            ignore: false,
            should_fail: false
        },
        testfn: DynTestFn(proc() { run_test(file) })
    }
}

fn run_test(file: ~str) {
    let path = os::make_absolute(&Path::new(file));
    // FIXME (#1094): not the right way to transform a path
    let infile = ~"file://" + path.display().to_str();
    let stdout = CreatePipe(true, false); // rustc #10228
    let stderr = InheritFd(2);

    let config = ProcessConfig {
        program: "./servo",
        args: &[~"-z", ~"-f", infile.clone()],
        stdin: Ignored,
        stdout: stdout,
        stderr: stderr,
        .. ProcessConfig::new()
    };
    let mut prc = match Process::configure(config) {
        Ok(p) => p,
        _ => fail!("Unable to configure process."),
    };
    let mut output = ~[];
    loop {
        let byte = prc.stdout.get_mut_ref().read_byte();
        match byte {
            Ok(byte) => {
                print!("{}", byte as char);
                output.push(byte);
            }
            _ => break
        }
    }

    let out = str::from_utf8(output);
    let lines: ~[&str] = out.unwrap().split('\n').collect();
    for &line in lines.iter() {
        if line.contains("TEST-UNEXPECTED-FAIL") {
            fail!(line.to_owned());
        }
    }

    let retval = prc.wait();
    if retval != ExitStatus(0) {
        fail!("Servo exited with non-zero status {}", retval);
    }
}
