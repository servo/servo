// Copyright (C) 2026 Test262 Contributors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  With mode "strict", when all iterators finish at the same step, the result
  iterator completes normally and .next() is called on all of them.
info: |
  IteratorZip ( iters, mode, padding, finishResults )
    3. Let closure be a new Abstract Closure with no parameters that captures
       iters, iterCount, openIters, mode, padding, and finishResults, and
       performs the following steps when called:
      ...
      b. Repeat,
        ...
        iii. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
          ...
          1. Let iter be iters[i].
          2. If iter is null, then
             a. Assert: mode is "longest".
             ...
          3. Else,
            a. Let result be Completion(IteratorStepValue(iter)). 
            ...
            d. If result is done, then
              ...
              iii. Else if mode is "strict", then
                i. If i ≠ 0, then
                  ...
                ii. For each integer k such that 1 ≤ k < iterCount, in ascending order, do
                  i. Let open be Completion(IteratorStep(iters[k])).
                  ...
                iii. Return ReturnCompletion(undefined).
includes: [compareArray.js]
features: [joint-iteration]
---*/

var log = [];

function makeIter(name, length) {
  var count = 0;
  return {
    next() {
      count++;
      log.push("call " + name + " next");
      return { done: count > length };
    },
    return() {
      log.push("unexpected call " + name + " return");
    }
  };
}

var it = Iterator.zip([makeIter("a", 2), makeIter("b", 2), makeIter("c", 2)], { mode: "strict" });

it.next();
it.next();

log.length = 0;

var result = it.next();
assert.sameValue(result.done, true);

assert.compareArray(log, [
  "call a next",
  "call b next",
  "call c next",
]);
