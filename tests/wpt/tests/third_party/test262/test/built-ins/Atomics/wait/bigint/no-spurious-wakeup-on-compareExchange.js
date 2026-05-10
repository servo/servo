// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Waiter does not spuriously notify on index which is subject to compareExchange operation
includes: [atomicsHelper.js]
features: [Atomics, BigInt, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 1;
const TIMEOUT = $262.agent.timeouts.small;

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

Atomics.compareExchange(i64a, 0, 0n, 1n);

const lapse = $262.agent.getReport();
assert(
  lapse >= TIMEOUT,
  'The result of `(lapse >= TIMEOUT)` is true'
);
assert.sameValue(
  $262.agent.getReport(),
  'timed-out',
  '$262.agent.getReport() returns "timed-out"'
);
assert.sameValue(Atomics.notify(i64a, 0), 0, 'Atomics.notify(i64a, 0) returns 0');
