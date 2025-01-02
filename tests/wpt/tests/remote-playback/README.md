# Remote Playback API specification Tests

The Remote Playback API can be found here:

GitHub repository: https://github.com/w3c/remote-playback/

File an issue: https://github.com/w3c/remote-playback/issues/new

## Hardware/network dependency

The Remote Playback API requires communication with a device over the network.
Tests that end in `-manual.html` require a compatible device available on the
local area network to run the tests; these tests must be run manually.

Known browser/device combinations that can be used to run manual tests:

| Browser             | Device |
| -------             | ------ |
| Chrome for Android  | [Chromecast](https://store.google.com/product/chromecast_google_tv?pli=1&hl=en-US) |
| Safari              | Apple TV |
