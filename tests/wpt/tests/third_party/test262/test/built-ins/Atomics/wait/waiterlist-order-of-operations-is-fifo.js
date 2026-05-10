// Copyright (C) 2018 Amal Hussein.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  New waiters should be applied to the end of the list and woken by order they entered the list (FIFO)
info: |
  Atomics.wait( typedArray, index, value, timeout )

  16.Perform AddWaiter(WL, W).
    ...
    3.Add W to the end of the list of waiters in WL.

includes: [atomicsHelper.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

var NUMAGENT = 3;

var WAIT_INDEX = 0;
var RUNNING = 1;
var LOCK_INDEX = 2;

for (var i = 0; i < NUMAGENT; i++) {
  var agentNum = i;

  $262.agent.start(`
    $262.agent.receiveBroadcast(function(sab) {
      const i32a = new Int32Array(sab);
      Atomics.add(i32a, ${RUNNING}, 1);

      // Synchronize workers before reporting the initial report.
      while (Atomics.compareExchange(i32a, ${LOCK_INDEX}, 0, 1) !== 0) ;

      // Report the agent number before waiting.
      $262.agent.report(${agentNum});

      // Wait until restarted by main thread.
      var status = Atomics.wait(i32a, ${WAIT_INDEX}, 0);

      // Report wait status.
      $262.agent.report(status);

      // Report the agent number after waiting.
      $262.agent.report(${agentNum});

      $262.agent.leaving();
    });
  `);
}

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

$262.agent.safeBroadcast(i32a);

// Wait until all agents started.
$262.agent.waitUntil(i32a, RUNNING, NUMAGENT);

// Agents may be started in any order.
const started = [];
for (var i = 0; i < NUMAGENT; i++) {
  // Wait until an agent entered its critical section.
  $262.agent.waitUntil(i32a, LOCK_INDEX, 1);

  // Record the agent number.
  started.push($262.agent.getReport());

  // The agent may have been interrupted between reporting its initial report
  // and the `Atomics.wait` call. Try to yield control to ensure the agent
  // actually started to wait.
  $262.agent.tryYield();

  // Now continue with the next agent.
  Atomics.store(i32a, LOCK_INDEX, 0);
}

// Agents must notify in the order they waited.
for (var i = 0; i < NUMAGENT; i++) {
  var woken = 0;
  while ((woken = Atomics.notify(i32a, WAIT_INDEX, 1)) === 0) ;

  assert.sameValue(woken, 1,
                   'Atomics.notify(i32a, WAIT_INDEX, 1) returns 1, at index = ' + i);

  assert.sameValue($262.agent.getReport(), 'ok',
                   '$262.agent.getReport() returns "ok", at index = ' + i);

  assert.sameValue($262.agent.getReport(), started[i],
                   '$262.agent.getReport() returns the value of `started[' + i + ']`');
}
