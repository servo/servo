# Servo code organization

## Servo components

* `components/servo`: Servo main program.
* `components/servo/main.rs`: Servo's entry point.
* `components/servo/lib.rs`: libservo entry point.
* `components/canvas`: HTML canvas graphics operations.
* `components/compositing`: The compositor and windowing systems.
* `components/devtools`: Server for remote Firefox developer tools.
* `components/gfx`: Graphics rendering, fonts, and text shaping.
* `components/layout`: The layout system.
* `components/msg`: Message structure definitions for inter-task communication.
* `components/net`: Networking, caching, image decoding.
* `components/plugins`: Various compiler plugins and macros used by the rest of Servo.
* `components/script`: The JavaScript and DOM systems.
* `components/style`: The CSS styling system.
* `components/util`: Various utility functions used by other Servo components.
* `components/*_traits`: Trait definitions to break crate dependencies.

## Supporting libraries

These libraries are either internal but used by Servo or external and need
special integration:

* `support/android`: Android-specific infrastructure.
* `support/android-rs-glue`: Android apk builder.
* `support/rust-task_info`: A binding to the task_info library on OS X.
* `support/time`: A temporary fork of libtime required for Android.

## Tests

* `tests/reftest.rs`: Reference (layout) test runner.
* `tests/ref`: Reference tests.
* `tests/html`: Manual test cases and examples.
* `tests/power`: Tests for measuring power usage.
* `tests/wpt`: Web platform tests and harness.

## Miscellaneous

* `etc`: Various scripts and files that don't belong anywhere else.
* `etc/patches`: Patches for upstream libraries.
