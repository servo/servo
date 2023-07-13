# wrench

`wrench` is a tool for debugging webrender outside of a browser engine.

## Build

Build `wrench` with `cargo build --release` within the `wrench` directory.

## headless

`wrench` has an optional headless mode for use in continuous integration. To run in headless mode, instead of using `cargo run -- args`, use `./headless.py args`.

## `show`

If you are working on gecko integration you can capture a frame via the following steps.
* Visit about:support and check that the "Compositing" value in the "Graphics" table says "WebRender". Enable `gfx.webrender.all` in about:config if necessary to enable WebRender.
* Hit ctrl-shift-3 to capture the frame. The data will be put in `~/wr-capture`.
* View the capture with `wrench show ~/wr-capture`.

## `reftest`

Wrench also has a reftest system for catching regressions.
* To run all reftests, run `script/headless.py reftest`
* To run specific reftests, run `script/headless.py reftest path/to/test/or/dir`
* To examine test failures, use the [reftest analyzer](https://hg.mozilla.org/mozilla-central/raw-file/tip/layout/tools/reftest/reftest-analyzer.xhtml)
* To add a new reftest, create an example frame and a reference frame in `reftests/` and then add an entry to `reftests/reftest.list`
