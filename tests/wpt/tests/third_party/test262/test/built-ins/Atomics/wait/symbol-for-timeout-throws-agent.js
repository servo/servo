// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Throws a TypeError if index arg can not be converted to an Integer
info: |
  Atomics.wait( typedArray, index, value, timeout )

  4. Let q be ? ToNumber(timeout).

    Symbol --> Throw a TypeError exception.

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, Symbol, Symbol.toPrimitive, TypedArray]
---*/

const RUNNING = 1;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    let status1 = "";
    let status2 = "";

    try {
      Atomics.wait(i32a, 0, 0, Symbol("1"));
    } catch (error) {
      status1 = 'Symbol("1")';
    }
    try {
      Atomics.wait(i32a, 0, 0, Symbol("2"));
    } catch (error) {
      status2 = 'Symbol("2")';
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
  'Symbol("1")',
  '$262.agent.getReport() returns "Symbol("1")"'
);
assert.sameValue(
  $262.agent.getReport(),
  'Symbol("2")',
  '$262.agent.getReport() returns "Symbol("2")"'
);

assert.sameValue(Atomics.notify(i32a, 0), 0, 'Atomics.notify(i32a, 0) returns 0');
