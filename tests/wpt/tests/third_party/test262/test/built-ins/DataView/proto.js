// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-dataview-constructor
description: >
  The prototype of DataView is Function.prototype
info: |
  The value of the [[Prototype]] internal slot of the DataView constructor is
  the intrinsic object %FunctionPrototype%.
---*/

assert.sameValue(Object.getPrototypeOf(DataView), Function.prototype);
