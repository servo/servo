# Android WebView

To run WPT on WebView on an Android device, some additional set-up is required.

Currently, Android WebView support is experimental.

* Please check [Chrome for Android](chrome_android) for the common
  instructions for Android support first.

* Install an up-to-date version of system webview shell:
  * Go to
  [chromium-browser-snapshots](https://commondatastorage.googleapis.com/chromium-browser-snapshots/index.html?prefix=Android/)
  * Find the subdirectory with the highest number and click it.
  * Download `chrome-android.zip` file and unzip it.
  * Install `SystemWebViewShell.apk`.
  * On emulator, system webview shell may already be installed by default. Then
    you may need to remove the existing apk:
     * Choose a userdebug build.
     * Run an emulator with
       [writable system partition from command line](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/android_emulator.md/)

* If you have an issue with ChromeDriver version, try removing
  `_venv/bin/chromedriver` such that wpt runner can install a matching version
  automatically. Failing that, please check your environment path and make
  sure that no other ChromeDriver is used.

Example command line:

```bash
./wpt run --test-type=testharness android_webview <TESTS>
```

* Note that there is no support for channel or automatic installation. The test
  will be run against the current WebView version installed on the device.

* Reftests are not supported at the moment.
