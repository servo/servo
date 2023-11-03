# This provides a shell with all the necesarry packages required to run mach and build servo
# NOTE: This does not work offline or for nix-build

with (import <nixpkgs> { config = { android_sdk.accept_license = true;  allowUnfree = true; }; });
let
    pinnedSha = "6adf48f53d819a7b6e15672817fa1e78e5f4e84f";
    pinnedNixpkgs = import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/${pinnedSha}.tar.gz";
    }) {};
    androidComposition = androidenv.composeAndroidPackages {
      includeEmulator = true;
      platformVersions = [ "30" ];
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
  # servoEmulator = androidenv.emulateApp {
  #    name = "servo-emulator";
  #    platformVersion = "30";
  #    abiVersion = "x86"; # armeabi-v7a, mips, x86_64
  #    systemImageType = "google_apis";
  # };
  androidSdk = androidComposition.androidsdk;
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

    # android build
    openjdk8_headless

    # android, make this conditional?
    androidSdk
  ] ++ (lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.AppKit
  ]);

  #LIBCLANG_PATH = llvmPackages.clang-unwrapped.lib + "/lib/";


  RUST_FONTCONFIG_DLOPEN = "on"; # to avoid link failure on fontconfig
  ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
  ANDROID_HOME = "${androidSdk}/libexec/android-sdk";
  ANDROID_NDK_ROOT = "${ANDROID_SDK_ROOT}/ndk-bundle";
  APP_PLATFORM = "30"; # blurdroid
  ANDROID_SDK_PLATFORM = "30"; # blurdroid
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
  '';
}
