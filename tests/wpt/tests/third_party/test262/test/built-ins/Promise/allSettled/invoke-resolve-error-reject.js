// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise rejection in response to error
esid: sec-promise.allsettled
info: |
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  6. Repeat
    ...
    i. Let nextPromise be ? Invoke(constructor, "resolve", « nextValue »).
flags: [async]
features: [Promise.allSettled]
---*/

var thrown = new Test262Error();
Promise.resolve = function() {
  throw thrown;
};

Promise.allSettled([1])
  .then(function() {
    throw new Test262Error('The promise should not be fulfilled.');
  }, function(reason) {
    if (reason !== thrown) {
      throw new Test262Error('The promise should be rejected with the thrown error object');
    }
  }).then($DONE, $DONE);
