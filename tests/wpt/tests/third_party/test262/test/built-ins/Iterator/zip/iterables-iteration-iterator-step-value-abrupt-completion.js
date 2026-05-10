// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Handle abrupt completions during iterables iteration.
info: |
  Iterator.zip ( iterables [ , options ] )
    ...
    12. Repeat, while next is not done,
      a. Set next to Completion(IteratorStepValue(inputIter)).
      b. IfAbruptCloseIterators(next, iters).
      ...

  IfAbruptCloseIterators ( value, iteratorRecords )
    1. Assert: value is a Completion Record.
    2. If value is an abrupt completion, return ? IteratorCloseAll(iteratorRecords, value).
    3. Else, set value to value.[[Value]].

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

class ExpectedError extends Error {}

var log = [];

var first = {
  next() {
    log.push("unexpected call to next method");
  },
  return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, first);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("close first iterator");

    // IteratorClose ignores new exceptions when called with a Throw completion.
    throw new Test262Error();
  },
};

var second = {
  next() {
    log.push("unexpected call to next method");
  },
  return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, second);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("close second iterator");

    // IteratorClose ignores new exceptions when called with a Throw completion.
    throw new Test262Error();
  },
};

var elements = [first, second];
var elementsIter = elements.values();

var iterables = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    log.push("call next");
    var result = elementsIter.next();
    if (result.done) {
      throw new ExpectedError();
    }
    return result;
  },
  return() {
    // This method shouldn't be called.
    log.push("UNEXPECTED - close iterables iterator");

    // IteratorClose ignores new exceptions when called with a Throw completion.
    throw new Test262Error();
  },
};

assert.throws(ExpectedError, function() {
  Iterator.zip(iterables);
});

// Ensure iterators are closed in the correct order.
assert.compareArray(log, [
  "call next",
  "call next",
  "call next",
  "close second iterator",
  "close first iterator",
]);
