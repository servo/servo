# Kun

Kun is another Servo port which is focussed on building ergonomic embedding APIs. It has a few
design choices in hope for better embedding. While it doesn't provide many features yet,
it already contains some building blocks that could solve some common use case scenarios:

- Multiple webview types like panel UI, context menu, and more.
- Basic multi-window / multi-webview support.
- Improve drawing order to overcome difficult rendering challenges like smoother resizing.

Winit primarily. See `main.rs` as an example of using it with the Winit event loop. You can
simple run it by `cargo run`. It's also to build if from mach by `./mach build --servo_kun`,
and then run it by `./mach run --bin PATH_TO_BBINARY`
