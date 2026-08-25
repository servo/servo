// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-immutable-prototype-exotic-objects-setprototypeof-v
description: >
  Object.prototype's [[SetPrototypeOf]] returns false if value is not the same
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
  Object.setPrototypeOf(ObjProto, {});
}, "Object.setPrototypeOf(ObjProto, {}) throws a TypeError");

assert.throws(TypeError, function() {
  Object.setPrototypeOf(ObjProto, Array.prototype);
}, "Object.setPrototypeOf(ObjProto, Array.prototype) throws a TypeError");

assert.throws(TypeError, function() {
  Object.setPrototypeOf(ObjProto, ObjProto);
}, "Object.setPrototypeOf(ObjProto, ObjProto) throws a TypeError");

assert.sameValue(
  Reflect.setPrototypeOf(ObjProto, {}),
  false,
  "Reflect.setPrototypeOf(ObjProto, {}) returns false"
);

assert.sameValue(
  Reflect.setPrototypeOf(ObjProto, Array.prototype),
  false,
  "Reflect.setPrototypeOf(ObjProto, Array.prototype) returns false"
);

assert.sameValue(
  Reflect.setPrototypeOf(ObjProto, ObjProto),
  false,
  "Reflect.setPrototypeOf(ObjProto, ObjProto) returns false"
);
