# Web Bluetooth Testing

Web Bluetooth testing relies on the [Web Bluetooth Testing API] which must be
provided by browsers under test.

In this test suite `resources/bluetooth-helpers.js` detects and triggers
the API to be loaded as needed.

The Chromium implementation is provided by
`../resources/chromium/web-bluetooth-test.js`.

[Web Bluetooth Testing API]: https://docs.google.com/document/d/1Nhv_oVDCodd1pEH_jj9k8gF4rPGb_84VYaZ9IG8M_WY/
