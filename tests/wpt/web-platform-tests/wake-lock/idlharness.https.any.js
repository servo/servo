// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/wake-lock/

'use strict';

idl_test(
  ['wake-lock'],
  ['dom', 'html', 'permissions'],
  async idl_array => {
    if (self.GLOBAL.isWorker()) {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    } else {
      idl_array.add_objects({ Navigator: ['navigator'] });
    }
    idl_array.add_objects({
      WakeLock: ['navigator.wakeLock'],
      WakeLockSentinel: ['sentinel'],
    });

    // For now, this assumes the request will be granted and the promise will
    // be fulfilled with a WakeLockSentinel object.
    self.sentinel = await navigator.wakeLock.request('screen');
    self.sentinel.release();
  }
);
