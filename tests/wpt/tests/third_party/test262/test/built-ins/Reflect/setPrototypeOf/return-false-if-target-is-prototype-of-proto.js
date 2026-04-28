// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.14
description: >
  Return false if target is found as a prototype of proto, without setting.
info: |
  26.1.14 Reflect.setPrototypeOf ( target, proto )

  ...
  3. Return target.[[SetPrototypeOf]](proto).

  9.1.2 [[SetPrototypeOf]] (V)

  ...
  8. Repeat while done is false,
    a. If p is null, let done be true.
    b. Else, if SameValue(p, O) is true, return false.
    c. Else,
      i. If the [[GetPrototypeOf]] internal method of p is not the ordinary
      object internal method defined in 9.1.1, let done be true.
      ii. Else, let p be the value of p’s [[Prototype]] internal slot.
  ...
features: [Reflect, Reflect.setPrototypeOf]
---*/

var target = {};
var proto = Object.create(target);
assert.sameValue(Reflect.setPrototypeOf(target, proto), false);
assert.sameValue(Object.getPrototypeOf(target), Object.prototype);
