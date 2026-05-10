// Copyright (C) 2019 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-forin-div-ofbodyevaluation-lhs-stmt-iterator-lhskind-labelset
description: >
  Ensure the number of ticks and Promise constructor lookups is correct with a Async-from-Sync iterator.
info: |
  13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation ( lhs, stmt, iteratorRecord, iterationKind,
                                                        lhsKind, labelSet [ , iteratorKind ] )
  25.1.4.2.1 %AsyncFromSyncIteratorPrototype%.next
  25.1.4.4 AsyncFromSyncIteratorContinuation
  25.6.4.5.1 PromiseResolve
  6.2.3.1 Await

includes: [compareArray.js]
flags: [async]
features: [async-iteration]
---*/

// The expected event log.
var expected = [
  // Before entering loop.
  "pre",

  // %AsyncFromSyncIteratorPrototype%.next
  // -> AsyncFromSyncIteratorContinuation
  //   -> PromiseResolve
  "constructor",

  // Await
  // -> PromiseResolve
  "constructor",

  // Async-from-Sync Iterator promise resolved.
  "tick 1",

  // Await promise resolved.
  "tick 2",

  // In loop body.
  "loop",

  // Await
  // -> PromiseResolve
  "constructor",

  // Async-from-Sync Iterator promise resolved.
  "tick 3",

  // Await promise resolved
  "tick 4",

  // After exiting loop.
  "post",
];

// The actual event log.
var actual = [];

// Test function using for-await with a single, already resolved Promise.
async function f() {
  var p = Promise.resolve(0);
  actual.push("pre");
  for await (var x of [p]) {
    actual.push("loop");
  }
  actual.push("post");
}

// Count the number of ticks needed to complete the loop and compare the actual log.
Promise.resolve(0)
  .then(() => actual.push("tick 1"))
  .then(() => actual.push("tick 2"))
  .then(() => actual.push("tick 3"))
  .then(() => actual.push("tick 4"))
  .then(() => {
    assert.compareArray(actual, expected, "Ticks and constructor lookups");
}).then($DONE, $DONE);

// Redefine `Promise.constructor` in order to intercept "constructor" lookups from PromiseResolve.
// (Perform last so that the lookups from SpeciesConstructor in `then` aren't logged.)
Object.defineProperty(Promise.prototype, "constructor", {
  get() {
    actual.push("constructor");
    return Promise;
  },
  configurable: true,
});

// Start the asynchronous function.
f();
