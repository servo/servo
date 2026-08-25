// Copyright (C) 2024 Sosuke Suzuki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  %WrapForValidIteratorPrototype%.return() call base iterator's return method when it exists.
info: |
  %WrapForValidIteratorPrototype%.return ( )
    5. Let returnMethod be ? GetMethod(iterator, "return").
    6. If returnMethod is undefined, then
      ...
    7. Return ? Call(returnMethod, iterator).

features: [iterator-helpers]
includes: [temporalHelpers.js, compareArray.js]
---*/

const calls = [];

const expectedIteratorResult = { value: 5, done: true };
const originalIter = {
    return () {
        return expectedIteratorResult;
    },
};
TemporalHelpers.observeMethod(calls, originalIter, "return", "originalIter");
const iter = TemporalHelpers.propertyBagObserver(calls, originalIter, "originalIter");

const wrapper = Iterator.from(iter);
assert.compareArray(calls, [
  "get originalIter[Symbol.iterator]",
  "get originalIter.next",
]);

assert.sameValue(wrapper.return(), expectedIteratorResult);
assert.compareArray(calls, [
  "get originalIter[Symbol.iterator]",
  "get originalIter.next",
  "get originalIter.return",
  "call originalIter.return",
]);
