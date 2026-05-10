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
features: [Atomics, BigInt, SharedArrayBuffer, TypedArray]
---*/

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)
);

const RUNNING = 1;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    $262.agent.report(Atomics.wait(i64a, 0, 0n, NaN));  // NaN => +Infinity
    $262.agent.leaving();
  });
`);

$262.agent.safeBroadcast(i64a);
$262.agent.waitUntil(i64a, RUNNING, 1n);

// Try to yield control to ensure the agent actually started to wait.
$262.agent.tryYield();

assert.sameValue(Atomics.notify(i64a, 0), 1, 'Atomics.notify(i64a, 0) returns 1');
assert.sameValue($262.agent.getReport(), 'ok', '$262.agent.getReport() returns "ok"');
