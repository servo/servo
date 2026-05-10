// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.7
description: >
  Reflect.getOwnPropertyDescriptor.name value and property descriptor
info: |
  26.1.7 Reflect.getOwnPropertyDescriptor ( target, propertyKey )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [Reflect]
---*/

verifyProperty(Reflect.getOwnPropertyDescriptor, "name", {
  value: "getOwnPropertyDescriptor",
  writable: false,
  enumerable: false,
  configurable: true
});
