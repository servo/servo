// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with an object with a "poisoned" `then` property
esid: sec-promise.allsettled
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

var value = {};
var promise;

try {
  Object.defineProperty(Array.prototype, 'then', {
    get() {
      throw value;
    },
    configurable: true
  });

  promise = Promise.allSettled([]);
} finally {
  delete Array.prototype.then;
}

promise.then(function() {
  $DONE('The promise should not be fulfilled.');
}, function(val) {
  if (val !== value) {
    $DONE('The promise should be rejected with the expected value.');
    return;
  }

  $DONE();
});
