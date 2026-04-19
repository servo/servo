// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-finalization-registry-constructor
description: >
  The prototype of FinalizationRegistry is Object.prototype
info: |
  The value of the [[Prototype]] internal slot of the FinalizationRegistry object is the
  intrinsic object %FunctionPrototype%.
features: [FinalizationRegistry]
---*/

assert.sameValue(
  Object.getPrototypeOf(FinalizationRegistry),
  Function.prototype,
  'Object.getPrototypeOf(FinalizationRegistry) returns the value of `Function.prototype`'
);
