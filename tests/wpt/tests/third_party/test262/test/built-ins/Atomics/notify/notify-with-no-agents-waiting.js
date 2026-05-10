// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test that Atomics.notify notifies zero waiters if there are no agents waiting.
includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 1;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);
    // THIS WILL NEVER WAIT.
    $262.agent.leaving();
  });
`);

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2)
);

$262.agent.safeBroadcast(i32a);
$262.agent.waitUntil(i32a, RUNNING, 1);

// There are ZERO agents waiting to notify...
assert.sameValue(Atomics.notify(i32a, 0, 1), 0, 'Atomics.notify(i32a, 0, 1) returns 0');
