// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a non-thenable object value
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
        a. Return FulfillPromise(promise, resolution).
flags: [async]
---*/

var value = {};

Promise.resolve(value).then(function(value) {
  if (value !== value) {
    $DONE('The promise should be fulfilled with the provided value.');
    return;
  }

  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});
