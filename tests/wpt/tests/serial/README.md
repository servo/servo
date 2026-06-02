# Web Serial Testing

Currently Web Serial only provide permission policy and maualy tests. The goal in future is to define test API specification similar to [WebUSB] and provide test-only interface to support more comprehensive tests.

Tests with the "-manual" suffix do not use the test-only interface and expect a
real hardware device to be connected. The specific characteristics of the device
are described in each test.

[WebUSB]: webusb
[Web Serial API]: https://wicg.github.io/serial
