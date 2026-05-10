// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Handle abrupt completions during iterables iteration.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    12. For each element key of allKeys, do
      ...
      3. If desc is not undefined and desc.[[Enumerable]] is true, then
        i. Let value be Completion(Get(iterables, key)).
        ii. IfAbruptCloseIterators(value, iters).
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

var symbol = {
  next() {
    log.push("unexpected call to next method");
  },
  return() {
    log.push("unexpected call to return method");
  },
};

var arrayIndex = {
  next() {
    log.push("unexpected call to next method");
  },
  return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, arrayIndex);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("close array-indexed iterator");

    // IteratorClose ignores new exceptions when called with a Throw completion.
    throw new Test262Error();
  },
};

var iterables = {
  [Symbol()]: symbol,
  first,
  second,
  get third() {
    throw new ExpectedError();
  },
  5: arrayIndex,
};

assert.throws(ExpectedError, function() {
  Iterator.zipKeyed(iterables);
});

// Ensure iterators are closed in the correct order.
assert.compareArray(log, [
  "close second iterator",
  "close first iterator",
  "close array-indexed iterator",
]);
