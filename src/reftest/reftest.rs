use std;

import result::{ok, err};
import std::test{test_opts, run_tests_console, test_desc};
import std::getopts::{getopts, reqopt, opt_opt, fail_str};

fn main(args: [str]) {
    let config = parse_config(args);
    let opts = test_options(config);
    let tests = find_tests(config);
    run_tests_console(opts, tests);
}

type Config = {
    source_dir: str,
    work_dir: str
};

fn parse_config(args: [str]) -> Config {
    let args = args.tail();
    let opts = [reqopt("source-dir"), reqopt("work-dir")];
    let match = alt getopts(args, opts) {
      ok(m) { m }
      err(f) { fail fail_str(f) }
    }

    {
        source_dir: opt_str(match, "source-dir"),
        work_dir: opt_str(match, "work-dir"),
        filter: if match.free.is_empty() {
            none
        } else {
            some(match.head())
        }
    }
}

fn test_options(config: config) -> test_opts {
    {
        filter: none,
        run_ignored: false,
        logfile: none
    }
}

fn find_tests(config: config) -> [test_desc] {
    fail;
}

// This is the script that uses phantom.js to render pages
fn rasterize_js() -> str { #include_str("rasterize.js") }
