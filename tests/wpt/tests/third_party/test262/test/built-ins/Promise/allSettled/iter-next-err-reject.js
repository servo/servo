// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: >
  Error when call an iterator next step (rejecting promise)
info: |
  Promise.allSettled ( iterable )

  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  ...
  6. Repeat
    a. Let next be IteratorStep(iteratorRecord).
    b. If next is an abrupt completion, set iteratorRecord.[[Done]] to true.
    c. ReturnIfAbrupt(next).
    ...

  IteratorStep ( iteratorRecord )

  1. Let result be ? IteratorNext(iteratorRecord).

  IteratorNext ( iteratorRecord [ , value ] )

  1. If value is not present, then
    a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]], « »).
  2. Else,
    a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]], « value »).
  ...
features: [Promise.allSettled, Symbol.iterator]
flags: [async]
---*/

var iterNextValThrows = {};
var error = new Test262Error();
iterNextValThrows[Symbol.iterator] = function() {
  return {
    next() {
      throw error;
    }
  };
};

Promise.allSettled(iterNextValThrows).then(function() {
  $DONE('The promise should be rejected.');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
