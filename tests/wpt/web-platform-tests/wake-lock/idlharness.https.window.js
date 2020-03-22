// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

// https://w3c.github.io/wake-lock/

'use strict';

idl_test(
  ['wake-lock'],
  ['dom', 'html', 'permissions'],
  async idl_array => {
    idl_array.add_objects({ Navigator: ['navigator'] });

    idl_array.add_objects({
      WakeLock: ['navigator.wakeLock'],
      WakeLockSentinel: ['sentinel'],
    });

    await test_driver.set_permission(
        { name: 'wake-lock', type: 'screen' }, 'granted', false);
    self.sentinel = await navigator.wakeLock.request('screen');
    self.sentinel.release();
  }
);
