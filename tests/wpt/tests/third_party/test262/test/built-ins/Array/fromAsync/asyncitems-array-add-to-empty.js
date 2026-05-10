// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync respects array mutation
info: |
  Array.fromAsync
    3.j.ii.3. Let next be ? Await(IteratorStep(iteratorRecord)).

  IteratorStep
    1. Let result be ? IteratorNext(iteratorRecord).

  IteratorNext
    1.a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).

  %AsyncFromSyncIteratorPrototype%.next
    6.a. Let result be Completion(IteratorNext(syncIteratorRecord)).

  IteratorNext
    1.a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).

  Array.prototype [ @@iterator ] ( )
  Array.prototype.values ( )
    2. Return CreateArrayIterator(O, value).

  CreateArrayIterator
    1.b.iii. If index ≥ len, return NormalCompletion(undefined).
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const items = [];
  const promise = Array.fromAsync(items);
  // By the time we get here, the first next() call has already happened, and returned
  // { done: true }. We then return from the loop in Array.fromAsync 3.j.ii. with the empty array,
  // and the following line no longer affects that.
  items.push(7);
  const result = await promise;
  assert.compareArray(result, []);
});
