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

var p0 = Promise.resolve(2).then(v => v + 1);
var p1 = Promise.reject(21).catch(v => v * 2);
var p2 = Promise.resolve('nope').then(() => { throw 'foo' });
var p3 = Promise.reject('yes').then(() => { throw 'nope'; });
var p4 = Promise.resolve('here').finally(() => 'nope');
var p5 = Promise.reject('here too').finally(() => 'nope');
var p6 = Promise.resolve('nope').finally(() => { throw 'finally'; });
var p7 = Promise.reject('nope').finally(() => { throw 'finally after rejected'; });
var p8 = Promise.reject(1).then(() => 'nope', () => 0);

Promise.allSettled([p0, p1, p2, p3, p4, p5, p6, p7, p8]).then(function(settled) {
  checkSettledPromises(settled, [
    { status: 'fulfilled', value: 3 },
    { status: 'fulfilled', value: 42 },
    { status: 'rejected', reason: 'foo' },
    { status: 'rejected', reason: 'yes' },
    { status: 'fulfilled', value: 'here' },
    { status: 'rejected', reason: 'here too' },
    { status: 'rejected', reason: 'finally' },
    { status: 'rejected', reason: 'finally after rejected' },
    { status: 'fulfilled', value: 0 },
  ], 'settled');
}).then($DONE, $DONE);
