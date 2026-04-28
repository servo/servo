// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%iteratorprototype%-@@iterator
description: Return value of @@iterator on %IteratorPrototype%
info: |
  %IteratorPrototype% [ @@iterator ] ( )
    1. Return the this value.
features: [Symbol.iterator]
---*/
const IteratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);

const getIterator = IteratorPrototype[Symbol.iterator];

const thisValues = [{}, Symbol(), 4, 4n, true, undefined, null];

for (const thisValue of thisValues) {
  assert.sameValue(getIterator.call(thisValue), thisValue);
}
