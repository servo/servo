// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Handle abrupt completion from IteratorStep in IteratorZip.
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
        v. Let completion be Completion(Yield(results)).
        vi. If completion is an abrupt completion, then
          1. Return ? IteratorCloseAll(openIters, completion).
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

function ExpectedError() {}

var log = [];

var first = {
  next() {
    log.push("call first next");
    return {done: false};
  },
  return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, first);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("call first return");

    // IteratorClose ignores new exceptions when called with a Throw completion.
    throw new Test262Error();
  }
};

var second = {
  next() {
    log.push("call second next");
    return {done: false};
  },
  return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, second);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("call second return");

    throw new ExpectedError();
  }
};

var third = {
  next() {
    log.push("call third next");
    return {done: false};
  },
  return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, third);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("call third return");

    return {};
  }
};

var fourth = {
  next() {
    log.push("call fourth next");
    return {done: true};
  },
  return() {
    log.push("unexpected call fourth return");
  }
};

var it = Iterator.zipKeyed({first, second, third, fourth}, {mode: "longest"});

it.next();

assert.throws(ExpectedError, function() {
  it.return();
});

assert.compareArray(log, [
  "call first next",
  "call second next",
  "call third next",
  "call fourth next",

  "call third return",
  "call second return",
  "call first return",
]);
