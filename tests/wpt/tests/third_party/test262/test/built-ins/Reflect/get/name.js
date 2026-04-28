// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.6
description: >
  Reflect.get.name value and property descriptor
info: |
  26.1.6 Reflect.get ( target, propertyKey [ , receiver ])

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [Reflect]
---*/

verifyProperty(Reflect.get, "name", {
  value: "get",
  writable: false,
  enumerable: false,
  configurable: true
});
