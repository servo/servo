// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallsettled
description: >
  Cannot tamper remainingElementsCount when Promise.allSettled resolve element function is called multiple times.
info: |
  Runtime Semantics: PerformPromiseAllSettled ( iteratorRecord, constructor, resultCapability )

  4. Let remainingElementsCount be a new Record { [[Value]]: 1 }.
  ...
  6.d ...
    ii. Set remainingElementsCount.[[value]] to remainingElementsCount.[[value]] − 1.
    iii. If remainingElementsCount.[[value]] is 0, then
      1. Let valuesArray be CreateArrayFromList(values).
      2. Perform ? Call(resultCapability.[[Resolve]], undefined, « valuesArray »).
  ...

  Promise.allSettled Resolve Element Functions

  2. Let alreadyCalled be F.[[AlreadyCalled]].
  3. If alreadyCalled.[[Value]] is true, return undefined.
  4. Set alreadyCalled.[[Value]] to true.
  ...
includes: [promiseHelper.js]
features: [Promise.allSettled]
---*/

var callCount = 0;

function Constructor(executor) {
  function resolve(values) {
    callCount += 1;
    checkSettledPromises(values, [
      {
        status: 'fulfilled',
        value: 'p1-fulfill'
      },
      {
        status: 'fulfilled',
        value: 'p2-fulfill'
      },
      {
        status: 'fulfilled',
        value: 'p3-fulfill'
      }
    ], 'values');
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

var p1OnFulfilled, p2OnFulfilled, p3OnFulfilled;

var p1 = {
  then(onFulfilled, onRejected) {
    p1OnFulfilled = onFulfilled;
  }
};
var p2 = {
  then(onFulfilled, onRejected) {
    p2OnFulfilled = onFulfilled;
  }
};
var p3 = {
  then(onFulfilled, onRejected) {
    p3OnFulfilled = onFulfilled;
  }
};

assert.sameValue(callCount, 0, 'callCount before call to allSettled()');

Promise.allSettled.call(Constructor, [p1, p2, p3]);

assert.sameValue(callCount, 0, 'callCount after call to allSettled()');

p1OnFulfilled('p1-fulfill');
p1OnFulfilled('p1-fulfill-unexpected-1');
p1OnFulfilled('p1-fulfill-unexpected-2');

assert.sameValue(callCount, 0, 'callCount after resolving p1');

p2OnFulfilled('p2-fulfill');
p3OnFulfilled('p3-fulfill');

assert.sameValue(callCount, 1, 'callCount after resolving all elements');
