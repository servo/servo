// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-object-prototype-object
description: >
  Object.prototype is still extensible and may have extensions prevented
info: |
  19.1.3 Properties of the Object Prototype Object

  The value of the [[Prototype]] internal slot of the Object prototype object is
  null and the initial value of the [[Extensible]] internal slot is true.
---*/

assert(
  Object.isExtensible(Object.prototype),
  "Object.prototype is extensible"
);

assert.sameValue(
  Object.preventExtensions(Object.prototype),
  Object.prototype,
  "Object.prototype may have extensions prevented"
);

assert.sameValue(
  Object.isExtensible(Object.prototype),
  false,
  "Object.prototype is not extensible after a preventExtensions operation"
);
