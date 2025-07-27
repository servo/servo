# Safari

To run Safari on macOS, some manual setup is required.

To enable Remote Automation, run either:

  * `safaridriver --enable`, for Safari, or
  * `"/Applications/Safari Technology Preview.app/Contents/MacOS/safaridriver" --enable`,
    for Safari Technology Preview.

You must also ensure you have
[configured the `hosts` file](from-local-system.html#hosts-file-setup).

Now, run the tests using the `safari` product:
```
./wpt run safari [test_list]
```

This will default to `--channel=preview` and run Safari Technology Preview.
To run the system Safari instead, use the `--channel=stable` argument:
```
./wpt run --channel=stable safari [test_list]
```

## Debugging

To debug problems with `safaridriver`, add the `--webdriver-arg=--diagnose`
option:
```
./wpt run --channel=preview --webdriver-arg=--diagnose safari [test_list]
```

The logs will be in `~/Library/Logs/com.apple.WebDriver/`.
See `man 1 safaridriver` for more information.
