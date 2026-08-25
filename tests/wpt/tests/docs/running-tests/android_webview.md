# Android WebView

To run WPT on WebView on an Android device, some additional set-up is required.

Currently, Android WebView support is experimental.

## Prerequisites

Please check [Chrome for Android](chrome_android.md) for the common instructions for Android support first.

Ensure you have a userdebug or eng Android build installed on the device.

Install an up-to-date version of system webview shell:
1. Go to [chromium-browser-snapshots](https://commondatastorage.googleapis.com/chromium-browser-snapshots/index.html?prefix=Android/)
2. Find the subdirectory with the highest number and click it, this number can be found
   in the "Commit Position" column of row "LAST_CHANGE" (at bottom of page).
3. Download `chrome-android.zip` file and unzip it.
4. Install `SystemWebViewShell.apk`.
5. On emulator, system webview shell may already be installed by default. Then you may need to remove the existing apk:
   * Choose a userdebug build.
   * Run an emulator with
     [writable system partition from command line](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/android_emulator.md/)

If you have an issue with ChromeDriver version mismatch, try one of the following:
  * Try removing `_venv/bin/chromedriver` such that wpt runner can install a matching version
  automatically. Failing that, please check your environment path and make
  sure that no other ChromeDriver is used.
  * Download the [ChromeDriver binary](https://chromedriver.chromium.org/) matching your WebView's major version and specify it on the command line
    ```
    ./wpt run --webdriver-binary <binary path> ...
    ```

Configure host remap rules in the [webview commandline file](https://cs.chromium.org/chromium/src/android_webview/docs/commandline-flags.md?l=57):
```
adb shell "echo '_ --host-resolver-rules=\"MAP nonexistent.*.test ^NOTFOUND, MAP *.test 127.0.0.1\"' > /data/local/tmp/webview-command-line"
```

Ensure that `adb` can be found on your system's PATH.

## Running Tests

Example command line:

```bash
./wpt run --test-type=testharness android_webview <TESTS>
```

* Note that there is no support for channel or automatic installation. The test
  will be run against the current WebView version installed on the device.

* Reftests are not supported at the moment.
