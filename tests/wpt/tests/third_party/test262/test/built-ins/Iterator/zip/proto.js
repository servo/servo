// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  The value of the [[Prototype]] internal slot of Iterator.zip is the
  intrinsic object %FunctionPrototype%.
features: [joint-iteration]
---*/

assert.sameValue(
  Object.getPrototypeOf(Iterator.zip),
  Function.prototype,
  "Object.getPrototypeOf(Iterator.zip) must return the value of Function.prototype"
);
