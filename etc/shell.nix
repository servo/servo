# This provides a shell with all the necesarry packages required to run mach and build servo
# NOTE: This does not work offline or for nix-build

with import <nixpkgs> {};
let
    pinnedSha = "6adf48f53d819a7b6e15672817fa1e78e5f4e84f";
    pinnedNixpkgs = import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/${pinnedSha}.tar.gz";
    }) {};
in
clangStdenv.mkDerivation rec {
  name = "servo-env";

  buildInputs = [
    # Native dependencies
    fontconfig freetype openssl libunwind
    xorg.libxcb
    xorg.libX11

    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-bad

    rustup
    llvmPackages.bintools # provides lld

    # Build utilities
    cmake dbus gcc git pkg-config which llvm autoconf213 perl yasm m4
    (python3.withPackages (ps: with ps; [virtualenv pip dbus]))
    # This pins gnumake to 4.3 since 4.4 breaks jobserver
    # functionality in mozjs and causes builds to be extremely
    # slow as it behaves as if -j1 was passed.
    # See https://github.com/servo/mozjs/issues/375
    pinnedNixpkgs.gnumake
  ] ++ (lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.AppKit
  ]);

  LIBCLANG_PATH = llvmPackages.clang-unwrapped.lib + "/lib/";

  # Allow cargo to download crates
  SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";

  # Enable colored cargo and rustc output
  TERMINFO = "${ncurses.out}/share/terminfo";

  # Fix missing libraries errors (those libraries aren't linked against, so we need to dynamically supply them)
  LD_LIBRARY_PATH = lib.makeLibraryPath [ xorg.libXcursor xorg.libXrandr xorg.libXi libxkbcommon ];

  shellHook = ''
    # Fix invalid option errors during linking
    # https://github.com/mozilla/nixpkgs-mozilla/commit/c72ff151a3e25f14182569679ed4cd22ef352328
    unset AS
  '';
}
