// Copyright (C) 2018 Amal Hussein.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Undefined count arg should result in an infinite count
info: |
  Atomics.notify( typedArray, index, count )

  3.If count is undefined, let c be +∞.

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 0; // Index to notify agent has started.
const WAIT_INDEX = 1; // Index all agents are waiting on.
const BUFFER_SIZE = 2;

const NUMAGENT = 4; // Total number of agents started

for (var i = 0; i < NUMAGENT; i++) {
  $262.agent.start(`
    $262.agent.receiveBroadcast(function(sab) {
      const i32a = new Int32Array(sab);
      Atomics.add(i32a, ${RUNNING}, 1);

      // Wait until restarted by main thread.
      var status = Atomics.wait(i32a, ${WAIT_INDEX}, 0);

      // Report wait status and then exit the agent.
      var name = String.fromCharCode(0x41 + ${i}); // "A", "B", "C", or "D"
      $262.agent.report(name + " " + status);
      $262.agent.leaving();
    });
  `);
}

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * BUFFER_SIZE)
);

$262.agent.safeBroadcast(i32a);
$262.agent.waitUntil(i32a, RUNNING, NUMAGENT);

// An agent may have been interrupted between reporting its initial report
// and the `Atomics.wait` call. Try to yield control to ensure the agent
// actually started to wait.
$262.agent.tryYield();

assert.sameValue(Atomics.notify(i32a, WAIT_INDEX, undefined), NUMAGENT,
                 'Atomics.notify(i32a, WAIT_INDEX, undefined) returns the value of `NUMAGENT`');

const reports = [];
for (var i = 0; i < NUMAGENT; i++) {
  reports.push($262.agent.getReport());
}
reports.sort();

assert.sameValue(reports[0], 'A ok', 'The value of reports[0] is "A ok"');
assert.sameValue(reports[1], 'B ok', 'The value of reports[1] is "B ok"');
assert.sameValue(reports[2], 'C ok', 'The value of reports[2] is "C ok"');
assert.sameValue(reports[3], 'D ok', 'The value of reports[3] is "D ok"');
