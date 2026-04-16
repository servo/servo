{ pkgs, lib, gnumakeSource ? null }:
let
  gnumakePkgs =
    if gnumakeSource != null
      then gnumakeSource
      else import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/6adf48f53d819a7b6e15672817fa1e78e5f4e84f.tar.gz";
      }) {};

  rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };

  crown = rustPlatform.buildRustPackage {
    name = "crown";

    src = ../support/crown;
    doCheck = false;
    cargoLock = {
      lockFile = ../support/crown/Cargo.lock;
    };

    RUSTC_BOOTSTRAP = "crown";
  };

  llvmPackages = pkgs.llvmPackages_20;
  stdenv = llvmPackages.stdenv;
in {

  envVars = {
    LIBCLANG_PATH = lib.makeLibraryPath [
      llvmPackages.clang-unwrapped.lib
    ];
    SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
    TERMINFO = "${pkgs.ncurses.out}/share/terminfo";
  };

  buildInputs = with pkgs; (
    [ # Native dependencies
      fontconfig
      freetype
      libunwind
      xorg.libxcb
      xorg.libX11
    ] ++

    [ # GST
      gst_all_1.gstreamer
      gst_all_1.gst-plugins-base
      gst_all_1.gst-plugins-good
      gst_all_1.gst-plugins-bad
      gst_all_1.gst-plugins-ugly
    ] ++

    [ # Rust
      llvmPackages.bintools
      crown
      rustToolchain

      taplo
      cargo-deny
      cargo-nextest
    ] ++

    [ # Build utilities
      cmake
      dbus
      gcc
      git
      pkg-config
      which
      llvm
      perl
      yasm
      m4
    ] ++

    [ # Python
      python311
      uv
      gnumakePkgs.gnumake
    ]
  ) ++ lib.optionals stdenv.isLinux [ # Linux specific
    udev
    wayland
    vulkan-loader
  ] ++ lib.optionals stdenv.isDarwin [ # Darwin specific
    # Frameworks provided automatically by  Darwin stdenv
  ];

  runtimeLibs = with pkgs; [
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libxkbcommon
    libGL
  ] ++ lib.optionals stdenv.isLinux [ # Linux specific
    wayland
    vulkan-loader
  ];

}
