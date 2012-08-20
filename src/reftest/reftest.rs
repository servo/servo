use std;
use servo;

import result::{ok, err};
import std::test::{test_opts, run_tests_console, test_desc};
import std::getopts::{getopts, reqopt, opt_str, fail_str};
import path::{connect, basename};
import os::list_dir_path;
import servo::run_pipeline_png;

fn main(args: ~[~str]) {
    let config = parse_config(args);
    let opts = test_options(config);
    let tests = find_tests(config);
    install_rasterize_py();
    run_tests_console(opts, tests);
}

struct Config {
    source_dir: ~str;
    work_dir: ~str;
    filter: option<~str>;
}

fn parse_config(args: ~[~str]) -> Config {
    let args = args.tail();
    let opts = ~[reqopt(~"source-dir"), reqopt(~"work-dir")];
    let matches = match getopts(args, opts) {
      ok(m) => m,
      err(f) => fail fail_str(f)
    };

    Config {
        source_dir: opt_str(matches, ~"source-dir"),
        work_dir: opt_str(matches, ~"work-dir"),
        filter: if matches.free.is_empty() {
            none
        } else {
            some(matches.free.head())
        }
    }
}

fn test_options(config: Config) -> test_opts {
    {
        filter: config.filter,
        run_ignored: false,
        logfile: none
    }
}

fn find_tests(config: Config) -> ~[test_desc] {
    let html_files = list_dir_path(config.source_dir).filter( |file| file.ends_with(".html") );
    return html_files.map(|file| make_test(config, file) );
}

fn make_test(config: Config, file: ~str) -> test_desc {
    {
        name: file,
        fn: fn~() { run_test(config, file) },
        ignore: false,
        should_fail: false
    }
}

fn run_test(config: Config, file: ~str) {
    let servo_render = render_servo(config, file);
    let ref_render = render_ref(config, file);
    if servo_render != ref_render {
        fail;
    }
}

type Render = ~[u8];

fn render_servo(config: Config, file: ~str) -> Render {
    let infile = file;
    let outfile = connect(config.work_dir, basename(file) + ".png");
    run_pipeline_png(infile, outfile);
    fail;
}

fn render_ref(config: Config, file: ~str) -> Render {
    fail
}

fn install_rasterize_py() { }

// This is the script that uses phantom.js to render pages
fn rasterize_py() -> ~str { #include_str("rasterize.py") }
