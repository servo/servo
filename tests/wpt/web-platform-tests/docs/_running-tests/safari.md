---
layout: page
title: Safari
---
To run Safari on macOS, some manual setup is required:

  * Allow Safari to be controlled by SafariDriver: `safaridriver --enable`

  * Allow pop-up windows:
    `defaults write com.apple.Safari WebKitJavaScriptCanOpenWindowsAutomatically 1`

  * Trust the certificate:
    `security add-trusted-cert -k "$(security default-keychain | cut -d\" -f2)" tools/certs/cacert.pem`

  * Set `OBJC_DISABLE_INITIALIZE_FORK_SAFETY=YES` in your environment. This is a
    workaround for a known
    [macOS High Sierra issue](https://github.com/web-platform-tests/wpt/issues/9007).

Now, run the tests using the `safari` product:
```
./wpt run safari [test_list]
```

This will use the `safaridriver` found on the path, which will be stable Safari.
To run Safari Technology Preview instead, use the `--webdriver-binary` argument:
```
./wpt run --webdriver-binary "/Applications/Safari Technology Preview.app/Contents/MacOS/safaridriver" safari [test_list]
```
