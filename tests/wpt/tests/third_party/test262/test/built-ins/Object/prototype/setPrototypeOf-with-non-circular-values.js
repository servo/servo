// Copyright (C) 2025 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-immutable-prototype-exotic-objects-setprototypeof-v
description: >
  Object.prototype's [[SetPrototypeOf]] returns false even in cases where
  OrdinarySetPrototypeOf(O, V) would return true
info: |
  9.4.7.1 [[SetPrototypeOf]] (V)

  ...
  2. Let current be the value of the [[Prototype]] internal slot of O.
  3. If SameValue(V, current), return true.
  4. Return false.

  19.1.3 Properties of the Object Prototype Object

  The value of the [[Prototype]] internal slot of the Object prototype object is
  null and the initial value of the [[Extensible]] internal slot is true.
features: [Reflect.setPrototypeOf]
---*/

var ObjProto = Object.prototype;

assert.throws(TypeError, function() {
  Object.setPrototypeOf(ObjProto, Object.create(null));
}, "Object.setPrototypeOf(ObjProto, Object.create(null)) throws a TypeError");

assert.sameValue(
  Reflect.setPrototypeOf(ObjProto, Object.create(null)),
  false,
  "Reflect.setPrototypeOf(ObjProto, Object.create(null)) returns false"
);
