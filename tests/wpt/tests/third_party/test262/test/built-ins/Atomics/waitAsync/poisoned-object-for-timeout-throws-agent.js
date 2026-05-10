// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  False timeout arg should result in an +0 timeout
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  6. Let q be ? ToNumber(timeout).

  Let primValue be ? ToPrimitive(argument, hint Number).
  Return ? ToNumber(primValue).

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const RUNNING = 1;

$262.agent.start(`
  const poisonedValueOf = {
    valueOf() {
      throw new Error('should not evaluate this code');
    }
  };

  const poisonedToPrimitive = {
    [Symbol.toPrimitive]() {
      throw new Error('passing a poisoned object using @@ToPrimitive');
    }
  };

  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    let status1 = '';
    let status2 = '';

    try {
      Atomics.waitAsync(i32a, 0, 0, poisonedValueOf);
    } catch (error) {
      status1 = 'poisonedValueOf';
    }
    try {
      Atomics.waitAsync(i32a, 0, 0, poisonedToPrimitive);
    } catch (error) {
      status2 = 'poisonedToPrimitive';
    }

    $262.agent.report(status1);
    $262.agent.report(status2);
    $262.agent.leaving();
  });
`);

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

$262.agent.safeBroadcastAsync(i32a, RUNNING, 1).then(async (agentCount) => {

  assert.sameValue(agentCount, 1, 'The value of `agentCount` is 1');

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'poisonedValueOf',
    '(await $262.agent.getReportAsync()) resolves to the value "poisonedValueOf"'
  );

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'poisonedToPrimitive',
    '(await $262.agent.getReportAsync()) resolves to the value "poisonedToPrimitive"'
  );

  assert.sameValue(Atomics.notify(i32a, 0), 0, 'Atomics.notify(new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)), 0) must return 0');

}).then($DONE, $DONE);

