# Servo code organization

## Servo components

* `src/bin.rs`: Servo's entry point
* `src/lib.rs`: libservo entry point
* `components/layout`: The layout system.
* `components/style`: The CSS styling system.
* `components/script`: The JavaScript and DOM systems.
* `components/compositing`: The compositor and windowing systems.
* `components/gfx`: Graphics rendering, fonts, and text shaping.
* `components/net`: Networking, caching, image decoding.
* `components/msg`: Message structure definitions for inter-task communication.
* `components/*_traits`: Trait definitions to break crate dependencies.
* `components/plugins`: Various compiler plugins and macros used by the rest of Servo.
* `components/util`: Various utility functions used by other Servo components.

## Supporting libraries

These libraries are either internal but used by Servo or external and need
special integration:

* `support/glfw-rs`: Wrapping for the GLFW library. Will eventually move to
  being completely out of tree.
* `support/rust-task_info`: A binding to the task_info library on OS X.

## Tests

* `tests/contenttest.rs`: Content (JavaScript) test runner
* `tests/contenttest`: Content tests
* `tests/reftest.rs`: Reference (layout) test runner
* `tests/reftest`: Reference tests
* `tests/html`: Manual test cases and examples
* `tests/power`: Tests for measuring power usage
* `tests/wpt`: Web platform tests and harness

## Miscellaneous

* `etc`: Various scripts and files that don't belong anywhere else.
* `etc/patches`: Patches for upstream libraries.
