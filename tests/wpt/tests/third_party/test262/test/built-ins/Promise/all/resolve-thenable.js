// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a thenable object value
es6id: 25.4.4.1
info: |
    [...]
    6. Let promiseCapability be NewPromiseCapability(C).
    [...]
    11. Let result be PerformPromiseAll(iteratorRecord, C, promiseCapability).
    [...]

    25.4.4.1.1 Runtime Semantics: PerformPromiseAll
    [...]
    6. Repeat
       [...]
       d. If next is false,
          [...]
          iii. If remainingElementsCount.[[value]] is 0,
             1. Let valuesArray be CreateArrayFromList(values).
             2. Let resolveResult be Call(resultCapability.[[Resolve]],
                undefined, «valuesArray»).
             3. ReturnIfAbrupt(resolveResult)
          iv. Return resultCapability.[[Promise]].

    7.3.16 CreateArrayFromList (elements)
    [...]
    2. Let array be ArrayCreate(0) (see 9.4.2.2).

    9.4.2.2 ArrayCreate(length, proto)
    [...]
    4. If the proto argument was not passed, let proto be the intrinsic object
       %ArrayPrototype%.
    5. Let A be a newly created Array exotic object.
    [...]
    8. Set the [[Prototype]] internal slot of A to proto.

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
flags: [async]
---*/

var value = {};
var promise;

try {
  Array.prototype.then = function(resolve) {
    resolve(value);
  };

  promise = Promise.all([]);
} finally {
  delete Array.prototype.then;
}

promise.then(function(val) {
  if (val !== value) {
    $DONE('The promise should be resolved with the expected value.');
    return;
  }

  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});
