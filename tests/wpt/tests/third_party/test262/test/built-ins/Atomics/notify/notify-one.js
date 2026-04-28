// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test that Atomics.notify notifies one waiter if that's what the count is.
includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const WAIT_INDEX = 0;             // Agents wait here
const RUNNING = 1;                // Accounting of live agents here
const NOTIFYCOUNT = 1;
const NUMAGENT = 3;
const BUFFER_SIZE = 4;

const TIMEOUT = $262.agent.timeouts.long;

for (var i = 0; i < NUMAGENT; i++ ) {
  $262.agent.start(`
    $262.agent.receiveBroadcast(function(sab) {
      const i32a = new Int32Array(sab);
      Atomics.add(i32a, ${RUNNING}, 1);

      // Waiters that are not woken will time out eventually.
      $262.agent.report(Atomics.wait(i32a, ${WAIT_INDEX}, 0, ${TIMEOUT}));
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

// Try to yield control to ensure the agent actually started to wait.
$262.agent.tryYield();

// There's a slight risk we'll fail to notify the desired count, if the preceding
// tryYield() took much longer than anticipated and workers have started timing
// out.
assert.sameValue(
  Atomics.notify(i32a, 0, NOTIFYCOUNT),
  NOTIFYCOUNT,
  'Atomics.notify(i32a, 0, NOTIFYCOUNT) returns the value of `NOTIFYCOUNT`'
);

// Try to sleep past the timeout.
$262.agent.trySleep(TIMEOUT);

// Collect and check results
const reports = [];
for (var i = 0; i < NUMAGENT; i++) {
  reports.push($262.agent.getReport());
}
reports.sort();

for (var i = 0; i < NOTIFYCOUNT; i++) {
  assert.sameValue(reports[i], 'ok', 'The value of reports[i] is "ok"');
}
for (var i = NOTIFYCOUNT; i < NUMAGENT; i++) {
  assert.sameValue(reports[i], 'timed-out', 'The value of reports[i] is "timed-out"');
}
