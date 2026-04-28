// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.waitasync
description: >
  Throws a TypeError if index arg can not be converted to an Integer
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  6. Let q be ? ToNumber(timeout).

    Symbol --> Throw a TypeError exception.

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, Symbol, Symbol.toPrimitive, TypedArray, Atomics, BigInt, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const RUNNING = 1;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    let status1 = '';
    let status2 = '';

    try {
      Atomics.waitAsync(i64a, 0, 0n, Symbol('1'));
    } catch (error) {
      status1 = 'A ' + error.name;
    }
    try {
      Atomics.waitAsync(i64a, 0, 0n, Symbol('2'));
    } catch (error) {
      status2 = 'B ' + error.name;
    }

    $262.agent.report(status1);
    $262.agent.report(status2);
    $262.agent.leaving();
  });
`);

const i64a = new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4));

$262.agent.safeBroadcastAsync(i64a, RUNNING, 1n).then(async agentCount => {
  assert.sameValue(agentCount, 1n, 'The value of `agentCount` is 1n');

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'A TypeError',
    '(await $262.agent.getReportAsync()) resolves to the value "A TypeError"'
  );

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'B TypeError',
    '(await $262.agent.getReportAsync()) resolves to the value "B TypeError"'
  );

  assert.sameValue(
    Atomics.notify(i64a, 0),
    0,
    'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)), 0) must return 0'
  );
}).then($DONE, $DONE);
