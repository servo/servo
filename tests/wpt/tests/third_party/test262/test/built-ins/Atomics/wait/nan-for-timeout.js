// Copyright (C) 2018 Amal Hussein.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  NaN timeout arg should result in an infinite timeout
info: |
  Atomics.wait( typedArray, index, value, timeout )

  4.Let q be ? ToNumber(timeout).
    ...
    Undefined    Return NaN.
  5.If q is NaN, let t be +∞, else let t be max(q, 0)

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 1;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    $262.agent.report(Atomics.wait(i32a, 0, 0, NaN));  // NaN => +Infinity
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

assert.sameValue(Atomics.notify(i32a, 0), 1, 'Atomics.notify(i32a, 0) returns 1');
assert.sameValue($262.agent.getReport(), "ok", '$262.agent.getReport() returns "ok"');
