// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.waitasync
description: >
  Object valueOf, toString, toPrimitive Zero timeout arg should result in an +0 timeout
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  6. Let q be ? ToNumber(timeout).

    Object -> Apply the following steps:

      Let primValue be ? ToPrimitive(argument, hint Number).
      Return ? ToNumber(primValue).

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, BigInt, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const RUNNING = 1;

$262.agent.start(`
  const valueOf = {
    valueOf() {
      return 0;
    }
  };

  const toString = {
    toString() {
      return "0";
    }
  };

  const toPrimitive = {
    [Symbol.toPrimitive]() {
      return 0;
    }
  };

  $262.agent.receiveBroadcast(async (sab) => {
    const i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);
    $262.agent.report(await Atomics.waitAsync(i64a, 0, 0n, valueOf).value);
    $262.agent.report(await Atomics.waitAsync(i64a, 0, 0n, toString).value);
    $262.agent.report(await Atomics.waitAsync(i64a, 0, 0n, toPrimitive).value);
    $262.agent.report(Atomics.waitAsync(i64a, 0, 0n, valueOf).value);
    $262.agent.report(Atomics.waitAsync(i64a, 0, 0n, toString).value);
    $262.agent.report(Atomics.waitAsync(i64a, 0, 0n, toPrimitive).value);
    $262.agent.leaving();
  });
`);

const i64a = new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4));

$262.agent.safeBroadcastAsync(i64a, RUNNING, 1n).then(async agentCount => {
  assert.sameValue(agentCount, 1n, 'The value of `agentCount` is 1n');

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'timed-out',
    '(await $262.agent.getReportAsync()) resolves to the value "timed-out"'
  );

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'timed-out',
    '(await $262.agent.getReportAsync()) resolves to the value "timed-out"'
  );

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'timed-out',
    '(await $262.agent.getReportAsync()) resolves to the value "timed-out"'
  );

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'timed-out',
    '(await $262.agent.getReportAsync()) resolves to the value "timed-out"'
  );

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'timed-out',
    '(await $262.agent.getReportAsync()) resolves to the value "timed-out"'
  );

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'timed-out',
    '(await $262.agent.getReportAsync()) resolves to the value "timed-out"'
  );

  assert.sameValue(
    Atomics.notify(i64a, 0),
    0,
    'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)), 0) must return 0'
  );
}).then($DONE, $DONE);
