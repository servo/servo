// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.2
description: >
  Reflect.construct.length value and property descriptor
info: |
  26.1.2 Reflect.construct ( target, argumentsList [, newTarget] )

  The length property of the construct function is 2.
includes: [propertyHelper.js]
features: [Reflect, Reflect.construct]
---*/

verifyProperty(Reflect.construct, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true
});
