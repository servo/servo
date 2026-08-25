// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Get the correct WaiterList
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  11. Let indexedPosition be (i × 4) + offset.
  12. Let WL be GetWaiterList(block, indexedPosition).

  GetWaiterList( block, i )

  ...
  4. Return the WaiterList that is referenced by the pair (block, i).

flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, BigInt, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const NUMAGENT = 2;
const RUNNING = 4;

$262.agent.start(`
  $262.agent.receiveBroadcast(async (sab) => {
    const i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    // Wait on index 0
    $262.agent.report(await Atomics.waitAsync(i64a, 0, 0n, Infinity).value);
    $262.agent.leaving();
  });
`);

$262.agent.start(`
  $262.agent.receiveBroadcast(async (sab) => {
    const i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    // Wait on index 2
    $262.agent.report(await Atomics.waitAsync(i64a, 2, 0n, Infinity).value);
    $262.agent.leaving();
  });
`);

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 5)
);

$262.agent.safeBroadcastAsync(i64a, RUNNING, BigInt(NUMAGENT)).then(async (agentCount) => {

  assert.sameValue(
    agentCount,
    BigInt(NUMAGENT),
    'The value of `agentCount` must return the same value returned by BigInt(NUMAGENT)'
  );

  // Notify index 1, notifies nothing
  assert.sameValue(Atomics.notify(i64a, 1), 0, 'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 5)), 1) must return 0');

  // Notify index 3, notifies nothing
  assert.sameValue(Atomics.notify(i64a, 3), 0, 'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 5)), 3) must return 0');

  // Notify index 2, notifies 1
  assert.sameValue(Atomics.notify(i64a, 2), 1, 'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 5)), 2) must return 1');
  assert.sameValue(
    await $262.agent.getReportAsync(),
    'ok',
    '(await $262.agent.getReportAsync()) resolves to the value "ok"'
  );

  // Notify index 0, notifies 1
  assert.sameValue(Atomics.notify(i64a, 0), 1, 'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 5)), 0) must return 1');
  assert.sameValue(
    await $262.agent.getReportAsync(),
    'ok',
    '(await $262.agent.getReportAsync()) resolves to the value "ok"'
  );

}).then($DONE, $DONE);

