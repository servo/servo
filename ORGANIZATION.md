# Servo code organization

## Servo components

* [`components/servo`][components/servo]: Servo main program.
* [`components/servo/main.rs`][components/servo/main.rs]: Servo's entry point.
* [`components/servo/lib.rs`][components/servo/lib.rs]: libservo entry point.
* [`components/canvas`][components/canvas]: HTML canvas graphics operations.
* [`components/compositing`][components/compositing]: The compositor and windowing systems.
* [`components/devtools`][components/devtools]: Server for remote Firefox developer tools.
* [`components/gfx`][components/gfx]: Graphics rendering, fonts, and text shaping.
* [`components/layout`][components/layout]: The layout system.
* [`components/msg`][components/msg]: Message structure definitions for inter-task communication.
* [`components/net`][components/net]: Networking, caching, image decoding.
* [`components/plugins`][components/plugins]: Various compiler plugins and macros used by the rest of Servo.
* [`components/script`][components/script]: The JavaScript and DOM systems.
* [`components/style`][components/style]: The CSS styling system.
* [`components/util`][components/util]: Various utility functions used by other Servo components.
* `components/*_traits`: Trait definitions to break crate dependencies.

## Supporting libraries

These libraries are either internal but used by Servo or external and need
special integration:

* [`support/android`][support/android]: Android-specific infrastructure.
* [`support/android-rs-glue`][support/android-rs-glue]: Android apk builder.
* [`support/rust-task_info`][support/rust-task_info]: A binding to the task_info library on OS X.

## Tests

* [`tests/reftest.rs`][tests/reftest.rs]: Reference (layout) test runner.
* [`tests/ref`][tests/ref]: Reference tests.
* [`tests/html`][tests/html]: Manual test cases and examples.
* [`tests/power`][tests/power]: Tests for measuring power usage.
* [`tests/wpt`][tests/wpt]: Web platform tests and harness.

## Miscellaneous

* [`etc`][etc]: Various scripts and files that don't belong anywhere else.
* [`etc/patches`][etc/patches]: Patches for upstream libraries.

[components/servo]: https://github.com/servo/servo/tree/master/components/servo
[components/servo/main.rs]: https://github.com/servo/servo/tree/master/components/servo/main.rs
[components/servo/lib.rs]: https://github.com/servo/servo/tree/master/components/servo/lib.rs
[components/canvas]: https://github.com/servo/servo/tree/master/components/canvas
[components/compositing]: https://github.com/servo/servo/tree/master/components/compositing
[components/devtools]: https://github.com/servo/servo/tree/master/components/devtools
[components/gfx]: https://github.com/servo/servo/tree/master/components/gfx
[components/layout]: https://github.com/servo/servo/tree/master/components/layout
[components/msg]: https://github.com/servo/servo/tree/master/components/msg
[components/net]: https://github.com/servo/servo/tree/master/components/net
[components/plugins]: https://github.com/servo/servo/tree/master/components/plugins
[components/script]: https://github.com/servo/servo/tree/master/components/script
[components/style]: https://github.com/servo/servo/tree/master/components/style
[components/util]: https://github.com/servo/servo/tree/master/components/util
[support/android]: https://github.com/servo/servo/tree/master/support/android
[support/android-rs-glue]: https://github.com/tomaka/android-rs-glue
[support/rust-task_info]: https://github.com/servo/servo/tree/master/support/rust-task_info
[tests/reftest.rs]: https://github.com/servo/servo/tree/master/tests/reftest.rs
[tests/ref]: https://github.com/servo/servo/tree/master/tests/ref
[tests/html]: https://github.com/servo/servo/tree/master/tests/html
[tests/power]: https://github.com/servo/servo/tree/master/tests/power
[tests/wpt]: https://github.com/servo/servo/tree/master/tests/wpt
[etc]: https://github.com/servo/servo/tree/master/etc
[etc/patches]: https://github.com/servo/servo/tree/master/etc/patches
