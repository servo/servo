// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.all
description: >
  Iterator is not closed when the "resolve" capability returns an abrupt
  completion.
info: |
  1. Let C be the this value.
  [...]
  3. Let promiseCapability be ? NewPromiseCapability(C).
  [...]
  7. Let result be PerformPromiseAll(iteratorRecord, C, promiseCapability).
  8. If result is an abrupt completion, then
     a. If iteratorRecord.[[Done]] is false, let result be
        IteratorClose(iterator, result).
     b. IfAbruptRejectPromise(result, promiseCapability).

  25.4.4.1.1 Runtime Semantics: PerformPromiseAll

  [...]
  6. Repeat
     [...]
     d. If next is false, then
        [...]
        iii. If remainingElementsCount.[[Value]] is 0, then
             1. Let valuesArray be CreateArrayFromList(values).
             2. Perform ? Call(resultCapability.[[Resolve]], undefined, «
                valuesArray »).

  25.4.1.1.1 IfAbruptRejectPromise

  IfAbruptRejectPromise is a short hand for a sequence of algorithm steps that
  use a PromiseCapability Record. An algorithm step of the form:

  1. IfAbruptRejectPromise(value, capability).

  means the same thing as:

  1. If value is an abrupt completion, then
     a. Perform ? Call(capability.[[Reject]], undefined, « value.[[Value]] »).
     b. Return capability.[[Promise]].
  2. Else if value is a Completion Record, let value be value.[[Value]].
features: [Symbol.iterator]
---*/

var nextCount = 0;
var returnCount = 0;
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      nextCount += 1;
      return {
        done: true
      };
    },
    return: function() {
      returnCount += 1;
      return {};
    }
  };
};
var P = function(executor) {
  return new Promise(function(_, reject) {
    executor(function() {
      throw new Test262Error();
    }, reject);
  });
};

P.resolve = Promise.resolve;

Promise.all.call(P, iter);

assert.sameValue(nextCount, 1);
assert.sameValue(returnCount, 0);
