# This provides a shell with all the necesarry packages required to run mach and build servo
# NOTE: This does not work offline or for nix-build

{
  buildAndroid ? false
}:
with import (builtins.fetchTarball {
  url = "https://github.com/NixOS/nixpkgs/archive/63d37ccd2d178d54e7fb691d7ec76000740ea24a.tar.gz";
}) {
  overlays = [
    (import (builtins.fetchTarball {
      # Bumped the channel in rust-toolchain.toml? Bump this commit too!
      url = "https://github.com/oxalica/rust-overlay/archive/7f0e3ef7b7fbed78e12e5100851175d28af4b7c6.tar.gz";
    }))
  ];
  config = {
    android_sdk.accept_license = buildAndroid;
    allowUnfree = buildAndroid;
  };
};
let
    rustToolchain = rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
    rustPlatform = makeRustPlatform {
      cargo = rustToolchain;
      rustc = rustToolchain;
    };
    pkgs_gnumake_4_3 = import (builtins.fetchTarball {
      url = "https://github.com/NixOS/nixpkgs/archive/6adf48f53d819a7b6e15672817fa1e78e5f4e84f.tar.gz";
    }) {};

    # We need clangStdenv with:
    # - clang < 16 (#30587)
    # - clang < 15 (#31059)
    # - glibc 2.38 (#31054)
    llvmPackages = llvmPackages_14;
    stdenv = llvmPackages.stdenv;

    buildToolsVersion = "33.0.2";
    androidComposition = androidenv.composeAndroidPackages {
      buildToolsVersions = [ buildToolsVersion ];
      includeEmulator = true;
      platformVersions = [ "33" ];
      includeSources = false;
      includeSystemImages = true;
      systemImageTypes = [ "google_apis" ];
      abiVersions = [ "x86" "armeabi-v7a" ];
      includeNDK = true;
      ndkVersion = "25.2.9519653";
      useGoogleAPIs = false;
      useGoogleTVAddOns = false;
      includeExtras = [
        "extras;google;gcm"
      ];
  };
  androidSdk = androidComposition.androidsdk;
  # Required by ./mach build --android
  androidEnvironment = lib.optionalAttrs buildAndroid rec {
    ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
    ANDROID_NDK_ROOT = "${ANDROID_SDK_ROOT}/ndk-bundle";
    GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${ANDROID_SDK_ROOT}/build-tools/${buildToolsVersion}/aapt2";
  };
in
stdenv.mkDerivation (androidEnvironment // {
  name = "servo-env";

  buildInputs = [
    # Native dependencies
    fontconfig freetype libunwind
    xorg.libxcb
    xorg.libX11

    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
    gst_all_1.gst-plugins-ugly

    rustup
    taplo
    cargo-deny
    llvmPackages.bintools # provides lld

    udev # Needed by libudev-sys for GamePad API.

    # Build utilities
    cmake dbus gcc git pkg-config which llvm perl yasm m4
    (python3.withPackages (ps: with ps; [virtualenv pip dbus]))

    # This pins gnumake to 4.3 since 4.4 breaks jobserver
    # functionality in mozjs and causes builds to be extremely
    # slow as it behaves as if -j1 was passed.
    # See https://github.com/servo/mozjs/issues/375
    pkgs_gnumake_4_3.gnumake

    # crown needs to be in our Cargo workspace so we can test it with `mach test`. This means its
    # dependency tree is listed in the main Cargo.lock, making it awkward to build with Nix because
    # all of Servo’s dependencies get pulled into the Nix store too, wasting over 1GB of disk space.
    # Filtering the lockfile to only the parts needed by crown saves space and builds faster.
    (let

      # Build and run filterlock over the main Cargo.lock.
      filteredLockFile = (clangStdenv.mkDerivation {
        name = "lock";
        buildInputs = [ rustToolchain ];
        nativeBuildInputs = [ rustPlatform.cargoSetupHook ];
        src = ../support/filterlock;
        cargoDeps = rustPlatform.importCargoLock {
          lockFile = ../support/filterlock/Cargo.lock;
        };
        buildPhase = ''
          > $out cargo run --offline -- ${../Cargo.lock} crown
        '';
        dontInstall = true;
      });
    in (rustPlatform.buildRustPackage {
      name = "crown";
      src = ../support/crown;
      doCheck = false;
      cargoLock = {
        lockFileContents = builtins.readFile filteredLockFile;

        # Needed when not filtering (filteredLockFile = ../Cargo.lock), else we’ll get errors like
        # “error: No hash was found while vendoring the git dependency blurmac-0.1.0.”
        # allowBuiltinFetchGit = true;
      };

      # Copy the filtered lockfile, making it writable by cargo --offline.
      postPatch = ''
        install -m 644 ${filteredLockFile} Cargo.lock
      '';

      # Reformat the filtered lockfile, so that cargo --frozen won’t complain
      # about the lockfile being dirty.
      # TODO maybe this can be avoided by using toml_edit in filterlock?
      preConfigure = ''
        cargo update --offline
      '';

      RUSTC_BOOTSTRAP = "crown";
    }))
  ] ++ (lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.AppKit
  ]) ++ (lib.optionals buildAndroid [
    # for android builds
    openjdk17_headless
    androidSdk
  ]);

  LIBCLANG_PATH = lib.makeLibraryPath [ llvmPackages.clang-unwrapped.lib ];

  # Allow cargo to download crates
  SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";

  # Enable colored cargo and rustc output
  TERMINFO = "${ncurses.out}/share/terminfo";


  # Provide libraries that aren’t linked against but somehow required
  LD_LIBRARY_PATH = lib.makeLibraryPath [
    # Fixes missing library errors
    zlib xorg.libXcursor xorg.libXrandr xorg.libXi libxkbcommon

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

      # Don’t pollute ~/.rustup with toolchains installed by nixpkgs rustup, because they
      # get patched in a way that makes them dependent on the Nix store.
      repo_root=$(git rev-parse --show-toplevel)
      export RUSTUP_HOME=$repo_root/.rustup
    fi
  '';
})
