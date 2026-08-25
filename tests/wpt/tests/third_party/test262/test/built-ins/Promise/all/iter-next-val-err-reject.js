// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.all
description: >
  Error when accessing an iterator result's `value` property (rejecting
  promise)
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
        [...]
        e. Let nextValue be IteratorValue(next).
        f. If nextValue is an abrupt completion, set iteratorRecord.[[done]] to
           true.
        g. ReturnIfAbrupt(nextValue).
features: [Symbol.iterator]
flags: [async]
---*/

var iterNextValThrows = {};
var poisonedVal = {
  done: false
};
var error = new Test262Error();
Object.defineProperty(poisonedVal, 'value', {
  get: function() {
    throw error;
  }
});
iterNextValThrows[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedVal;
    }
  };
};

Promise.all(iterNextValThrows).then(function() {
  $DONE('The promise should be rejected.');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
