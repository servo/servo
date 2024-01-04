# This provides a shell with all the necesarry packages required to run mach and build servo
# NOTE: This does not work offline or for nix-build

with import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/6adf48f53d819a7b6e15672817fa1e78e5f4e84f.tar.gz";
    }) {};
clangStdenv.mkDerivation rec {
  name = "servo-env";

  buildInputs = [
    # Native dependencies
    fontconfig freetype libunwind
    xorg.libxcb
    xorg.libX11

    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-bad

    # rustup # TODO NixOS only or set RUSTUP_HOME
    taplo
    llvmPackages.bintools # provides lld

    # Build utilities
    cmake dbus gcc git pkg-config which llvm perl yasm m4
    (python3.withPackages (ps: with ps; [virtualenv pip dbus]))
    # This pins gnumake to 4.3 since 4.4 breaks jobserver
    # functionality in mozjs and causes builds to be extremely
    # slow as it behaves as if -j1 was passed.
    # See https://github.com/servo/mozjs/issues/375
    gnumake
  ] ++ (lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.AppKit
  ]);

  LIBCLANG_PATH = llvmPackages.clang-unwrapped.lib + "/lib/";

  # Allow cargo to download crates
  SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";

  # Enable colored cargo and rustc output
  TERMINFO = "${ncurses.out}/share/terminfo";

  # Provide libraries that aren’t linked against but somehow required
  LD_LIBRARY_PATH = lib.makeLibraryPath [
    # Fixes missing library errors
    xorg.libXcursor xorg.libXrandr xorg.libXi libxkbcommon

    # [WARN  script::dom::gpu] Could not get GPUAdapter ("NotFound")
    # TLA Err: Error: Couldn't request WebGPU adapter.
    vulkan-loader
  ];

  shellHook = ''
    # Fix invalid option errors during linking
    # https://github.com/mozilla/nixpkgs-mozilla/commit/c72ff151a3e25f14182569679ed4cd22ef352328
    unset AS

    # Compiling programs under Nix sets the interpreter (ELF INTERP) and rpath (ELF DT_RUNPATH [1])
    # to ensure that it can find the needed (ELF DT_NEEDED) libraries in the Nix store.
    #
    # This is good on NixOS, but bad everywhere else. Using the Nix interpreter makes the programs
    # dependent on the Nix store, making them impossible to distribute and run on other machines
    # without `nix bundle`. Even on the same machine, the program will crash in a variety of ways
    # because of the “OpenGL problem” [2] and other mismatches in X11 and Wayland libraries. Worse
    # still, it makes everyone else suffer the problems NixOS has, like needing $LD_LIBRARY_PATH
    # (see above) and needing to disable rust-lld (servo#30123).
    #
    # We can make the programs independent of Nix by resetting $NIX_DYNAMIC_LINKER to the system
    # interpreter, setting $NIX_DONT_SET_RPATH to prevent the clang and ld wrappers from adding
    # -rpath options to $NIX_LDFLAGS [3][4], and removing any -rpath options that get added by
    # clangStdenv despite $NIX_DONT_SET_RPATH.
    #
    # This is comparable to fixing target/*/servo after the fact with:
    #
    #     patchelf --remove-rpath --set-interpreter $(patchelf --print-interpreter /usr/bin/env)
    #
    # [1] DT_RPATH breaks LD_LIBRARY_PATH and is no longer used
    #     https://medium.com/obscure-system/rpath-vs-runpath-883029b17c45
    # [2] “Using Nix on non-NixOS distros, it’s common to see GL application errors:”
    #     https://github.com/nix-community/nixGL
    # [3] https://ryantm.github.io/nixpkgs/stdenv/stdenv/#bintools-wrapper
    # [4] https://matklad.github.io/2022/03/14/rpath-or-why-lld-doesnt-work-on-nixos.html
    if ! [ -e /etc/NIXOS ]; then
      set -- $NIX_LDFLAGS
      for i; do
        shift
        if [ "$i" = -rpath ]; then
          shift
        else
          set -- "$@" "$i"
        fi
      done
      export NIX_DYNAMIC_LINKER=$(patchelf --print-interpreter /usr/bin/env)
      export NIX_DONT_SET_RPATH=1
      export NIX_LDFLAGS="$@"
    fi
  '';
}
