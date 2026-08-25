// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.1.1
description: >
  Indexed setter properties on Array.prototype are not invoked.
info: |
  Runtime Semantics: PerformPromiseAll( iteratorRecord, constructor, resultCapability)

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
---*/

Object.defineProperty(Array.prototype, 0, {
  set: function() {
    throw new Test262Error("Setter on Array.prototype called");
  }
});

Promise.all([42]).then(function() {
  $DONE();
}, $DONE);
