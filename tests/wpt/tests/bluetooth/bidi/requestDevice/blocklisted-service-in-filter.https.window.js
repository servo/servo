// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'Reject with SecurityError if requesting a blocklisted ' +
    'service.';
const expected = new DOMException(
    'requestDevice() called with a filter containing a blocklisted UUID ' +
        'or manufacturer data. https://goo.gl/4NeimX',
    'SecurityError');

bluetooth_bidi_test(async () => {
  await assert_promise_rejects_with_message(
      setUpPreconnectedFakeDevice({
        fakeDeviceOptions: {knownServiceUUIDs: ['human_interface_device']},
        requestDeviceOptions:
            {filters: [{services: ['human_interface_device']}]}
      }),
      expected, 'Requesting blocklisted service rejects.');
}, test_desc);
