// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics-object
description: >
  The prototype of Atomics is Object.prototype
info: |
  The Atomics Object

  The value of the [[Prototype]] internal slot of the Atomics object is the
  intrinsic object %ObjectPrototype%.
features: [Atomics]
---*/

assert.sameValue(
  Object.getPrototypeOf(Atomics),
  Object.prototype,
  'Object.getPrototypeOf(Atomics) returns the value of `Object.prototype`'
);
