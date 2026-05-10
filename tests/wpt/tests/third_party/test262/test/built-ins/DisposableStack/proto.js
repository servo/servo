// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-disposablestack-constructor
description: >
  The prototype of DisposableStack is Function.prototype
info: |
  The value of the [[Prototype]] internal slot of the DisposableStack object is the
  intrinsic object %FunctionPrototype%.
features: [explicit-resource-management]
---*/

assert.sameValue(
  Object.getPrototypeOf(DisposableStack),
  Function.prototype,
  'Object.getPrototypeOf(DisposableStack) returns the value of `Function.prototype`'
);
