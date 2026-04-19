// Copyright (C) 2018 Amal Hussein.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Undefined timeout arg should result in an infinite timeout
info: |
  Atomics.wait( typedArray, index, value, timeout )

  4.Let q be ? ToNumber(timeout).
    ...
    Undefined    Return NaN.
  5.If q is NaN, let t be +∞, else let t be max(q, 0)

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const WAIT_INDEX = 0; // Index all agents are waiting on
const RUNNING = 1;
const NUMAGENT = 2;   // Total number of agents started
const NOTIFYCOUNT = 2;  // Total number of agents to notify up

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    var i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    // undefined => NaN => +Infinity
    $262.agent.report("A " + Atomics.wait(i32a, 0, 0, undefined));
    $262.agent.leaving();
  });
`);

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    var i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    // undefined timeout arg => NaN => +Infinity
    $262.agent.report("B " + Atomics.wait(i32a, 0, 0));
    $262.agent.leaving();
  });
`);

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

$262.agent.safeBroadcast(i32a);
$262.agent.waitUntil(i32a, RUNNING, NUMAGENT);

// Try to yield control to ensure the agent actually started to wait.
$262.agent.tryYield();

assert.sameValue(
  Atomics.notify(i32a, WAIT_INDEX, NOTIFYCOUNT),
  NOTIFYCOUNT,
  'Atomics.notify(i32a, WAIT_INDEX, NOTIFYCOUNT) returns the value of `NOTIFYCOUNT` (2)'
);

const reports = [];
for (var i = 0; i < NUMAGENT; i++) {
  reports.push($262.agent.getReport());
}
reports.sort();

assert.sameValue(reports[0], 'A ok', 'The value of reports[0] is "A ok"');
assert.sameValue(reports[1], 'B ok', 'The value of reports[1] is "B ok"');
