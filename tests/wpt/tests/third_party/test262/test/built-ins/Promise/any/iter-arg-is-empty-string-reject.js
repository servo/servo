// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Promise.any('') rejects with AggregateError, empty errors array.
info: |
  Runtime Semantics: PerformPromiseAny ( iteratorRecord, constructor, resultCapability )

  ...
  3. Let errors be a new empty List.
  ...
  8. Repeat,
    a. Let next be IteratorStep(iteratorRecord).
    b. If next is an abrupt completion, set iteratorRecord.[[Done]] to true.
    c. ReturnIfAbrupt(next).
    d. If next is false, then
      i. Set iteratorRecord.[[Done]] to true.
      ii. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
      iii. If remainingElementsCount.[[Value]] is 0, then
        1. Let error be a newly created AggregateError object.
        2. Set error.[[AggregateErrors]] to errors.
        3. Return ThrowCompletion(error).
  ...

features: [AggregateError, Promise.any, arrow-function]
flags: [async]
---*/

Promise.any('')
  .then(
    () => $DONE('The promise should be rejected, but was resolved'),
    error => {
      assert.sameValue(Object.getPrototypeOf(error), AggregateError.prototype);
      assert(error instanceof AggregateError);
      assert.sameValue(error.errors.length, 0);
    }
  ).then($DONE, $DONE);
