Descriptions of the Servo source directories. For the most part these represent a single
crate or library.

## Servo and Rust projects

These are the main servo projects, as well as projects that are pure Rust (not bindings).

* _rust-css_ - A CSS parsing and selector matching library, based on libcss and currently
    using libcss for the implementation, but not exposing any libcss internals
* _rust-geom_ - Generic linear algebra library
* _rust-http-client_ - An HTTP library in Rust, using libuv via the Rust standard library
* _rust-layers_ - A simple GPU-accelerated 2D scene graph library
* _servo_ - The Servo web browser engine
* _servo-gfx_ - Servo's graphics subsystem
* _sharegl_ - A library for sharing 3D textures between processes

## Third-party projects

* _libcss_ - The CSS library from the NetSurf web browser. We are using this for parsing
    and selector matching until we have a Rust solution
* _libhubbub_ - The HTML parser from the NetSurf web browser. We are using this for parsing
    until we have a Rust solution
* _libparserutils_ - A parsing library used by libcss and libhubbub
* _libwapcaplet_ - A string internment library used by libcss and libhubbub
* _mozjs_ - The SpiderMonkey JavaScript engine, forked from mozilla-central
* _rust-azure_ - mozilla-central's 2D vector graphics abstraction layer and bindings
* _rust-harfbuzz_ - The harfbuzz text shaping library and bindings
* _rust-stb-image_ - The stb_image library and bindings. We are using this for image decoding
    until we have a higher-performance and/or Rust solution
* _skia_ - The Skia drawing library, a backend for Azure

## Library bindings

* _rust-cocoa_ - Bindings to OS X's Cocoa framework
* _rust-core-foundation_ - Bindings to OS X's Core Foundation framework
* _rust-core-graphics_ - Bindings to OS X's Core Graphics framework, a backend for Azure
* _rust-core-text_ - Bindings to OS X's Core Text framework
* _rust-fontconfig_ - Bindings to fontconfig
* _rust-freetype_ - Bindings to FreeType
* _rust-glut_ - Bindings to GLUT
* _rust-hubbub_ - Bindings to libhubbub
* _rust-io-surface_ - OS X implementation of cross-process surface sharing. Used by sharegl
* _rust-mozjs_ - Bindings to SpiderMonkey
* _rust-netsurfcss_ - Bindings to libcss
* _rust-opengles_ - Bindings to OpenGL ES
* _rust-wapcaplet_ - Bindings to libwapcaplet
* _rust-xlib_ - Bindings to xlib

## Other

* _contenttest_ - Test harness for JavaScript bindings
* _etc_ - Miscellaneous
* _reftest_ - Test harness for comparing Servo output to Firefox
* _test_ - Test cases
