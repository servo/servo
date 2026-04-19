// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `Promise.resolve` invoked with an object whose `then` property is not callable
es6id: 25.4.4.5
info: |
    6. Let resolveResult be Call(promiseCapability.[[Resolve]], undefined,
       «x»).

    [...]

    25.4.1.3.2 Promise Resolve Functions

    11. If IsCallable(thenAction) is false, then
        a. Return FulfillPromise(promise, resolution).
    12. Perform EnqueueJob ("PromiseJobs", PromiseResolveThenableJob,
        «promise, resolution, thenAction»)
    13. Return undefined.
flags: [async]
---*/

var nonThenable = {
  then: null
};

Promise.resolve(nonThenable).then(function(value) {
  assert.sameValue(value, nonThenable);
}).then($DONE, $DONE);
