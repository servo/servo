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

  2. Let i be ? ValidateAtomicAccess(typedArray, index).

  ValidateAtomicAccess( typedArray, requestIndex )

  2. Let accessIndex be ? ToIndex(requestIndex).

  ToIndex ( value )

  2. Else,
    a. Let integerIndex be ? ToInteger(value).

  ToInteger(value)

  1. Let number be ? ToNumber(argument).

    Symbol --> Throw a TypeError exception.

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, Symbol, Symbol.toPrimitive, TypedArray, Atomics, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const RUNNING = 1;

$262.agent.start(`
  const poisonedValueOf = {
    valueOf() {
      throw new Test262Error('should not evaluate this code');
    }
  };

  const poisonedToPrimitive = {
    [Symbol.toPrimitive]() {
      throw new Test262Error('should not evaluate this code');
    }
  };

  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    let status1 = '';
    let status2 = '';

    try {
      Atomics.waitAsync(i32a, Symbol('1'), poisonedValueOf, poisonedValueOf);
    } catch (error) {
      status1 = 'A ' + error.name;
    }
    try {
      Atomics.waitAsync(i32a, Symbol('2'), poisonedToPrimitive, poisonedToPrimitive);
    } catch (error) {
      status2 = 'B ' + error.name;
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
    'A TypeError',
    '(await $262.agent.getReportAsync()) resolves to the value "A TypeError"'
  );

  assert.sameValue(
    await $262.agent.getReportAsync(),
    'B TypeError',
    '(await $262.agent.getReportAsync()) resolves to the value "B TypeError"'
  );

  assert.sameValue(Atomics.notify(i32a, 0), 0, 'Atomics.notify(new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)), 0) must return 0');

}).then($DONE, $DONE);
