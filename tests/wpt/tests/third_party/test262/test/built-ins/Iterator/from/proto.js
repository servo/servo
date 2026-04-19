// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  The value of the [[Prototype]] internal slot of Iterator.from is the
  intrinsic object %FunctionPrototype%.
features: [iterator-helpers]
---*/

assert.sameValue(
  Object.getPrototypeOf(Iterator.from),
  Function.prototype,
  'Object.getPrototypeOf(Iterator.from) must return the value of Function.prototype'
);
