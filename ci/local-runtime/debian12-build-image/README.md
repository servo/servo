# Debian 12 Servo build-room image

This directory defines the Debian 12/bookworm build-room image for local-runtime
Servo artifacts.

The image intentionally does **not** include:

- Servo source checkout
- `/var/servo-cargo-target` build output
- compiled Rust dependencies
- compiled Servo artifacts
- artifact archives or debug symbols
- release-upload logic

It exists to avoid reinstalling the known-working Debian 12 toolchain and setup
every time a local-runtime Debian artifact build runs. The image contains the
pre-bootstrap tools needed by `./mach`, the Linux build packages from Servo's
Debian-compatible package list, `uv`, Rust bootstrap tooling, and the ELF tools
used by the local-runtime symbol-split and GLIBC-floor reporting path.

Update this image only when the Debian build prerequisites change. Do not use it
as a place to cache Servo source or compiled output.
