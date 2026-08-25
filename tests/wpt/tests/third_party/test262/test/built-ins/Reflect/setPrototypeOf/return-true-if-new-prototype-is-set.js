// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.14
description: >
  Return true if the new prototype is set.
info: |
  26.1.14 Reflect.setPrototypeOf ( target, proto )

  ...
  3. Return target.[[SetPrototypeOf]](proto).

  9.1.2 [[SetPrototypeOf]] (V)

  ...
  9. Set the value of the [[Prototype]] internal slot of O to V.
  10. Return true.
  ...
features: [Reflect, Reflect.setPrototypeOf]
---*/

var o1 = {};
assert.sameValue(Reflect.setPrototypeOf(o1, null), true);
assert.sameValue(Object.getPrototypeOf(o1), null);

var o2 = Object.create(null);
assert.sameValue(Reflect.setPrototypeOf(o2, Object.prototype), true);
assert.sameValue(Object.getPrototypeOf(o2), Object.prototype);

var o3 = {};
var proto = {};
assert.sameValue(Reflect.setPrototypeOf(o3, proto), true);
assert.sameValue(Object.getPrototypeOf(o3), proto);
