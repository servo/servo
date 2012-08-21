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
    install_rasterize_py(config);
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
        fail ~"rendered pages to not match";
    }
}

type Render = ~[u8];

const WIDTH: uint = 800;
const HEIGHT: uint = 600;

fn render_servo(config: Config, file: ~str) -> Render {
    let infile = file;
    let outfile = connect(config.work_dir, basename(file) + ".png");
    run_pipeline_png(infile, outfile);
    return sanitize_image(outfile);
}

fn render_ref(config: Config, file: ~str) -> Render {
    let infile = file;
    let outfile = connect(config.work_dir, basename(file) + ".ref.png");
    let rasterize_path = rasterize_path(config);
    let prog = run::start_program("python", ~[rasterize_path, infile, outfile]);
    prog.finish();
    return sanitize_image(outfile);
}

fn sanitize_image(file: ~str) -> Render {
    let buf = io::read_whole_file(file).get();
    let image = servo::image::base::load_from_memory(buf).get();

    // I don't know how to precisely control the rendered height of
    // the Firefox output, so it is larger than we want. Trim it down.
    assert image.width == WIDTH;
    assert image.height >= HEIGHT;
    vec::slice(image.data, 0, image.width * HEIGHT * 4)
}

fn install_rasterize_py(config: Config) {
    import io::WriterUtil;
    let path = rasterize_path(config);
    let writer = io::file_writer(path, ~[io::Create, io::Truncate]).get();
    writer.write_str(rasterize_py());
}

fn rasterize_path(config: Config) -> ~str {
    connect(config.work_dir, ~"rasterize.py")
}

// This is the script that uses phantom.js to render pages
fn rasterize_py() -> ~str { #include_str("rasterize.py") }
