// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: >
  Resolution is a collection of all the settled values (all fulfilled)
info: |
  Runtime Semantics: PerformPromiseAllSettled

  6. Repeat,
    ...
    j. Let steps be the algorithm steps defined in Promise.allSettled Resolve Element Functions.
    k. Let resolveElement be ! CreateBuiltinFunction(steps, « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
    ...
    r. Let rejectSteps be the algorithm steps defined in Promise.allSettled Reject Element Functions.
    s. Let rejectElement be ! CreateBuiltinFunction(rejectSteps, « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
    ...
    z. Perform ? Invoke(nextPromise, "then", « resolveElement, rejectElement »).

  Promise.allSettled Resolve Element Functions

  9. Let obj be ! ObjectCreate(%ObjectPrototype%).
  10. Perform ! CreateDataProperty(obj, "status", "fulfilled").
  11. Perform ! CreateDataProperty(obj, "value", x).
  12. Set values[index] to be obj.
  13. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
  14. If remainingElementsCount.[[Value]] is 0, then
    a. Let valuesArray be ! CreateArrayFromList(values).
    b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).

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
includes: [promiseHelper.js]
features: [Promise.allSettled]
---*/

var obj = {};
var p1 = new Promise(function(resolve) {
  resolve(1);
});
var p2 = new Promise(function(resolve) {
  resolve('test262');
});
var p3 = new Promise(function(resolve) {
  resolve(obj);
});

Promise.allSettled([p1, p2, p3]).then(function(settled) {
  checkSettledPromises(settled, [
    { status: 'fulfilled', value: 1 },
    { status: 'fulfilled', value: 'test262' },
    { status: 'fulfilled', value: obj }
  ], 'settled');
}).then($DONE, $DONE);
