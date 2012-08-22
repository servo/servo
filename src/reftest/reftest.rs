use std;
use servo;

import result::{ok, err};
import std::test::{test_opts, run_tests_console, test_desc};
import std::getopts::{getopts, reqopt, opt_str, fail_str};
import path::{connect, basename};
import os::list_dir_path;
import servo::run_pipeline_png;
import servo::image::base::Image;

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
    let directives = load_test_directives(file);

    {
        name: file,
        fn: fn~() { run_test(config, file) },
        ignore: directives.ignore,
        should_fail: false
    }
}

struct Directives {
    ignore: bool;
}

fn load_test_directives(file: ~str) -> Directives {
    let data = match io::read_whole_file_str(file) {
      result::ok(data) => data,
      result::err(*) => fail #fmt("unable to load directives for %s", file)
    };

    let mut ignore = false;

    for str::lines(data).each |line| {
        if is_comment(line) {
            if line.contains("ignore") {
                ignore = true;
                break;
            }
        }
    }

    fn is_comment(line: ~str) -> bool {
        line.starts_with("<!--")
    }

    return Directives {
        ignore: ignore
    }
}

fn run_test(config: Config, file: ~str) {
    let servo_image = render_servo(config, file);
    let ref_image = render_ref(config, file);

    assert servo_image.width == ref_image.width;
    assert servo_image.height == ref_image.height;
    #debug("image depth: ref: %?, servo: %?", ref_image.depth, servo_image.depth);

    for uint::range(0, servo_image.height) |h| {
        for uint::range(0, servo_image.width) |w| {
            let i = (h * servo_image.width + w) * 4;
            let servo_pixel = (
                servo_image.data[i + 0],
                servo_image.data[i + 1],
                servo_image.data[i + 2],
                servo_image.data[i + 3]
            );
            let ref_pixel = (
                ref_image.data[i + 0],
                ref_image.data[i + 1],
                ref_image.data[i + 2],
                ref_image.data[i + 3]
            );
            #debug("i: %?, x: %?, y: %?, ref: %?, servo: %?", i, w, h, ref_pixel, servo_pixel);

            if servo_pixel != ref_pixel {
                fail #fmt("mismatched pixel. x: %?, y: %?, ref: %?, servo: %?", w, h, ref_pixel, servo_pixel)
            }
        }
    }
}

const WIDTH: uint = 800;
const HEIGHT: uint = 600;

fn render_servo(config: Config, file: ~str) -> Image {
    let infile = ~"file://" + os::make_absolute(file);
    let outfile = connect(config.work_dir, basename(file) + ".png");
    run_pipeline_png(infile, outfile);
    return sanitize_image(outfile);
}

fn render_ref(config: Config, file: ~str) -> Image {
    let infile = file;
    let outfile = connect(config.work_dir, basename(file) + ".ref.png");
    // After we've generated the reference image once, we don't need
    // to keep launching Firefox
    if !os::path_exists(outfile) {
        let rasterize_path = rasterize_path(config);
        let prog = run::start_program("python", ~[rasterize_path, infile, outfile]);
        prog.finish();
    }
    return sanitize_image(outfile);
}

fn sanitize_image(file: ~str) -> Image {
    let buf = io::read_whole_file(file).get();
    let image = servo::image::base::load_from_memory(buf).get();

    // I don't know how to precisely control the rendered height of
    // the Firefox output, so it is larger than we want. Trim it down.
    assert image.width == WIDTH;
    assert image.height >= HEIGHT;
    let data = vec::slice(image.data, 0, image.width * HEIGHT * 4);

    return Image(image.width, HEIGHT, image.depth, data);
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
