// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.1.4
description: >
  Map instances are ordinary objects that inherit properties from the Map
  prototype.
---*/

assert.sameValue(
  Object.getPrototypeOf(new Map()),
  Map.prototype,
  '`Object.getPrototypeOf(new Map())` returns `Map.prototype`'
);
