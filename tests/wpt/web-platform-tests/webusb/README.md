# WebUSB Testing

WebUSB testing relies on the [WebUSB Testing API] which must be
provided by browsers under test.

In this test suite `resources/usb-helpers.js` detects and triggers
the API to be loaded as needed.

The Chromium implementation is provided by
`../resources/chromium/webusb-test.js`.

[WebUSB Testing API]: https://wicg.github.io/webusb/test/
