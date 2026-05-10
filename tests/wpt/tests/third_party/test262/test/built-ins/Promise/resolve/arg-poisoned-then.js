// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `Promise.resolve` invoked with an object with a "poisoned" `then` property
es6id: 25.4.4.5
info: |
    6. Let resolveResult be Call(promiseCapability.[[Resolve]], undefined,
       «x»).

    [...]

    25.4.1.3.2 Promise Resolve Functions

    8. Let then be Get(resolution, "then").
    9. If then is an abrupt completion, then
       a. Return RejectPromise(promise, then.[[value]]).
flags: [async]
---*/

var poisonedThen = {};
var err = new Test262Error();
Object.defineProperty(poisonedThen, 'then', {
  get: function() {
    throw err;
  }
});

Promise.resolve(poisonedThen).then(function() {
  throw new Test262Error(
    'Promise should be rejected when retrieving `then` property throws an error'
  );
}, function(reason) {
  assert.sameValue(reason, err);
}).then($DONE, $DONE);
