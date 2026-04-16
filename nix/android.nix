{ androidPkgs, lib }:
let
  buildToolsVersion = "34.0.0";

  androidComposition = androidPkgs.androidenv.composeAndroidPackages {
    buildToolsVersions = [
      buildToolsVersion
    ];
    includeEmulator = true;
    platformVersions = [
      "33"
    ];
    includeSources = false;
    includeSystemImages = true;
    systemImageTypes = [
      "google_apis"
    ];
    abiVersions = if androidPkgs.stdenv.hostPlatform.isAarch64
      then [
        "armeabi-v7a"
        "arm64-v8a"
      ]
      else [
        "armeabi-v7a"
        "x86_64"
        "x86"
      ];
    includeNDK = true;
    ndkVersion = "28.2.13676358";
    useGoogleAPIs = false;
    useGoogleTVAddOns = false;
    includeExtras = [
      "extras;google;gcm"
    ];
  };

  androidSdk = androidComposition.androidsdk;
  in {

    buildInputs = [
      androidPkgs.openjdk17_headless
      androidSdk
    ];

    envVars = {
      ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
      ANDROID_NDK_ROOT = "${androidSdk}/libexec/android-sdk/ndk-bundle";
      GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidSdk}/libexec/android-sdk/build-tools/${buildToolsVersion}/aapt2";
    };

  }
