// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-immutable-prototype-exotic-objects-setprototypeof-v
description: >
  Object.prototype's [[SetPrototypeOf]] returns true if value is same
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

assert.sameValue(
  Object.setPrototypeOf(ObjProto, null),
  ObjProto,
  "Object.setPrototypeOf(ObjProto, null) returns the Object.prototype"
);

assert(
  Object.isExtensible(ObjProto),
  "Object.prototype is still extensible after a setPrototypeOf operation - #1"
);

assert.sameValue(
  Reflect.setPrototypeOf(ObjProto, null),
  true,
  "Reflect.setPrototypeOf(ObjProto, null) returns true"
);

assert(
  Object.isExtensible(ObjProto),
  "Object.prototype is still extensible after a setPrototypeOf operation - #2"
);
