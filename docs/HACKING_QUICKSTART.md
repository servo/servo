# Hacking Servo - Quickstart

This guide covers the basic things one needs to know to start hacking Servo.
It doesn't cover how Servo works or how to use Git effectively (see the
[documentation](#documentation) section for those), but describes how to
set up your environment to compile, run, and debug Servo.

- [Building Servo](#building-servo)
- [Running Servo](#running-servo)
- [mach](#mach)
  - [mach and Servo options](#mach-and-servo-options)
- [Some basic Rust](#some-basic-rust)
  - [Cargo and crates](#cargo-and-crates)
  - [Working on a crate](#working-on-a-crate)
- [Editor support](#editor-support)
  - [Visual Studio Code](#visual-studio-code)
- [Debugging](#debugging)
  - [Logging](#logging)
  - [println!()](#println)
  - [Debugger](#debugger)
- [Tests](#tests)
  - [Updating a test](#updating-a-test)
  - [Add a new test](#add-a-new-test)
  - [Debugging a test](#debugging-a-test)
- [Documentation](#documentation)
- [Ask questions](#ask-questions)

## Building Servo

Building Servo is quite easy. Install the prerequisites described in the [README](../README.md) file, then type:

```shell
./mach build -d
```

There are three main build profiles, which you can build and use independently of one another:

- debug builds, which allow you to use a debugger (lldb)
- release builds, which are slower to build but more performant
- production builds, which are used for official releases only

| profile    | mach option            | optimised? | debug<br>info? | debug<br>assertions? | finds resources in<br>current working dir? |
| ---------- | ---------------------- | ---------- | -------------- | -------------------- | ------------------------------------------ |
| debug      | `-d`                   | no         | yes            | yes                  | yes                                        |
| release    | `-r`                   | yes        | no             | yes(!)               | yes                                        |
| production | `--profile production` | yes        | yes            | no                   | no                                         |

You can change these settings in a servobuild file (see [servobuild.example](../servobuild.example)) or in the root [Cargo.toml](../Cargo.toml).

## Running Servo

The servo binary is located in `target/debug/servo` (or `target/release/servo`). You can directly run this binary, but we recommend using `./mach` instead:

```shell
./mach run -d -- https://github.com
```

… is equivalent to:

```shell
./target/debug/servo https://github.com
```

If you build with `-d`, run with `-d`. If you build with `-r`, run with `-r`.

## mach

`mach` is a python utility that does plenty of things to make our life easier (build, run, run tests, update dependencies… see `./mach --help`). Beside editing files and git commands, everything else is done via `mach`.

```shell
./mach run -d [mach options] -- [servo options]
```

The `--` separates `mach` options from `servo` options. This is not required, but we recommend it. `mach` and `servo` have some options with the same name (`--help`, `--debug`), so the `--` makes it clear where options apply.

### mach and Servo options

This guide only covers the most important options. Be sure to look at all the available mach commands and the servo options:

```shell
./mach --help         # mach options
./mach run -- --help  # servo options
```

## Some basic Rust

Even if you have never seen any Rust code, it's not too hard to read Servo's code. But there are some basics things one must know:

- [Match](https://doc.rust-lang.org/stable/rust-by-example/flow_control/match.html) and [Patterns](https://doc.rust-lang.org/book/ch18-00-patterns.html)
- [Options](https://doc.rust-lang.org/stable/rust-by-example/std/option.html)
- [Expression](https://doc.rust-lang.org/stable/rust-by-example/expression.html)
- [Traits](https://doc.rust-lang.org/stable/rust-by-example/trait.html)
- That doesn't sound important, but be sure to understand how `println!()` works, especially the [formatting traits](https://doc.rust-lang.org/std/fmt/#formatting-traits)

This won't be enough to do any serious work at first, but if you want to navigate the code and fix basic bugs, that should do it. It's a good starting point, and as you dig into Servo source code, you'll learn more.

For more exhaustive documentation:

- [doc.rust-lang.org](https://doc.rust-lang.org)
- [rust by example](https://doc.rust-lang.org/stable/rust-by-example)

### Cargo and crates

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

See [Cargo's documentation about Cargo.toml and Cargo.lock files](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html).

### Working on a crate

As explained above, Servo depends on a lot of libraries, which makes it very modular. While working on a bug in Servo, you'll often end up in one of its dependencies. You will then want to compile your own version of the dependency (and maybe compiling against the HEAD of the library will fix the issue!).

For example, I'm trying to bring some cocoa events to Servo. The Servo window on Desktop is constructed with a library named [winit](https://github.com/rust-windowing/winit). winit itself depends on a cocoa library named [cocoa-rs](https://github.com/servo/cocoa-rs). When building Servo, magically, all these dependencies are downloaded and built for you. But because I want to work on this cocoa event feature, I want Servo to use my own version of _winit_ and _cocoa-rs_.

This is how my projects are laid out:

```
~/my-projects/servo/
~/my-projects/cocoa-rs/
```

Both folder are git repositories.

To make it so that servo uses `~/my-projects/cocoa-rs/`, first ascertain which version of the crate Servo is using and whether it is a git dependency or one from crates.io.

Both information can be found using, in this example, `cargo pkgid cocoa`(`cocoa` is the name of the package, which doesn't necessarily match the repo folder name).

If the output is in the format `https://github.com/servo/cocoa-rs#cocoa:0.0.0`, you are dealing with a git dependency and you will have to edit the `~/my-projects/servo/Cargo.toml` file and add at the bottom:

```toml
[patch]
"https://github.com/servo/cocoa-rs#cocoa:0.0.0" = { path = '../cocoa-rs' }
```

If the output is in the format `https://github.com/rust-lang/crates.io-index#cocoa#0.0.0`, you are dealing with a crates.io dependency and you will have to edit the `~/my-projects/servo/Cargo.toml` in the following way:

```toml
[patch]
"cocoa:0.0.0" = { path = '../cocoa-rs' }
```

Both will tell any cargo project to not use the online version of the dependency, but your local clone.

For more details about overriding dependencies, see [Cargo's documentation](https://doc.crates.io/specifying-dependencies.html#overriding-dependencies).

## Editor support

### Visual Studio Code

Running plain `cargo` will cause problems! For example, you might get rust-analyzer extension errors
about build scripts like

- The style crate requires enabling one of its 'servo' or 'gecko' feature flags and, in the 'servo'
  case, one of 'servo-layout-2013' or 'servo-layout-2020'.

- (if you are on NixOS) thread 'main' panicked at 'called \`Result::unwrap()\` on an \`Err\` value:
  "Could not run \`PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=\\"1\\" PKG_CONFIG_ALLOW_SYSTEM_LIBS=\\"1\\"
  \\"pkg-config\\" \\"--libs\\" \\"--cflags\\" \\"fontconfig\\"\`

- (if you are on NixOS) [ERROR rust_analyzer::main_loop] FetchWorkspaceError: rust-analyzer failed to load workspace: Failed to load the project at /path/to/servo/Cargo.toml: Failed to read Cargo metadata from Cargo.toml file /path/to/servo/Cargo.toml, Some(Version { major: 1, minor: 74, patch: 1 }): Failed to run `cd "/path/to/servo" && "cargo" "metadata" "--format-version" "1" "--manifest-path" "/path/to/servo/Cargo.toml" "--filter-platform" "x86_64-unknown-linux-gnu"`: `cargo metadata` exited with an error: error: could not execute process `crown -vV` (never executed)

This is because the rustflags (flags passed to the rust compiler) that standard `cargo` provides are
different to what `./mach` uses, and so every time Servo is built using `cargo` it will undo all the
work done by `./mach` (and vice versa).

You can override this in a `.vscode/settings.json` file:

```
{
    "rust-analyzer.check.overrideCommand": [
        "./mach", "check", "--message-format=json" ],
    "rust-analyzer.cargo.buildScripts.overrideCommand": [
        "./mach", "check", "--message-format=json" ],
    "rust-analyzer.rustfmt.overrideCommand": [ "./mach", "fmt" ],
}
```

If that still causes problems, then supplying a different target directory should fix this (although it will increase
the amount of disc space used).

```
{
    "rust-analyzer.checkOnSave.overrideCommand": [
        "./mach", "check", "--message-format=json", "--target-dir", "target/lsp" ],
    "rust-analyzer.cargo.buildScripts.overrideCommand": [
        "./mach", "check", "--message-format=json", "--target-dir", "target/lsp" ],
    "rust-analyzer.rustfmt.overrideCommand": [ "./mach", "fmt" ],
}
```

If you are on NixOS, you should also set CARGO_BUILD_RUSTC in `.vscode/settings.json` as follows,
where `/nix/store/.../crown` is the output of `nix-shell etc/shell.nix --run 'command -v crown'`.
These settings should be enough to not need to run `code .` from within a `nix-shell etc/shell.nix`,
but it wouldn’t hurt to try that if you still have problems.

```
{
    "rust-analyzer.server.extraEnv": {
        "CARGO_BUILD_RUSTC": "/nix/store/.../crown",
    },
}
```

When enabling rust-analyzer’s proc macro support, you may start to see errors like

- proc macro \`MallocSizeOf\` not expanded: Cannot create expander for /path/to/servo/target/debug/deps/libfoo-0781e5a02b945749.so: unsupported ABI \`rustc 1.69.0-nightly (dc1d9d50f 2023-01-31)\` rust-analyzer(unresolved-proc-macro)

This means rust-analyzer is using the wrong proc macro server, and you will need to configure the correct one manually. Use mach to query the current sysroot path, and copy the last line of output:

```
$ ./mach rustc --print sysroot
NOTE: Entering nix-shell etc/shell.nix
info: component 'llvm-tools' for target 'x86_64-unknown-linux-gnu' is up to date
/home/me/.rustup/toolchains/nightly-2023-02-01-x86_64-unknown-linux-gnu
```

Then configure either your sysroot path or proc macro server path in `.vscode/settings.json`:

```
{
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.cargo.sysroot": "[paste what you copied]",
    "rust-analyzer.procMacro.server": "[paste what you copied]/libexec/rust-analyzer-proc-macro-srv",
}
```

## Debugging

### Logging

Before starting the debugger right away, you might want to get some information about what's happening, how, and when. Luckily, Servo comes with plenty of logs that will help us. Type these 2 commands:

```shell
./mach run -d -- --help
./mach run -d -- --debug help
```

A typical command might be:

```shell
./mach run -d -- -i -y 1 --debug dump-style-tree /tmp/a.html
```

… to avoid using too many threads and make things easier to understand.

On macOS, you can add some Cocoa-specific debug options:

```shell
./mach run -d -- /tmp/a.html -- -NSShowAllViews YES
```

You can also enable some extra logging (warning: verbose!):

```
RUST_LOG="debug" ./mach run -d -- /tmp/a.html
```

Using `RUST_LOG="debug"` is usually the very first thing you might want to do if you have no idea what to look for. Because this is very verbose, you can combine these with `ts` (`moreutils` package (apt-get, brew)) to add timestamps and `tee` to save the logs (while keeping them in the console):

```
RUST_LOG="debug" ./mach run -d -- -i -y 1 /tmp/a.html 2>&1 | ts -s "%.S: " | tee /tmp/log.txt
```

You can filter by crate or module, for example `RUST_LOG="layout::inline=debug" ./mach run …`. Check the [env_logger](https://docs.rs/env_logger) documentation for more details.

Use `RUST_BACKTRACE=1` to dump the backtrace when Servo panics.

### println!()

You will want to add your own logs. Luckily, many structures [implement the `fmt::Debug` trait](https://doc.rust-lang.org/std/fmt/#fmtdisplay-vs-fmtdebug), so adding:

```rust
println!("foobar: {:?}", foobar)
```

usually just works. If it doesn't, maybe some of foobar's properties don't implement the right trait.

### Debugger

To run the debugger:

```shell
./mach run -d --debug -- -y 1 /tmp/a.html
```

This will start `lldb` on Mac, and `gdb` on Linux.

From here, use:

```shell
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

See this [lldb tutorial](https://lldb.llvm.org/tutorial.html) and this [gdb tutorial](http://www.unknownroad.com/rtfm/gdbtut/gdbtoc.html).

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

For your PR to get accepted, source code also has to satisfy certain tidiness requirements.

To check code tidiness:

```
./mach test-tidy
```

### Updating a test

In some cases, extensive tests for the feature you're working on already exist under tests/wpt:

- Make a release build
- run `./mach test-wpt --release --log-raw=/path/to/some/logfile`
- run [`update-wpt` on it](https://github.com/servo/servo/blob/main/tests/wpt/README.md#updating-test-expectations)

This may create a new commit with changes to expectation ini files. If there are lots of changes,
it's likely that your feature had tests in wpt already.

Include this commit in your pull request.

### Add a new test

If you need to create a new test file, it should be located in `tests/wpt/mozilla/tests` or in `tests/wpt/web-platform-tests` if it's something that doesn't depend on servo-only features. You'll then need to update the list of tests and the list of expected results:

```
./mach test-wpt --manifest-update
```

### Debugging a test

See the [debugging guide](./debugging.md) to get started in how to debug Servo.

## Documentation

- Servo's directory structure: [ORGANIZATION.md](./ORGANIZATION.md)
- https://doc.servo.org/servo/index.html
- https://github.com/servo/servo/wiki
- https://doc.rust-lang.org/rust-by-example/
- https://doc.rust-lang.org
- Cargo & crates: https://doc.crates.io/guide.html
- mach help: `./mach --help`
- servo options: `./mach run -- --help`
- servo debug options: `./mach run -- --debug help`

## Ask questions

### [Servo’s Zulip](https://servo.zulipchat.com/)

### Discord servers

The official [Rust Discord server](https://discordapp.com/invite/rust-lang) is a great place to ask Rust specific questions, including questions related to cargo.
