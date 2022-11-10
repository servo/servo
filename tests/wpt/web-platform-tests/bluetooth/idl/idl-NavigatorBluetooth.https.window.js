// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';
const test_desc = '[SameObject] test for navigator.bluetooth';

test(() => {
  assert_true('bluetooth' in navigator, 'navigator.bluetooth exists.');
}, 'navigator.bluetooth IDL test');

test(() => {
  assert_equals(navigator.bluetooth, navigator.bluetooth);
}, test_desc);
