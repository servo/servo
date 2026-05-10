// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Rejecting through deferred invocation of the provided resolving function
esid: sec-promise.allsettled
info: |
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled
  
  6. Repeat
    ...
    z. Perform ? Invoke(nextPromise, "then", « resolveElement, rejectElement »).

  Promise.allSettled Reject Element Functions

  9. Let obj be ! ObjectCreate(%ObjectPrototype%).
  10. Perform ! CreateDataProperty(obj, "status", "rejected").
  11. Perform ! CreateDataProperty(obj, "reason", x).
  12. Set values[index] to be obj.
  13. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
  14. If remainingElementsCount.[[Value]] is 0, then
    a. Let valuesArray be CreateArrayFromList(values).
    b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).
flags: [async]
features: [Promise.allSettled]
---*/

var simulation = {};
var thenable = {
  then(_, reject) {
    new Promise(function(resolve) {
        resolve();
      })
      .then(function() {
        reject(simulation);
      });
  }
};

Promise.allSettled([thenable])
  .then((settleds) => {
    assert.sameValue(settleds.length, 1);
    assert.sameValue(settleds[0].status, 'rejected');
    assert.sameValue(settleds[0].reason, simulation);
  }).then($DONE, $DONE);
