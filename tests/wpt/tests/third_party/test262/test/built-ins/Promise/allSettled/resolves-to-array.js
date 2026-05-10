// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: Promise.allSettled returned a promise resolves into an array
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
flags: [async]
features: [Promise.allSettled]
---*/

var arg = [];

Promise.allSettled([]).then(function(result) {
  assert(Array.isArray(result));
  assert.sameValue(Object.getPrototypeOf(result), Array.prototype);
  assert.notSameValue(result, arg, 'the resolved array is a new array');
}).then($DONE, $DONE);
