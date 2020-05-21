# Safari

To run Safari on macOS, some manual setup is required. Some steps are different
for Safari and Safari Technology Preview.

  * Allow Safari to be controlled by SafariDriver:
    * `safaridriver --enable` or
    * `"/Applications/Safari Technology Preview.app/Contents/MacOS/safaridriver" --enable`

  * Allow pop-up windows:
    * `defaults write com.apple.Safari WebKitJavaScriptCanOpenWindowsAutomatically 1` or
    * `defaults write com.apple.SafariTechnologyPreview WebKitJavaScriptCanOpenWindowsAutomatically 1`

  * Turn on additional experimental features Safari Technology Preview:
    * `defaults write com.apple.SafariTechnologyPreview ExperimentalServerTimingEnabled 1`

  * Trust the certificate:
    * `security add-trusted-cert -k "$(security default-keychain | cut -d\" -f2)" tools/certs/cacert.pem`

  * Set `no_proxy='*'` in your environment. This is a
    workaround for a known
    [macOS High Sierra issue](https://github.com/web-platform-tests/wpt/issues/9007).

Now, run the tests using the `safari` product:
```
./wpt run safari [test_list]
```

This will use the `safaridriver` found on the path, which will be stable Safari.
To run Safari Technology Preview instead, use the `--channel=preview` argument:
```
./wpt run --channel=preview safari [test_list]
```

## Debugging

To debug problems with `safaridriver`, add the `--webdriver-arg=--diagnose`
argument:
```
./wpt run --channel=preview --webdriver-arg=--diagnose safari [test_list]
```

The logs will be in `~/Library/Logs/com.apple.WebDriver/`.
See `man safaridriver` for more information.

To enable safaridriver diagnostics in Azure Pipelines, set
`safaridriver_diagnose` to `true` in `.azure-pipelines.yml`.
