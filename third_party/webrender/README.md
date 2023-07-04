# WebRender

[![Version](https://img.shields.io/crates/v/webrender.svg)](https://crates.io/crates/webrender)

WebRender is a GPU-based 2D rendering engine written in [Rust](https://www.rust-lang.org/). [Firefox](https://www.mozilla.org/firefox), the research web browser [Servo](https://github.com/servo/servo), and other GUI frameworks draw with it. It currently uses the OpenGL API internally.

Note that the canonical home for this code is in gfx/wr folder of the
mozilla-central repository at https://hg.mozilla.org/mozilla-central. The
Github repository at https://github.com/servo/webrender should be considered
a downstream mirror, although it contains additional metadata (such as Github
wiki pages) that do not exist in mozilla-central. Pull requests against the
Github repository are still being accepted, although once reviewed, they will
be landed on mozilla-central first and then mirrored back. If you are familiar
with the mozilla-central contribution workflow, filing bugs in
[Bugzilla](https://bugzilla.mozilla.org/enter_bug.cgi?product=Core&component=Graphics%3A%20WebRender)
and submitting patches there would be preferred.

## Update as a Dependency
After updating shaders in WebRender, go to servo and:

  * Go to the servo directory and do ./mach update-cargo -p webrender
  * Create a pull request to servo


## Use WebRender with Servo
To use a local copy of WebRender with servo, go to your servo build directory and:

  * Edit Cargo.toml
  * Add at the end of the file:

```
[patch."https://github.com/servo/webrender"]
"webrender" = { path = "<path>/webrender" }
"webrender_api" = { path = "<path>/webrender_api" }
```

where `<path>` is the path to your local copy of WebRender.

  * Build as normal

## Documentation

The Wiki has a [few pages](https://github.com/servo/webrender/wiki/) describing the internals and conventions of WebRender.

## Testing

Tests run using OSMesa to get consistent rendering across platforms.

Still there may be differences depending on font libraries on your system, for
example.

See [this gist](https://gist.github.com/finalfantasia/129cae811e02bf4551ac) for
how to make the text tests useful in Fedora, for example.
