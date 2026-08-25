// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-promise-prototype-object
description: Promise.prototype [[Prototype]] is %ObjectPrototype%
info: |
  The Promise prototype object is the intrinsic object %PromisePrototype%. The
  value of the [[Prototype]] internal slot of the Promise prototype object is
  the intrinsic object %ObjectPrototype%. The Promise prototype object is an
  ordinary object. It does not have a [[PromiseState]] internal slot or any of
  the other internal slots of Promise instances.
---*/

assert.sameValue(Object.getPrototypeOf(Promise.prototype), Object.prototype);
