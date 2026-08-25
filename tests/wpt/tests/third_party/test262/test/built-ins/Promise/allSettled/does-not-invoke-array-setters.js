// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallsettled
description: >
  Indexed setter properties on Array.prototype are not invoked.
info: |
  Promise.allSettled ( iterable )

  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b, IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled ( iteratorRecord, constructor, resultCapability )

  ...
  4. Let remainingElementsCount be a new Record { [[value]]: 1 }.
  ...
  6.d ...
    ii. Set remainingElementsCount.[[value]] to remainingElementsCount.[[value]] − 1.
    iii. If remainingElementsCount.[[value]] is 0,
      1. Let valuesArray be CreateArrayFromList(values).
      ...
  ...

  7.3.16 CreateArrayFromList (elements)
    ...
    4. For each element e of elements
      a. Let status be CreateDataProperty(array, ToString(n), e).
      b. Assert: status is true.
    ...
flags: [async]
features: [Promise.allSettled]
---*/

Object.defineProperty(Array.prototype, 0, {
  set() {
    throw new Test262Error('Setter on Array.prototype called');
  }
});

Promise.allSettled([42]).then(function() {
  $DONE();
}, $DONE);
