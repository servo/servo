// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a resolved Promise instance whose `then` method has been overridden
es6id: 25.4.4.5
info: |
    [...]
    6. Let resolveResult be Call(promiseCapability.[[Resolve]], undefined,
       «x»).
    [...]

    25.4.1.3.2 Promise Resolve Functions
    [...]
    8. Let then be Get(resolution, "then").
    9. If then is an abrupt completion, then
       [...]
    10. Let thenAction be then.[[value]].
    11. If IsCallable(thenAction) is false, then
        [...]
    12. Perform EnqueueJob ("PromiseJobs", PromiseResolveThenableJob,
        «promise, resolution, thenAction»)
---*/

var value = {};
var rejectCallCount = 0;
var thenable = new Promise(function(resolve) {
  resolve();
});
var resolvedValue;

thenable.then = function(resolve) {
  resolve(value);
};

Promise.resolve(thenable).then(function(val) {
  resolvedValue = val;
}, function() {
  rejectCallCount += 1;
});

assert.sameValue(resolvedValue, value);
assert.sameValue(rejectCallCount, 0);
