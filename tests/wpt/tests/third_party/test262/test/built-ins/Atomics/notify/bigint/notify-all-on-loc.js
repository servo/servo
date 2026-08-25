// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test that Atomics.notify notifies all waiters on a location, but does not
  notify waiters on other locations.
includes: [atomicsHelper.js]
features: [Atomics, BigInt, SharedArrayBuffer, TypedArray]
---*/

const WAIT_INDEX = 0;             // Waiters on this will be woken
const WAIT_FAKE = 1;              // Waiters on this will not be woken
const RUNNING = 2;                // Accounting of live agents
const NOTIFY_INDEX = 3;             // Accounting for too early timeouts
const NUMAGENT = 3;
const TIMEOUT_AGENT_MESSAGES = 2; // Number of messages for the timeout agent
const BUFFER_SIZE = 4;

// Long timeout to ensure the agent doesn't timeout before the main agent calls
// `Atomics.notify`.
const TIMEOUT = $262.agent.timeouts.long;

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * BUFFER_SIZE)
);

for (var i = 0; i < NUMAGENT; i++) {
  $262.agent.start(`
    $262.agent.receiveBroadcast(function(sab) {
      const i64a = new BigInt64Array(sab);
      Atomics.add(i64a, ${RUNNING}, 1n);

      $262.agent.report("A " + Atomics.wait(i64a, ${WAIT_INDEX}, 0n));
      $262.agent.leaving();
    });
  `);
}

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    // This will always time out.
    $262.agent.report("B " + Atomics.wait(i64a, ${WAIT_FAKE}, 0n, ${TIMEOUT}));

    // If this value is not 1n, then the agent timeout before the main agent
    // called Atomics.notify.
    const result = Atomics.load(i64a, ${NOTIFY_INDEX}) === 1n
                   ? "timeout after Atomics.notify"
                   : "timeout before Atomics.notify";
    $262.agent.report("W " + result);

    $262.agent.leaving();
  });
`);

$262.agent.safeBroadcast(i64a);

// Wait for agents to be running.
$262.agent.waitUntil(i64a, RUNNING, BigInt(NUMAGENT + 1));

// Try to yield control to ensure the agent actually started to wait. If we
// don't, we risk sending the notification before agents are sleeping, and we hang.
$262.agent.tryYield();

// Notify all waiting on WAIT_INDEX, should be 3 always, they won't time out.
assert.sameValue(
  Atomics.notify(i64a, WAIT_INDEX),
  NUMAGENT,
  'Atomics.notify(i64a, WAIT_INDEX) returns the value of `NUMAGENT`'
);

Atomics.store(i64a, NOTIFY_INDEX, 1n);

const reports = [];
for (var i = 0; i < NUMAGENT  + TIMEOUT_AGENT_MESSAGES; i++) {
  reports.push($262.agent.getReport());
}
reports.sort();

for (var i = 0; i < NUMAGENT; i++) {
  assert.sameValue(reports[i], 'A ok', 'The value of reports[i] is "A ok"');
}
assert.sameValue(reports[NUMAGENT], 'B timed-out', 'The value of reports[NUMAGENT] is "B timed-out"');
assert.sameValue(reports[NUMAGENT + 1], "W timeout after Atomics.notify",
                 'The value of reports[NUMAGENT + 1] is "W timeout after Atomics.notify"');
