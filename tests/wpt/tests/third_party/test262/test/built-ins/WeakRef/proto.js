// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-weak-ref-constructor
description: >
  The prototype of WeakRef is Object.prototype
info: |
  The value of the [[Prototype]] internal slot of the WeakRef object is the
  intrinsic object %FunctionPrototype%.
features: [WeakRef]
---*/

assert.sameValue(
  Object.getPrototypeOf(WeakRef),
  Function.prototype,
  'Object.getPrototypeOf(WeakRef) returns the value of `Function.prototype`'
);
