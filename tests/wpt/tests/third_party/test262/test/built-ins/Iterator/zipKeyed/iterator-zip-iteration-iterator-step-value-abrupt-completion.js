// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Handle abrupt completion from IteratorStepValue in IteratorZip.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    16. Return IteratorZip(iters, mode, padding, finishResults).

  IteratorZip ( iters, mode, padding, finishResults )
    3. Let closure be a new Abstract Closure with no parameters that captures
       iters, iterCount, openIters, mode, padding, and finishResults, and
       performs the following steps when called:
      ...
      b. Repeat,
        ...
        iii. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
          ...
          3. Else,
            a. Let result be Completion(IteratorStepValue(iter)).
            b. If result is an abrupt completion, then
              i. Remove iter from openIters.
              ii. Return ? IteratorCloseAll(openIters, result).
            ...
            d. If result is done, then
              i. Remove iter from openIters.
              ...

  IteratorCloseAll ( iters, completion )
    1. For each element iter of iters, in reverse List order, do
       a. Set completion to Completion(IteratorClose(iter, completion)).
    2. Return ? completion.

  IteratorClose ( iteratorRecord, completion )
    1. Assert: iteratorRecord.[[Iterator]] is an Object.
    2. Let iterator be iteratorRecord.[[Iterator]].
    3. Let innerResult be Completion(GetMethod(iterator, "return")).
    4. If innerResult is a normal completion, then
      a. Let return be innerResult.[[Value]].
      b. If return is undefined, return ? completion.
      c. Set innerResult to Completion(Call(return, iterator)).
    5. If completion is a throw completion, return ? completion.
    ...
includes: [compareArray.js]
features: [joint-iteration]
---*/

var modes = [
  "shortest",
  "longest",
  "strict",
];

function ExpectedError() {}

var log = [];

var first = {
  next() {
    log.push("call first next");
    throw new ExpectedError();
  },
  return() {
    log.push("unexpected call first return");
  }
};

var second = {
  next() {
    log.push("unexpected call second next");
  },
  return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, second);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("call second return");

    // IteratorClose ignores new exceptions when called with a Throw completion.
    throw new Test262Error();
  }
};

var third = {
  next() {
    log.push("unexpected call third next");
  },
  return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, third);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("call third return");

    // IteratorClose ignores new exceptions when called with a Throw completion.
    throw new Test262Error();
  }
};

// Empty iterator to ensure |return| is not called for closed iterators.
var empty = {
  next() {
    log.push("call empty next");
    return {done: true};
  },
  return() {
    log.push("unexpected call empty return");
  }
};

for (var mode of modes) {
  var it = Iterator.zipKeyed({first, second, third}, {mode});

  assert.throws(ExpectedError, function() {
    it.next();
  });

  assert.compareArray(log, [
    "call first next",
    "call third return",
    "call second return",
  ]);

  // Clear log.
  log.length = 0;
}

// This case applies only when mode is "longest".
var it = Iterator.zipKeyed({empty, first, second, third}, {mode: "longest"});

assert.throws(ExpectedError, function() {
  it.next();
});

assert.compareArray(log, [
  "call empty next",
  "call first next",
  "call third return",
  "call second return",
]);
