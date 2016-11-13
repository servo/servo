# Servo debugging guide

There are a few ways to debug Servo. `mach` supports a `--debug` flag that
searches a suitable debugger for you and runs servo with the appropriate
arguments under it:

```
./mach run --debug test.html
```

You can also specify an alternative debugger using the `--debugger` flag:

```
./mach run --debugger=my-debugger test.html
```

You can also, of course, run directly your debugger on the Servo binary:

```
$ gdb --args ./target/debug/servo test.html
```

## Debugging SpiderMonkey.

You can build Servo with a debug version of SpiderMonkey passing the
`--debug-mozjs` flag to `./mach build`.

Note that this sometimes can cause problems when an existing build exists, so
you might have to delete the `mozjs` build directory, or run `./mach clean`
before your first `--debug-mozjs` build.

## Debugging Servo with [rr][rr].

To record a trace under rr you can either use:

```
$ ./mach run --debugger=rr testcase.html
```

Or:

```
$ rr record ./target/debug/servo testcase.html
```

### Running WPT tests under rr's chaos mode.

Matt added a mode to Servo's testing commands to record traces of Servo running
a test or set of tests until the result is unexpected.

To use this, you can pass the `--chaos` argument to `mach test-wpt`:

```
$ ./mach test-wpt --chaos path/to/test
```

Note that for this to work you need to have `rr` in your `PATH`.

Also, note that this might generate a lot of traces, so you might want to delete
them when you're done. They're under `$HOME/.local/share/rr`.

### Known gotchas

If you use a Haswell processor that supports Hardware Lock Ellision, rr might
not work for you. There's a `rr` [bug][rr-bug] open about this. Until that gets
fixed, you can ensure that the `parking_lot` dependency isn't built with the
`nightly` feature, which as of this writing is the only dependency that uses it.

[rr]: http://rr-project.org/
[rr-bug]: https://github.com/mozilla/rr/issues/1883
