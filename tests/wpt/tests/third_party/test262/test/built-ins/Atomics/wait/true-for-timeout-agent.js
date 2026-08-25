// Copyright (C) 2018 Amal Hussein.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  True timeout arg should result in an +0 timeout
info: |
  Atomics.wait( typedArray, index, value, timeout )

  4. Let q be ? ToNumber(timeout).

    Boolean -> If argument is true, return 1. If argument is false, return +0.

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 1;

$262.agent.start(`
  const valueOf = {
    valueOf: function() {
      return true;
    }
  };

  const toPrimitive = {
    [Symbol.toPrimitive]: function() {
      return true;
    }
  };

  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    const status1 = Atomics.wait(i32a, 0, 0, true);
    const status2 = Atomics.wait(i32a, 0, 0, valueOf);
    const status3 = Atomics.wait(i32a, 0, 0, toPrimitive);

    $262.agent.report(status1);
    $262.agent.report(status2);
    $262.agent.report(status3);
    $262.agent.leaving();
  });
`);

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

$262.agent.safeBroadcast(i32a);
$262.agent.waitUntil(i32a, RUNNING, 1);

// Try to yield control to ensure the agent actually started to wait.
$262.agent.tryYield();

assert.sameValue(
  $262.agent.getReport(),
  'timed-out',
  '$262.agent.getReport() returns "timed-out"'
);
assert.sameValue(
  $262.agent.getReport(),
  'timed-out',
  '$262.agent.getReport() returns "timed-out"'
);
assert.sameValue(
  $262.agent.getReport(),
  'timed-out',
  '$262.agent.getReport() returns "timed-out"'
);

assert.sameValue(Atomics.notify(i32a, 0), 0, 'Atomics.notify(i32a, 0) returns 0');
