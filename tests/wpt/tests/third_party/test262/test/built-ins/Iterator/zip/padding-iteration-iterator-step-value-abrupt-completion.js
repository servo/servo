// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Abrupt completion for IteratorStepValue in "padding" option iteration.
info: |
  Iterator.zip ( iterables [ , options ] )
    ...
    14. If mode is "longest", then
      ...
      b. Else,
        i. Let paddingIter be Completion(GetIterator(paddingOption, sync)).
        ...
        iv. Perform the following steps iterCount times:
          1. If usingIterator is true, then
            a. Set next to Completion(IteratorStepValue(paddingIter)).
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
    log.push("first return");

    // This exception is ignored.
    throw new Test262Error();
  }
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
    log.push("second return");

    // This exception is ignored.
    throw new Test262Error();
  }
};

var third = {
  next() {
    log.push("unexpected call to next method");
  },
  get return() {
    // Called with the correct receiver and no arguments.
    assert.sameValue(this, third);
    assert.sameValue(arguments.length, 0);

    // NB: Log after above asserts, because failures aren't propagated.
    log.push("third return");

    // This exception is ignored.
    throw new Test262Error();
  }
};

function ExpectedError() {}

// Padding iterator throws from |next|.
var padding = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    throw new ExpectedError();
  },
  return() {
    log.push("unexpected call to return method");
  },
};

assert.throws(ExpectedError, function() {
  Iterator.zip([first, second, third], {mode: "longest", padding});
});

assert.compareArray(log, [
  "third return",
  "second return",
  "first return",
]);

// Clear log
log.length = 0;

// Padding iterator throws from |done|.
var padding = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    return {
      get done() {
        throw new ExpectedError();
      },
      get value() {
        log.push("unexpected access to value");
      },
    };
  },
  return() {
    log.push("unexpected call to return method");
  },
};

assert.throws(ExpectedError, function() {
  Iterator.zip([first, second, third], {mode: "longest", padding});
});

assert.compareArray(log, [
  "third return",
  "second return",
  "first return",
]);


// Clear log
log.length = 0;

// Padding iterator throws from |value|.
var padding = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    return {
      done: false,
      get value() {
        throw new ExpectedError();
      },
    };
  },
  return() {
    log.push("unexpected call to return method");
  },
};

assert.throws(ExpectedError, function() {
  Iterator.zip([first, second, third], {mode: "longest", padding});
});

assert.compareArray(log, [
  "third return",
  "second return",
  "first return",
]);
