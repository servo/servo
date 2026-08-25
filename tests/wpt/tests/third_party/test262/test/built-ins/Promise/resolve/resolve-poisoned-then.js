// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with an object with a "poisoned" then property
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
       a. Return RejectPromise(promise, then.[[value]]).
flags: [async]
---*/

var value = {};
var resolve;
var poisonedThen = Object.defineProperty({}, 'then', {
  get: function() {
    throw value;
  }
});

Promise.resolve(poisonedThen).then(function() {
  $DONE('The promise should not be fulfilled.');
}, function(val) {
  if (val !== value) {
    $DONE('The promise should be rejected with the provided value.');
    return;
  }

  $DONE();
});
