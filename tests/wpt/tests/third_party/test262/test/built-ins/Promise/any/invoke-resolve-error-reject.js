// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise rejection in response to error
esid: sec-promise.any
info: |
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  6. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  8. Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).

flags: [async]
features: [Promise.any, arrow-function]
---*/

let error = new Test262Error();
Promise.resolve = function() {
  throw error;
};

Promise.any([1]).then(() => {
    $DONE('The promise should be rejected, but was resolved');
  }, (reason) => {
    assert.sameValue(reason, error);
  }).then($DONE, $DONE);
