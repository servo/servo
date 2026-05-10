// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.waitasync
description: >
  Test that Atomics.waitAsync returns the right result when it was awoken before
  a timeout
flags: [async]
includes: [atomicsHelper.js]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, BigInt, arrow-function, async-functions]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const RUNNING = 1;
const TIMEOUT = $262.agent.timeouts.huge;
const i64a = new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4));

$262.agent.start(`
  $262.agent.receiveBroadcast(async (sab) => {
    const i64a = new BigInt64Array(sab);
    Atomics.add(i64a, ${RUNNING}, 1n);

    const before = $262.agent.monotonicNow();
    const unpark = await Atomics.waitAsync(i64a, 0, 0n, ${TIMEOUT}).value;
    const duration = $262.agent.monotonicNow() - before;

    $262.agent.report(duration);
    $262.agent.report(unpark);
    $262.agent.leaving();
  });
`);

$262.agent.safeBroadcastAsync(i64a, RUNNING, 1n).then(async agentCount => {
  assert.sameValue(agentCount, 1n, 'The value of `agentCount` is 1n');

  assert.sameValue(
    Atomics.notify(i64a, 0),
    1,
    'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)), 0) must return 1'
  );

  const lapse = await $262.agent.getReportAsync();
  assert(lapse < TIMEOUT, 'The result of evaluating `(lapse < TIMEOUT)` is true');
  const result = await $262.agent.getReportAsync();
  assert.sameValue(result, 'ok', 'The value of `result` is "ok"');
  assert.sameValue(result, 'ok', 'The value of `result` is "ok"');

  assert.sameValue(
    Atomics.notify(i64a, 0),
    0,
    'Atomics.notify(new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)), 0) must return 0'
  );
}).then($DONE, $DONE);
