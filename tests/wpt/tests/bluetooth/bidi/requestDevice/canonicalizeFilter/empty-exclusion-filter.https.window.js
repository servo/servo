// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'An exclusion filter must restrict the devices in some way.';
const expected = new TypeError();

bluetooth_bidi_test(
    () => assert_promise_rejects_with_message(
        requestDeviceWithTrustedClick(
            {filters: [{name: 'Name'}], exclusionFilters: [{}]}),
        expected),
    test_desc);
