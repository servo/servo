// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'An exclusion filter must restrict the devices in some way.';
const expected = new TypeError();

bluetooth_test(
    () => assert_promise_rejects_with_message(
        requestDeviceWithTrustedClick(
            {filters: [{name: 'Name'}], exclusionFilters: [{}]}),
        expected),
    test_desc);
