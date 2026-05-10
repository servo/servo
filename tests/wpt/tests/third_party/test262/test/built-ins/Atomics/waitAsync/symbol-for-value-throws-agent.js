// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Throws a TypeError if value arg is a Symbol
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  5. Otherwise, let v be ? ToInt32(value).

  ToInt32(value)

  1.Let number be ? ToNumber(argument).

    Symbol --> Throw a TypeError exception.

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, Symbol, Symbol.toPrimitive, TypedArray, Atomics, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const RUNNING = 1;

$262.agent.start(`
  const poisonedValueOf = {
    valueOf: function() {
      throw new Test262Error('should not evaluate this code');
    }
  };

  const poisonedToPrimitive = {
    [Symbol.toPrimitive]: function() {
      throw new Test262Error("passing a poisoned object using @@ToPrimitive");
    }
  };

  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    let status1 = "";
    let status2 = "";

    try {
      Atomics.waitAsync(i32a, 0, Symbol("1"), poisonedValueOf);
    } catch (error) {
      status1 = 'A ' + error.name;
    }
    try {
      Atomics.waitAsync(i32a, 0, Symbol("2"), poisonedToPrimitive);
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


