// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  The value of the [[Prototype]] internal slot of Iterator.zipKeyed is the
  intrinsic object %FunctionPrototype%.
features: [joint-iteration]
---*/

assert.sameValue(
  Object.getPrototypeOf(Iterator.zipKeyed),
  Function.prototype,
  "Object.getPrototypeOf(Iterator.zipKeyed) must return the value of Function.prototype"
);
