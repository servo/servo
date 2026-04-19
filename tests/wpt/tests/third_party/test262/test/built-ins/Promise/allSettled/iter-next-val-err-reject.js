// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: >
  Error when accessing an iterator result's `value` property (rejecting promise)
info: |
  Promise.allSettled ( iterable )

  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  ...
  6. Repeat
    ...
    e. Let nextValue be IteratorValue(next).
    f. If nextValue is an abrupt completion, set iteratorRecord.[[Done]] to true.
    g. ReturnIfAbrupt(nextValue).
features: [Promise.allSettled, Symbol.iterator]
flags: [async]
---*/

var iterNextValThrows = {};
var poisonedVal = {
  done: false
};
var error = new Test262Error();
Object.defineProperty(poisonedVal, 'value', {
  get() {
    throw error;
  }
});
iterNextValThrows[Symbol.iterator] = function() {
  return {
    next() {
      return poisonedVal;
    }
  };
};

Promise.allSettled(iterNextValThrows).then(function() {
  $DONE('The promise should be rejected.');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
