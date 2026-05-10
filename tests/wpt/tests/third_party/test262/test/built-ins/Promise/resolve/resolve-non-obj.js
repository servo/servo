// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a non-object value
es6id: 25.4.4.5
info: |
    [...]
    6. Let resolveResult be Call(promiseCapability.[[Resolve]], undefined,
       «x»).
    [...]

    25.4.1.3.2 Promise Resolve Functions
    [...]
    7. If Type(resolution) is not Object, then
       a. Return FulfillPromise(promise, resolution).
flags: [async]
---*/

Promise.resolve(23).then(function(value) {
  if (value !== 23) {
    $DONE('The promise should be fulfilled with the provided value.');
    return;
  }

  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});
