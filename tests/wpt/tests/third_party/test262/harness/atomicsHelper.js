// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Collection of functions used to interact with Atomics.* operations across agent boundaries.
defines:
  - $262.agent.getReportAsync
  - $262.agent.getReport
  - $262.agent.safeBroadcastAsync
  - $262.agent.safeBroadcast
  - $262.agent.setTimeout
  - $262.agent.tryYield
  - $262.agent.trySleep
---*/

/**
 * @return {String} A report sent from an agent.
 */
{
  // This is only necessary because the original
  // $262.agent.getReport API was insufficient.
  //
  // All runtimes currently have their own
  // $262.agent.getReport which is wrong, so we
  // will pave over it with a corrected version.
  //
  // Binding $262.agent is necessary to prevent
  // breaking SpiderMonkey's $262.agent.getReport
  let getReport = $262.agent.getReport.bind($262.agent);

  $262.agent.getReport = function() {
    var r;
    while ((r = getReport()) == null) {
      $262.agent.sleep(1);
    }
    return r;
  };

  if (this.setTimeout === undefined) {
    (function(that) {
      that.setTimeout = function(callback, delay) {
        let p = Promise.resolve();
        let start = Date.now();
        let end = start + delay;
        function check() {
          if ((end - Date.now()) > 0) {
            p.then(check);
          }
          else {
            callback();
          }
        }
        p.then(check);
      }
    })(this);
  }

  $262.agent.setTimeout = setTimeout;

  $262.agent.getReportAsync = function() {
    return new Promise(function(resolve) {
      (function loop() {
        let result = getReport();
        if (!result) {
          setTimeout(loop, 1000);
        } else {
          resolve(result);
        }
      })();
    });
  };
}

/**
 *
 * Share a given Int32Array or BigInt64Array to all running agents. Ensure that the
 * provided TypedArray is a "shared typed array".
 *
 * NOTE: Migrating all tests to this API is necessary to prevent tests from hanging
 * indefinitely when a SAB is sent to a worker but the code in the worker attempts to
 * create a non-sharable TypedArray (something that is not Int32Array or BigInt64Array).
 * When that scenario occurs, an exception is thrown and the agent worker can no
 * longer communicate with any other threads that control the SAB. If the main
 * thread happens to be spinning in the $262.agent.waitUntil() while loop, it will never
 * meet its termination condition and the test will hang indefinitely.
 *
 * Because we've defined $262.agent.broadcast(SAB) in
 * https://github.com/tc39/test262/blob/HEAD/INTERPRETING.md, there are host implementations
 * that assume compatibility, which must be maintained.
 *
 *
 * $262.agent.safeBroadcast(TA) should not be included in
 * https://github.com/tc39/test262/blob/HEAD/INTERPRETING.md
 *
 *
 * @param {(Int32Array|BigInt64Array)} typedArray An Int32Array or BigInt64Array with a SharedArrayBuffer
 */
$262.agent.safeBroadcast = function(typedArray) {
  let Constructor = Object.getPrototypeOf(typedArray).constructor;
  let temp = new Constructor(
    new SharedArrayBuffer(Constructor.BYTES_PER_ELEMENT)
  );
  try {
    // This will never actually wait, but that's fine because we only
    // want to ensure that this typedArray CAN be waited on and is shareable.
    Atomics.wait(temp, 0, Constructor === Int32Array ? 1 : BigInt(1));
  } catch (error) {
    throw new Test262Error(`${Constructor.name} cannot be used as a shared typed array. (${error})`);
  }

  $262.agent.broadcast(typedArray.buffer);
};

$262.agent.safeBroadcastAsync = async function(ta, index, expected) {
  await $262.agent.broadcast(ta.buffer);
  await $262.agent.waitUntil(ta, index, expected);
  await $262.agent.tryYield();
  return await Atomics.load(ta, index);
};


/**
 * With a given Int32Array or BigInt64Array, wait until the expected number of agents have
 * reported themselves by calling:
 *
 *    Atomics.add(typedArray, index, 1);
 *
 * @param {(Int32Array|BigInt64Array)} typedArray An Int32Array or BigInt64Array with a SharedArrayBuffer
 * @param {number} index    The index of which all agents will report.
 * @param {number} expected The number of agents that are expected to report as active.
 */
$262.agent.waitUntil = function(typedArray, index, expected) {

  var agents = 0;
  while ((agents = Atomics.load(typedArray, index)) !== expected) {
    /* nothing */
  }
  assert.sameValue(agents, expected, "Reporting number of 'agents' equals the value of 'expected'");
};

/**
 * Timeout values used throughout the Atomics tests. All timeouts are specified in milliseconds.
 *
 * @property {number} yield Used for `$262.agent.tryYield`. Must not be used in other functions.
 * @property {number} small Used when agents will always timeout and `Atomics.wake` is not part
 *                          of the test semantics. Must be larger than `$262.agent.timeouts.yield`.
 * @property {number} long  Used when some agents may timeout and `Atomics.wake` is called on some
 *                          agents. The agents are required to wait and this needs to be observable
 *                          by the main thread.
 * @property {number} huge  Used when `Atomics.wake` is called on all waiting agents. The waiting
 *                          must not timeout. The agents are required to wait and this needs to be
 *                          observable by the main thread. All waiting agents must be woken by the
 *                          main thread.
 *
 * Usage for `$262.agent.timeouts.small`:
 *   const WAIT_INDEX = 0;
 *   const RUNNING = 1;
 *   const TIMEOUT = $262.agent.timeouts.small;
 *   const i32a = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2));
 *
 *   $262.agent.start(`
 *     $262.agent.receiveBroadcast(function(sab) {
 *       const i32a = new Int32Array(sab);
 *       Atomics.add(i32a, ${RUNNING}, 1);
 *
 *       $262.agent.report(Atomics.wait(i32a, ${WAIT_INDEX}, 0, ${TIMEOUT}));
 *
 *       $262.agent.leaving();
 *     });
 *   `);
 *   $262.agent.safeBroadcast(i32a.buffer);
 *
 *   // Wait until the agent was started and then try to yield control to increase
 *   // the likelihood the agent has called `Atomics.wait` and is now waiting.
 *   $262.agent.waitUntil(i32a, RUNNING, 1);
 *   $262.agent.tryYield();
 *
 *   // The agent is expected to time out.
 *   assert.sameValue($262.agent.getReport(), "timed-out");
 *
 *
 * Usage for `$262.agent.timeouts.long`:
 *   const WAIT_INDEX = 0;
 *   const RUNNING = 1;
 *   const NUMAGENT = 2;
 *   const TIMEOUT = $262.agent.timeouts.long;
 *   const i32a = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2));
 *
 *   for (let i = 0; i < NUMAGENT; i++) {
 *     $262.agent.start(`
 *       $262.agent.receiveBroadcast(function(sab) {
 *         const i32a = new Int32Array(sab);
 *         Atomics.add(i32a, ${RUNNING}, 1);
 *
 *         $262.agent.report(Atomics.wait(i32a, ${WAIT_INDEX}, 0, ${TIMEOUT}));
 *
 *         $262.agent.leaving();
 *       });
 *     `);
 *   }
 *   $262.agent.safeBroadcast(i32a.buffer);
 *
 *   // Wait until the agents were started and then try to yield control to increase
 *   // the likelihood the agents have called `Atomics.wait` and are now waiting.
 *   $262.agent.waitUntil(i32a, RUNNING, NUMAGENT);
 *   $262.agent.tryYield();
 *
 *   // Wake exactly one agent.
 *   assert.sameValue(Atomics.wake(i32a, WAIT_INDEX, 1), 1);
 *
 *   // When it doesn't matter how many agents were woken at once, a while loop
 *   // can be used to make the test more resilient against intermittent failures
 *   // in case even though `tryYield` was called, the agents haven't started to
 *   // wait.
 *   //
 *   // // Repeat until exactly one agent was woken.
 *   // var woken = 0;
 *   // while ((woken = Atomics.wake(i32a, WAIT_INDEX, 1)) !== 0) ;
 *   // assert.sameValue(woken, 1);
 *
 *   // One agent was woken and the other one timed out.
 *   const reports = [$262.agent.getReport(), $262.agent.getReport()];
 *   assert(reports.includes("ok"));
 *   assert(reports.includes("timed-out"));
 *
 *
 * Usage for `$262.agent.timeouts.huge`:
 *   const WAIT_INDEX = 0;
 *   const RUNNING = 1;
 *   const NUMAGENT = 2;
 *   const TIMEOUT = $262.agent.timeouts.huge;
 *   const i32a = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2));
 *
 *   for (let i = 0; i < NUMAGENT; i++) {
 *     $262.agent.start(`
 *       $262.agent.receiveBroadcast(function(sab) {
 *         const i32a = new Int32Array(sab);
 *         Atomics.add(i32a, ${RUNNING}, 1);
 *
 *         $262.agent.report(Atomics.wait(i32a, ${WAIT_INDEX}, 0, ${TIMEOUT}));
 *
 *         $262.agent.leaving();
 *       });
 *     `);
 *   }
 *   $262.agent.safeBroadcast(i32a.buffer);
 *
 *   // Wait until the agents were started and then try to yield control to increase
 *   // the likelihood the agents have called `Atomics.wait` and are now waiting.
 *   $262.agent.waitUntil(i32a, RUNNING, NUMAGENT);
 *   $262.agent.tryYield();
 *
 *   // Wake all agents.
 *   assert.sameValue(Atomics.wake(i32a, WAIT_INDEX), NUMAGENT);
 *
 *   // When it doesn't matter how many agents were woken at once, a while loop
 *   // can be used to make the test more resilient against intermittent failures
 *   // in case even though `tryYield` was called, the agents haven't started to
 *   // wait.
 *   //
 *   // // Repeat until all agents were woken.
 *   // for (var wokenCount = 0; wokenCount < NUMAGENT; ) {
 *   //   var woken = 0;
 *   //   while ((woken = Atomics.wake(i32a, WAIT_INDEX)) !== 0) ;
 *   //   // Maybe perform an action on the woken agents here.
 *   //   wokenCount += woken;
 *   // }
 *
 *   // All agents were woken and none timeout.
 *   for (var i = 0; i < NUMAGENT; i++) {
 *     assert($262.agent.getReport(), "ok");
 *   }
 */
$262.agent.timeouts = {
  yield: 100,
  small: 200,
  long: 1000,
  huge: 10000,
};

/**
 * Try to yield control to the agent threads.
 *
 * Usage:
 *   const VALUE = 0;
 *   const RUNNING = 1;
 *   const i32a = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2));
 *
 *   $262.agent.start(`
 *     $262.agent.receiveBroadcast(function(sab) {
 *       const i32a = new Int32Array(sab);
 *       Atomics.add(i32a, ${RUNNING}, 1);
 *
 *       Atomics.store(i32a, ${VALUE}, 1);
 *
 *       $262.agent.leaving();
 *     });
 *   `);
 *   $262.agent.safeBroadcast(i32a.buffer);
 *
 *   // Wait until agent was started and then try to yield control.
 *   $262.agent.waitUntil(i32a, RUNNING, 1);
 *   $262.agent.tryYield();
 *
 *   // Note: This result is not guaranteed, but should hold in practice most of the time.
 *   assert.sameValue(Atomics.load(i32a, VALUE), 1);
 *
 * The default implementation simply waits for `$262.agent.timeouts.yield` milliseconds.
 */
$262.agent.tryYield = function() {
  $262.agent.sleep($262.agent.timeouts.yield);
};

/**
 * Try to sleep the current agent for the given amount of milliseconds. It is acceptable,
 * but not encouraged, to ignore this sleep request and directly continue execution.
 *
 * The default implementation calls `$262.agent.sleep(ms)`.
 *
 * @param {number} ms Time to sleep in milliseconds.
 */
$262.agent.trySleep = function(ms) {
  $262.agent.sleep(ms);
};
