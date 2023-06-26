# WebUSB Testing

WebUSB testing relies on the [WebUSB Testing API] which must be
provided by browsers under test.

In this test suite `resources/usb-helpers.js` detects and triggers
the API to be loaded as needed.

The Chromium implementation is provided by
`../resources/chromium/webusb-test.js` using [MojoJS].

Tests with the "-manual" suffix do not use the test-only interface and expect a
real hardware device to be connected. The specific characteristics of the device
are described in each test.

[MojoJS]: https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/testing/web_platform_tests.md#mojojs
[WebUSB Testing API]: https://wicg.github.io/webusb/test/
