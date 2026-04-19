// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.any
description: Rejecting through immediate invocation of the provided resolving function
info: |
    ...
    Let promiseCapability be NewPromiseCapability(C).
    ...
    Let result be PerformPromiseAny(iteratorRecord, promiseCapability, C).
    ...

    Runtime Semantics: PerformPromiseAny
    ...
    8. Repeat
       ...
       r. Perform ? Invoke(nextPromise, "then",
          « resultCapability.[[Resolve]], rejectElement »).


    Promise.any Reject Element Functions
    ...
    6. Return RejectPromise(promise, reason).
flags: [async]
features: [Promise.any, arrow-function]
---*/

let callCount = 0;
let thenable = {
  then(_, reject) {
    callCount++;
    reject('reason');
  }
};

Promise.any([thenable])
  .then(() => {
    $DONE('The promise should not be fulfilled.');
  }, (error) => {
    assert.sameValue(callCount, 1, "callCount === 1");
    assert(error instanceof AggregateError, "error instanceof AggregateError");
    assert.sameValue(error.errors[0], "reason", "error.errors[0] === 'reason'");
    $DONE();
  });
