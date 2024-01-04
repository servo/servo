# This provides a shell with all the necesarry packages required to run mach and build servo
# NOTE: This does not work offline or for nix-build

with import <nixpkgs> {
  overlays = [
    (import (builtins.fetchTarball {
      # Bumped the channel in rust-toolchain.toml? Bump this commit too!
      url = "https://github.com/oxalica/rust-overlay/archive/a0df72e106322b67e9c6e591fe870380bd0da0d5.tar.gz";
    }))
  ];
};
let
    pinnedNixpkgs = import (builtins.fetchTarball {
      url = "https://github.com/NixOS/nixpkgs/archive/6adf48f53d819a7b6e15672817fa1e78e5f4e84f.tar.gz";
    }) {};
    rustToolchain = rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
    rustPlatform = makeRustPlatform {
      cargo = rustToolchain;
      rustc = rustToolchain;
    };
in
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

    rustup
    taplo
    llvmPackages.bintools # provides lld

    # Build utilities
    cmake dbus gcc git pkg-config which llvm perl yasm m4
    (python3.withPackages (ps: with ps; [virtualenv pip dbus]))
    # This pins gnumake to 4.3 since 4.4 breaks jobserver
    # functionality in mozjs and causes builds to be extremely
    # slow as it behaves as if -j1 was passed.
    # See https://github.com/servo/mozjs/issues/375
    pinnedNixpkgs.gnumake

    # crown needs to be in our Cargo workspace so we can test it with `mach test`. This means its
    # dependency tree is listed in the main Cargo.lock, making it awkward to build with Nix because
    # all of Servo’s dependencies get pulled into the Nix store too, wasting over 1GB of disk space.
    # Filtering the lockfile to only the parts needed by crown saves space and builds faster.
    (let
      vendorTarball = rustPlatform.fetchCargoTarball {
        src = ../support/filterlock;
        hash = "sha256-/kJNDtmv2uI7Qlmpi3DMWSw88rzEJSbroO0/QrgQrSc=";
      };
      vendorConfig = builtins.toFile "toml" ''
        [source.crates-io]
        replace-with = "vendor"
        [source.vendor]
        directory = "vendor"
      '';

      # Build and run filterlock over the main Cargo.lock.
      filteredLockFile = (clangStdenv.mkDerivation {
        name = "lock";
        buildInputs = [ rustToolchain ];
        src = ../support/filterlock;
        buildPhase = ''
          tar xzf ${vendorTarball}
          mv cargo-deps-vendor.tar.gz vendor
          mkdir .cargo
          cp -- ${vendorConfig} .cargo/config.toml
          > $out cargo run --offline -- ${../Cargo.lock} crown
        '';
      });
    in (rustPlatform.buildRustPackage rec {
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
  '';
}
