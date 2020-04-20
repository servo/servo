// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

// https://w3c.github.io/screen-wake-lock/

'use strict';

idl_test(
  ['screen-wake-lock'],
  ['dom', 'html'],
  async idl_array => {
    idl_array.add_objects({ Navigator: ['navigator'] });

    idl_array.add_objects({
      WakeLock: ['navigator.wakeLock'],
      WakeLockSentinel: ['sentinel'],
    });

    await test_driver.set_permission(
        { name: 'screen-wake-lock' }, 'granted', false);
    self.sentinel = await navigator.wakeLock.request('screen');
    self.sentinel.release();
  }
);
