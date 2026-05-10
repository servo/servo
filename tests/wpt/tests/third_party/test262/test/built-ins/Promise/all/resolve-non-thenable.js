// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a non-thenable object value
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

var v1 = {};
var v2 = {};
var v3 = {};

Promise.all([v1, v2, v3])
  .then(function(values) {
    if (!values) {
      $DONE('The promise should be resolved with a value.');
      return;
    }
    if (values.constructor !== Array) {
      $DONE('The promise should be resolved with an Array instance.');
      return;
    }

    if (values.length !== 3) {
      $DONE('The promise should be resolved with an array of proper length.');
      return;
    }

    if (values[0] !== v1) {
      $DONE('The promise should be resolved with the correct element values (#1)');
      return;
    }

    if (values[1] !== v2) {
      $DONE('The promise should be resolved with the correct element values (#2)');
      return;
    }

    if (values[2] !== v3) {
      $DONE('The promise should be resolved with the correct element values (#3)');
      return;
    }

    $DONE();
  }, function() {
    $DONE('The promise should not be rejected.');
  });
