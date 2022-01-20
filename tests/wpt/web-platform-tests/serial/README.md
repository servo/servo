# Web Serial Testing

Automated testing for the [Web Serial API] relies on a test-only interface which
must be provided by browsers under test. This is similar to [WebUSB] however
there is no separate specification of the API other than the tests themselves
and the Chromium implementation.

Tests in this suite include `resources/automation.js` to detect and load the
test API as needed.

The Chromium implementation is provided by
`../resources/chromium/fake-serial.js` using [MojoJS].

Tests with the "-manual" suffix do not use the test-only interface and expect a
real hardware device to be connected. The specific characteristics of the device
are described in each test.

[MojoJS]: https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/testing/web_platform_tests.md#mojojs
[WebUSB]: ../webusb
[Web Serial API]: https://wicg.github.io/serial
