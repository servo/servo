// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Reflect.set.name value and property descriptor
info: |
  26.1.13 Reflect.set ( target, propertyKey, V [ , receiver ] )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [Reflect, Reflect.set]
---*/

verifyProperty(Reflect.set, "name", {
  value: "set",
  writable: false,
  enumerable: false,
  configurable: true
});
