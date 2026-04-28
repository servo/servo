// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.all
description: >
  Error when advancing the provided iterable (rejecting promise)
info: |
    11. Let result be PerformPromiseAll(iteratorRecord, C, promiseCapability).
    12. If result is an abrupt completion,
        a. If iteratorRecord.[[done]] is false, let result be
           IteratorClose(iterator, result).
        b. IfAbruptRejectPromise(result, promiseCapability).

    [...]

    25.4.4.1.1 Runtime Semantics: PerformPromiseAll

    [...]
    6. Repeat
        a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
        b. If next is an abrupt completion, set iteratorRecord.[[done]] to
           true.
        c. ReturnIfAbrupt(next).
features: [Symbol.iterator]
flags: [async]
---*/

var iterStepThrows = {};
var poisonedDone = {};
var error = new Test262Error();
Object.defineProperty(poisonedDone, 'done', {
  get: function() {
    throw error;
  }
});
Object.defineProperty(poisonedDone, 'value', {
  get: function() {
    $DONE('The `value` property should not be accessed.');
  }
});

iterStepThrows[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedDone;
    }
  };
};

Promise.all(iterStepThrows).then(function() {
  $DONE('The promise should be rejected.');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
