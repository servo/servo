// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  The value of the [[Prototype]] internal slot of the return value of Iterator.zip
  is the intrinsic object %IteratorHelperPrototype%.
includes: [wellKnownIntrinsicObjects.js]
features: [joint-iteration]
---*/

var iter = Iterator.zip([]);
assert(iter instanceof Iterator, "Iterator.zip([]) must return an Iterator");

assert.sameValue(
  Object.getPrototypeOf(iter),
  getWellKnownIntrinsicObject("%IteratorHelperPrototype%"),
  "[[Prototype]] is %IteratorHelperPrototype%"
);
