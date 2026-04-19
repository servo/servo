// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Get the correct WaiterList
info: |
  Atomics.wait( typedArray, index, value, timeout )

  ...
  11. Let WL be GetWaiterList(block, indexedPosition).
  ...


  GetWaiterList( block, i )

  ...
  4. Return the WaiterList that is referenced by the pair (block, i).

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

var NUMAGENT = 2;
var RUNNING = 4;

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    // Wait on index 0
    $262.agent.report(Atomics.wait(i32a, 0, 0, Infinity));
    $262.agent.leaving();
  });
`);

$262.agent.start(`
  $262.agent.receiveBroadcast(function(sab) {
    const i32a = new Int32Array(sab);
    Atomics.add(i32a, ${RUNNING}, 1);

    // Wait on index 2
    $262.agent.report(Atomics.wait(i32a, 2, 0, Infinity));
    $262.agent.leaving();
  });
`);

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 5)
);

$262.agent.safeBroadcast(i32a);

// Wait until all agents started.
$262.agent.waitUntil(i32a, RUNNING, NUMAGENT);

// Notify index 1, notifies nothing
assert.sameValue(Atomics.notify(i32a, 1), 0, 'Atomics.notify(i32a, 1) returns 0');

// Notify index 3, notifies nothing
assert.sameValue(Atomics.notify(i32a, 3), 0, 'Atomics.notify(i32a, 3) returns 0');

// Notify index 2, notifies 1
var woken = 0;
while ((woken = Atomics.notify(i32a, 2)) === 0) ;
assert.sameValue(woken, 1, 'Atomics.notify(i32a, 2) returns 1');
assert.sameValue($262.agent.getReport(), 'ok', '$262.agent.getReport() returns "ok"');

// Notify index 0, notifies 1
var woken = 0;
while ((woken = Atomics.notify(i32a, 0)) === 0) ;
assert.sameValue(woken, 1, 'Atomics.notify(i32a, 0) returns 1');
assert.sameValue($262.agent.getReport(), 'ok', '$262.agent.getReport() returns "ok"');
