// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-iterator-constructor
description: >
  The value of the [[Prototype]] internal slot of the Iterator constructor is the
  intrinsic object %FunctionPrototype%.
features: [iterator-helpers]
---*/

assert.sameValue(
  Object.getPrototypeOf(Iterator),
  Function.prototype,
  'Object.getPrototypeOf(Iterator) must return the value of Function.prototype'
);
