// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.14
description: >
  Return true if proto has the same value as current target's prototype.
info: |
  26.1.14 Reflect.setPrototypeOf ( target, proto )

  ...
  3. Return target.[[SetPrototypeOf]](proto).

  9.1.2 [[SetPrototypeOf]] (V)

  ...
  4. If SameValue(V, current), return true.
  ...
features: [Reflect, Reflect.setPrototypeOf]
---*/

var o1 = {};
Object.preventExtensions(o1);
assert.sameValue(Reflect.setPrototypeOf(o1, Object.prototype), true);

var o2 = Object.create(null);
Object.preventExtensions(o2);
assert.sameValue(Reflect.setPrototypeOf(o2, null), true);

var proto = {};
var o3 = Object.create(proto);
Object.preventExtensions(o3);
assert.sameValue(Reflect.setPrototypeOf(o3, proto), true);
