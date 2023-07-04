# WebRender Debugger
A web based debugger for WebRender.

## Using the debugger
Build your application with the debugger feature enabled, for example in wrench:

```
cargo build --features=debugger
```

Now, open your browser and open the debugger/index.html file. Click Connect and
the debugger will attempt to connect to WR via websocket.

## Using the debugger with Gecko

In the Gecko source tree, open ```gfx/webrender_bindings/Cargo.toml``` in a text editor.

Add ```features = ['debugger']``` to the end of the file (in the ```dependencies.webrender``` section).

Vendor the rust dependencies locally for the debugger (we don't want these committed to the repo):
```./mach vendor rust```

Now, build and run as usual, and the debugger will be available.
