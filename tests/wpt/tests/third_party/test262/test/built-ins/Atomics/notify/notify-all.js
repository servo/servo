// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test that Atomics.notify notifies all waiters if that's what the count is.
includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const WAIT_INDEX = 0;             // Waiters on this will be woken
const RUNNING = 1;                // Accounting of live agents
const NUMAGENT = 3;
const BUFFER_SIZE = 4;

for (var i = 0; i < NUMAGENT; i++) {
  $262.agent.start(`
    $262.agent.receiveBroadcast(function(sab) {
      const i32a = new Int32Array(sab);
      Atomics.add(i32a, ${RUNNING}, 1);

      $262.agent.report("A " + Atomics.wait(i32a, ${WAIT_INDEX}, 0));
      $262.agent.leaving();
    });
  `);
}

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * BUFFER_SIZE)
);

$262.agent.safeBroadcast(i32a);

// Wait for agents to be running.
$262.agent.waitUntil(i32a, RUNNING, NUMAGENT);

// Try to yield control to ensure the agent actually started to wait. If we
// don't, we risk sending the notification before agents are sleeping, and we hang.
$262.agent.tryYield();

// Notify all waiting on WAIT_INDEX, should be 3 always, they won't time out.
assert.sameValue(
  Atomics.notify(i32a, WAIT_INDEX),
  NUMAGENT,
  'Atomics.notify(i32a, WAIT_INDEX) returns the value of `NUMAGENT`'
);

for (var i = 0; i < NUMAGENT; i++) {
  assert.sameValue($262.agent.getReport(), 'A ok', 'The value of reports[i] is "A ok"');
}
