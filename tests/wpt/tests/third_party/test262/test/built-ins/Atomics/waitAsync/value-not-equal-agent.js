// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Returns "not-equal" when value arg does not match an index in the typedArray
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  16. Let w be ! AtomicLoad(typedArray, i).
  17. If v is not equal to w, then
    a. Perform LeaveCriticalSection(WL).
    b. If mode is sync, then
      i. Return the String "not-equal".
    c. Perform ! Call(capability.[[Resolve]], undefined, « "not-equal" »).
    d. Return promiseCapability.[[Promise]].

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const RUNNING = 1;
const value = 42;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    $262.agent.report(Atomics.store(i32a, 0, ${value}));
    $262.agent.report(Atomics.waitAsync(i32a, 0, 0).value);
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
    '42',
    '(await $262.agent.getReportAsync()) resolves to the value "42"'
  );
  assert.sameValue(
    await $262.agent.getReportAsync(),
    'not-equal',
    '(await $262.agent.getReportAsync()) resolves to the value "not-equal"'
  );
  assert.sameValue(
    Atomics.notify(i32a, 0, 1),
    0,
    'Atomics.notify(new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)), 0, 1) must return 0'
  );
}).then($DONE, $DONE);
