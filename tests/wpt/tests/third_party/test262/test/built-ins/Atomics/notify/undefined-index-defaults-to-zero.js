// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  An undefined index arg should translate to 0
info: |
  Atomics.notify( typedArray, index, count )

  2.Let i be ? ValidateAtomicAccess(typedArray, index).
    ...
      2.Let accessIndex be ? ToIndex(requestIndex).

      9.If IsSharedArrayBuffer(buffer) is false, throw a TypeError exception.
        ...
          3.If bufferData is a Data Block, return false

          If value is undefined, then
          Let index be 0.
includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

var WAIT_INDEX = 0;
var RUNNING = 1;

var NUMAGENT = 2;

for (var i = 0; i < NUMAGENT; i++) {
  $262.agent.start(`
    $262.agent.receiveBroadcast(function(sab) {
      const i32a = new Int32Array(sab);

      // Notify main thread that the agent was started.
      Atomics.add(i32a, ${RUNNING}, 1);

      // Wait until restarted by main thread.
      var status = Atomics.wait(i32a, ${WAIT_INDEX}, 0);

      // Report wait status.
      $262.agent.report(status);

      $262.agent.leaving();
    });
  `);
}

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

$262.agent.safeBroadcast(i32a);

// Wait until both agents started.
$262.agent.waitUntil(i32a, RUNNING, NUMAGENT);

// Try to yield control to ensure the agents actually started to wait.
$262.agent.tryYield();

// Notify at index 0, undefined => 0.
var woken = 0;
while ((woken = Atomics.notify(i32a, undefined, 1)) === 0) ;
assert.sameValue(woken, 1, 'Atomics.notify(i32a, undefined, 1) returns 1');

assert.sameValue($262.agent.getReport(), 'ok', '$262.agent.getReport() returns "ok"');

// Notify again at index 0, default => 0.
var woken = 0;
while ((woken = Atomics.notify(i32a /*, default values used */)) === 0) ;
assert.sameValue(woken, 1, 'Atomics.notify(i32a /*, default values used */) returns 1');

assert.sameValue($262.agent.getReport(), 'ok', '$262.agent.getReport() returns "ok"');
