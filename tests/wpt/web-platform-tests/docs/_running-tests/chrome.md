---
layout: page
title: Chrome
---
When running Chrome, there are some additional useful command line arguments.

As with most products, you can use a different binary with `--binary`, e.g., to
run Chrome Dev on Linux:

```
./wpt run --binary `which google-chrome-unstable` chrome
```

Extra arguments to Chrome can be passed with `--binary-args`.

To enable all [experimental web platform features](https://www.chromium.org/blink/runtime-enabled-features) (chrome://flags/#enable-experimental-web-platform-features):

```
./wpt run --binary-arg=--enable-experimental-web-platform-features chrome fullscreen/
```

To enable a specific [runtime enabled feature](http://dev.chromium.org/blink/runtime-enabled-features):

```
./wpt run --binary-arg=--enable-blink-features=AsyncClipboard chrome clipboard-apis/
```

To bypass device selection and use mock media for tests using `getUserMedia`:

```
./wpt run --binary-arg=--use-fake-ui-for-media-stream --binary-arg=--use-fake-device-for-media-stream chrome mediacapture-streams/
```

Note: there's an [open issue for doing this using WebDriver](https://github.com/w3c/web-platform-tests/issues/7424).

Some of the above are most useful in combination, e.g., to run all tests in
Chrome Dev with experimental web platform features enabled:

```
./wpt run --binary `which google-chrome-unstable` --binary-arg=--enable-experimental-web-platform-features chrome
```
