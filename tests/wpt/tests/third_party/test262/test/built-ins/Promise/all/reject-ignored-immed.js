// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolved promises ignore rejections through immediate invocation of the
    provided resolving function
es6id: 25.4.4.1
info: |
    [...]
    6. Let promiseCapability be NewPromiseCapability(C).
    [...]
    11. Let result be PerformPromiseAll(iteratorRecord, promiseCapability, C).
    [...]

    25.4.4.1.1 Runtime Semantics: PerformPromiseAll
    [...]
    6. Repeat
       [...]
       r. Let result be Invoke(nextPromise, "then", resolveElement,
          promiseCapability.[[Reject]]»).

    25.4.1.3.1 Promise Reject Functions
    [...]
    2. Let promise be the value of F's [[Promise]] internal slot.
    3. Let alreadyResolved be the value of F's [[AlreadyResolved]] internal
       slot.
    4. If alreadyResolved.[[value]] is true, return undefined.
flags: [async]
---*/

var fulfiller = {
  then: function(resolve) {
    resolve();
  }
};
var lateRejector = {
  then: function(resolve, reject) {
    resolve();
    reject();
  }
};

Promise.all([fulfiller, lateRejector])
  .then(function() {
    $DONE();
  }, function() {
    $DONE('The promise should not be rejected.');
  });
