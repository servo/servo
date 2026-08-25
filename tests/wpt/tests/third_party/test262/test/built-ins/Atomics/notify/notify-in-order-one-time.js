// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test that Atomics.notify notifies agents in the order they are waiting.
includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const NUMAGENT = 3;
const WAIT_INDEX = 0;             // Waiters on this will be woken
const SPIN = 1;                   // Worker i (zero-based) spins on location SPIN+i
const RUNNING = SPIN + NUMAGENT;  // Accounting of live agents
const BUFFER_SIZE = RUNNING + 1;

// Create workers and start them all spinning.  We set atomic slots to make
// them go into a wait, thus controlling the waiting order.  Then we notify them
// one by one and observe the notification order.

for (var i = 0; i < NUMAGENT; i++) {
  $262.agent.start(`
    $262.agent.receiveBroadcast(function(sab) {
      const i32a = new Int32Array(sab);
      Atomics.add(i32a, ${RUNNING}, 1);

      while (Atomics.load(i32a, ${SPIN + i}) === 0) {
        /* nothing */
      }

      $262.agent.report(${i});
      Atomics.wait(i32a, ${WAIT_INDEX}, 0);
      $262.agent.report(${i});

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

var waiterlist = [];
for (var i = 0; i < NUMAGENT; i++) {
  assert.sameValue(
    Atomics.store(i32a, SPIN + i, 1),
    1,
    `Atomics.store(i32a, SPIN + ${i}, 1) returns 1`
  );

  waiterlist.push($262.agent.getReport());

  // Try to yield control to ensure the agent actually started to wait.
  $262.agent.tryYield();
}

var notified = [];
for (var i = 0; i < NUMAGENT; i++) {
  assert.sameValue(
    Atomics.notify(i32a, WAIT_INDEX, 1),
    1,
    `Atomics.notify(i32a, WAIT_INDEX, 1) returns 1 (${i})`
  );

  notified.push($262.agent.getReport());
}

assert.sameValue(
  notified.join(''),
  waiterlist.join(''),
  'notified.join(\'\') returns waiterlist.join(\'\')'
);
