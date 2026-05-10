// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.waitasync
description: >
  Undefined index arg is coerced to zero
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  2. Let i be ? ValidateAtomicAccess(typedArray, index).
      ...
      2.Let accessIndex be ? ToIndex(requestIndex).

      9.If IsSharedArrayBuffer(buffer) is false, throw a TypeError exception.
        ...
          3.If bufferData is a Data Block, return false

          If value is undefined, then
          Let index be 0.

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const WAIT_INDEX = 0;
const RUNNING = 1;
const NUMAGENT = 2;
const NOTIFYCOUNT = 2;

$262.agent.start(`
  $262.agent.receiveBroadcast(async (sab) => {
    var i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    $262.agent.report("A " + (await Atomics.waitAsync(i32a, undefined, 0).value));
    $262.agent.leaving();
  });
`);

$262.agent.start(`
  $262.agent.receiveBroadcast(async (sab) => {
    var i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    $262.agent.report("B " + (await Atomics.waitAsync(i32a, undefined, 0).value));
    $262.agent.leaving();
  });
`);

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

$262.agent.safeBroadcastAsync(i32a, RUNNING, NUMAGENT).then(async (agentCount) => {

  assert.sameValue(agentCount, NUMAGENT, 'The value of `agentCount` is expected to equal the value of NUMAGENT');

  assert.sameValue(
    Atomics.notify(i32a, WAIT_INDEX, NOTIFYCOUNT),
    NOTIFYCOUNT,
    'Atomics.notify(new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)), 0, 2) must return the value of NOTIFYCOUNT'
  );

  const reports = [
    await $262.agent.getReportAsync(),
    await $262.agent.getReportAsync(),
  ];

  reports.sort();
  assert.sameValue(reports[0], 'A ok', 'The value of reports[0] is "A ok"');
  assert.sameValue(reports[1], 'B ok', 'The value of reports[1] is "B ok"');
}).then($DONE, $DONE);
