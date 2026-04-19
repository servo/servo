// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test that Atomics.notify on awoken waiter is a noop.
includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 1;
const TIMEOUT = $262.agent.timeouts.long;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    $262.agent.report(Atomics.wait(i32a, 0, 0, ${TIMEOUT}));
    $262.agent.leaving();
  });
`);

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2)
);

$262.agent.safeBroadcast(i32a);

$262.agent.waitUntil(i32a, RUNNING, 1);

// Try to yield control to ensure the agent actually started to wait.
$262.agent.tryYield();

assert.sameValue(Atomics.notify(i32a, 0, 1), 1, 'Atomics.notify(i32a, 0, 1) returns 1');

assert.sameValue($262.agent.getReport(), 'ok', '$262.agent.getReport() returns "ok"');

// Already notified, this should be a noop
assert.sameValue(Atomics.notify(i32a, 0, 1), 0, 'Atomics.notify(i32a, 0, 1) returns 0');
