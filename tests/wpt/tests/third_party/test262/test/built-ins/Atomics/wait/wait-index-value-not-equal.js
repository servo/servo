// Copyright (C) 2018 Amal Hussein.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Returns "not-equal" when value of index is not equal
info: |
  Atomics.wait( typedArray, index, value, timeout )

  14.If v is not equal to w, then
    a.Perform LeaveCriticalSection(WL).
    b. Return the String "not-equal".

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const RUNNING = 1;
const TIMEOUT = $262.agent.timeouts.small;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    $262.agent.report(Atomics.wait(i32a, 0, 44, ${TIMEOUT}));
    $262.agent.report(Atomics.wait(i32a, 0, 251.4, ${TIMEOUT}));
    $262.agent.leaving();
  });
`);

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

// NB: We don't actually explicitly need to wait for the agent to start in this
// test case, we only do it for consistency with other test cases which do
// require the main agent to wait and yield control.

$262.agent.safeBroadcast(i32a);
$262.agent.waitUntil(i32a, RUNNING, 1);

// Try to yield control to ensure the agent actually started to wait.
$262.agent.tryYield();

assert.sameValue(
  $262.agent.getReport(),
  'not-equal',
  '$262.agent.getReport() returns "not-equal"'
);
assert.sameValue(
  $262.agent.getReport(),
  'not-equal',
  '$262.agent.getReport() returns "not-equal"'
);
assert.sameValue(Atomics.notify(i32a, 0), 0, 'Atomics.notify(i32a, 0) returns 0');
