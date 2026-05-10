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
  const items = [1, 2, 3];
  const promise = Array.fromAsync(items);
  // At this point, the first element of `items` has been read, but the iterator will take other
  // changes into account.
  items.pop();
  const result = await promise;
  assert.compareArray(result, [1, 2]);
});
