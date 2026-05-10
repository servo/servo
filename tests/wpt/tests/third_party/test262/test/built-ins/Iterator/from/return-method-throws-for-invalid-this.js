// Copyright (C) 2024 Sosuke Suzuki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  %WrapForValidIteratorPrototype%.return() requires [[iterated]] internal slot
info: |
  %WrapForValidIteratorPrototype%.return ( )
    ...
    2. Perform ? RequireInternalSlot(O, [[Iterated]]).

features: [iterator-helpers]
includes: [temporalHelpers.js, compareArray.js]
---*/

const WrapForValidIteratorPrototype = Object.getPrototypeOf(Iterator.from({}));

{
  assert.throws(TypeError, function() {
      WrapForValidIteratorPrototype.return.call({});
  });
}

{
  const originalIter = {
    return() {
      return { value: 5, done: true };
    },
  };

  const calls = [];
  TemporalHelpers.observeMethod(calls, originalIter, "return", "originalIter");
  const iter = TemporalHelpers.propertyBagObserver(calls, originalIter, "originalIter");

  assert.throws(TypeError, function() {
      WrapForValidIteratorPrototype.return.call(iter);
  });
  assert.compareArray(calls, []);
}
