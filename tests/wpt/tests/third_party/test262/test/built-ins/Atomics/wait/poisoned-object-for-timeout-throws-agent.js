// Copyright (C) 2018 Amal Hussein.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  False timeout arg should result in an +0 timeout
info: |
  Atomics.wait( typedArray, index, value, timeout )

  4. Let q be ? ToNumber(timeout).

    Null -> Return +0.

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 1;

$262.agent.start(`
  const poisonedValueOf = {
    valueOf: function() {
      throw new Error("should not evaluate this code");
    }
  };

  const poisonedToPrimitive = {
    [Symbol.toPrimitive]: function() {
      throw new Error("passing a poisoned object using @@ToPrimitive");
    }
  };

  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    let status1 = "";
    let status2 = "";

    try {
      Atomics.wait(i32a, 0, 0, poisonedValueOf);
    } catch (error) {
      status1 = "poisonedValueOf";
    }
    try {
      Atomics.wait(i32a, 0, 0, poisonedToPrimitive);
    } catch (error) {
      status2 = "poisonedToPrimitive";
    }

    $262.agent.report(status1);
    $262.agent.report(status2);
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
  'poisonedValueOf',
  '$262.agent.getReport() returns "poisonedValueOf"'
);
assert.sameValue(
  $262.agent.getReport(),
  'poisonedToPrimitive',
  '$262.agent.getReport() returns "poisonedToPrimitive"'
);

assert.sameValue(Atomics.notify(i32a, 0), 0, 'Atomics.notify(i32a, 0) returns 0');
