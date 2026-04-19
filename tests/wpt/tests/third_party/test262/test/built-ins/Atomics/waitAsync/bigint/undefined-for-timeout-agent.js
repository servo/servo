// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Undefined timeout arg should result in an infinite timeout
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  6. Let q be ? ToNumber(timeout).
    ...
    Undefined    Return NaN.

  5.If q is NaN, let t be +∞, else let t be max(q, 0)

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, BigInt, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const WAIT_INDEX = 0;
const RUNNING = 1;
const NUMAGENT = 2;
const NOTIFYCOUNT = 2;

$262.agent.start(`
  $262.agent.receiveBroadcast(async (sab) => {
    var i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    // undefined => NaN => +Infinity
    $262.agent.report("A " + (await Atomics.waitAsync(i64a, 0, 0n, undefined).value));
    $262.agent.leaving();
  });
`);

$262.agent.start(`
  $262.agent.receiveBroadcast(async (sab) => {
    var i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    // undefined timeout arg => NaN => +Infinity
    $262.agent.report("B " + (await Atomics.waitAsync(i64a, 0, 0n).value));
    $262.agent.leaving();
  });
`);

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)
);

$262.agent.safeBroadcastAsync(i64a, RUNNING, BigInt(NUMAGENT)).then(async (agentCount) => {

  assert.sameValue(
    agentCount,
    BigInt(NUMAGENT),
    'The value of `agentCount` must return the same value returned by BigInt(NUMAGENT)'
  );

  assert.sameValue(
    Atomics.notify(i64a, WAIT_INDEX, NOTIFYCOUNT),
    NOTIFYCOUNT,
    'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)), 0, 2) must return the value of NOTIFYCOUNT'
  );

  const reports = [
    await $262.agent.getReportAsync(),
    await $262.agent.getReportAsync(),
  ];

  reports.sort();
  assert.sameValue(reports[0], 'A ok', 'The value of reports[0] is "A ok"');
  assert.sameValue(reports[1], 'B ok', 'The value of reports[1] is "B ok"');
}).then($DONE, $DONE);

