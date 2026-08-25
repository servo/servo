// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  The value of the [[Prototype]] internal slot of Iterator.concat is the
  intrinsic object %FunctionPrototype%.
features: [iterator-sequencing]
---*/

assert.sameValue(
  Object.getPrototypeOf(Iterator.concat),
  Function.prototype,
  "Object.getPrototypeOf(Iterator.concat) must return the value of Function.prototype"
);
