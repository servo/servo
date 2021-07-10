# Chrome

When running Chrome, there are some useful command line arguments.

You can inform `wpt` of the release channel of Chrome using `--channel`.
`wpt` is able to find the correct binary in the following cases:
* On Linux for stable, beta and dev channels if
  `google-chrome-{stable,beta,unstable}` are in `PATH`;
* On Mac for stable and canary channels if the official DMGs are installed.

In other cases, you will need to specify the path to the Chrome binary with
`--binary`. For example:

```bash
./wpt run --channel dev --binary /path/to/non-default/google-chrome chrome
```

Note: when the channel is "dev", `wpt` will *automatically* enable all
[experimental web platform features][1]
(chrome://flags/#enable-experimental-web-platform-features) by passing
`--enable-experimental-web-platform-features` to Chrome.

If you want to enable a specific [runtime enabled feature][1], use
`--binary-arg` to specify the flag(s) that you want to pass to Chrome:

```bash
./wpt run --binary-arg=--enable-blink-features=AsyncClipboard chrome clipboard-apis/
```

[1]: https://www.chromium.org/blink/runtime-enabled-features
