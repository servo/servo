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


## Copy of original preflight script used:
```
#!/usr/bin/env bash
# Debian 12 preflight for the manual local-runtime symbol-split build.
# Keep this intentionally small: Servo's own mach/bootstrap installs the broad
# Linux build dependency set from python/servo/platform/linux_packages/apt/*.txt.
set -Eeuo pipefail

summary_file="${GITHUB_STEP_SUMMARY:-}"

log() {
  printf '[debian12-preflight] %s\n' "$*"
}

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf '[debian12-preflight] ERROR: required command not found: %s\n' "$command_name" >&2
    return 1
  fi
}

select_required_tool() {
  local label="$1"
  shift
  local tool
  for tool in "$@"; do
    if command -v "$tool" >/dev/null 2>&1; then
      printf '%s' "$tool"
      return 0
    fi
  done
  printf '[debian12-preflight] ERROR: missing %s; expected one of: %s\n' "$label" "$*" >&2
  return 1
}

print_context() {
  log '/etc/os-release:'
  cat /etc/os-release
  log 'uname -a:'
  uname -a
  log 'id:'
  id
  log 'pwd:'
  pwd
  log 'ldd --version:'
  ldd --version | sed -n '1,2p'
}

apt_install_pre_mach_tools() {
  export DEBIAN_FRONTEND=noninteractive
  apt-get update

  local packages=(
    # Servo devcontainer: curl/ca-certificates are present before bootstrap and
    # are needed here to fetch the same uv tool that the devcontainer copies from
    # ghcr.io/astral-sh/uv.
    ca-certificates
    curl

    # Servo bootstrap/mach requirement: ./mach reaches Python through uv before
    # mach_bootstrap can install Servo's broader Linux package set. python3 also
    # parses rust-toolchain.toml in this preflight.
    python3

    # local-runtime packaging/ABI reporting requirement: the symbol-split and
    # GLIBC-floor steps require GNU readelf/objcopy/strip-compatible tools.
    binutils

    # local-runtime release-upload requirement: later Debian workflow steps call
    # git for safe.directory setup and gh for stable release asset uploads.
    git
    gh

    # local-runtime packaging requirement: later steps unpack Servo's tarball and
    # write the split runtime/symbol ZIP artifacts.
    tar
    zip
  )

  apt-get install -y --no-install-recommends "${packages[@]}"
}

rust_toolchain_channel() {
  python3 - <<'PY_TOOLCHAIN'
import re
from pathlib import Path
text = Path("rust-toolchain.toml").read_text(encoding="utf-8")
match = re.search(r'^channel\s*=\s*["\']([^"\']+)["\']', text, re.MULTILINE)
if not match:
    raise SystemExit("rust-toolchain.toml does not declare a channel")
print(match.group(1))
PY_TOOLCHAIN
}

install_rustup_if_missing() {
  if command -v rustup >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1 && command -v rustc >/dev/null 2>&1; then
    log "Rust toolchain already available at $(command -v rustup), $(command -v cargo), $(command -v rustc)"
    return 0
  fi

  # Source: Servo devcontainer installs rustup into /usr/local with the
  # rust-toolchain.toml/.devcontainer RUST_VERSION toolchain before running
  # mach bootstrap. Servo bootstrap's base platform code expects rustup/cargo
  # to exist before it can install cargo-nextest and other Cargo tools.
  local rust_version
  rust_version="$(rust_toolchain_channel)"
  log "Installing rustup/Rust ${rust_version} because Servo bootstrap installs Cargo tools with cargo."
  export RUSTUP_HOME=/usr/local/rustup
  export CARGO_HOME=/usr/local/cargo
  export PATH="${CARGO_HOME}/bin:${PATH}"
  curl https://sh.rustup.rs -sSf \
    | sh -s -- --default-toolchain "${rust_version}" -y --component clippy,llvm-tools,llvm-tools-preview,rustc-dev,rustfmt,rust-src

  if [ -n "${GITHUB_PATH:-}" ]; then
    echo "${CARGO_HOME}/bin" >> "${GITHUB_PATH}"
  fi
}

install_uv_if_missing() {
  if command -v uv >/dev/null 2>&1; then
    log "uv already available at $(command -v uv)"
    return 0
  fi

  # Source: Servo's ./mach re-execs through `uv run --frozen ...` before
  # mach_bootstrap can install build dependencies. The devcontainer provides uv
  # by copying it from ghcr.io/astral-sh/uv into /bin, so this Debian-only
  # preflight provides the same pre-bootstrap command explicitly.
  log 'Installing uv because ./mach requires it before Servo bootstrap can run.'
  curl -LsSf https://astral.sh/uv/install.sh | UV_INSTALL_DIR=/usr/local/bin sh
}

verify_preflight_tools() {
  # Servo bootstrap/mach requirement: ./mach invokes uv and Python before the
  # Python bootstrap code can install Servo's full Linux dependency set.
  require_command uv
  require_command python3
  require_command curl

  # Servo devcontainer/bootstrap requirement: bootstrap installs the pinned Rust
  # toolchain and Cargo tools such as cargo-nextest through rustup/cargo.
  require_command rustup
  require_command rustc
  require_command cargo

  # local-runtime packaging/release requirement: later workflow steps use git/gh
  # for safe-directory setup and stable release uploads.
  require_command git
  require_command gh

  # local-runtime packaging/ABI reporting requirement: later workflow steps use
  # tar/zip plus readelf/objcopy/strip or LLVM-compatible equivalents to split
  # debug symbols and report the GLIBC floor.
  require_command tar
  require_command zip
  READELF_TOOL="$(select_required_tool 'ELF header reader' readelf llvm-readelf)"
  OBJCOPY_TOOL="$(select_required_tool 'debug objcopy tool' objcopy llvm-objcopy)"
  STRIP_TOOL="$(select_required_tool 'strip tool' strip llvm-strip)"
  export READELF_TOOL OBJCOPY_TOOL STRIP_TOOL
}

print_versions() {
  log 'Tool versions:'
  uv --version
  python3 --version
  curl --version | sed -n '1p'
  rustup --version
  rustc --version --verbose | sed -n '1,4p'
  cargo --version --verbose | sed -n '1,2p'
  git --version
  gh --version | sed -n '1p'
  tar --version | sed -n '1p'
  zip --version | sed -n '1p'
  "$READELF_TOOL" --version | sed -n '1p'
  "$OBJCOPY_TOOL" --version | sed -n '1p'
  "$STRIP_TOOL" --version | sed -n '1p'
}

write_summary() {
  if [ -z "$summary_file" ]; then
    return 0
  fi

  {
    echo '### Debian 12 preflight'
    echo
    echo '- Printed OS, kernel, identity, working-directory, and glibc context before Servo bootstrap.'
    echo '- Provided `uv` because `./mach` re-execs through `uv run --frozen` before bootstrap.'
    echo '- Installed only pre-mach essentials from Servo devcontainer/bootstrap needs plus local-runtime packaging, ABI reporting, and release-upload tools.'
    echo '- Provided `rustup`, `rustc`, and `cargo` from Servo devcontainer/bootstrap requirements so bootstrap can install Cargo tools such as `cargo-nextest`.'
    echo '- Verified `uv`, `python3`, `curl`, `rustup`, `rustc`, `cargo`, `git`, `gh`, `tar`, `zip`, and ELF tooling before long build steps.'
    echo
    echo '| Tool | Selected command |'
    echo '| --- | --- |'
    echo "| uv | $(command -v uv) |"
    echo "| python3 | $(command -v python3) |"
    echo "| rustup | $(command -v rustup) |"
    echo "| rustc | $(command -v rustc) |"
    echo "| cargo | $(command -v cargo) |"
    echo "| git | $(command -v git) |"
    echo "| gh | $(command -v gh) |"
    echo "| readelf | $(command -v "$READELF_TOOL") |"
    echo "| objcopy | $(command -v "$OBJCOPY_TOOL") |"
    echo "| strip | $(command -v "$STRIP_TOOL") |"
  } >> "$summary_file"
}

print_context
apt_install_pre_mach_tools
install_uv_if_missing
install_rustup_if_missing
verify_preflight_tools
print_versions
write_summary
log 'Preflight complete.'
```
