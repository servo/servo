// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    [...]
    5. Let rejectResult be Call(promiseCapability.[[Reject]], undefined, «r»).
    [...]

    25.4.1.3.1 Promise Reject Functions
    [...]
    6. Return RejectPromise(promise, reason).
es6id: 25.4.4.4
author: Sam Mikes
description: Promise.reject creates a new settled promise
flags: [async]
---*/

var p = Promise.reject(3);

assert(!!(p instanceof Promise), 'The value of !!(p instanceof Promise) is expected to be true');

p.then(function() {
  throw new Test262Error("Promise should not be fulfilled.");
}, function(result) {
  assert.sameValue(result, 3, 'The value of result is expected to be 3');
}).then($DONE, $DONE);
