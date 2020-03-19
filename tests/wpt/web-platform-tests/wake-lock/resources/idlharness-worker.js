'use strict';

// https://w3c.github.io/wake-lock/

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

idl_test(
  ['wake-lock'],
  ['dom', 'html', 'permissions'],
  async idl_array => {
    idl_array.add_objects({ WorkerNavigator: ['navigator'] });

    idl_array.add_objects({
      WakeLock: ['navigator.wakeLock'],
      WakeLockSentinel: ['sentinel'],
    });

    self.sentinel = await navigator.wakeLock.request('system');
    self.sentinel.release();
  }
);

done();
