# Debian 12 Plain-Base Preflight Recovery Notes

This note preserves the retired plain-`debian:12` bootstrap/preflight path that
was previously used by the manual local-runtime Debian artifact workflow.

The current default artifact path should continue to use the reusable Debian 12
build-room image:

```text
ghcr.io/thesepeoplearenotyourfriends/servo-debian12-build:bookworm
```

That image carries the Debian/Servo "breakfast" tooling layer before the
workflow starts, so the live artifact workflow no longer needs an executable
`ci/local-runtime/debian12-preflight.sh` script. The script was removed from
`ci/` so Servo tidy does not treat obsolete recovery logic as active shell code.

Keep this file as documentation only. It is not an active workflow script and is
not intended to be invoked by CI. The recipe may still be useful if the GHCR
image becomes unavailable, registry authentication breaks, tags are lost, tool
URLs or signing keys change, or the Debian artifact path must be reconstructed
from a plain `debian:12` container.

## Recovery recipe from the retired script

The retired preflight did four things before the long Servo build/package steps:

1. Printed OS, kernel, user, working-directory, and glibc context.
2. Installed only pre-`mach` essentials needed by Servo bootstrap plus the
   local-runtime packaging, ABI-reporting, and release-upload steps.
3. Installed `uv` because `./mach` re-execs through `uv run --frozen` before
   Servo bootstrap can install broader dependencies.
4. Installed the pinned Rust toolchain from `rust-toolchain.toml` when
   `rustup`, `rustc`, or `cargo` were missing, because Servo bootstrap installs
   Cargo tools during `./mach bootstrap --yes`.

The package/tooling set was:

```text
apt-get update
apt-get install -y --no-install-recommends \
  ca-certificates \
  curl \
  python3 \
  binutils \
  git \
  gh \
  tar \
  zip
```

The `uv` install step was:

```text
curl -LsSf https://astral.sh/uv/install.sh | UV_INSTALL_DIR=/usr/local/bin sh
```

The Rust recovery logic read the `channel = "..."` value from
`rust-toolchain.toml`, then installed rustup into `/usr/local` with Servo's
expected components:

```text
export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PATH="${CARGO_HOME}/bin:${PATH}"
curl https://sh.rustup.rs -sSf \
  | sh -s -- --default-toolchain "${rust_version}" -y --component clippy,llvm-tools,llvm-tools-preview,rustc-dev,rustfmt,rust-src
```

When run in GitHub Actions, the old script also appended
`/usr/local/cargo/bin` to `GITHUB_PATH` so later workflow steps could find the
Rust tools.

The preflight considered these commands mandatory before returning success:

```text
uv
python3
curl
rustup
rustc
cargo
git
gh
tar
zip
```

It also selected one command from each ELF tooling group:

```text
readelf or llvm-readelf
objcopy or llvm-objcopy
strip or llvm-strip
```

The old summary output recorded selected command paths for `uv`, `python3`,
`rustup`, `rustc`, `cargo`, `git`, `readelf`, `objcopy`, and `strip`, plus short
version output for the required tools. If this path is ever revived, keep it as
a deliberate recovery workflow or image-rebuild recipe rather than restoring an
obsolete live script beside the current build-room image path.
