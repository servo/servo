# Hacking Servo - Quickstart

This guide covers the basic things one needs to know to start hacking Servo.
It doesn't cover how Servo works (see the [documentation](#documentation) section for that),
but describes how to setup your environment to compile, run, and debug Servo. For information
on the [Github Workflow](https://github.com/servo/servo/wiki/Github-workflow) and some helpful
[Git Tips](https://github.com/servo/servo/wiki/Github-workflow#git-tips), see the
[Wiki](https://github.com/servo/servo/wiki).

## Building Servo

Building Servo is quite easy. Install the prerequisites described in the [README](../README.md) file, then type:

```shell
./mach build -d
```

*Note: on Mac, you might run into an SSL issue while compiling. You'll find a solution to this problem [here](https://github.com/sfackler/rust-openssl/issues/255).*

The `-d` option means "debug build". You can also build with the `-r` option which means "release build". Building with `-d` will allow you to use a debugger (lldb). A `-r` build is more performant. Release builds are slower to build.

You can use and build a release build and a debug build in parallel.

## Running Servo

The servo binary is located in `target/debug/servo` (or `target/release/servo`). You can directly run this binary, but we recommend using `./mach` instead:

``` shell
./mach run -d -- http://github.com
```

… is equivalent to:

``` shell
./target/debug/servo http://github.com
```

If you build with `-d`, run with `-d`. If you build with `-r`, run with `-r`.

## ./mach

`mach` is a python utility that does plenty of things to make our life easier (build, run, run tests, update dependencies… see `./mach --help`). Beside editing files and git commands, everything else is done via `mach`.

``` shell
./mach run -d [mach options] -- [servo options]
```

The `--`  separates `mach` options from `servo` options. This is not required, but we recommend it. `mach` and `servo` have some options with the same name (`--help`, `--debug`), so the `--` makes it clear where options apply.

## Mach and Servo options

This guide only covers the most important options. Be sure to look at all the available mach commands and the servo options:

``` shell
./mach --help         # mach options
./mach run -- --help  # servo options
```

## Some basic Rust

Even if you have never seen any Rust code, it's not too hard to read Servo's code. But there are some basics things one must know:

- [Match](https://doc.rust-lang.org/book/match.html) and [Patterns](https://doc.rust-lang.org/book/patterns.html)
- [Options](http://rustbyexample.com/std/option.html)
- [Expression](http://rustbyexample.com/expression.html)
- [Traits](http://rustbyexample.com/trait.html)
- That doesn't sound important, but be sure to understand how `println!()` works, especially the [formatting traits](https://doc.rust-lang.org/std/fmt/#formatting-traits)

This won't be enough to do any serious work at first, but if you want to navigate the code and fix basic bugs, that should do it. It's a good starting point, and as you dig into Servo source code, you'll learn more.

For more exhaustive documentation:

- [doc.rust-lang.org](https://doc.rust-lang.org)
- [rust by example](http://rustbyexample.com)

## Cargo and Crates

A Rust library is called a crate. Servo uses plenty of crates. These crates are dependencies. They are listed in files called `Cargo.toml`. Servo is split into components and ports (see `components` and `ports` directories). Each has its own dependencies, and each has its own `Cargo.toml` file.

`Cargo.toml` files list the dependencies. You can edit this file.

For example, `components/net_traits/Cargo.toml` includes:

```
 [dependencies.stb_image]
 git = "https://github.com/servo/rust-stb-image"
```

But because `rust-stb-image` API might change over time, it's not safe to compile against the `HEAD` of `rust-stb-image`. A `Cargo.lock` file is a snapshot of a `Cargo.toml` file which includes a reference to an exact revision, ensuring everybody is always compiling with the same configuration:

```
[[package]]
name = "stb_image"
source = "git+https://github.com/servo/rust-stb-image#f4c5380cd586bfe16326e05e2518aa044397894b"
```

This file should not be edited by hand. In a normal Rust project, to update the git revision, you would use `cargo update -p stb_image`, but in Servo, use `./mach cargo-update -p stb_image`. Other arguments to cargo are also understood, e.g. use --precise '0.2.3' to update that crate to version 0.2.3.

See [Cargo's documentation about Cargo.toml and Cargo.lock files](http://doc.crates.io/guide.html#cargotoml-vs-cargolock).

## Working on a Crate

As explained above, Servo depends on a lot of libraries, which makes it very modular. While working on a bug in Servo, you'll often end up in one of its dependencies. You will then want to compile your own version of the dependency (and maybe compiling against the HEAD of the library will fix the issue!).

For example, I'm trying to bring some cocoa events to Servo. The Servo window on Desktop is constructed with a library named [Glutin](https://github.com/tomaka/glutin). Glutin itself depends on a cocoa library named [cocoa-rs](http://github.com/servo/cocoa-rs). When building Servo, magically, all these dependencies are downloaded and built for you. But because I want to work on this cocoa event feature, I want Servo to use my own version of *glutin* and *cocoa-rs*.

This is how my projects are laid out:

```
~/my-projects/servo/
~/my-projects/cocoa-rs/
~/my-projects/glutin/
```

These are all git repositories.

To make it so that servo uses `~/my-projects/cocoa-rs/` and `~/my-projects/glutin/` , create a `~/my-projects/servo/.cargo/config` file:

``` shell
$ cat ~/my-projects/servo/.cargo/config
paths = ['../glutin', '../cocoa-rs']
```

This will tell any cargo project to not use the online version of the dependency, but your local clone.

For more details about overriding dependencies, see [Cargo's documentation](http://doc.crates.io/guide.html#overriding-dependencies).

## Debugging

### Logging:

Before starting the debugger right away, you might want to get some information about what's happening, how, and when. Luckily, Servo comes with plenty of logs that will help us. Type these 2 commands:

``` shell
./mach run -d -- --help
./mach run -d -- --debug help
```

A typical command might be:

``` shell
./mach run -d -- -i -y 1 -t 1 --debug dump-layer-tree /tmp/a.html
```

… to avoid using too many threads and make things easier to understand.

On OSX, you can add some Cocoa-specific debug options:

``` shell
./mach run -d -- /tmp/a.html -- -NSShowAllViews YES
```

You can also enable some extra logging (warning: verbose!):

```
RUST_LOG="debug" ./mach run -d -- /tmp/a.html
```

Using `RUST_LOG="debug"` is usually the very first thing you might want to do if you have no idea what to look for. Because this is very verbose, you can combine these with `ts` (`moreutils` package (apt-get, brew)) to add timestamps and `tee` to save the logs (while keeping them in the console):

```
RUST_LOG="debug" ./mach run -d -- -i -y 1 -t 1  /tmp/a.html 2>&1 | ts -s "%.S: " | tee /tmp/log.txt
```

You can filter by crate or module, for example `RUST_LOG="layout::inline=debug" ./mach run …`. Check the [env_logger](http://doc.rust-lang.org/log/env_logger/index.html) documentation for more details.

Use `RUST_BACKTRACE=1` to dump the backtrace when Servo panics.

### println!()

You will want to add your own logs. Luckily, many structures [implement the `fmt::Debug` trait](https://doc.rust-lang.org/std/fmt/#fmt::display-vs-fmt::debug), so adding:

``` rust
println!("foobar: {:?}", foobar)
```

usually just works. If it doesn't, maybe some of foobar's properties don't implement the right trait.

### Debugger

To run the debugger:

``` shell
./mach run -d --debug -- -y 1 -t 1 /tmp/a.html
```

This will start `lldb` on Mac, and `gdb` on Linux.

From here, use:

``` shell
(lldb) b a_servo_function # add a breakpoint
(lldb) run # run until breakpoint is reached
(lldb) bt # see backtrace
(lldb) frame n # choose the stack frame from the number in the bt
(lldb) thread list
(lldb) next / step / …
(lldb) print varname
```

And to search for a function's full name/regex:

```shell
(lldb) image lookup -r -n <name> #lldb
(gdb) info functions <name> #gdb
```

See this [lldb tutorial](http://lldb.llvm.org/tutorial.html) and this [gdb tutorial](http://www.unknownroad.com/rtfm/gdbtut/gdbtoc.html).

To inspect variables and you are new with lldb, we recommend using the `gui` mode (use left/right to expand variables):

```
(lldb) gui
┌──<Variables>───────────────────────────────────────────────────────────────────────────┐
│ ◆─(&mut gfx::paint_task::PaintTask<Box<CompositorProxy>>) self = 0x000070000163a5b0    │
│ ├─◆─(msg::constellation_msg::PipelineId) id                                            │
│ ├─◆─(url::Url) _url                                                                    │
│ │ ├─◆─(collections::string::String) scheme                                             │
│ │ │ └─◆─(collections::vec::Vec<u8>) vec                                                │
│ │ ├─◆─(url::SchemeData) scheme_data                                                    │
│ │ ├─◆─(core::option::Option<collections::string::String>) query                        │
│ │ └─◆─(core::option::Option<collections::string::String>) fragment                     │
│ ├─◆─(std::sync::mpsc::Receiver<gfx::paint_task::LayoutToPaintMsg>) layout_to_paint_port│
│ ├─◆─(std::sync::mpsc::Receiver<gfx::paint_task::ChromeToPaintMsg>) chrome_to_paint_port│
└────────────────────────────────────────────────────────────────────────────────────────┘
```

If lldb crashes on certain lines involving the `profile()` function, it's not just you. Comment out the profiling code, and only keep the inner function, and that should do it.

## Tests

This is boring. But your PR won't get accepted without a test. Tests are located in the `tests` directory. You'll see that there are a lot of files in there, so finding the proper location for your test is not always obvious.

First, look at the "Testing" section in `./mach --help` to understand the different test categories. You'll also find some `update-*` commands. It's used to update the list of expected results.

To run a test:

```
./mach test-wpt tests/wpt/yourtest
```

### Updating a test:

In some cases, extensive tests for the feature you're working on already exist under tests/wpt:

- Make a release build
- run `./mach test-wpt --release --log-raw=/path/to/some/logfile`
- run [`update-wpt` on it](https://github.com/servo/servo/blob/master/tests/wpt/README.md#updating-test-expectations)

This may create a new commit with changes to expectation ini files. If there are lots of changes,
it's likely that your feature had tests in wpt already.

Include this commit in your pull request.

### Add a new test:

If you need to create a new test file, it should be located in `tests/wpt/mozilla/tests` or in `tests/wpt/web-platform-tests` if it's something that doesn't depend on servo-only features. You'll then need to update the list of tests and the list of expected results:

```
./mach test-wpt --manifest-update
```

### Debugging a test

See the [debugging guide](./debugging.md) to get started in how to debug Servo.

## Documentation:

- Servo's directory structure: [ORGANIZATION.md](./ORGANIZATION.md)
- http://doc.servo.org/servo/index.html
- https://github.com/servo/servo/wiki
- http://rustbyexample.com
- https://doc.rust-lang.org
- Cargo & crates: http://doc.crates.io/guide.html
- mach help: `./mach --help`
- servo options: `./mach run -- --help`
- servo debug options: `./mach run -- --debug help`

## Ask questions

### IRC

IRC channels (irc.mozilla.org):

- #servo
- #rust
- #cargo

### Mailing list

https://lists.mozilla.org/listinfo/dev-servo
