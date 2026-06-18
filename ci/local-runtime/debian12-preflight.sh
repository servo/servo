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
    # mach_bootstrap can install Servo's broader Linux package set.
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
    echo '- Verified `uv`, `python3`, `curl`, `git`, `gh`, `tar`, `zip`, and ELF tooling before long build steps.'
    echo
    echo '| Tool | Selected command |'
    echo '| --- | --- |'
    echo "| uv | $(command -v uv) |"
    echo "| python3 | $(command -v python3) |"
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
verify_preflight_tools
print_versions
write_summary
log 'Preflight complete.'
