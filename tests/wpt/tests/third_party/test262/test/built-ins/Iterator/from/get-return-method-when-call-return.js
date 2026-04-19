// Copyright (C) 2024 Sosuke Suzuki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  Gets the base iterator return method when the wrapper return method is called.
info: |
  %WrapForValidIteratorPrototype%.return ( )
    ...
    5. Let returnMethod be ? GetMethod(iterator, "return").

features: [iterator-helpers]
includes: [temporalHelpers.js, compareArray.js]
---*/

const calls = [];

const iter = TemporalHelpers.propertyBagObserver(calls, {
  return () {
    return { value: 5, done: true };
  },
}, "originalIter");

const wrapper = Iterator.from(iter);
assert.compareArray(calls, [
  "get originalIter[Symbol.iterator]",
  "get originalIter.next",
]);

wrapper.return();
assert.compareArray(calls, [
  "get originalIter[Symbol.iterator]",
  "get originalIter.next",
  "get originalIter.return"
]);
