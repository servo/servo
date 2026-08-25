// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise rejection in response to error
esid: sec-promise.all
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
        i. Let nextPromise be Invoke(constructor, "resolve", «nextValue»).
        j. ReturnIfAbrupt(nextPromise ).
flags: [async]
---*/

var thrown = new Test262Error();
Promise.resolve = function() {
  throw thrown;
};

Promise.all([1])
  .then(function() {
    throw new Test262Error('The promise should not be fulfilled.');
  }, function(reason) {
    if (reason !== thrown) {
      throw new Test262Error('The promise should be rejected with the thrown error object');
    }
  }).then($DONE, $DONE);
