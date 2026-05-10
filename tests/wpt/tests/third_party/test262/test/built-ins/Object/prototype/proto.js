// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-object-prototype-object
description: >
  The value of the [[Prototype]] internal slot of Object.prototype is null
info: |
  19.1.3 Properties of the Object Prototype Object

  The value of the [[Prototype]] internal slot of the Object prototype object is
  null and the initial value of the [[Extensible]] internal slot is true.
---*/

assert.sameValue(Object.getPrototypeOf(Object.prototype), null);
