// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern mod std;
extern mod servo;

use std::test::{TestOpts, run_tests_console, TestDesc};
use std::getopts::{getopts, reqopt, opt_str, fail_str};
use os::list_dir_path;
use servo::run_pipeline_png;
use servo::image::base::Image;

fn main(args: ~[~str]) {
    let config = parse_config(args);
    let opts = test_options(config);
    let tests = find_tests(config);
    install_rasterize_py(config);
    run_tests_console(opts, tests);
}

struct Config {
    source_dir: ~str,
    work_dir: ~str,
    filter: Option<~str>
}

fn parse_config(args: ~[~str]) -> Config {
    let args = args.tail();
    let opts = ~[reqopt(~"source-dir"), reqopt(~"work-dir")];
    let matches = match getopts(args, opts) {
      Ok(m) => m,
      Err(f) => fail fail_str(f)
    };

    Config {
        source_dir: opt_str(matches, ~"source-dir"),
        work_dir: opt_str(matches, ~"work-dir"),
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
    let directives = load_test_directives(file);

    {
        name: file,
        testfn: fn~() { run_test(config, file) },
        ignore: directives.ignore,
        should_fail: false
    }
}

struct Directives {
    ignore: bool
}

fn load_test_directives(file: ~str) -> Directives {
    let data = match io::read_whole_file_str(&Path(file)) {
      result::Ok(data) => data,
      result::Err(e) => fail #fmt("unable to load directives for %s: %s", file, e)
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

            let (sr, sg, sb, sa) = servo_pixel;
            let (rr, rg, rb, ra) = ref_pixel;

            if sr != rr
                || sg != rg
                || sb != rb
                || sa != ra {
                fail #fmt("mismatched pixel. x: %?, y: %?, ref: %?, servo: %?", w, h, ref_pixel, servo_pixel)
            }
        }
    }
}

const WIDTH: uint = 800;
const HEIGHT: uint = 600;

fn render_servo(config: Config, file: ~str) -> Image {
    let infile = ~"file://" + os::make_absolute(&Path(file)).to_str();
    let outfilename = Path(file).filename().get().to_str() + ".png";
    let outfile = Path(config.work_dir).push(outfilename).to_str();
    run_pipeline_png(infile, outfile);
    return sanitize_image(outfile);
}

fn render_ref(config: Config, file: ~str) -> Image {
    let infile = file;
    let outfilename = Path(file).filename().get().to_str() + "ref..png";
    let outfile = Path(config.work_dir).push(outfilename);
    // After we've generated the reference image once, we don't need
    // to keep launching Firefox
    if !os::path_exists(&outfile) {
        let rasterize_path = rasterize_path(config);
        let prog = run::start_program("python", ~[rasterize_path, infile, outfile.to_str()]);
        prog.finish();
    }
    return sanitize_image(outfile.to_str());
}

fn sanitize_image(file: ~str) -> Image {
    let buf = io::read_whole_file(&Path(file)).get();
    let image = servo::image::base::load_from_memory(buf).get();

    // I don't know how to precisely control the rendered height of
    // the Firefox output, so it is larger than we want. Trim it down.
    assert image.width == WIDTH;
    assert image.height >= HEIGHT;
    let data = vec::slice(image.data, 0, image.width * HEIGHT * 4);

    return Image(image.width, HEIGHT, image.depth, data);
}

fn install_rasterize_py(config: Config) {
    use io::WriterUtil;
    let path = rasterize_path(config);
    let writer = io::file_writer(&Path(path), ~[io::Create, io::Truncate]).get();
    writer.write_str(rasterize_py());
}

fn rasterize_path(config: Config) -> ~str {
    Path(config.work_dir).push(~"rasterize.py").to_str()
}

// This is the script that uses phantom.js to render pages
fn rasterize_py() -> ~str { #include_str("rasterize.py") }
