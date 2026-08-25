// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.wait
description: >
  Test that Atomics.wait returns the right result when it was awoken before
  a timeout
info: |
  Atomics.wait( typedArray, index, value, timeout )

  2.Let i be ? ValidateAtomicAccess(typedArray, index).
    ...
      2.Let accessIndex be ? ToIndex(requestIndex).

      9.If IsSharedArrayBuffer(buffer) is false, throw a TypeError exception.
        ...
          3.If bufferData is a Data Block, return false

          If value is undefined, then
          Let index be 0.
includes: [atomicsHelper.js]
features: [Atomics, BigInt, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 1;
const TIMEOUT = $262.agent.timeouts.huge;

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)
);

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    const before = $262.agent.monotonicNow();
    const unpark = Atomics.wait(i64a, 0, 0n, ${TIMEOUT});
    const duration = $262.agent.monotonicNow() - before;

    $262.agent.report(duration);
    $262.agent.report(unpark);
    $262.agent.leaving();
  });
`);

$262.agent.safeBroadcast(i64a);
$262.agent.waitUntil(i64a, RUNNING, 1n);

// Try to yield control to ensure the agent actually started to wait.
$262.agent.tryYield();

assert.sameValue(Atomics.notify(i64a, 0), 1, 'Atomics.notify(i64a, 0) returns 1');

const lapse = $262.agent.getReport();

assert(
  lapse < TIMEOUT,
  'The result of `(lapse < TIMEOUT)` is true'
);
assert.sameValue($262.agent.getReport(), 'ok', '$262.agent.getReport() returns "ok"');
