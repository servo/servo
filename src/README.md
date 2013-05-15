Descriptions of the Servo source directories. For the most part these represent a single
crate or library.

## Servo components

* `components/contenttest`: JavaScript test runner.
* `components/reftest`: Reference (layout) test runner.
* `components/servo`: The top-level Servo crate. Contains compositing, DOM, layout, and script.
* `components/servo-gfx`: Graphics rendering, fonts, and text shaping.
* `components/servo-net`: Networking, caching, image decoding.
* `components/servo-util`: Various utility functions used by other Servo components.

## The Rust compiler

* `compiler/rust`: The Rust compiler.

## Supporting libraries

These libraries are used in all Servo ports. In keeping with Servo's philosophy of modularity,
they are designed to be useful in other Rust projects.

* `support/azure`: A cross-platform 2D drawing library from the Mozilla project. Azure can render
  with Direct2D, Core Graphics (Quartz), Skia, and Cairo.
* `support/css`: A general CSS parsing and selector matching library. This abstraction layer
  exists to prevent `libcss` internals from leaking into Servo.
* `support/geom`: A simple Euclidean geometry and linear algebra library.
* `support/glut`: Bindings to the GLUT windowing framework. This bare-bones windowing framework is
  useful for testing the engine without browser chrome.
* `support/harfbuzz`: A mature Unicode- and OpenType-aware text shaping library, used by many
  rendering engines and toolkits.
* `support/http-client`: An HTTP client library for Rust.
* `support/hubbub`: The HTML parser from the NetSurf project. This is a temporary solution for HTML
  parsing until a pure-Rust solution is available.
* `support/layers`: A simple GPU-accelerated 2D scene graph library, somewhat similar to libraries
  like Clutter.
* `support/libparserutils`: A parsing library used by `hubbub` and `netsurfcss`.
* `support/netsurfcss`: The CSS library from the NetSurf project. This is a temporary stopgap for
  CSS parsing until a pure-Rust solution is available.
* `support/opengles`: Bindings to OpenGL ES 2.0.
* `support/sharegl`: A library for sharing OpenGL or Direct3D textures between processes.
* `support/skia`: Google's accelerated 2D rendering library.
* `support/spidermonkey`: Mozilla's JavaScript engine.
* `support/stb-image`: A minimalist image decoding library. This is a temporary stopgap for image
  decoding until a higher-performance solution is available.
* `support/wapcaplet`: A string storage library used by `hubbub` and `netsurfcss`.

## Platform-specfic bindings

### Linux

* `platform/linux/rust-fontconfig`: Bindings to the freedesktop.org `fontconfig` library.
* `platform/linux/rust-freetype`: Bindings to the FreeType library.
* `platform/linux/rust-xlib`: Bindings to the X Window System libraries.

### Mac

* `platform/macos/rust-cocoa`: General Cocoa bindings.
* `platform/macos/rust-core-foundation`: Bindings to Core Foundation.
* `platform/macos/rust-core-graphics`: Bindings to Core Graphics/Quartz.
* `platform/macos/rust-core-text`: Bindings to Core Text.
* `platform/macos/rust-io-surface`: Bindings to the `IOSurface` library.

## Miscellaneous

* `etc`: Various scripts and files that don't belong anywhere else.
* `etc/patches`: Patches for upstream libraries.
* `test`: Test cases.

